//! The producer half of the data hub.

use std::marker::PhantomData;

use chrono::{DateTime, FixedOffset};

use crate::enumerations::DeliveryMethod;
use crate::framework::{
    CheckStatusResponse, DataReadyNotification, ErrorCondition, HeartbeatNotification,
    ServiceDelivery, ServiceRequestError, Siri, SiriPayload, StatusResponse, SubscriptionRequest,
    SubscriptionResponse, TerminateSubscriptionRequest, TerminateSubscriptionResponse,
    TerminationError, TerminationResponseStatus, TerminationScope, UnknownSubscriptionError,
};
use crate::pubsub::envelope;
use crate::pubsub::service::{Service, Source};
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

impl<S: Service> Subscription<S> {
    /// Whether this is the subscription `subscriber_ref` holds as `subscription_ref`.
    ///
    /// SIRI scopes a subscription identifier to its subscriber, so both are needed
    /// to name one: two subscribers may each call theirs `1`.
    fn is(&self, subscriber_ref: &ParticipantRef, subscription_ref: &SubscriptionRef) -> bool {
        self.subscriber_ref == *subscriber_ref && self.subscription_ref == *subscription_ref
    }

    /// Whether a message sent for this subscription arrives where one sent for
    /// `other` does: at the same address, or — when neither named one — at the same
    /// subscriber.
    fn same_endpoint(&self, other: &Self) -> bool {
        match (&self.consumer_address, &other.consumer_address) {
            (Some(here), Some(there)) => here == there,
            (None, None) => self.subscriber_ref == other.subscriber_ref,
            _ => false,
        }
    }
}

/// What a producer decided about one entry of a subscription request.
enum Decision<S: Service> {
    /// The entry is held from now on.
    Accepted(Subscription<S>),
    /// The entry is answered with a refusal and holds nothing.
    Refused {
        subscription_ref: SubscriptionRef,
        reason: ErrorCondition<ServiceRequestError>,
    },
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
    /// acknowledged by silence. A request the producer can read but not act on —
    /// one for another service, one lacking what the answer needs — is an error
    /// rather than a reply; a request it can act on is always answered, entry by
    /// entry, refusals included.
    pub fn handle(&mut self, message: &Siri, now: DateTime<FixedOffset>) -> Result<Option<Siri>> {
        match &message.payload {
            SiriPayload::SubscriptionRequest(request) => {
                Ok(Some(self.open_subscriptions(request, now)?))
            }
            SiriPayload::TerminateSubscriptionRequest(request) => {
                Ok(Some(self.close_subscriptions(request, now)))
            }
            SiriPayload::DataSupplyRequest(request) => {
                let consumer = request.consumer_ref.as_ref().ok_or(Error::MissingElement {
                    message: "DataSupplyRequest",
                    element: "ConsumerRef",
                })?;
                Ok(Some(self.supply(consumer, now)))
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
                if request.requests.is_empty() {
                    return Err(Error::MissingElement {
                        message: "ServiceRequest",
                        element: "AbstractFunctionalServiceRequest",
                    });
                }
                let mut deliveries = Vec::with_capacity(request.requests.len());
                for payload in &request.requests {
                    let asked = Svc::request_of(payload).ok_or(Error::UnexpectedMessage {
                        expected: "a request for the service this producer serves",
                        found: request_name(payload),
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
            other => Err(Error::UnexpectedMessage {
                expected: "a request a producer answers",
                found: payload_name(other),
            }),
        }
    }

    /// The messages that have become due since the last call.
    pub fn poll(&mut self, now: DateTime<FixedOffset>) -> Vec<Outbound> {
        let mut outbound = self.expire_subscriptions(now);
        outbound.extend(self.pending_deliveries(now));
        outbound.extend(self.due_heartbeats(now));
        outbound
    }

    /// Answers a subscription request with one status per subscription it names.
    ///
    /// Every entry is judged before any is held: a refusal is a status in the
    /// answer, not a failure of the request, and a request that fails as a whole —
    /// one naming no subscription at all, which the schema does not allow — has
    /// changed nothing by the time it fails.
    fn open_subscriptions(
        &mut self,
        request: &SubscriptionRequest,
        now: DateTime<FixedOffset>,
    ) -> Result<Siri> {
        if request.subscriptions.is_empty() {
            return Err(Error::MissingElement {
                message: "SubscriptionRequest",
                element: "AbstractFunctionalServiceSubscriptionRequest",
            });
        }
        let decisions: Vec<Decision<Svc>> = request
            .subscriptions
            .iter()
            .map(|payload| self.judge(payload, request, now))
            .collect();

        let mut statuses = Vec::with_capacity(decisions.len());
        for decision in decisions {
            match decision {
                Decision::Refused {
                    subscription_ref,
                    reason,
                } => statuses.push(StatusResponse::refused(now, subscription_ref, reason)),
                Decision::Accepted(subscription) => {
                    // Reopening a subscription replaces the one held under that name.
                    self.subscriptions.retain(|held| {
                        !held.is(&subscription.subscriber_ref, &subscription.subscription_ref)
                    });
                    statuses.push(StatusResponse {
                        shortest_possible_cycle: self.config.shortest_possible_cycle.clone(),
                        valid_until: Some(subscription.initial_termination_time),
                        ..StatusResponse::accepted(now, subscription.subscription_ref.clone())
                    });
                    self.subscriptions.push(subscription);
                }
            }
        }

        Ok(envelope(SubscriptionResponse {
            request_message_ref: request.message_identifier.as_ref().map(|id| id.as_str().into()),
            service_started_time: self.service_started_time,
            ..SubscriptionResponse::new(now, self.config.producer_ref.clone(), statuses)
        }))
    }

    /// Decides one entry of a subscription request without holding anything yet.
    fn judge(
        &self,
        payload: &crate::framework::SubscriptionRequestPayload,
        request: &SubscriptionRequest,
        now: DateTime<FixedOffset>,
    ) -> Decision<Svc> {
        let subscription_ref = SubscriptionRef::new(payload.subscription_identifier().as_str());
        let refused = |description: String| Decision::Refused {
            subscription_ref: subscription_ref.clone(),
            reason: ErrorCondition::with_description(
                ServiceRequestError::OtherError(Default::default()),
                description,
            ),
        };

        let Some(subscription) = Svc::subscription_of(payload) else {
            return refused(format!(
                "{} is not a subscription to the service this producer serves",
                subscription_name(payload)
            ));
        };
        if subscription.initial_termination_time <= now {
            return refused("InitialTerminationTime is in the past".to_owned());
        }
        Decision::Accepted(Subscription {
            subscriber_ref: subscription
                .subscriber_ref
                .unwrap_or_else(|| request.requestor_ref.clone()),
            subscription_ref,
            consumer_address: request.consumer_address.clone(),
            initial_termination_time: subscription.initial_termination_time,
            request: subscription.request,
            incremental_updates: subscription.incremental_updates.unwrap_or(false),
            state: SubscriptionState::DeliveryDue,
        })
    }

    /// Answers a termination request with one status per subscription it names.
    ///
    /// A subscription the subscriber does not hold is reported as unknown, which is
    /// the error the schema provides for it; a request to close all of a
    /// subscriber's subscriptions when there are none is answered with no status at
    /// all, which the schema allows. Neither is a failure of the request.
    fn close_subscriptions(
        &mut self,
        request: &TerminateSubscriptionRequest,
        now: DateTime<FixedOffset>,
    ) -> Siri {
        let subscriber = request
            .subscriber_ref
            .clone()
            .unwrap_or_else(|| request.requestor_ref.clone());
        let closed = |subscription_ref: SubscriptionRef| TerminationResponseStatus {
            subscriber_ref: Some(subscriber.clone()),
            ..TerminationResponseStatus::terminated(now, subscription_ref)
        };

        let statuses: Vec<TerminationResponseStatus> = match request.scope() {
            Some(TerminationScope::All) => {
                let (closing, kept) = std::mem::take(&mut self.subscriptions)
                    .into_iter()
                    .partition(|held| held.subscriber_ref == subscriber);
                self.subscriptions = kept;
                closing
                    .into_iter()
                    .map(|held| closed(held.subscription_ref))
                    .collect()
            }
            Some(TerminationScope::Subscriptions(qualifiers)) => qualifiers
                .iter()
                .map(|qualifier| {
                    let subscription_ref = SubscriptionRef::new(qualifier.as_str());
                    let held_before = self.subscriptions.len();
                    self.subscriptions
                        .retain(|held| !held.is(&subscriber, &subscription_ref));
                    if self.subscriptions.len() < held_before {
                        closed(subscription_ref)
                    } else {
                        TerminationResponseStatus {
                            subscriber_ref: Some(subscriber.clone()),
                            ..TerminationResponseStatus::refused(
                                now,
                                subscription_ref,
                                ErrorCondition::new(TerminationError::UnknownSubscriptionError(
                                    UnknownSubscriptionError {
                                        subscription_code: Some(qualifier.clone()),
                                        ..Default::default()
                                    },
                                )),
                            )
                        }
                    }
                })
                .collect(),
            None => Vec::new(),
        };

        envelope(TerminateSubscriptionResponse {
            request_message_ref: request.message_identifier.as_ref().map(|id| id.as_str().into()),
            ..TerminateSubscriptionResponse::new(now, self.config.producer_ref.clone(), statuses)
        })
    }

    /// Builds the delivery a consumer's outstanding fetch is waiting for.
    ///
    /// Noted by position first and worked through afterwards, for the reason
    /// [`Self::pending_deliveries`] gives.
    ///
    /// A fetch that finds nothing waiting — a retry after a lost answer, say — is
    /// answered with a delivery of this service carrying no records. The schema
    /// requires at least one functional-service delivery in every `ServiceDelivery`
    /// (`ServiceDeliveryBodyGroup`), and a delivery with an empty record list is
    /// what it provides for "nothing"; a `ServiceDelivery` with no functional
    /// delivery at all is not a document the schema accepts.
    fn supply(&mut self, consumer: &ParticipantRef, now: DateTime<FixedOffset>) -> Siri {
        let waiting: Vec<usize> = self
            .subscriptions
            .iter()
            .enumerate()
            .filter(|(_, held)| {
                held.subscriber_ref == *consumer
                    && matches!(held.state, SubscriptionState::AwaitingFetch(_))
            })
            .map(|(index, _)| index)
            .collect();

        let mut deliveries = Vec::with_capacity(waiting.len().max(1));
        for index in waiting {
            deliveries.push(Svc::service_delivery(
                self.subscription_delivery(&self.subscriptions[index], now),
            ));
            self.subscriptions[index].state = SubscriptionState::Idle;
        }
        if deliveries.is_empty() {
            deliveries.push(Svc::service_delivery(Svc::delivery(now, Vec::new())));
        }

        envelope(ServiceDelivery::new(
            now,
            self.config.producer_ref.clone(),
            deliveries,
        ))
    }

    /// The messages the subscriptions that are owed one have fallen due for.
    ///
    /// The subscriptions that are due are noted by position first, and worked
    /// through afterwards, because building a message reads the subscription while
    /// recording what was sent writes to it. Taking them out of the producer instead
    /// would settle that by copying every one of them — including the request it
    /// holds — for each turn of the cycle.
    fn pending_deliveries(&mut self, now: DateTime<FixedOffset>) -> Vec<Outbound> {
        let due: Vec<usize> = self
            .subscriptions
            .iter()
            .enumerate()
            .filter(|(_, held)| held.state == SubscriptionState::DeliveryDue)
            .map(|(index, _)| index)
            .collect();

        let mut outbound = Vec::with_capacity(due.len());
        for index in due {
            let message = match self.config.delivery_method {
                // A method the crate does not know is answered the default way.
                DeliveryMethod::Direct | DeliveryMethod::Unrecognised(_) => {
                    let delivery = self.subscription_delivery(&self.subscriptions[index], now);
                    self.subscriptions[index].state = SubscriptionState::Idle;
                    envelope(ServiceDelivery::new(
                        now,
                        self.config.producer_ref.clone(),
                        vec![Svc::service_delivery(delivery)],
                    ))
                }
                DeliveryMethod::Fetched => {
                    let identifier = self.mint_message_id();
                    self.subscriptions[index].state =
                        SubscriptionState::AwaitingFetch(identifier.clone());
                    envelope(DataReadyNotification {
                        message_identifier: Some(identifier),
                        ..DataReadyNotification::new(now, self.config.producer_ref.clone())
                    })
                }
            };
            outbound.push(Outbound {
                recipient: self.subscriptions[index].subscriber_ref.clone(),
                address: self.subscriptions[index].consumer_address.clone(),
                message,
            });
        }
        outbound
    }

    fn expire_subscriptions(&mut self, now: DateTime<FixedOffset>) -> Vec<Outbound> {
        let producer_ref = &self.config.producer_ref;
        let mut outbound = Vec::new();
        self.subscriptions.retain(|held| {
            if held.initial_termination_time > now {
                return true;
            }
            outbound.push(Outbound {
                recipient: held.subscriber_ref.clone(),
                address: held.consumer_address.clone(),
                message: envelope(crate::framework::SubscriptionTerminatedNotification {
                    response_timestamp: now,
                    producer_ref: Some(producer_ref.clone()),
                    address: None,
                    response_message_identifier: None,
                    request_message_ref: None,
                    delegator_address: None,
                    delegator_ref: None,
                    subscriber_ref: Some(held.subscriber_ref.clone()),
                    subscription_filter_ref: None,
                    subscription_ref: held.subscription_ref.clone(),
                    error_condition: None,
                    extensions: None,
                }),
            });
            false
        });
        outbound
    }

    /// The heartbeat that has fallen due, addressed to every endpoint holding a
    /// subscription.
    ///
    /// A heartbeat is about the producer, not about a subscription, so each endpoint
    /// gets one: the subscriptions that named the same address share it, and a
    /// subscriber that named no address is reached once, by name. The interval is
    /// kept from the producer's side — one clock, however many consumers.
    fn due_heartbeats(&mut self, now: DateTime<FixedOffset>) -> Vec<Outbound> {
        let Some(interval) = self
            .config
            .heartbeat_interval
            .as_ref()
            .and_then(Duration::to_std)
            .and_then(|interval| chrono::Duration::from_std(interval).ok())
        else {
            return Vec::new();
        };
        if self.last_heartbeat.is_some_and(|last| now - last < interval) {
            return Vec::new();
        }
        self.last_heartbeat = Some(now);

        let heartbeat = envelope(HeartbeatNotification {
            valid_until: Some(now + interval),
            shortest_possible_cycle: self.config.shortest_possible_cycle.clone(),
            ..HeartbeatNotification::healthy(
                now,
                self.config.producer_ref.clone(),
                self.service_started_time.unwrap_or(now),
            )
        });
        let mut endpoints: Vec<&Subscription<Svc>> = Vec::new();
        for held in &self.subscriptions {
            if !endpoints.iter().any(|reached| reached.same_endpoint(held)) {
                endpoints.push(held);
            }
        }
        endpoints
            .into_iter()
            .map(|held| Outbound {
                recipient: held.subscriber_ref.clone(),
                address: held.consumer_address.clone(),
                message: heartbeat.clone(),
            })
            .collect()
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
