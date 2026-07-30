//! Asking a service what it can do, and the answer it gives.
//!
//! A consumer that has never spoken to a producer before does not know which
//! interaction patterns, filters or languages that producer supports. The
//! capabilities exchange settles that: the consumer sends one
//! [`CapabilitiesRequest`] naming the functional services it is interested in, and
//! the producer answers with a [`CapabilitiesResponse`] describing each of them,
//! optionally including the permissions granted to the asking participant.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::{CommunicationsTransportMethod, CompressionMethod};
use crate::et::EstimatedTimetableCapabilitiesResponse;
use crate::pt::ProductionTimetableCapabilitiesResponse;
use crate::vm::VehicleMonitoringCapabilitiesResponse;
use crate::framework::error_condition::{ErrorCondition, ServiceRequestError};
use crate::model::{ConnectionLinkRef, DirectionRef, LineRef, OperatorRef};
use crate::types::{
    Duration, Empty, EndpointAddress, Extensions, MessageQualifier, MessageRef, ParticipantRef,
};
use crate::xml::SiriRoot;

siri_ref! {
    /// Identifies an edition of a permission set.
    ///
    /// A producer that revises who may see what can advertise the new edition under
    /// a fresh version, letting a consumer notice that its cached permissions are
    /// stale without comparing them entry by entry.
    PermissionVersionRef;
}

/// A request for the capabilities of one or more functional services.
///
/// Which services are asked about is decided by the [`payload`](Self::payload):
/// the schema lists one capability request element per functional service, and a
/// producer answers only for those that are present.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilitiesRequest {
    /// Version of SIRI the request conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// When the request was made.
    #[serde(rename = "RequestTimestamp")]
    pub request_timestamp: DateTime<FixedOffset>,
    /// Account the requestor authenticates as.
    #[serde(rename = "AccountId", default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    /// Secret authenticating the account.
    #[serde(rename = "AccountKey", default, skip_serializing_if = "Option::is_none")]
    pub account_key: Option<String>,
    /// Address to send the answer to.
    #[serde(rename = "Address", default, skip_serializing_if = "Option::is_none")]
    pub address: Option<EndpointAddress>,
    /// Who is asking.
    #[serde(rename = "RequestorRef")]
    pub requestor_ref: ParticipantRef,
    /// Address of the participant this request is made on behalf of.
    #[serde(rename = "DelegatorAddress", default, skip_serializing_if = "Option::is_none")]
    pub delegator_address: Option<EndpointAddress>,
    /// Participant this request is made on behalf of.
    #[serde(rename = "DelegatorRef", default, skip_serializing_if = "Option::is_none")]
    pub delegator_ref: Option<ParticipantRef>,
    /// One entry per functional service the requestor wants described.
    #[serde(rename = "$value")]
    pub payload: Vec<CapabilitiesRequestPayload>,
}

impl CapabilitiesRequest {
    /// A capabilities request from `requestor_ref` covering the given services.
    pub fn new(
        request_timestamp: DateTime<FixedOffset>,
        requestor_ref: impl Into<ParticipantRef>,
        payload: Vec<CapabilitiesRequestPayload>,
    ) -> Self {
        Self {
            version: None,
            request_timestamp,
            account_id: None,
            account_key: None,
            address: None,
            requestor_ref: requestor_ref.into(),
            delegator_address: None,
            delegator_ref: None,
            payload,
        }
    }
}

/// Which functional service a capability request inside a [`CapabilitiesRequest`]
/// asks about.
///
/// Every functional service takes the same trivially shaped capability request, so
/// all variants carry a [`ServiceCapabilitiesRequest`]; only the element name — and
/// therefore the variant — says which service is meant. The enum is non-exhaustive
/// because the schema may gain further services.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CapabilitiesRequestPayload {
    /// The planned timetable service.
    ProductionTimetableCapabilitiesRequest(ServiceCapabilitiesRequest),
    /// The real-time timetable service.
    EstimatedTimetableCapabilitiesRequest(ServiceCapabilitiesRequest),
    /// The timetable-at-a-stop service.
    StopTimetableCapabilitiesRequest(ServiceCapabilitiesRequest),
    /// The departures-at-a-stop service.
    StopMonitoringCapabilitiesRequest(ServiceCapabilitiesRequest),
    /// The vehicle-tracking service.
    VehicleMonitoringCapabilitiesRequest(ServiceCapabilitiesRequest),
    /// The planned-interchange service.
    ConnectionTimetableCapabilitiesRequest(ServiceCapabilitiesRequest),
    /// The interchange-feeder-tracking service.
    ConnectionMonitoringCapabilitiesRequest(ServiceCapabilitiesRequest),
    /// The free-text message service.
    GeneralMessageCapabilitiesRequest(ServiceCapabilitiesRequest),
    /// The facility-status service.
    FacilityMonitoringCapabilitiesRequest(ServiceCapabilitiesRequest),
    /// The incident and disruption service.
    SituationExchangeCapabilitiesRequest(ServiceCapabilitiesRequest),
}

/// A capability request for a single functional service.
///
/// The service being asked about is carried by the element name rather than by any
/// field, which is why one type serves all of them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceCapabilitiesRequest {
    /// Version of SIRI the request conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// When the request was made.
    #[serde(rename = "RequestTimestamp")]
    pub request_timestamp: DateTime<FixedOffset>,
    /// Identifier the requestor puts on this message.
    #[serde(rename = "MessageIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub message_identifier: Option<MessageQualifier>,
    /// Whether the answer should spell out what this requestor in particular is
    /// allowed to see, rather than only what the service offers in general.
    ///
    /// Only meaningful against a producer that does access control at all.
    #[serde(rename = "ParticipantPermissions", default, skip_serializing_if = "Option::is_none")]
    pub participant_permissions: Option<bool>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl ServiceCapabilitiesRequest {
    /// A capability request bearing only the timestamp the schema requires.
    pub fn new(request_timestamp: DateTime<FixedOffset>) -> Self {
        Self {
            version: None,
            request_timestamp,
            message_identifier: None,
            participant_permissions: None,
            extensions: None,
        }
    }
}

/// The capabilities of one or more functional services.
///
/// Answers a [`CapabilitiesRequest`], with one entry per service described.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilitiesResponse {
    /// Version of SIRI the response conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// When the response was made.
    #[serde(rename = "ResponseTimestamp")]
    pub response_timestamp: DateTime<FixedOffset>,
    /// Who answered.
    #[serde(rename = "ProducerRef", default, skip_serializing_if = "Option::is_none")]
    pub producer_ref: Option<ParticipantRef>,
    /// Address of the producer.
    #[serde(rename = "Address", default, skip_serializing_if = "Option::is_none")]
    pub address: Option<EndpointAddress>,
    /// Identifier the producer puts on this message.
    #[serde(rename = "ResponseMessageIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub response_message_identifier: Option<MessageQualifier>,
    /// The capabilities request this answers.
    #[serde(rename = "RequestMessageRef", default, skip_serializing_if = "Option::is_none")]
    pub request_message_ref: Option<MessageRef>,
    /// Address of the participant the answer is given on behalf of.
    #[serde(rename = "DelegatorAddress", default, skip_serializing_if = "Option::is_none")]
    pub delegator_address: Option<EndpointAddress>,
    /// Participant the answer is given on behalf of.
    #[serde(rename = "DelegatorRef", default, skip_serializing_if = "Option::is_none")]
    pub delegator_ref: Option<ParticipantRef>,
    /// One entry per functional service described.
    #[serde(rename = "$value")]
    pub payload: Vec<CapabilitiesResponsePayload>,
}

impl CapabilitiesResponse {
    /// A capabilities response from `producer_ref` describing the given services.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        producer_ref: impl Into<ParticipantRef>,
        payload: Vec<CapabilitiesResponsePayload>,
    ) -> Self {
        Self {
            version: None,
            response_timestamp,
            producer_ref: Some(producer_ref.into()),
            address: None,
            response_message_identifier: None,
            request_message_ref: None,
            delegator_address: None,
            delegator_ref: None,
            payload,
        }
    }
}

/// The capabilities of one functional service inside a [`CapabilitiesResponse`].
///
/// Each functional service adds one variant describing its own capabilities; the
/// enum is non-exhaustive so that the services this crate does not implement yet
/// can be added without a breaking change for callers that match on it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CapabilitiesResponsePayload {
    /// What the Production Timetable service offers.
    ProductionTimetableCapabilitiesResponse(ProductionTimetableCapabilitiesResponse),
    /// What the Estimated Timetable service offers.
    EstimatedTimetableCapabilitiesResponse(EstimatedTimetableCapabilitiesResponse),
    /// What the Vehicle Monitoring service offers.
    VehicleMonitoringCapabilitiesResponse(VehicleMonitoringCapabilitiesResponse),
    /// What the Situation Exchange service offers.
    SituationExchangeCapabilitiesResponse(SituationExchangeCapabilitiesResponse),
}

impl From<ProductionTimetableCapabilitiesResponse> for CapabilitiesResponsePayload {
    fn from(response: ProductionTimetableCapabilitiesResponse) -> Self {
        Self::ProductionTimetableCapabilitiesResponse(response)
    }
}

impl From<EstimatedTimetableCapabilitiesResponse> for CapabilitiesResponsePayload {
    fn from(response: EstimatedTimetableCapabilitiesResponse) -> Self {
        Self::EstimatedTimetableCapabilitiesResponse(response)
    }
}

impl From<VehicleMonitoringCapabilitiesResponse> for CapabilitiesResponsePayload {
    fn from(response: VehicleMonitoringCapabilitiesResponse) -> Self {
        Self::VehicleMonitoringCapabilitiesResponse(response)
    }
}

impl From<SituationExchangeCapabilitiesResponse> for CapabilitiesResponsePayload {
    fn from(response: SituationExchangeCapabilitiesResponse) -> Self {
        Self::SituationExchangeCapabilitiesResponse(response)
    }
}

/// What the Situation Exchange service offers, and to whom.
///
/// Appears both inside a [`CapabilitiesResponse`] and, when only this one service
/// is of interest, as a document in its own right.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SituationExchangeCapabilitiesResponse {
    /// Version of SIRI the response conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// When the response was made.
    #[serde(rename = "ResponseTimestamp")]
    pub response_timestamp: DateTime<FixedOffset>,
    /// The capability request this answers.
    #[serde(rename = "RequestMessageRef", default, skip_serializing_if = "Option::is_none")]
    pub request_message_ref: Option<MessageRef>,
    /// Address of the participant the answer is given on behalf of.
    #[serde(rename = "DelegatorAddress", default, skip_serializing_if = "Option::is_none")]
    pub delegator_address: Option<EndpointAddress>,
    /// Participant the answer is given on behalf of.
    #[serde(rename = "DelegatorRef", default, skip_serializing_if = "Option::is_none")]
    pub delegator_ref: Option<ParticipantRef>,
    /// Whether the capability request was processed successfully.
    #[serde(rename = "Status", default, skip_serializing_if = "Option::is_none")]
    pub status: Option<bool>,
    /// Why the capability request could not be processed.
    #[serde(rename = "ErrorCondition", default, skip_serializing_if = "Option::is_none")]
    pub error_condition: Option<ErrorCondition<ServiceRequestError>>,
    /// What the service can do.
    #[serde(
        rename = "SituationExchangeServiceCapabilities",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub situation_exchange_service_capabilities: Option<SituationExchangeServiceCapabilities>,
    /// What participants are allowed to see.
    #[serde(
        rename = "SituationExchangePermissions",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub situation_exchange_permissions: Option<SituationExchangePermissions>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl SituationExchangeCapabilitiesResponse {
    /// A successful answer describing the given capabilities.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        capabilities: SituationExchangeServiceCapabilities,
    ) -> Self {
        Self {
            version: None,
            response_timestamp,
            request_message_ref: None,
            delegator_address: None,
            delegator_ref: None,
            status: Some(true),
            error_condition: None,
            situation_exchange_service_capabilities: Some(capabilities),
            situation_exchange_permissions: None,
            extensions: None,
        }
    }

    /// Whether the producer reported the capability request as processed.
    ///
    /// `Status` is optional in the schema and defaults to true.
    pub fn is_success(&self) -> bool {
        self.status.unwrap_or(true)
    }
}

impl SiriRoot for SituationExchangeCapabilitiesResponse {
    const ELEMENT_NAME: &'static str = "SituationExchangeCapabilitiesResponse";
}

/// What a Situation Exchange service can do.
///
/// Every field is optional: a producer describes only the aspects it wants to
/// commit to, and a consumer treats an absent field as "the schema default".
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SituationExchangeServiceCapabilities {
    /// Interaction patterns and delivery mechanics common to all SIRI services.
    #[serde(rename = "GeneralInteraction", default, skip_serializing_if = "Option::is_none")]
    pub general_interaction: Option<GeneralInteractionCapability>,
    /// How messages are carried and whether they are compressed.
    #[serde(rename = "TransportDescription", default, skip_serializing_if = "Option::is_none")]
    pub transport_description: Option<TransportDescription>,
    /// Which criteria a requestor may narrow situations by.
    #[serde(rename = "TopicFiltering", default, skip_serializing_if = "Option::is_none")]
    pub topic_filtering: Option<SituationExchangeTopicFiltering>,
    /// Languages, coordinate format and volume limits applied to requests.
    #[serde(rename = "RequestPolicy", default, skip_serializing_if = "Option::is_none")]
    pub request_policy: Option<SituationExchangeRequestPolicy>,
    /// What may be asked for when opening a subscription.
    #[serde(rename = "SubscriptionPolicy", default, skip_serializing_if = "Option::is_none")]
    pub subscription_policy: Option<SubscriptionPolicyCapability>,
    /// Whether and how requests are checked against per-participant permissions.
    #[serde(rename = "AccessControl", default, skip_serializing_if = "Option::is_none")]
    pub access_control: Option<SituationExchangeAccessControl>,
    /// Optional features of the response content.
    ///
    /// The schema reserves the element without giving it content yet, so its
    /// presence is all a producer can currently state.
    #[serde(rename = "ResponseFeatures", default, skip_serializing_if = "Option::is_none")]
    pub response_features: Option<Empty>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// Interaction patterns and delivery mechanics a service supports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneralInteractionCapability {
    /// Whether data can be pulled, pushed, or both.
    #[serde(rename = "Interaction")]
    pub interaction: InteractionCapability,
    /// Whether data arrives unprompted or has to be fetched.
    #[serde(rename = "Delivery")]
    pub delivery: DeliveryCapability,
    /// Whether one logical delivery may be split over several messages, chained by
    /// the `MoreData` flag.
    #[serde(rename = "MultipartDespatch")]
    pub multipart_despatch: bool,
    /// Whether one subscription may carry the filters of several subscribers.
    #[serde(rename = "MultipleSubscriberFilter")]
    pub multiple_subscriber_filter: bool,
    /// Whether the producer accepts acknowledgements of the deliveries it sends.
    #[serde(rename = "HasConfirmDelivery")]
    pub has_confirm_delivery: bool,
    /// Whether the producer emits heartbeats between deliveries.
    #[serde(rename = "HasHeartbeat")]
    pub has_heartbeat: bool,
    /// Whether a visit number is a strict position within the journey pattern
    /// rather than merely an arbitrary label.
    #[serde(rename = "VisitNumberisOrder", default, skip_serializing_if = "Option::is_none")]
    pub visit_number_is_order: Option<bool>,
}

/// Which of the two SIRI interaction patterns a service supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InteractionCapability {
    /// Whether a consumer may ask for data directly and get it in the answer.
    #[serde(rename = "RequestResponse")]
    pub request_response: bool,
    /// Whether a consumer may register a standing interest and be sent data as it
    /// changes.
    #[serde(rename = "PublishSubscribe")]
    pub publish_subscribe: bool,
}

/// How a service hands subscribed data over.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliveryCapability {
    /// Whether the producer pushes deliveries to the consumer as they arise.
    #[serde(rename = "DirectDelivery")]
    pub direct_delivery: bool,
    /// Whether the producer only announces that data is waiting, leaving the
    /// consumer to fetch it.
    #[serde(rename = "FetchedDelivery")]
    pub fetched_delivery: bool,
}

/// How SIRI messages are carried between the two endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportDescription {
    /// The protocol and encoding messages are exchanged with.
    #[serde(rename = "CommunicationsTransportMethod")]
    pub communications_transport_method: CommunicationsTransportMethod,
    /// How message bodies are compressed for transmission, if at all.
    #[serde(rename = "CompressionMethod")]
    pub compression_method: CompressionMethod,
}

/// Which criteria a requestor may narrow a situation request by.
///
/// A `false` — or an absent field, which the schema gives a default for — means the
/// producer ignores that filter rather than rejecting a request that uses it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SituationExchangeTopicFiltering {
    /// The look-ahead window applied when a request does not name one.
    #[serde(rename = "DefaultPreviewInterval")]
    pub default_preview_interval: Duration,
    /// Whether situations can be narrowed to a particular passenger facility.
    #[serde(rename = "FilterByFacilityRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_facility_ref: Option<bool>,
    /// Whether situations can be narrowed to a geographic area.
    #[serde(rename = "FilterByLocationRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_location_ref: Option<bool>,
    /// Whether situations can be narrowed to a particular vehicle.
    #[serde(rename = "FilterByVehicleRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_vehicle_ref: Option<bool>,
    /// Whether situations can be narrowed to a mode of transport.
    #[serde(rename = "FilterByMode", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_mode: Option<bool>,
    /// Whether situations can be narrowed to a network of lines.
    #[serde(rename = "FilterByNetworkRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_network_ref: Option<bool>,
    /// Whether situations can be narrowed to a line.
    #[serde(rename = "FilterByLineRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_line_ref: Option<bool>,
    /// Whether situations can be narrowed to a scheduled stop point.
    #[serde(rename = "FilterByStopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_stop_point_ref: Option<bool>,
    /// Whether situations can be narrowed to a stop place.
    #[serde(rename = "FilterByStopPlaceRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_stop_place_ref: Option<bool>,
    /// Whether situations can be narrowed to a vehicle journey.
    #[serde(rename = "FilterByVehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_vehicle_journey_ref: Option<bool>,
    /// Whether situations can be narrowed to a connection link.
    #[serde(rename = "FilterByConnectionLinkRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_connection_link_ref: Option<bool>,
    /// Whether situations can be narrowed to a planned interchange.
    #[serde(rename = "FilterByInterchangeRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_interchange_ref: Option<bool>,
    /// Whether situations can be narrowed to those affecting a stated accessibility
    /// need.
    #[serde(rename = "FilterBySpecificNeed", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_specific_need: Option<bool>,
    /// Whether situations can be narrowed by producer-defined keywords.
    #[serde(rename = "FilterByKeyword", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_keyword: Option<bool>,
}

/// Languages, coordinate format and volume limits a service applies to requests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SituationExchangeRequestPolicy {
    /// Languages the service can return texts in, most preferred first.
    #[serde(rename = "NationalLanguage")]
    pub national_language: Vec<String>,
    /// Whether one text can be returned in several languages at once, rather than
    /// only in the single language the request asked for.
    #[serde(rename = "Translations", default, skip_serializing_if = "Option::is_none")]
    pub translations: Option<bool>,
    /// How positions are written in responses.
    #[serde(rename = "$value")]
    pub coordinate_format: CoordinateFormat,
    /// Whether a request may cap how many situations come back.
    #[serde(
        rename = "HasMaximumNumberOfSituations",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub has_maximum_number_of_situations: Option<bool>,
}

/// How a service writes the positions it returns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoordinateFormat {
    /// Positions come as a coordinate list in the named GML projection.
    GmlCoordinateFormat(String),
    /// Positions come as WGS 84 longitude and latitude in decimal degrees.
    WgsDecimalDegrees(Empty),
}

/// What a service allows a subscriber to ask for when opening a subscription.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionPolicyCapability {
    /// Whether a subscriber may ask to be sent only what changed since the last
    /// delivery instead of the full current picture.
    #[serde(rename = "HasIncrementalUpdates", default, skip_serializing_if = "Option::is_none")]
    pub has_incremental_updates: Option<bool>,
    /// Whether a subscriber may set how large a change has to be before it is worth
    /// a delivery.
    #[serde(rename = "HasChangeSensitivity", default, skip_serializing_if = "Option::is_none")]
    pub has_change_sensitivity: Option<bool>,
}

/// Whether and how a Situation Exchange service checks requests against the
/// permissions of the participant making them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SituationExchangeAccessControl {
    /// Whether requests are checked against permissions at all.
    #[serde(rename = "RequestChecking")]
    pub request_checking: bool,
    /// Whether the operator a request names is checked against its permissions.
    #[serde(rename = "CheckOperatorRef", default, skip_serializing_if = "Option::is_none")]
    pub check_operator_ref: Option<bool>,
    /// Whether the line a request names is checked against its permissions.
    #[serde(rename = "CheckLineRef", default, skip_serializing_if = "Option::is_none")]
    pub check_line_ref: Option<bool>,
}

/// What participants are allowed to see of a Situation Exchange service.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SituationExchangePermissions {
    /// The edition of the permission set these entries belong to.
    #[serde(rename = "PermissionVersionRef", default, skip_serializing_if = "Option::is_none")]
    pub permission_version_ref: Option<PermissionVersionRef>,
    /// One entry per participant, plus optionally one covering everybody else.
    #[serde(
        rename = "SituationExchangePermission",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub situation_exchange_permission: Vec<SituationExchangePermission>,
}

/// What one participant — or every participant without an entry of their own — may
/// see of a Situation Exchange service.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SituationExchangePermission {
    /// Who this entry applies to.
    #[serde(rename = "$value")]
    pub scope: PermissionScope,
    /// Which interaction patterns the participant may use.
    #[serde(rename = "GeneralCapabilities", default, skip_serializing_if = "Option::is_none")]
    pub general_capabilities: Option<GeneralPermissions>,
    /// Whose situations the participant may see.
    #[serde(rename = "OperatorPermissions")]
    pub operator_permissions: OperatorPermissions,
    /// Which lines' situations the participant may see.
    #[serde(rename = "LinePermissions")]
    pub line_permissions: LinePermissions,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl SituationExchangePermission {
    /// A permission entry applying to every participant that has no entry of its own.
    pub fn for_all_participants(
        operator_permissions: OperatorPermissions,
        line_permissions: LinePermissions,
    ) -> Self {
        Self {
            scope: PermissionScope::AllParticipants(Empty::new()),
            general_capabilities: None,
            operator_permissions,
            line_permissions,
            extensions: None,
        }
    }

    /// A permission entry applying to one named participant.
    pub fn for_participant(
        participant_ref: impl Into<ParticipantRef>,
        operator_permissions: OperatorPermissions,
        line_permissions: LinePermissions,
    ) -> Self {
        Self {
            scope: PermissionScope::ParticipantRef(participant_ref.into()),
            general_capabilities: None,
            operator_permissions,
            line_permissions,
            extensions: None,
        }
    }
}

/// Who a permission entry applies to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionScope {
    /// The entry is the default, applying to any participant that has no entry
    /// naming it specifically.
    AllParticipants(Empty),
    /// The entry applies to the named participant only.
    ParticipantRef(ParticipantRef),
}

/// Which interaction patterns a participant may use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneralPermissions {
    /// Whether the participant may ask for data directly.
    #[serde(rename = "RequestResponse")]
    pub request_response: bool,
    /// Whether the participant may open subscriptions.
    #[serde(rename = "PublishSubscribe")]
    pub publish_subscribe: bool,
}

/// Whose situations a participant may see.
///
/// Either a blanket permission for every operator the service knows, or one entry
/// per operator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperatorPermissions {
    /// The entries, all of the same kind.
    #[serde(rename = "$value")]
    pub items: Vec<OperatorPermissionItem>,
}

impl OperatorPermissions {
    /// Permission covering every operator the service knows about.
    pub fn allow_all() -> Self {
        Self {
            items: vec![OperatorPermissionItem::AllowAll(true)],
        }
    }

    /// Permission listed operator by operator.
    pub fn per_operator(permissions: Vec<OperatorPermission>) -> Self {
        Self {
            items: permissions
                .into_iter()
                .map(OperatorPermissionItem::OperatorPermission)
                .collect(),
        }
    }
}

/// One entry of an [`OperatorPermissions`] list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperatorPermissionItem {
    /// Whether every operator known to the service is covered.
    AllowAll(bool),
    /// A decision about one named operator.
    OperatorPermission(OperatorPermission),
}

/// Whether a participant may see one named operator's situations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperatorPermission {
    /// Whether access is granted or withheld.
    #[serde(rename = "Allow")]
    pub allow: bool,
    /// The operator the decision is about.
    #[serde(rename = "OperatorRef")]
    pub operator_ref: OperatorRef,
}

/// Which lines' situations a participant may see.
///
/// Either a blanket permission for every line the service knows, or one entry per
/// line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinePermissions {
    /// The entries, all of the same kind.
    #[serde(rename = "$value")]
    pub items: Vec<LinePermissionItem>,
}

impl LinePermissions {
    /// Permission covering every line the service knows about.
    pub fn allow_all() -> Self {
        Self {
            items: vec![LinePermissionItem::AllowAll(true)],
        }
    }

    /// Permission listed line by line.
    pub fn per_line(permissions: Vec<LinePermission>) -> Self {
        Self {
            items: permissions
                .into_iter()
                .map(LinePermissionItem::LinePermission)
                .collect(),
        }
    }
}

/// One entry of a [`LinePermissions`] list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinePermissionItem {
    /// Whether every line known to the service is covered.
    AllowAll(bool),
    /// A decision about one named line.
    LinePermission(LinePermission),
}

/// Whether a participant may see one named line's situations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinePermission {
    /// Whether access is granted or withheld.
    #[serde(rename = "Allow")]
    pub allow: bool,
    /// The line the decision is about.
    #[serde(rename = "LineRef")]
    pub line_ref: LineRef,
    /// The directions of that line the decision is limited to; empty means both.
    #[serde(rename = "DirectionRef", default, skip_serializing_if = "Vec::is_empty")]
    pub direction_ref: Vec<DirectionRef>,
}

/// Languages and coordinate format a service applies to every request.
///
/// Each functional service states this; the services that need more than the common
/// fields add them to their own request policy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityRequestPolicy {
    /// Languages the service can return texts in, most preferred first.
    #[serde(rename = "NationalLanguage")]
    pub national_language: Vec<String>,
    /// Whether one text can be returned in several languages at once.
    #[serde(rename = "Translations", default, skip_serializing_if = "Option::is_none")]
    pub translations: Option<bool>,
    /// How positions are written in responses.
    #[serde(rename = "$value")]
    pub coordinate_format: CoordinateFormat,
}

impl CapabilityRequestPolicy {
    /// A policy offering the given language and WGS 84 decimal degrees.
    pub fn in_language(national_language: impl Into<String>) -> Self {
        Self {
            national_language: vec![national_language.into()],
            translations: None,
            coordinate_format: CoordinateFormat::WgsDecimalDegrees(Empty::new()),
        }
    }
}

/// Whether and how a service that filters by connection link checks requests
/// against the permissions of the participant making them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionCapabilityAccessControl {
    /// Whether requests are checked against permissions at all.
    #[serde(rename = "RequestChecking")]
    pub request_checking: bool,
    /// Whether the operator a request names is checked against its permissions.
    #[serde(rename = "CheckOperatorRef", default, skip_serializing_if = "Option::is_none")]
    pub check_operator_ref: Option<bool>,
    /// Whether the line a request names is checked against its permissions.
    #[serde(rename = "CheckLineRef", default, skip_serializing_if = "Option::is_none")]
    pub check_line_ref: Option<bool>,
    /// Whether the connection link a request names is checked against its permissions.
    #[serde(rename = "CheckConnectionLinkRef", default, skip_serializing_if = "Option::is_none")]
    pub check_connection_link_ref: Option<bool>,
}

/// What one participant may see of a timetable service.
///
/// Production Timetable and Estimated Timetable grant the same three permissions,
/// so the schema gives them one structure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionServicePermission {
    /// Who this entry applies to.
    #[serde(rename = "$value")]
    pub scope: PermissionScope,
    /// Which interaction patterns the participant may use.
    #[serde(rename = "GeneralCapabilities", default, skip_serializing_if = "Option::is_none")]
    pub general_capabilities: Option<GeneralPermissions>,
    /// Whose services the participant may see.
    #[serde(rename = "OperatorPermissions")]
    pub operator_permissions: OperatorPermissions,
    /// Which lines' services the participant may see.
    #[serde(rename = "LinePermissions")]
    pub line_permissions: LinePermissions,
    /// Which connection links' services the participant may see.
    #[serde(rename = "ConnectionLinkPermissions")]
    pub connection_link_permissions: ConnectionLinkPermissions,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl ConnectionServicePermission {
    /// A permission entry applying to every participant that has no entry of its own.
    pub fn for_all_participants() -> Self {
        Self {
            scope: PermissionScope::AllParticipants(Empty::new()),
            general_capabilities: None,
            operator_permissions: OperatorPermissions::allow_all(),
            line_permissions: LinePermissions::allow_all(),
            connection_link_permissions: ConnectionLinkPermissions::allow_all(),
            extensions: None,
        }
    }
}

/// Which connection links' services a participant may see.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionLinkPermissions {
    /// The entries, all of the same kind.
    #[serde(rename = "$value")]
    pub items: Vec<ConnectionLinkPermissionItem>,
}

impl ConnectionLinkPermissions {
    /// Permission covering every connection link the service knows about.
    pub fn allow_all() -> Self {
        Self {
            items: vec![ConnectionLinkPermissionItem::AllowAll(true)],
        }
    }

    /// Permission listed link by link.
    pub fn per_link(permissions: Vec<ConnectionLinkPermission>) -> Self {
        Self {
            items: permissions
                .into_iter()
                .map(ConnectionLinkPermissionItem::ConnectionLinkPermission)
                .collect(),
        }
    }
}

/// One entry of a [`ConnectionLinkPermissions`] list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionLinkPermissionItem {
    /// Whether every connection link known to the service is covered.
    AllowAll(bool),
    /// A decision about one named connection link.
    ConnectionLinkPermission(ConnectionLinkPermission),
}

/// Whether a participant may see one named connection link's services.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionLinkPermission {
    /// Whether access is granted or withheld.
    #[serde(rename = "Allow")]
    pub allow: bool,
    /// The connection link the decision is about.
    #[serde(rename = "ConnectionLinkRef")]
    pub connection_link_ref: ConnectionLinkRef,
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAPABILITY_RESPONSE: &str = r#"<SituationExchangeCapabilitiesResponse xmlns="http://www.siri.org.uk/siri" version="2.0">
	<ResponseTimestamp>2001-12-17T09:30:47-05:00</ResponseTimestamp>
	<RequestMessageRef>http://www.mytransportco.eu</RequestMessageRef>
	<Status>true</Status>
	<SituationExchangeServiceCapabilities>
		<GeneralInteraction>
			<Interaction>
				<RequestResponse>true</RequestResponse>
				<PublishSubscribe>true</PublishSubscribe>
			</Interaction>
			<Delivery>
				<DirectDelivery>true</DirectDelivery>
				<FetchedDelivery>false</FetchedDelivery>
			</Delivery>
			<MultipartDespatch>true</MultipartDespatch>
			<MultipleSubscriberFilter>true</MultipleSubscriberFilter>
			<HasConfirmDelivery>false</HasConfirmDelivery>
			<HasHeartbeat>false</HasHeartbeat>
		</GeneralInteraction>
		<TopicFiltering>
			<DefaultPreviewInterval>PT60M</DefaultPreviewInterval>
			<FilterByFacilityRef>false</FilterByFacilityRef>
			<FilterByLocationRef>true</FilterByLocationRef>
			<FilterByStopPointRef>true</FilterByStopPointRef>
		</TopicFiltering>
		<RequestPolicy>
			<NationalLanguage>en-uk</NationalLanguage>
			<WgsDecimalDegrees/>
		</RequestPolicy>
		<AccessControl>
			<RequestChecking>false</RequestChecking>
		</AccessControl>
	</SituationExchangeServiceCapabilities>
	<SituationExchangePermissions>
		<SituationExchangePermission>
			<AllParticipants/>
			<GeneralCapabilities>
				<RequestResponse>true</RequestResponse>
				<PublishSubscribe>true</PublishSubscribe>
			</GeneralCapabilities>
			<OperatorPermissions>
				<AllowAll>true</AllowAll>
			</OperatorPermissions>
			<LinePermissions>
				<AllowAll>true</AllowAll>
			</LinePermissions>
		</SituationExchangePermission>
		<SituationExchangePermission>
			<ParticipantRef>NADER</ParticipantRef>
			<OperatorPermissions>
				<AllowAll>true</AllowAll>
			</OperatorPermissions>
			<LinePermissions>
				<AllowAll>true</AllowAll>
			</LinePermissions>
		</SituationExchangePermission>
	</SituationExchangePermissions>
</SituationExchangeCapabilitiesResponse>"#;

    #[test]
    fn a_capability_response_is_read_as_a_document_of_its_own() {
        let response: SituationExchangeCapabilitiesResponse =
            crate::from_str(CAPABILITY_RESPONSE).unwrap();

        assert!(response.is_success());
        let capabilities = response
            .situation_exchange_service_capabilities
            .as_ref()
            .unwrap();
        let interaction = &capabilities.general_interaction.as_ref().unwrap().interaction;
        assert!(interaction.request_response && interaction.publish_subscribe);

        let filtering = capabilities.topic_filtering.as_ref().unwrap();
        assert_eq!(filtering.default_preview_interval.as_str(), "PT60M");
        assert_eq!(filtering.filter_by_facility_ref, Some(false));
        assert_eq!(filtering.filter_by_stop_point_ref, Some(true));
        assert_eq!(filtering.filter_by_keyword, None);

        let policy = capabilities.request_policy.as_ref().unwrap();
        assert_eq!(policy.national_language, ["en-uk"]);
        assert_eq!(
            policy.coordinate_format,
            CoordinateFormat::WgsDecimalDegrees(Empty::new())
        );
    }

    #[test]
    fn permissions_distinguish_the_default_entry_from_a_named_participant() {
        let response: SituationExchangeCapabilitiesResponse =
            crate::from_str(CAPABILITY_RESPONSE).unwrap();
        let permissions = response.situation_exchange_permissions.unwrap();
        let entries = permissions.situation_exchange_permission;

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].scope, PermissionScope::AllParticipants(Empty::new()));
        assert_eq!(
            entries[0].general_capabilities,
            Some(GeneralPermissions {
                request_response: true,
                publish_subscribe: true
            })
        );
        assert_eq!(
            entries[1].scope,
            PermissionScope::ParticipantRef(ParticipantRef::new("NADER"))
        );
        assert_eq!(entries[1].general_capabilities, None);
        assert_eq!(entries[1].line_permissions, LinePermissions::allow_all());
    }

    #[test]
    fn a_capability_response_survives_a_round_trip() {
        let response: SituationExchangeCapabilitiesResponse =
            crate::from_str(CAPABILITY_RESPONSE).unwrap();
        let written = crate::to_string(&response).unwrap();
        let reread: SituationExchangeCapabilitiesResponse = crate::from_str(&written).unwrap();
        assert_eq!(response, reread);
    }

    #[test]
    fn a_capabilities_request_names_one_element_per_service() {
        let timestamp = DateTime::parse_from_rfc3339("2001-12-17T09:30:47Z").unwrap();
        let request = CapabilitiesRequest::new(
            timestamp,
            "12345",
            vec![
                CapabilitiesRequestPayload::ProductionTimetableCapabilitiesRequest(
                    ServiceCapabilitiesRequest::new(timestamp),
                ),
                CapabilitiesRequestPayload::SituationExchangeCapabilitiesRequest(
                    ServiceCapabilitiesRequest::new(timestamp),
                ),
            ],
        );

        let xml = quick_xml::se::to_string_with_root("CapabilitiesRequest", &request).unwrap();
        assert!(xml.contains("<ProductionTimetableCapabilitiesRequest>"));
        assert!(xml.contains("<SituationExchangeCapabilitiesRequest>"));
        assert!(xml.contains("<RequestorRef>12345</RequestorRef>"));

        let reread: CapabilitiesRequest = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(reread, request);
    }

    #[test]
    fn a_capabilities_response_names_one_element_per_service_described() {
        let timestamp = DateTime::parse_from_rfc3339("2001-12-17T09:30:47Z").unwrap();
        let response = CapabilitiesResponse::new(
            timestamp,
            "KUBRICK",
            vec![
                EstimatedTimetableCapabilitiesResponse::new(
                    timestamp,
                    crate::et::EstimatedTimetableServiceCapabilities {
                        topic_filtering: Some(
                            crate::et::EstimatedTimetableTopicFiltering::by_operator_and_line(),
                        ),
                        request_policy: Some(CapabilityRequestPolicy::in_language("en-uk")),
                        ..Default::default()
                    },
                )
                .into(),
                VehicleMonitoringCapabilitiesResponse::new(
                    timestamp,
                    crate::vm::VehicleMonitoringServiceCapabilities::default(),
                )
                .into(),
            ],
        );

        let xml = quick_xml::se::to_string_with_root("CapabilitiesResponse", &response).unwrap();
        assert!(xml.contains("<EstimatedTimetableCapabilitiesResponse>"), "{xml}");
        assert!(xml.contains("<VehicleMonitoringCapabilitiesResponse>"), "{xml}");
        assert!(xml.contains("<FilterByOperatorRef>true</FilterByOperatorRef>"), "{xml}");
        assert!(xml.contains("<WgsDecimalDegrees/>"), "{xml}");

        let reread: CapabilitiesResponse = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(reread, response);
    }

    #[test]
    fn a_timetable_permission_grants_operators_lines_and_connection_links() {
        let permission = ConnectionServicePermission::for_all_participants();
        let xml =
            quick_xml::se::to_string_with_root("EstimatedTimetablePermission", &permission).unwrap();
        let operators = xml.find("<OperatorPermissions>").expect("operators are written");
        let lines = xml.find("<LinePermissions>").expect("lines are written");
        let links = xml
            .find("<ConnectionLinkPermissions>")
            .expect("connection links are written");
        assert!(operators < lines && lines < links, "{xml}");

        let reread: ConnectionServicePermission = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(reread, permission);
        assert_eq!(
            reread.connection_link_permissions,
            ConnectionLinkPermissions::allow_all()
        );
    }

    #[test]
    fn per_line_permissions_keep_the_directions_they_are_limited_to() {
        let permissions = LinePermissions::per_line(vec![LinePermission {
            allow: true,
            line_ref: LineRef::new("Line564"),
            direction_ref: vec![DirectionRef::new("NORTH")],
        }]);

        let xml = quick_xml::se::to_string_with_root("LinePermissions", &permissions).unwrap();
        assert!(xml.contains("<DirectionRef>NORTH</DirectionRef>"));
        assert!(!xml.contains("AllowAll"));

        let reread: LinePermissions = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(reread, permissions);
    }
}
