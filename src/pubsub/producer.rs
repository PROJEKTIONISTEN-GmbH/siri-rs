//! The producer half of the data hub.

use std::marker::PhantomData;

use chrono::{DateTime, FixedOffset};

use crate::enumerations::DeliveryMethod;
use crate::framework::{
    CheckStatusResponse, DataReadyNotification, ErrorCondition, HeartbeatNotification,
    ServiceDelivery, ServiceRequestError, Siri, SiriPayload, StatusResponse, SubscriptionRequest,
    SubscriptionResponse, TerminateSubscriptionRequest, TerminateSubscriptionResponse,
    TerminationResponseStatus, TerminationScope,
};
use crate::pubsub::envelope;
use crate::pubsub::service::{Service, Source, SubscriptionParts};
use crate::types::{Duration, EndpointAddress, MessageQualifier, ParticipantRef, SubscriptionRef};
use crate::{Error, Result};

/// How a producer identifies itself and answers.
#[derive(Debug, Clone)]
pub struct ProducerConfig {
    /// Who this producer is, quoted in every message it sends.
    pub producer_ref: ParticipantRef,
    /// Whether deliveries are pushed, or announced for the consumer to fetch.
    pub delivery_method: DeliveryMethod,
    /// How often to send a heartbeat, if at all.
    pub heartbeat_interval: Option<Duration>,
    /// The shortest interval at which this producer will accept requests.
    pub shortest_possible_cycle: Option<Duration>,
    /// Prefix for the message identifiers this producer mints.
    pub message_id_prefix: String,
}

impl ProducerConfig {
    /// A producer that pushes deliveries and sends no heartbeat.
    pub fn new(producer_ref: impl Into<ParticipantRef>) -> Self {
        let producer_ref = producer_ref.into();
        Self {
            message_id_prefix: producer_ref.as_str().to_owned(),
            producer_ref,
            delivery_method: DeliveryMethod::Direct,
            heartbeat_interval: None,
            shortest_possible_cycle: None,
        }
    }

    /// The same producer, announcing data instead of pushing it.
    pub fn with_fetched_delivery(mut self) -> Self {
        self.delivery_method = DeliveryMethod::Fetched;
        self
    }

    /// The same producer, sending a heartbeat at the given interval.
    pub fn with_heartbeat(mut self, interval: Duration) -> Self {
        self.heartbeat_interval = Some(interval);
        self
    }
}

/// A subscription a producer holds on a consumer's behalf.
#[derive(Debug, Clone)]
pub struct Subscription<S: Service> {
    /// Who subscribed.
    pub subscriber_ref: ParticipantRef,
    /// The producer's handle on this subscription.
    pub subscription_ref: SubscriptionRef,
    /// Where deliveries for it should be sent.
    pub consumer_address: Option<EndpointAddress>,
    /// When it lapses unless renewed.
    pub initial_termination_time: DateTime<FixedOffset>,
    /// What was subscribed to.
    pub request: S::Request,
    /// Whether the consumer asked for changes only.
    pub incremental_updates: bool,
    /// Where the subscription is in its lifecycle.
    pub state: SubscriptionState,
}

/// Where a [`Subscription`] is in its lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubscriptionState {
    /// Nothing is owed to the consumer.
    Idle,
    /// Data has changed and the consumer has not been told yet.
    DeliveryDue,
    /// The consumer has been told data is ready and has not fetched it yet.
    AwaitingFetch(MessageQualifier),
}

/// A message a producer wants to send, and who to.
#[derive(Debug, Clone, PartialEq)]
pub struct Outbound {
    /// Who the message is for.
    pub recipient: ParticipantRef,
    /// Where to send it, when the consumer named an address.
    pub address: Option<EndpointAddress>,
    /// The message.
    pub message: Siri,
}

/// The producer half of a SIRI exchange.
///
/// Feed it incoming messages with [`Producer::handle`] and ask it for the messages
/// that are due with [`Producer::poll`]. It never reads a clock of its own: the
/// caller passes the current instant, which keeps its behaviour reproducible.
///
/// Which functional service it serves follows from the source it is given: a
/// [`SituationSource`](super::SituationSource) makes it a SIRI-SX producer, an
/// [`EstimatedTimetableSource`](super::EstimatedTimetableSource) a SIRI-ET one, and
/// so on. Messages addressed to another service are refused rather than
/// misinterpreted.
#[derive(Debug)]
pub struct Producer<Src, Svc: Service> {
    config: ProducerConfig,
    source: Src,
    service_started_time: Option<DateTime<FixedOffset>>,
    subscriptions: Vec<Subscription<Svc>>,
    last_heartbeat: Option<DateTime<FixedOffset>>,
    next_message_id: u64,
    service: PhantomData<Svc>,
}

impl<Src: Source<Svc>, Svc: Service> Producer<Src, Svc> {
    /// A producer publishing what `source` holds.
    pub fn new(config: ProducerConfig, source: Src) -> Self {
        Self {
            config,
            source,
            service_started_time: None,
            subscriptions: Vec::new(),
            last_heartbeat: None,
            next_message_id: 1,
            service: PhantomData,
        }
    }

    /// Records when this producer's service started.
    ///
    /// Consumers watch this: a value later than the one they last saw means the
    /// producer restarted and their subscriptions are gone.
    pub fn started_at(mut self, service_started_time: DateTime<FixedOffset>) -> Self {
        self.service_started_time = Some(service_started_time);
        self
    }

    /// The subscriptions this producer currently holds.
    pub fn subscriptions(&self) -> &[Subscription<Svc>] {
        &self.subscriptions
    }

    /// The source, so the application can keep updating it.
    pub fn source_mut(&mut self) -> &mut Src {
        &mut self.source
    }

    /// Marks every subscription as owing its consumer a delivery.
    ///
    /// Call this when the underlying data changes. Subscriptions already waiting for
    /// a fetch keep waiting: the consumer will get the new data when it collects, so
    /// a second notification would be noise.
    pub fn data_changed(&mut self) {
        for subscription in &mut self.subscriptions {
            if subscription.state == SubscriptionState::Idle {
                subscription.state = SubscriptionState::DeliveryDue;
            }
        }
    }

    /// Answers an incoming message.
    ///
    /// Returns the reply to send straight back, or `None` for messages that are
    /// acknowledged by silence.
    pub fn handle(&mut self, message: &Siri, now: DateTime<FixedOffset>) -> Result<Option<Siri>> {
        match &message.payload {
            SiriPayload::SubscriptionRequest(request) => {
                Ok(Some(self.open_subscriptions(request, now)?))
            }
            SiriPayload::TerminateSubscriptionRequest(request) => {
                Ok(Some(self.close_subscriptions(request, now)))
            }
            SiriPayload::DataSupplyRequest(request) => {
                let consumer = request.consumer_ref.clone().ok_or_else(|| Error::InvalidValue {
                    datatype: "DataSupplyRequest/ConsumerRef",
                    value: String::new(),
                })?;
                Ok(Some(self.supply(&consumer, now)))
            }
            SiriPayload::CheckStatusRequest(_) => Ok(Some(envelope(CheckStatusResponse {
                shortest_possible_cycle: self.config.shortest_possible_cycle.clone(),
                ..CheckStatusResponse::healthy(
                    now,
                    self.config.producer_ref.clone(),
                    self.service_started_time.unwrap_or(now),
                )
            }))),
            SiriPayload::ServiceRequest(request) => {
                let mut deliveries = Vec::with_capacity(request.requests.len());
                for payload in &request.requests {
                    let asked = Svc::request_of(payload).ok_or(Error::UnexpectedRoot {
                        expected: "a request for the service this producer serves",
                        found: request_name(payload).to_owned(),
                    })?;
                    deliveries.push(Svc::service_delivery(self.delivery_for(asked, now)));
                }
                Ok(Some(envelope(ServiceDelivery {
                    request_message_ref: request
                        .message_identifier
                        .as_ref()
                        .map(|id| id.as_str().into()),
                    ..ServiceDelivery::new(now, self.config.producer_ref.clone(), deliveries)
                })))
            }
            SiriPayload::DataReadyAcknowledgement(_)
            | SiriPayload::DataReceivedAcknowledgement(_) => Ok(None),
            other => Err(Error::UnexpectedRoot {
                expected: "a request a producer answers",
                found: payload_name(other).to_owned(),
            }),
        }
    }

    /// The messages that have become due since the last call.
    pub fn poll(&mut self, now: DateTime<FixedOffset>) -> Vec<Outbound> {
        let mut outbound = self.expire_subscriptions(now);
        outbound.extend(self.pending_deliveries(now));
        outbound.extend(self.due_heartbeat(now));
        outbound
    }

    fn open_subscriptions(
        &mut self,
        request: &SubscriptionRequest,
        now: DateTime<FixedOffset>,
    ) -> Result<Siri> {
        let mut statuses = Vec::new();
        for payload in &request.subscriptions {
            let subscription: SubscriptionParts<Svc> =
                Svc::subscription_of(payload).ok_or(Error::UnexpectedRoot {
                    expected: "a subscription to the service this producer serves",
                    found: subscription_name(payload).to_owned(),
                })?;
            let subscriber_ref = subscription
                .subscriber_ref
                .clone()
                .unwrap_or_else(|| request.requestor_ref.clone());
            let subscription_ref =
                SubscriptionRef::new(subscription.subscription_identifier.as_str());

            if subscription.initial_termination_time <= now {
                statuses.push(StatusResponse::refused(
                    now,
                    subscription_ref,
                    ErrorCondition::with_description(
                        ServiceRequestError::OtherError(Default::default()),
                        "InitialTerminationTime is in the past",
                    ),
                ));
                continue;
            }

            self.subscriptions
                .retain(|held| held.subscription_ref != subscription_ref);
            self.subscriptions.push(Subscription {
                subscriber_ref,
                subscription_ref: subscription_ref.clone(),
                consumer_address: request.consumer_address.clone(),
                initial_termination_time: subscription.initial_termination_time,
                request: subscription.request,
                incremental_updates: subscription.incremental_updates.unwrap_or(false),
                state: SubscriptionState::DeliveryDue,
            });
            statuses.push(StatusResponse {
                shortest_possible_cycle: self.config.shortest_possible_cycle.clone(),
                valid_until: Some(subscription.initial_termination_time),
                ..StatusResponse::accepted(now, subscription_ref)
            });
        }

        Ok(envelope(SubscriptionResponse {
            request_message_ref: request.message_identifier.as_ref().map(|id| id.as_str().into()),
            service_started_time: self.service_started_time,
            ..SubscriptionResponse::new(now, self.config.producer_ref.clone(), statuses)
        }))
    }

    fn close_subscriptions(
        &mut self,
        request: &TerminateSubscriptionRequest,
        now: DateTime<FixedOffset>,
    ) -> Siri {
        let subscriber = request
            .subscriber_ref
            .clone()
            .unwrap_or_else(|| request.requestor_ref.clone());
        let closing: Vec<SubscriptionRef> = match request.scope() {
            Some(TerminationScope::All) => self
                .subscriptions
                .iter()
                .filter(|held| held.subscriber_ref == subscriber)
                .map(|held| held.subscription_ref.clone())
                .collect(),
            Some(TerminationScope::Subscriptions(refs)) => refs
                .iter()
                .map(|qualifier| SubscriptionRef::new(qualifier.as_str()))
                .collect(),
            None => Vec::new(),
        };

        let statuses = closing
            .into_iter()
            .map(|subscription_ref| {
                self.subscriptions.retain(|held| {
                    held.subscription_ref != subscription_ref || held.subscriber_ref != subscriber
                });
                TerminationResponseStatus {
                    subscriber_ref: Some(subscriber.clone()),
                    ..TerminationResponseStatus::terminated(now, subscription_ref)
                }
            })
            .collect();

        envelope(TerminateSubscriptionResponse {
            request_message_ref: request.message_identifier.as_ref().map(|id| id.as_str().into()),
            ..TerminateSubscriptionResponse::new(now, self.config.producer_ref.clone(), statuses)
        })
    }

    /// Builds the delivery a consumer's outstanding fetch is waiting for.
    fn supply(&mut self, consumer: &ParticipantRef, now: DateTime<FixedOffset>) -> Siri {
        let waiting: Vec<Subscription<Svc>> = self
            .subscriptions
            .iter()
            .filter(|held| {
                held.subscriber_ref == *consumer
                    && matches!(held.state, SubscriptionState::AwaitingFetch(_))
            })
            .cloned()
            .collect();

        let deliveries = waiting
            .iter()
            .map(|subscription| Svc::service_delivery(self.subscription_delivery(subscription, now)))
            .collect();

        for held in &mut self.subscriptions {
            if held.subscriber_ref == *consumer
                && matches!(held.state, SubscriptionState::AwaitingFetch(_))
            {
                held.state = SubscriptionState::Idle;
            }
        }

        envelope(ServiceDelivery::new(
            now,
            self.config.producer_ref.clone(),
            deliveries,
        ))
    }

    fn pending_deliveries(&mut self, now: DateTime<FixedOffset>) -> Vec<Outbound> {
        let due: Vec<Subscription<Svc>> = self
            .subscriptions
            .iter()
            .filter(|held| held.state == SubscriptionState::DeliveryDue)
            .cloned()
            .collect();

        due.into_iter()
            .map(|subscription| {
                let message = match self.config.delivery_method {
                    DeliveryMethod::Direct => envelope(ServiceDelivery::new(
                        now,
                        self.config.producer_ref.clone(),
                        vec![Svc::service_delivery(
                            self.subscription_delivery(&subscription, now),
                        )],
                    )),
                    DeliveryMethod::Fetched => {
                        let identifier = self.mint_message_id();
                        self.set_state(
                            &subscription.subscription_ref,
                            SubscriptionState::AwaitingFetch(identifier.clone()),
                        );
                        envelope(DataReadyNotification {
                            message_identifier: Some(identifier),
                            ..DataReadyNotification::new(now, self.config.producer_ref.clone())
                        })
                    }
                };
                if self.config.delivery_method == DeliveryMethod::Direct {
                    self.set_state(&subscription.subscription_ref, SubscriptionState::Idle);
                }
                Outbound {
                    recipient: subscription.subscriber_ref.clone(),
                    address: subscription.consumer_address.clone(),
                    message,
                }
            })
            .collect()
    }

    fn expire_subscriptions(&mut self, now: DateTime<FixedOffset>) -> Vec<Outbound> {
        let expired: Vec<Subscription<Svc>> = self
            .subscriptions
            .iter()
            .filter(|held| held.initial_termination_time <= now)
            .cloned()
            .collect();
        self.subscriptions
            .retain(|held| held.initial_termination_time > now);

        expired
            .into_iter()
            .map(|subscription| Outbound {
                recipient: subscription.subscriber_ref.clone(),
                address: subscription.consumer_address.clone(),
                message: envelope(crate::framework::SubscriptionTerminatedNotification {
                    response_timestamp: now,
                    producer_ref: Some(self.config.producer_ref.clone()),
                    address: None,
                    response_message_identifier: None,
                    request_message_ref: None,
                    delegator_address: None,
                    delegator_ref: None,
                    subscriber_ref: Some(subscription.subscriber_ref.clone()),
                    subscription_filter_ref: None,
                    subscription_ref: subscription.subscription_ref.clone(),
                    error_condition: None,
                    extensions: None,
                }),
            })
            .collect()
    }

    fn due_heartbeat(&mut self, now: DateTime<FixedOffset>) -> Option<Outbound> {
        let interval = self.config.heartbeat_interval.as_ref()?.to_std()?;
        let interval = chrono::Duration::from_std(interval).ok()?;
        if self.last_heartbeat.is_some_and(|last| now - last < interval) {
            return None;
        }
        self.last_heartbeat = Some(now);

        let recipient = self.subscriptions.first()?.subscriber_ref.clone();
        let address = self.subscriptions.first()?.consumer_address.clone();
        Some(Outbound {
            recipient,
            address,
            message: envelope(HeartbeatNotification {
                valid_until: Some(now + interval),
                shortest_possible_cycle: self.config.shortest_possible_cycle.clone(),
                ..HeartbeatNotification::healthy(
                    now,
                    self.config.producer_ref.clone(),
                    self.service_started_time.unwrap_or(now),
                )
            }),
        })
    }

    fn subscription_delivery(
        &self,
        subscription: &Subscription<Svc>,
        now: DateTime<FixedOffset>,
    ) -> Svc::Delivery {
        let mut delivery = self.delivery_for(&subscription.request, now);
        Svc::attribute(
            &mut delivery,
            subscription.subscriber_ref.clone(),
            subscription.subscription_ref.clone(),
        );
        delivery
    }

    fn delivery_for(&self, request: &Svc::Request, now: DateTime<FixedOffset>) -> Svc::Delivery {
        Svc::delivery(now, self.source.items(request))
    }

    fn set_state(&mut self, subscription_ref: &SubscriptionRef, state: SubscriptionState) {
        if let Some(held) = self
            .subscriptions
            .iter_mut()
            .find(|held| held.subscription_ref == *subscription_ref)
        {
            held.state = state;
        }
    }

    fn mint_message_id(&mut self) -> MessageQualifier {
        let id = self.next_message_id;
        self.next_message_id += 1;
        MessageQualifier::new(format!("{}-{id}", self.config.message_id_prefix))
    }
}

fn request_name(payload: &crate::framework::ServiceRequestPayload) -> &'static str {
    use crate::framework::ServiceRequestPayload as Payload;
    match payload {
        Payload::ProductionTimetableRequest(_) => "ProductionTimetableRequest",
        Payload::EstimatedTimetableRequest(_) => "EstimatedTimetableRequest",
        Payload::StopTimetableRequest(_) => "StopTimetableRequest",
        Payload::StopMonitoringRequest(_) => "StopMonitoringRequest",
        Payload::StopMonitoringMultipleRequest(_) => "StopMonitoringMultipleRequest",
        Payload::VehicleMonitoringRequest(_) => "VehicleMonitoringRequest",
        Payload::ConnectionTimetableRequest(_) => "ConnectionTimetableRequest",
        Payload::ConnectionMonitoringRequest(_) => "ConnectionMonitoringRequest",
        Payload::GeneralMessageRequest(_) => "GeneralMessageRequest",
        Payload::FacilityMonitoringRequest(_) => "FacilityMonitoringRequest",
        Payload::ControlActionRequest(_) => "ControlActionRequest",
        Payload::ControlActionMultipleRequest(_) => "ControlActionMultipleRequest",
        Payload::SituationExchangeRequest(_) => "SituationExchangeRequest",
    }
}

fn subscription_name(payload: &crate::framework::SubscriptionRequestPayload) -> &'static str {
    use crate::framework::SubscriptionRequestPayload as Payload;
    match payload {
        Payload::ProductionTimetableSubscriptionRequest(_) => {
            "ProductionTimetableSubscriptionRequest"
        }
        Payload::EstimatedTimetableSubscriptionRequest(_) => "EstimatedTimetableSubscriptionRequest",
        Payload::StopTimetableSubscriptionRequest(_) => "StopTimetableSubscriptionRequest",
        Payload::StopMonitoringSubscriptionRequest(_) => "StopMonitoringSubscriptionRequest",
        Payload::VehicleMonitoringSubscriptionRequest(_) => "VehicleMonitoringSubscriptionRequest",
        Payload::ConnectionTimetableSubscriptionRequest(_) => {
            "ConnectionTimetableSubscriptionRequest"
        }
        Payload::ConnectionMonitoringSubscriptionRequest(_) => {
            "ConnectionMonitoringSubscriptionRequest"
        }
        Payload::GeneralMessageSubscriptionRequest(_) => "GeneralMessageSubscriptionRequest",
        Payload::FacilityMonitoringSubscriptionRequest(_) => "FacilityMonitoringSubscriptionRequest",
        Payload::ControlActionSubscriptionRequest(_) => "ControlActionSubscriptionRequest",
        Payload::SituationExchangeSubscriptionRequest(_) => "SituationExchangeSubscriptionRequest",
    }
}

fn payload_name(payload: &SiriPayload) -> &'static str {
    match payload {
        SiriPayload::ServiceRequest(_) => "ServiceRequest",
        SiriPayload::SubscriptionRequest(_) => "SubscriptionRequest",
        SiriPayload::TerminateSubscriptionRequest(_) => "TerminateSubscriptionRequest",
        SiriPayload::DataReadyNotification(_) => "DataReadyNotification",
        SiriPayload::DataSupplyRequest(_) => "DataSupplyRequest",
        SiriPayload::CheckStatusRequest(_) => "CheckStatusRequest",
        SiriPayload::HeartbeatNotification(_) => "HeartbeatNotification",
        SiriPayload::CapabilitiesRequest(_) => "CapabilitiesRequest",
        SiriPayload::StopPointsRequest(_) => "StopPointsRequest",
        SiriPayload::LinesRequest(_) => "LinesRequest",
        SiriPayload::ServiceFeaturesRequest(_) => "ServiceFeaturesRequest",
        SiriPayload::VehicleFeaturesRequest(_) => "VehicleFeaturesRequest",
        SiriPayload::ProductCategoriesRequest(_) => "ProductCategoriesRequest",
        SiriPayload::SubscriptionResponse(_) => "SubscriptionResponse",
        SiriPayload::TerminateSubscriptionResponse(_) => "TerminateSubscriptionResponse",
        SiriPayload::SubscriptionTerminatedNotification(_) => "SubscriptionTerminatedNotification",
        SiriPayload::DataReadyAcknowledgement(_) => "DataReadyAcknowledgement",
        SiriPayload::ServiceDelivery(_) => "ServiceDelivery",
        SiriPayload::DataReceivedAcknowledgement(_) => "DataReceivedAcknowledgement",
        SiriPayload::CheckStatusResponse(_) => "CheckStatusResponse",
        SiriPayload::CapabilitiesResponse(_) => "CapabilitiesResponse",
        SiriPayload::StopPointsDelivery(_) => "StopPointsDelivery",
        SiriPayload::LinesDelivery(_) => "LinesDelivery",
        SiriPayload::ServiceFeaturesDelivery(_) => "ServiceFeaturesDelivery",
        SiriPayload::VehicleFeaturesDelivery(_) => "VehicleFeaturesDelivery",
        SiriPayload::ProductCategoriesDelivery(_) => "ProductCategoriesDelivery",
    }
}
