//! What a Stop Monitoring service can do, and to whom.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::StopMonitoringDetail;
use crate::framework::{
    CoordinateFormat, ErrorCondition, GeneralInteractionCapability, GeneralPermissions,
    LinePermissions, MonitoringCapabilityAccessControl, OperatorPermissions, PermissionScope,
    PermissionVersionRef, ServiceRequestError, StopMonitorPermissions, SubscriptionPolicyCapability,
    TransportDescription,
};
use crate::types::{Duration, Empty, EndpointAddress, Extensions, MessageRef, ParticipantRef};
use crate::xml::SiriRoot;

/// What the Stop Monitoring service offers, and to whom.
///
/// Appears both inside a [`CapabilitiesResponse`](crate::framework::CapabilitiesResponse)
/// and, when only this one service is of interest, as a document in its own right.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopMonitoringCapabilitiesResponse {
    /// Version of SIRI-SM the response conforms to.
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
        rename = "StopMonitoringServiceCapabilities",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub stop_monitoring_service_capabilities: Option<StopMonitoringServiceCapabilities>,
    /// What participants are allowed to see.
    #[serde(rename = "StopMonitoringPermissions", default, skip_serializing_if = "Option::is_none")]
    pub stop_monitoring_permissions: Option<StopMonitoringPermissions>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl StopMonitoringCapabilitiesResponse {
    /// A successful answer describing the given capabilities.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        capabilities: StopMonitoringServiceCapabilities,
    ) -> Self {
        Self {
            version: None,
            response_timestamp,
            request_message_ref: None,
            delegator_address: None,
            delegator_ref: None,
            status: Some(true),
            error_condition: None,
            stop_monitoring_service_capabilities: Some(capabilities),
            stop_monitoring_permissions: None,
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

impl SiriRoot for StopMonitoringCapabilitiesResponse {
    const ELEMENT_NAME: &'static str = "StopMonitoringCapabilitiesResponse";
}

/// What a Stop Monitoring service can do.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StopMonitoringServiceCapabilities {
    /// Interaction patterns and delivery mechanics common to all SIRI services.
    #[serde(rename = "GeneralInteraction", default, skip_serializing_if = "Option::is_none")]
    pub general_interaction: Option<GeneralInteractionCapability>,
    /// How messages are carried and whether they are compressed.
    #[serde(rename = "TransportDescription", default, skip_serializing_if = "Option::is_none")]
    pub transport_description: Option<TransportDescription>,
    /// Which criteria a requestor may narrow the visits by.
    #[serde(rename = "TopicFiltering", default, skip_serializing_if = "Option::is_none")]
    pub topic_filtering: Option<StopMonitoringTopicFiltering>,
    /// Languages, coordinate format and volume limits applied to requests.
    #[serde(rename = "RequestPolicy", default, skip_serializing_if = "Option::is_none")]
    pub request_policy: Option<StopMonitoringRequestPolicy>,
    /// What may be asked for when opening a subscription.
    #[serde(rename = "SubscriptionPolicy", default, skip_serializing_if = "Option::is_none")]
    pub subscription_policy: Option<SubscriptionPolicyCapability>,
    /// Whether and how requests are checked against per-participant permissions.
    #[serde(rename = "AccessControl", default, skip_serializing_if = "Option::is_none")]
    pub access_control: Option<MonitoringCapabilityAccessControl>,
    /// Optional content the responses may carry.
    #[serde(rename = "ResponseFeatures", default, skip_serializing_if = "Option::is_none")]
    pub response_features: Option<StopMonitoringResponseFeatures>,
}

/// Which criteria a requestor may narrow a stop's visits by.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopMonitoringTopicFiltering {
    /// The look-ahead window applied when a request does not name one.
    #[serde(rename = "DefaultPreviewInterval")]
    pub default_preview_interval: Duration,
    /// Whether that window may be made to start at a stated time.
    #[serde(rename = "ByStartTime", default, skip_serializing_if = "Option::is_none")]
    pub by_start_time: Option<bool>,
    /// Whether visits can be narrowed to one monitoring point.
    ///
    /// The schema fixes this to `true`: naming the monitoring point is how a
    /// stop-monitoring request states its topic at all.
    #[serde(rename = "FilterByMonitoringRef")]
    pub filter_by_monitoring_ref: bool,
    /// Whether visits can be narrowed to a line.
    #[serde(rename = "FilterByLineRef")]
    pub filter_by_line_ref: bool,
    /// Whether visits can be narrowed to a direction.
    #[serde(rename = "FilterByDirectionRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_direction_ref: Option<bool>,
    /// Whether visits can be narrowed to a destination.
    #[serde(rename = "FilterByDestination", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_destination: Option<bool>,
    /// Whether visits can be narrowed to arrivals or to departures.
    #[serde(rename = "FilterByVisitType", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_visit_type: Option<bool>,
}

/// Languages, coordinate format and volume limits a service applies to requests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopMonitoringRequestPolicy {
    /// Languages the service can return texts in, most preferred first.
    #[serde(rename = "NationalLanguage")]
    pub national_language: Vec<String>,
    /// Whether one text can be returned in several languages at once.
    #[serde(rename = "Translations", default, skip_serializing_if = "Option::is_none")]
    pub translations: Option<bool>,
    /// How positions are written in responses.
    #[serde(rename = "$value")]
    pub coordinate_format: CoordinateFormat,
    /// Whether the responses name entities by reference.
    #[serde(rename = "UseReferences", default, skip_serializing_if = "Option::is_none")]
    pub use_references: Option<bool>,
    /// Whether they name them by name.
    #[serde(rename = "UseNames", default, skip_serializing_if = "Option::is_none")]
    pub use_names: Option<bool>,
    /// Whether a request may ask for a particular level of detail.
    #[serde(rename = "HasDetailLevel", default, skip_serializing_if = "Option::is_none")]
    pub has_detail_level: Option<bool>,
    /// The level of detail applied when a request does not ask for one.
    #[serde(rename = "DefaultDetailLevel", default, skip_serializing_if = "Option::is_none")]
    pub default_detail_level: Option<StopMonitoringDetail>,
    /// Whether a request may cap how many visits come back.
    #[serde(rename = "HasMaximumVisits", default, skip_serializing_if = "Option::is_none")]
    pub has_maximum_visits: Option<bool>,
    /// Whether a request may reserve a minimum number of visits per line.
    #[serde(rename = "HasMinimumVisitsPerLine", default, skip_serializing_if = "Option::is_none")]
    pub has_minimum_visits_per_line: Option<bool>,
    /// Whether it may reserve them per via point instead.
    #[serde(rename = "HasMinimumVisitsPerVia", default, skip_serializing_if = "Option::is_none")]
    pub has_minimum_visits_per_via: Option<bool>,
    /// Whether a request may cap how many calls after this stop come back.
    #[serde(rename = "HasNumberOfOnwardsCalls", default, skip_serializing_if = "Option::is_none")]
    pub has_number_of_onwards_calls: Option<bool>,
    /// Whether it may cap how many calls before this stop come back.
    #[serde(rename = "HasNumberOfPreviousCalls", default, skip_serializing_if = "Option::is_none")]
    pub has_number_of_previous_calls: Option<bool>,
}

impl StopMonitoringRequestPolicy {
    /// A policy offering the given language and WGS 84 decimal degrees.
    pub fn in_language(national_language: impl Into<String>) -> Self {
        Self {
            national_language: vec![national_language.into()],
            translations: None,
            coordinate_format: CoordinateFormat::WgsDecimalDegrees(Empty::new()),
            use_references: None,
            use_names: None,
            has_detail_level: None,
            default_detail_level: None,
            has_maximum_visits: None,
            has_minimum_visits_per_line: None,
            has_minimum_visits_per_via: None,
            has_number_of_onwards_calls: None,
            has_number_of_previous_calls: None,
        }
    }
}

/// Optional content a Stop Monitoring service's responses may carry.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StopMonitoringResponseFeatures {
    /// Whether the responses carry notices about the lines calling at the stop.
    #[serde(rename = "HasLineNotices", default, skip_serializing_if = "Option::is_none")]
    pub has_line_notices: Option<bool>,
    /// Whether they carry the situations affecting the services.
    #[serde(rename = "HasSituations", default, skip_serializing_if = "Option::is_none")]
    pub has_situations: Option<bool>,
}

/// What participants are allowed to see of a Stop Monitoring service.
///
/// Also a document in its own right: the schema declares it globally so that a
/// producer can publish its permission set on its own.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StopMonitoringPermissions {
    /// The edition of the permission set these entries belong to.
    #[serde(rename = "PermissionVersionRef", default, skip_serializing_if = "Option::is_none")]
    pub permission_version_ref: Option<PermissionVersionRef>,
    /// One entry per participant, plus optionally one covering everybody else.
    #[serde(rename = "StopMonitoringPermission", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_monitoring_permission: Vec<StopMonitoringServicePermission>,
}

impl SiriRoot for StopMonitoringPermissions {
    const ELEMENT_NAME: &'static str = "StopMonitoringPermissions";
}

/// What one participant may see of a Stop Monitoring service.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopMonitoringServicePermission {
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
    /// Which monitoring points the participant may ask about.
    #[serde(rename = "StopMonitorPermissions")]
    pub stop_monitor_permissions: StopMonitorPermissions,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl StopMonitoringServicePermission {
    /// A permission entry applying to every participant that has no entry of its own.
    pub fn for_all_participants() -> Self {
        Self {
            scope: PermissionScope::AllParticipants(Empty::new()),
            general_capabilities: None,
            operator_permissions: OperatorPermissions::allow_all(),
            line_permissions: LinePermissions::allow_all(),
            stop_monitor_permissions: StopMonitorPermissions::allow_all(),
            extensions: None,
        }
    }
}
