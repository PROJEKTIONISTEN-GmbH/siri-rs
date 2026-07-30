//! The consumer half of the data hub.

use std::marker::PhantomData;

use chrono::{DateTime, FixedOffset};

use crate::framework::{
    DataReadyAcknowledgement, DataReceivedAcknowledgement, DataSupplyRequest, Siri, SiriPayload,
    SubscriptionContext, SubscriptionRequest, TerminateSubscriptionRequest,
};
use crate::pubsub::envelope;
use crate::pubsub::service::{Service, SubscriptionParts};
use crate::types::{
    Duration, EndpointAddress, MessageQualifier, ParticipantRef, SubscriptionQualifier,
    SubscriptionRef,
};
use crate::Result;

/// The outcome of a subscription request, per subscription.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subscribed {
    /// The subscription the producer answered about.
    pub subscription_ref: SubscriptionRef,
    /// Whether the producer accepted it.
    pub accepted: bool,
}

/// What an incoming message meant.
#[derive(Debug, Clone, PartialEq)]
pub enum ConsumerEvent<S: Service> {
    /// The producer answered a subscription request.
    Subscribed {
        /// One entry per subscription in the request.
        outcomes: Vec<Subscribed>,
    },
    /// The producer has data waiting; `reply` acknowledges it and `fetch` collects it.
    DataReady {
        /// Acknowledgement to send straight back.
        reply: Box<Siri>,
        /// The request that collects the data.
        fetch: Box<Siri>,
    },
    /// The producer delivered data.
    Delivered {
        /// The records, flattened across the deliveries in the message.
        items: Vec<S::Item>,
        /// Acknowledgement to send back, when the producer asked for confirmation.
        reply: Option<Box<Siri>>,
    },
    /// The producer reported itself alive.
    Alive {
        /// When the producer's service last started.
        service_started_time: Option<DateTime<FixedOffset>>,
    },
    /// The producer ended a subscription of its own accord.
    SubscriptionEnded {
        /// The subscription that ended.
        subscription_ref: SubscriptionRef,
    },
    /// The producer answered a termination request.
    Terminated {
        /// The subscriptions the producer confirmed as closed.
        subscription_refs: Vec<SubscriptionRef>,
    },
    /// The message needed no action.
    Ignored,
}

/// The consumer half of a SIRI exchange.
///
/// It builds the requests that drive a subscription and interprets what comes
/// back. Like [`Producer`](super::Producer) it reads no clock of its own.
///
/// Which functional service it speaks is the type parameter, so it has to be named
/// when a consumer is built: `Consumer::<SituationExchange>::new("MY-APP")`.
/// Deliveries from another service are reported as
/// [`ConsumerEvent::Ignored`] rather than misread.
#[derive(Debug)]
pub struct Consumer<S: Service> {
    requestor_ref: ParticipantRef,
    consumer_address: Option<EndpointAddress>,
    confirm_delivery: bool,
    subscriptions: Vec<SubscriptionRef>,
    next_message_id: u64,
    service: PhantomData<S>,
}

impl<S: Service> Consumer<S> {
    /// A consumer identifying itself as `requestor_ref`.
    pub fn new(requestor_ref: impl Into<ParticipantRef>) -> Self {
        Self {
            requestor_ref: requestor_ref.into(),
            consumer_address: None,
            confirm_delivery: false,
            subscriptions: Vec::new(),
            next_message_id: 1,
            service: PhantomData,
        }
    }

    /// Tells the producer where to send deliveries.
    pub fn at_address(mut self, consumer_address: impl Into<EndpointAddress>) -> Self {
        self.consumer_address = Some(consumer_address.into());
        self
    }

    /// Acknowledge every delivery rather than only data-ready notifications.
    pub fn confirming_deliveries(mut self) -> Self {
        self.confirm_delivery = true;
        self
    }

    /// The subscriptions the producer has accepted.
    pub fn subscriptions(&self) -> &[SubscriptionRef] {
        &self.subscriptions
    }

    /// Builds a request opening one subscription.
    pub fn subscribe(
        &mut self,
        subscription_identifier: impl Into<SubscriptionQualifier>,
        initial_termination_time: DateTime<FixedOffset>,
        request: S::Request,
        now: DateTime<FixedOffset>,
    ) -> Siri {
        self.subscribe_with_heartbeat(
            subscription_identifier,
            initial_termination_time,
            request,
            None,
            now,
        )
    }

    /// Builds a request opening one subscription and asking for a heartbeat.
    pub fn subscribe_with_heartbeat(
        &mut self,
        subscription_identifier: impl Into<SubscriptionQualifier>,
        initial_termination_time: DateTime<FixedOffset>,
        request: S::Request,
        heartbeat_interval: Option<Duration>,
        now: DateTime<FixedOffset>,
    ) -> Siri {
        let subscription = S::subscription_request(SubscriptionParts {
            subscriber_ref: None,
            subscription_identifier: subscription_identifier.into(),
            initial_termination_time,
            request,
            incremental_updates: None,
        });
        envelope(SubscriptionRequest {
            message_identifier: Some(self.mint_message_id()),
            consumer_address: self.consumer_address.clone(),
            subscription_context: heartbeat_interval
                .map(SubscriptionContext::with_heartbeat_interval),
            ..SubscriptionRequest::new(now, self.requestor_ref.clone(), vec![subscription])
        })
    }

    /// Builds a direct request for data, outside any subscription.
    pub fn request(&mut self, request: S::Request, now: DateTime<FixedOffset>) -> Siri {
        envelope(crate::framework::ServiceRequest {
            message_identifier: Some(self.mint_message_id()),
            address: self.consumer_address.clone(),
            ..crate::framework::ServiceRequest::new(
                now,
                self.requestor_ref.clone(),
                vec![S::service_request(request)],
            )
        })
    }

    /// Builds a request closing every subscription this consumer holds.
    pub fn terminate_all(&mut self, now: DateTime<FixedOffset>) -> Siri {
        envelope(TerminateSubscriptionRequest {
            message_identifier: Some(self.mint_message_id()),
            ..TerminateSubscriptionRequest::all(now, self.requestor_ref.clone())
        })
    }

    /// Interprets an incoming message.
    pub fn handle(&mut self, message: &Siri, now: DateTime<FixedOffset>) -> Result<ConsumerEvent<S>> {
        match &message.payload {
            SiriPayload::SubscriptionResponse(response) => {
                let outcomes: Vec<Subscribed> = response
                    .response_status
                    .iter()
                    .map(|status| Subscribed {
                        subscription_ref: status.subscription_ref.clone(),
                        accepted: status.is_accepted(),
                    })
                    .collect();
                for outcome in outcomes.iter().filter(|outcome| outcome.accepted) {
                    if !self.subscriptions.contains(&outcome.subscription_ref) {
                        self.subscriptions.push(outcome.subscription_ref.clone());
                    }
                }
                Ok(ConsumerEvent::Subscribed { outcomes })
            }
            SiriPayload::DataReadyNotification(notification) => {
                let notification_ref = notification
                    .message_identifier
                    .as_ref()
                    .map(|id| id.as_str().into());
                let reply = envelope(DataReadyAcknowledgement {
                    request_message_ref: notification_ref.clone(),
                    ..DataReadyAcknowledgement::accepted(now, self.requestor_ref.clone())
                });
                let mut fetch = DataSupplyRequest::new(
                    now,
                    self.requestor_ref.clone(),
                    notification_ref.unwrap_or_else(|| "".into()),
                );
                fetch.message_identifier = Some(self.mint_message_id());
                fetch.address = self.consumer_address.clone();
                Ok(ConsumerEvent::DataReady {
                    reply: Box::new(reply),
                    fetch: Box::new(envelope(fetch)),
                })
            }
            SiriPayload::ServiceDelivery(delivery) => {
                let items = delivery
                    .deliveries
                    .iter()
                    .filter_map(S::items_of)
                    .flatten()
                    .collect();
                let reply = self.confirm_delivery.then(|| {
                    Box::new(envelope(DataReceivedAcknowledgement {
                        request_message_ref: delivery
                            .response_message_identifier
                            .as_ref()
                            .map(|id| id.as_str().into()),
                        ..DataReceivedAcknowledgement::accepted(now, self.requestor_ref.clone())
                    }))
                });
                Ok(ConsumerEvent::Delivered { items, reply })
            }
            SiriPayload::HeartbeatNotification(heartbeat) => Ok(ConsumerEvent::Alive {
                service_started_time: heartbeat.service_started_time,
            }),
            SiriPayload::CheckStatusResponse(response) => Ok(ConsumerEvent::Alive {
                service_started_time: response.service_started_time,
            }),
            SiriPayload::SubscriptionTerminatedNotification(notification) => {
                self.subscriptions
                    .retain(|held| *held != notification.subscription_ref);
                Ok(ConsumerEvent::SubscriptionEnded {
                    subscription_ref: notification.subscription_ref.clone(),
                })
            }
            SiriPayload::TerminateSubscriptionResponse(response) => {
                let subscription_refs: Vec<SubscriptionRef> = response
                    .termination_response_status
                    .iter()
                    .filter(|status| status.status.unwrap_or(true))
                    .map(|status| status.subscription_ref.clone())
                    .collect();
                self.subscriptions
                    .retain(|held| !subscription_refs.contains(held));
                Ok(ConsumerEvent::Terminated { subscription_refs })
            }
            _ => Ok(ConsumerEvent::Ignored),
        }
    }

    fn mint_message_id(&mut self) -> MessageQualifier {
        let id = self.next_message_id;
        self.next_message_id += 1;
        MessageQualifier::new(format!("{}-{id}", self.requestor_ref))
    }
}
