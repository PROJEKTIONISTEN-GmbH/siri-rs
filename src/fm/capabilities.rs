//! What a Facility Monitoring service can do, and to whom.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::framework::{
    CoordinateFormat, ErrorCondition, GeneralInteractionCapability, GeneralPermissions,
    LinePermissions, OperatorPermissions, PermissionScope, PermissionVersionRef,
    ServiceRequestError, SubscriptionPolicyCapability, TransportDescription,
};
use crate::types::{Duration, Empty, EndpointAddress, Extensions, MessageRef, ParticipantRef};
use crate::xml::SiriRoot;

/// What the Facility Monitoring service offers, and to whom.
///
/// Appears both inside a [`CapabilitiesResponse`](crate::framework::CapabilitiesResponse)
/// and, when only this one service is of interest, as a document in its own right.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FacilityMonitoringCapabilitiesResponse {
    /// Version of SIRI-FM the response conforms to.
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
        rename = "FacilityMonitoringServiceCapabilities",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub facility_monitoring_service_capabilities: Option<FacilityMonitoringServiceCapabilities>,
    /// What participants are allowed to see.
    #[serde(
        rename = "FacilityMonitoringPermissions",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub facility_monitoring_permissions: Option<FacilityMonitoringPermissions>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl FacilityMonitoringCapabilitiesResponse {
    /// A successful answer describing the given capabilities.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        capabilities: FacilityMonitoringServiceCapabilities,
    ) -> Self {
        Self {
            version: None,
            response_timestamp,
            request_message_ref: None,
            delegator_address: None,
            delegator_ref: None,
            status: Some(true),
            error_condition: None,
            facility_monitoring_service_capabilities: Some(capabilities),
            facility_monitoring_permissions: None,
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

impl SiriRoot for FacilityMonitoringCapabilitiesResponse {
    const ELEMENT_NAME: &'static str = "FacilityMonitoringCapabilitiesResponse";
}

/// What a Facility Monitoring service can do.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FacilityMonitoringServiceCapabilities {
    /// Interaction patterns and delivery mechanics common to all SIRI services.
    #[serde(rename = "GeneralInteraction", default, skip_serializing_if = "Option::is_none")]
    pub general_interaction: Option<GeneralInteractionCapability>,
    /// How messages are carried and whether they are compressed.
    #[serde(rename = "TransportDescription", default, skip_serializing_if = "Option::is_none")]
    pub transport_description: Option<TransportDescription>,
    /// Which criteria a requestor may narrow the facilities by.
    #[serde(rename = "TopicFiltering", default, skip_serializing_if = "Option::is_none")]
    pub topic_filtering: Option<FacilityMonitoringTopicFiltering>,
    /// Languages, coordinate format and volume limits applied to requests.
    #[serde(rename = "RequestPolicy", default, skip_serializing_if = "Option::is_none")]
    pub request_policy: Option<FacilityMonitoringRequestPolicy>,
    /// What may be asked for when opening a subscription.
    #[serde(rename = "SubscriptionPolicy", default, skip_serializing_if = "Option::is_none")]
    pub subscription_policy: Option<SubscriptionPolicyCapability>,
    /// Whether and how requests are checked against per-participant permissions.
    #[serde(rename = "AccessControl", default, skip_serializing_if = "Option::is_none")]
    pub access_control: Option<FacilityMonitoringAccessControl>,
    /// Optional content the responses may carry.
    #[serde(rename = "ResponseFeatures", default, skip_serializing_if = "Option::is_none")]
    pub response_features: Option<FacilityMonitoringResponseFeatures>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// Which criteria a requestor may narrow a producer's facilities by.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FacilityMonitoringTopicFiltering {
    /// The look-ahead window applied when a request does not name one.
    #[serde(rename = "DefaultPreviewInterval")]
    pub default_preview_interval: Duration,
    /// Whether facilities can be narrowed to named facilities.
    #[serde(rename = "FilterByFacilityRef")]
    pub filter_by_facility_ref: bool,
    /// Whether they can be narrowed to a place or a service.
    ///
    /// The schema fixes this to `true`: naming where the facility is, is how a
    /// facility-monitoring request states its topic at all.
    #[serde(rename = "FilterByLocationRef")]
    pub filter_by_location_ref: bool,
    /// Whether they can be narrowed to one vehicle.
    #[serde(rename = "FilterByVehicleRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_vehicle_ref: Option<bool>,
    /// Whether they can be narrowed to one line.
    #[serde(rename = "FilterByLineRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_line_ref: Option<bool>,
    /// Whether they can be narrowed to one stop.
    #[serde(rename = "FilterByStopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_stop_point_ref: Option<bool>,
    /// Whether they can be narrowed to one journey.
    #[serde(rename = "FilterByVehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_vehicle_journey_ref: Option<bool>,
    /// Whether they can be narrowed to one connection link.
    #[serde(rename = "FilterByConnectionLinkRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_connection_link_ref: Option<bool>,
    /// Whether they can be narrowed to one interchange.
    #[serde(rename = "FilterByInterchangeRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_interchange_ref: Option<bool>,
    /// Whether they can be narrowed to the facilities bearing on a passenger need.
    #[serde(rename = "FilterBySpecificNeed", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_specific_need: Option<bool>,
}

/// Languages, coordinate format and volume limits a service applies to requests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FacilityMonitoringRequestPolicy {
    /// Languages the service can return texts in, most preferred first.
    #[serde(rename = "NationalLanguage")]
    pub national_language: Vec<String>,
    /// Whether one text can be returned in several languages at once.
    #[serde(rename = "Translations", default, skip_serializing_if = "Option::is_none")]
    pub translations: Option<bool>,
    /// How positions are written in responses.
    #[serde(rename = "$value")]
    pub coordinate_format: CoordinateFormat,
    /// Whether a request may cap how many facility conditions come back.
    #[serde(rename = "HasMaximumFacilityStatus", default, skip_serializing_if = "Option::is_none")]
    pub has_maximum_facility_status: Option<bool>,
}

impl FacilityMonitoringRequestPolicy {
    /// A policy offering the given language and WGS 84 decimal degrees.
    pub fn in_language(national_language: impl Into<String>) -> Self {
        Self {
            national_language: vec![national_language.into()],
            translations: None,
            coordinate_format: CoordinateFormat::WgsDecimalDegrees(Empty::new()),
            has_maximum_facility_status: None,
        }
    }
}

/// Whether and how a Facility Monitoring service checks requests against the
/// permissions of the participant making them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FacilityMonitoringAccessControl {
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

/// Optional content a Facility Monitoring service's responses may carry.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FacilityMonitoringResponseFeatures {
    /// Whether the responses say what is being done about a failure.
    #[serde(rename = "HasRemedy", default, skip_serializing_if = "Option::is_none")]
    pub has_remedy: Option<bool>,
    /// Whether they say where the facility is.
    #[serde(rename = "HasFacilityLocation", default, skip_serializing_if = "Option::is_none")]
    pub has_facility_location: Option<bool>,
}

/// What participants are allowed to see of a Facility Monitoring service.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FacilityMonitoringPermissions {
    /// The edition of the permission set these entries belong to.
    #[serde(rename = "PermissionVersionRef", default, skip_serializing_if = "Option::is_none")]
    pub permission_version_ref: Option<PermissionVersionRef>,
    /// One entry per participant, plus optionally one covering everybody else.
    #[serde(
        rename = "FacilityMonitoringPermission",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub facility_monitoring_permission: Vec<FacilityMonitoringServicePermission>,
}

/// What one participant may see of a Facility Monitoring service.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FacilityMonitoringServicePermission {
    /// Who this entry applies to.
    #[serde(rename = "$value")]
    pub scope: PermissionScope,
    /// Which interaction patterns the participant may use.
    #[serde(rename = "GeneralCapabilities", default, skip_serializing_if = "Option::is_none")]
    pub general_capabilities: Option<GeneralPermissions>,
    /// Whose facilities the participant may see.
    #[serde(rename = "OperatorPermissions")]
    pub operator_permissions: OperatorPermissions,
    /// Which lines' facilities the participant may see.
    #[serde(rename = "LinePermissions")]
    pub line_permissions: LinePermissions,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl FacilityMonitoringServicePermission {
    /// A permission entry applying to every participant that has no entry of its own.
    pub fn for_all_participants() -> Self {
        Self {
            scope: PermissionScope::AllParticipants(Empty::new()),
            general_capabilities: None,
            operator_permissions: OperatorPermissions::allow_all(),
            line_permissions: LinePermissions::allow_all(),
            extensions: None,
        }
    }
}
