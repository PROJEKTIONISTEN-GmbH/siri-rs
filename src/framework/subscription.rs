//! Opening, renewing and closing subscriptions.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::ca::ControlActionSubscriptionRequest;
use crate::cm::ConnectionMonitoringSubscriptionRequest;
use crate::ct::ConnectionTimetableSubscriptionRequest;
use crate::et::EstimatedTimetableSubscriptionRequest;
use crate::fm::FacilityMonitoringSubscriptionRequest;
use crate::framework::error_condition::{
    ApplicationError, ErrorCondition, ServiceRequestError, TerminationError,
};
use crate::gm::GeneralMessageSubscriptionRequest;
use crate::pt::ProductionTimetableSubscriptionRequest;
use crate::sm::StopMonitoringSubscriptionRequest;
use crate::st::StopTimetableSubscriptionRequest;
use crate::sx::SituationExchangeSubscriptionRequest;
use crate::types::{
    Duration, Empty, EndpointAddress, Extensions, MessageQualifier, MessageRef,
    NaturalLanguageString, ParticipantRef, SubscriptionFilterRef, SubscriptionQualifier,
    SubscriptionRef,
};
use crate::vm::VehicleMonitoringSubscriptionRequest;

/// A request to open one or more subscriptions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubscriptionRequest {
    /// When the request was made.
    #[serde(rename = "RequestTimestamp")]
    pub request_timestamp: DateTime<FixedOffset>,
    /// Account the subscriber authenticates as.
    #[serde(rename = "AccountId", default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    /// Secret authenticating the account.
    #[serde(rename = "AccountKey", default, skip_serializing_if = "Option::is_none")]
    pub account_key: Option<String>,
    /// Address to send the subscription response to.
    #[serde(rename = "Address", default, skip_serializing_if = "Option::is_none")]
    pub address: Option<EndpointAddress>,
    /// Who is subscribing.
    #[serde(rename = "RequestorRef")]
    pub requestor_ref: ParticipantRef,
    /// Identifier the subscriber puts on this message.
    #[serde(rename = "MessageIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub message_identifier: Option<MessageQualifier>,
    /// Address of the participant this subscription is made on behalf of.
    #[serde(rename = "DelegatorAddress", default, skip_serializing_if = "Option::is_none")]
    pub delegator_address: Option<EndpointAddress>,
    /// Participant this subscription is made on behalf of.
    #[serde(rename = "DelegatorRef", default, skip_serializing_if = "Option::is_none")]
    pub delegator_ref: Option<ParticipantRef>,
    /// Where deliveries for these subscriptions should be sent.
    #[serde(rename = "ConsumerAddress", default, skip_serializing_if = "Option::is_none")]
    pub consumer_address: Option<EndpointAddress>,
    /// Filter shared by the subscriptions in this request.
    #[serde(rename = "SubscriptionFilterIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub subscription_filter_identifier: Option<String>,
    /// Settings that apply to every subscription in this request.
    #[serde(rename = "SubscriptionContext", default, skip_serializing_if = "Option::is_none")]
    pub subscription_context: Option<SubscriptionContext>,
    /// The subscriptions to open, all of the same functional service.
    #[serde(rename = "$value")]
    pub subscriptions: Vec<SubscriptionRequestPayload>,
}

impl SubscriptionRequest {
    /// A subscription request from `requestor_ref` carrying the given subscriptions.
    pub fn new(
        request_timestamp: DateTime<FixedOffset>,
        requestor_ref: impl Into<ParticipantRef>,
        subscriptions: Vec<SubscriptionRequestPayload>,
    ) -> Self {
        Self {
            request_timestamp,
            account_id: None,
            account_key: None,
            address: None,
            requestor_ref: requestor_ref.into(),
            message_identifier: None,
            delegator_address: None,
            delegator_ref: None,
            consumer_address: None,
            subscription_filter_identifier: None,
            subscription_context: None,
            subscriptions,
        }
    }
}

/// A functional service subscription inside a [`SubscriptionRequest`].
///
/// Non-exhaustive so that further services can be added without a breaking change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SubscriptionRequestPayload {
    /// A subscription to a planned timetable.
    ProductionTimetableSubscriptionRequest(Box<ProductionTimetableSubscriptionRequest>),
    /// A subscription to a real-time timetable.
    EstimatedTimetableSubscriptionRequest(Box<EstimatedTimetableSubscriptionRequest>),
    /// A subscription to the timetable at a stop.
    StopTimetableSubscriptionRequest(Box<StopTimetableSubscriptionRequest>),
    /// A subscription to what is due at a stop.
    StopMonitoringSubscriptionRequest(Box<StopMonitoringSubscriptionRequest>),
    /// A subscription to vehicle positions.
    VehicleMonitoringSubscriptionRequest(Box<VehicleMonitoringSubscriptionRequest>),
    /// A subscription to the connections planned over a connection link.
    ConnectionTimetableSubscriptionRequest(Box<ConnectionTimetableSubscriptionRequest>),
    /// A subscription to how those connections are going.
    ConnectionMonitoringSubscriptionRequest(Box<ConnectionMonitoringSubscriptionRequest>),
    /// A subscription to free-form messages.
    GeneralMessageSubscriptionRequest(Box<GeneralMessageSubscriptionRequest>),
    /// A subscription to the state of passenger facilities.
    FacilityMonitoringSubscriptionRequest(Box<FacilityMonitoringSubscriptionRequest>),
    /// A subscription to what a control room decides.
    ControlActionSubscriptionRequest(Box<ControlActionSubscriptionRequest>),
    /// A subscription to situations.
    SituationExchangeSubscriptionRequest(Box<SituationExchangeSubscriptionRequest>),
}

impl SubscriptionRequestPayload {
    /// The subscriber's name for the subscription, whichever service it is to.
    pub fn subscription_identifier(&self) -> &SubscriptionQualifier {
        match self {
            Self::ProductionTimetableSubscriptionRequest(r) => &r.subscription_identifier,
            Self::EstimatedTimetableSubscriptionRequest(r) => &r.subscription_identifier,
            Self::StopTimetableSubscriptionRequest(r) => &r.subscription_identifier,
            Self::StopMonitoringSubscriptionRequest(r) => &r.subscription_identifier,
            Self::VehicleMonitoringSubscriptionRequest(r) => &r.subscription_identifier,
            Self::ConnectionTimetableSubscriptionRequest(r) => &r.subscription_identifier,
            Self::ConnectionMonitoringSubscriptionRequest(r) => &r.subscription_identifier,
            Self::GeneralMessageSubscriptionRequest(r) => &r.subscription_identifier,
            Self::FacilityMonitoringSubscriptionRequest(r) => &r.subscription_identifier,
            Self::ControlActionSubscriptionRequest(r) => &r.subscription_identifier,
            Self::SituationExchangeSubscriptionRequest(r) => &r.subscription_identifier,
        }
    }
}

impl From<ProductionTimetableSubscriptionRequest> for SubscriptionRequestPayload {
    fn from(request: ProductionTimetableSubscriptionRequest) -> Self {
        Self::ProductionTimetableSubscriptionRequest(Box::new(request))
    }
}

impl From<EstimatedTimetableSubscriptionRequest> for SubscriptionRequestPayload {
    fn from(request: EstimatedTimetableSubscriptionRequest) -> Self {
        Self::EstimatedTimetableSubscriptionRequest(Box::new(request))
    }
}

impl From<StopTimetableSubscriptionRequest> for SubscriptionRequestPayload {
    fn from(request: StopTimetableSubscriptionRequest) -> Self {
        Self::StopTimetableSubscriptionRequest(Box::new(request))
    }
}

impl From<StopMonitoringSubscriptionRequest> for SubscriptionRequestPayload {
    fn from(request: StopMonitoringSubscriptionRequest) -> Self {
        Self::StopMonitoringSubscriptionRequest(Box::new(request))
    }
}

impl From<VehicleMonitoringSubscriptionRequest> for SubscriptionRequestPayload {
    fn from(request: VehicleMonitoringSubscriptionRequest) -> Self {
        Self::VehicleMonitoringSubscriptionRequest(Box::new(request))
    }
}

impl From<ConnectionTimetableSubscriptionRequest> for SubscriptionRequestPayload {
    fn from(request: ConnectionTimetableSubscriptionRequest) -> Self {
        Self::ConnectionTimetableSubscriptionRequest(Box::new(request))
    }
}

impl From<ConnectionMonitoringSubscriptionRequest> for SubscriptionRequestPayload {
    fn from(request: ConnectionMonitoringSubscriptionRequest) -> Self {
        Self::ConnectionMonitoringSubscriptionRequest(Box::new(request))
    }
}

impl From<GeneralMessageSubscriptionRequest> for SubscriptionRequestPayload {
    fn from(request: GeneralMessageSubscriptionRequest) -> Self {
        Self::GeneralMessageSubscriptionRequest(Box::new(request))
    }
}

impl From<FacilityMonitoringSubscriptionRequest> for SubscriptionRequestPayload {
    fn from(request: FacilityMonitoringSubscriptionRequest) -> Self {
        Self::FacilityMonitoringSubscriptionRequest(Box::new(request))
    }
}

impl From<ControlActionSubscriptionRequest> for SubscriptionRequestPayload {
    fn from(request: ControlActionSubscriptionRequest) -> Self {
        Self::ControlActionSubscriptionRequest(Box::new(request))
    }
}

impl From<SituationExchangeSubscriptionRequest> for SubscriptionRequestPayload {
    fn from(request: SituationExchangeSubscriptionRequest) -> Self {
        Self::SituationExchangeSubscriptionRequest(Box::new(request))
    }
}

/// Settings that apply to every subscription in a request.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SubscriptionContext {
    /// How often the producer should send a heartbeat while the subscription lives.
    #[serde(rename = "HeartbeatInterval", default, skip_serializing_if = "Option::is_none")]
    pub heartbeat_interval: Option<Duration>,
}

impl SubscriptionContext {
    /// A context asking for a heartbeat at the given interval.
    pub fn with_heartbeat_interval(heartbeat_interval: Duration) -> Self {
        Self {
            heartbeat_interval: Some(heartbeat_interval),
        }
    }
}

/// The outcome of a [`SubscriptionRequest`], one [`StatusResponse`] per subscription.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubscriptionResponse {
    /// When the response was made.
    #[serde(rename = "ResponseTimestamp")]
    pub response_timestamp: DateTime<FixedOffset>,
    /// Address of the responder.
    #[serde(rename = "Address", default, skip_serializing_if = "Option::is_none")]
    pub address: Option<EndpointAddress>,
    /// Who answered.
    #[serde(rename = "ResponderRef", default, skip_serializing_if = "Option::is_none")]
    pub responder_ref: Option<ParticipantRef>,
    /// The subscription request this answers.
    #[serde(rename = "RequestMessageRef", default, skip_serializing_if = "Option::is_none")]
    pub request_message_ref: Option<MessageRef>,
    /// Address of the participant the answer is given on behalf of.
    #[serde(rename = "DelegatorAddress", default, skip_serializing_if = "Option::is_none")]
    pub delegator_address: Option<EndpointAddress>,
    /// Participant the answer is given on behalf of.
    #[serde(rename = "DelegatorRef", default, skip_serializing_if = "Option::is_none")]
    pub delegator_ref: Option<ParticipantRef>,
    /// One status per requested subscription.
    #[serde(rename = "ResponseStatus")]
    pub response_status: Vec<StatusResponse>,
    /// Where to send later subscription management requests.
    #[serde(rename = "SubscriptionManagerAddress", default, skip_serializing_if = "Option::is_none")]
    pub subscription_manager_address: Option<EndpointAddress>,
    /// When the producer's service last started.
    #[serde(rename = "ServiceStartedTime", default, skip_serializing_if = "Option::is_none")]
    pub service_started_time: Option<DateTime<FixedOffset>>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl SubscriptionResponse {
    /// A response from `responder_ref` carrying the given per-subscription statuses.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        responder_ref: impl Into<ParticipantRef>,
        response_status: Vec<StatusResponse>,
    ) -> Self {
        Self {
            response_timestamp,
            address: None,
            responder_ref: Some(responder_ref.into()),
            request_message_ref: None,
            delegator_address: None,
            delegator_ref: None,
            response_status,
            subscription_manager_address: None,
            service_started_time: None,
            extensions: None,
        }
    }

    /// Whether every subscription in the request was accepted.
    pub fn all_accepted(&self) -> bool {
        self.response_status.iter().all(StatusResponse::is_accepted)
    }
}

/// The outcome for one subscription within a [`SubscriptionResponse`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatusResponse {
    /// When this status was determined.
    #[serde(rename = "ResponseTimestamp")]
    pub response_timestamp: DateTime<FixedOffset>,
    /// The request this status answers.
    #[serde(rename = "RequestMessageRef", default, skip_serializing_if = "Option::is_none")]
    pub request_message_ref: Option<MessageRef>,
    /// Who subscribed.
    #[serde(rename = "SubscriberRef", default, skip_serializing_if = "Option::is_none")]
    pub subscriber_ref: Option<ParticipantRef>,
    /// The shared filter the subscription uses.
    #[serde(rename = "SubscriptionFilterRef", default, skip_serializing_if = "Option::is_none")]
    pub subscription_filter_ref: Option<SubscriptionFilterRef>,
    /// The subscription this status is about.
    #[serde(rename = "SubscriptionRef")]
    pub subscription_ref: SubscriptionRef,
    /// Whether the subscription was accepted.
    #[serde(rename = "Status", default, skip_serializing_if = "Option::is_none")]
    pub status: Option<bool>,
    /// Why the subscription was refused.
    #[serde(rename = "ErrorCondition", default, skip_serializing_if = "Option::is_none")]
    pub error_condition: Option<ErrorCondition<ServiceRequestError>>,
    /// How long the producer guarantees this answer.
    #[serde(rename = "ValidUntil", default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<FixedOffset>>,
    /// The shortest interval at which the producer will send updates.
    #[serde(rename = "ShortestPossibleCycle", default, skip_serializing_if = "Option::is_none")]
    pub shortest_possible_cycle: Option<Duration>,
}

impl StatusResponse {
    /// A status saying the subscription was accepted.
    pub fn accepted(
        response_timestamp: DateTime<FixedOffset>,
        subscription_ref: impl Into<SubscriptionRef>,
    ) -> Self {
        Self {
            response_timestamp,
            request_message_ref: None,
            subscriber_ref: None,
            subscription_filter_ref: None,
            subscription_ref: subscription_ref.into(),
            status: Some(true),
            error_condition: None,
            valid_until: None,
            shortest_possible_cycle: None,
        }
    }

    /// A status saying the subscription was refused, with the reason.
    pub fn refused(
        response_timestamp: DateTime<FixedOffset>,
        subscription_ref: impl Into<SubscriptionRef>,
        error_condition: ErrorCondition<ServiceRequestError>,
    ) -> Self {
        Self {
            status: Some(false),
            error_condition: Some(error_condition),
            ..Self::accepted(response_timestamp, subscription_ref)
        }
    }

    /// Whether the subscription was accepted.
    ///
    /// `Status` is optional in the schema and defaults to true, so a status
    /// without the element counts as accepted.
    pub fn is_accepted(&self) -> bool {
        self.status.unwrap_or(true)
    }
}

/// A request to close subscriptions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TerminateSubscriptionRequest {
    /// When the request was made.
    #[serde(rename = "RequestTimestamp")]
    pub request_timestamp: DateTime<FixedOffset>,
    /// Account the requestor authenticates as.
    #[serde(rename = "AccountId", default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    /// Secret authenticating the account.
    #[serde(rename = "AccountKey", default, skip_serializing_if = "Option::is_none")]
    pub account_key: Option<String>,
    /// Address to send the response to.
    #[serde(rename = "Address", default, skip_serializing_if = "Option::is_none")]
    pub address: Option<EndpointAddress>,
    /// Who is asking.
    #[serde(rename = "RequestorRef")]
    pub requestor_ref: ParticipantRef,
    /// Identifier the requestor puts on this message.
    #[serde(rename = "MessageIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub message_identifier: Option<MessageQualifier>,
    /// Address of the participant this request is made on behalf of.
    #[serde(rename = "DelegatorAddress", default, skip_serializing_if = "Option::is_none")]
    pub delegator_address: Option<EndpointAddress>,
    /// Participant this request is made on behalf of.
    #[serde(rename = "DelegatorRef", default, skip_serializing_if = "Option::is_none")]
    pub delegator_ref: Option<ParticipantRef>,
    /// Whose subscriptions to close, if not the requestor's own.
    #[serde(rename = "SubscriberRef", default, skip_serializing_if = "Option::is_none")]
    pub subscriber_ref: Option<ParticipantRef>,
    /// Present to close every subscription the subscriber holds.
    #[serde(rename = "All", default, skip_serializing_if = "Option::is_none")]
    pub all: Option<Empty>,
    /// The individual subscriptions to close.
    #[serde(rename = "SubscriptionRef", default, skip_serializing_if = "Vec::is_empty")]
    pub subscription_ref: Vec<SubscriptionQualifier>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// Which subscriptions a [`TerminateSubscriptionRequest`] closes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TerminationScope<'a> {
    /// Every subscription the subscriber holds.
    All,
    /// The listed subscriptions.
    Subscriptions(&'a [SubscriptionQualifier]),
}

impl TerminateSubscriptionRequest {
    /// A request closing every subscription the requestor holds.
    pub fn all(
        request_timestamp: DateTime<FixedOffset>,
        requestor_ref: impl Into<ParticipantRef>,
    ) -> Self {
        Self {
            request_timestamp,
            account_id: None,
            account_key: None,
            address: None,
            requestor_ref: requestor_ref.into(),
            message_identifier: None,
            delegator_address: None,
            delegator_ref: None,
            subscriber_ref: None,
            all: Some(Empty::new()),
            subscription_ref: Vec::new(),
            extensions: None,
        }
    }

    /// A request closing the named subscriptions.
    pub fn subscriptions(
        request_timestamp: DateTime<FixedOffset>,
        requestor_ref: impl Into<ParticipantRef>,
        subscription_ref: Vec<SubscriptionQualifier>,
    ) -> Self {
        Self {
            all: None,
            subscription_ref,
            ..Self::all(request_timestamp, requestor_ref)
        }
    }

    /// Which alternative of the schema's choice this request carries, or `None`
    /// when neither is present.
    pub fn scope(&self) -> Option<TerminationScope<'_>> {
        if self.all.is_some() {
            Some(TerminationScope::All)
        } else if self.subscription_ref.is_empty() {
            None
        } else {
            Some(TerminationScope::Subscriptions(&self.subscription_ref))
        }
    }
}

/// The outcome of a [`TerminateSubscriptionRequest`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TerminateSubscriptionResponse {
    /// When the response was made.
    #[serde(rename = "ResponseTimestamp")]
    pub response_timestamp: DateTime<FixedOffset>,
    /// Address of the responder.
    #[serde(rename = "Address", default, skip_serializing_if = "Option::is_none")]
    pub address: Option<EndpointAddress>,
    /// Who answered.
    #[serde(rename = "ResponderRef", default, skip_serializing_if = "Option::is_none")]
    pub responder_ref: Option<ParticipantRef>,
    /// The termination request this answers.
    #[serde(rename = "RequestMessageRef", default, skip_serializing_if = "Option::is_none")]
    pub request_message_ref: Option<MessageRef>,
    /// Address of the participant the answer is given on behalf of.
    #[serde(rename = "DelegatorAddress", default, skip_serializing_if = "Option::is_none")]
    pub delegator_address: Option<EndpointAddress>,
    /// Participant the answer is given on behalf of.
    #[serde(rename = "DelegatorRef", default, skip_serializing_if = "Option::is_none")]
    pub delegator_ref: Option<ParticipantRef>,
    /// One status per subscription the request named.
    #[serde(rename = "TerminationResponseStatus", default, skip_serializing_if = "Vec::is_empty")]
    pub termination_response_status: Vec<TerminationResponseStatus>,
}

impl TerminateSubscriptionResponse {
    /// A response from `responder_ref` carrying the given per-subscription statuses.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        responder_ref: impl Into<ParticipantRef>,
        termination_response_status: Vec<TerminationResponseStatus>,
    ) -> Self {
        Self {
            response_timestamp,
            address: None,
            responder_ref: Some(responder_ref.into()),
            request_message_ref: None,
            delegator_address: None,
            delegator_ref: None,
            termination_response_status,
        }
    }
}

/// The outcome for one subscription within a [`TerminateSubscriptionResponse`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TerminationResponseStatus {
    /// When this status was determined.
    #[serde(rename = "ResponseTimestamp", default, skip_serializing_if = "Option::is_none")]
    pub response_timestamp: Option<DateTime<FixedOffset>>,
    /// The request this status answers.
    #[serde(rename = "RequestMessageRef", default, skip_serializing_if = "Option::is_none")]
    pub request_message_ref: Option<MessageRef>,
    /// Who held the subscription.
    #[serde(rename = "SubscriberRef", default, skip_serializing_if = "Option::is_none")]
    pub subscriber_ref: Option<ParticipantRef>,
    /// The shared filter the subscription used.
    #[serde(rename = "SubscriptionFilterRef", default, skip_serializing_if = "Option::is_none")]
    pub subscription_filter_ref: Option<SubscriptionFilterRef>,
    /// The subscription this status is about.
    #[serde(rename = "SubscriptionRef")]
    pub subscription_ref: SubscriptionRef,
    /// Whether the subscription was closed.
    #[serde(rename = "Status", default, skip_serializing_if = "Option::is_none")]
    pub status: Option<bool>,
    /// Why the subscription could not be closed.
    #[serde(rename = "ErrorCondition", default, skip_serializing_if = "Option::is_none")]
    pub error_condition: Option<ErrorCondition<TerminationError>>,
}

impl TerminationResponseStatus {
    /// A status saying the subscription was closed.
    pub fn terminated(
        response_timestamp: DateTime<FixedOffset>,
        subscription_ref: impl Into<SubscriptionRef>,
    ) -> Self {
        Self {
            response_timestamp: Some(response_timestamp),
            request_message_ref: None,
            subscriber_ref: None,
            subscription_filter_ref: None,
            subscription_ref: subscription_ref.into(),
            status: Some(true),
            error_condition: None,
        }
    }

    /// A status saying the subscription could not be closed, with the reason.
    pub fn refused(
        response_timestamp: DateTime<FixedOffset>,
        subscription_ref: impl Into<SubscriptionRef>,
        error_condition: ErrorCondition<TerminationError>,
    ) -> Self {
        Self {
            status: Some(false),
            error_condition: Some(error_condition),
            ..Self::terminated(response_timestamp, subscription_ref)
        }
    }
}

/// A producer telling a consumer it has ended a subscription of its own accord.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubscriptionTerminatedNotification {
    /// When the notification was made.
    #[serde(rename = "ResponseTimestamp")]
    pub response_timestamp: DateTime<FixedOffset>,
    /// Who ended the subscription.
    #[serde(rename = "ProducerRef", default, skip_serializing_if = "Option::is_none")]
    pub producer_ref: Option<ParticipantRef>,
    /// Address of the producer.
    #[serde(rename = "Address", default, skip_serializing_if = "Option::is_none")]
    pub address: Option<EndpointAddress>,
    /// Identifier the producer puts on this message.
    #[serde(rename = "ResponseMessageIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub response_message_identifier: Option<MessageQualifier>,
    /// The message this notification relates to.
    #[serde(rename = "RequestMessageRef", default, skip_serializing_if = "Option::is_none")]
    pub request_message_ref: Option<MessageRef>,
    /// Address of the participant the notification is sent on behalf of.
    #[serde(rename = "DelegatorAddress", default, skip_serializing_if = "Option::is_none")]
    pub delegator_address: Option<EndpointAddress>,
    /// Participant the notification is sent on behalf of.
    #[serde(rename = "DelegatorRef", default, skip_serializing_if = "Option::is_none")]
    pub delegator_ref: Option<ParticipantRef>,
    /// Who held the subscription.
    #[serde(rename = "SubscriberRef", default, skip_serializing_if = "Option::is_none")]
    pub subscriber_ref: Option<ParticipantRef>,
    /// The shared filter the subscription used.
    #[serde(rename = "SubscriptionFilterRef", default, skip_serializing_if = "Option::is_none")]
    pub subscription_filter_ref: Option<SubscriptionFilterRef>,
    /// The subscription that ended.
    #[serde(rename = "SubscriptionRef")]
    pub subscription_ref: SubscriptionRef,
    /// Why the subscription ended.
    ///
    /// The element name carries a spelling mistake that has been in the published
    /// schema since SIRI 2.0 and is kept here so documents stay valid.
    #[serde(rename = "ErrrorCondition", default, skip_serializing_if = "Option::is_none")]
    pub error_condition: Option<SubscriptionTerminatedErrorCondition>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// Why a producer ended a subscription.
///
/// Unlike the other error conditions this one describes itself with a
/// [`NaturalLanguageString`], which is what the schema prescribes here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubscriptionTerminatedErrorCondition {
    /// Which error occurred.
    #[serde(rename = "$value")]
    pub code: ApplicationError,
    /// Free text describing the error.
    #[serde(rename = "Description", default, skip_serializing_if = "Option::is_none")]
    pub description: Option<NaturalLanguageString>,
}
