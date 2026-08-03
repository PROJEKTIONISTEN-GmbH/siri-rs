//! What a Stop Timetable service can do, and to whom.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::framework::{
    CoordinateFormat, ErrorCondition, GeneralInteractionCapability, GeneralPermissions,
    LinePermissions, MonitoringCapabilityAccessControl, OperatorPermissions, PermissionScope,
    PermissionVersionRef, ServiceRequestError, StopMonitorPermissions, TransportDescription,
};
use crate::types::{Empty, EndpointAddress, Extensions, MessageRef, ParticipantRef};
use crate::xml::SiriRoot;

/// What the Stop Timetable service offers, and to whom.
///
/// Appears both inside a [`CapabilitiesResponse`](crate::framework::CapabilitiesResponse)
/// and, when only this one service is of interest, as a document in its own right.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopTimetableCapabilitiesResponse {
    /// Version of SIRI-ST the response conforms to.
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
        rename = "StopTimetableServiceCapabilities",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub stop_timetable_service_capabilities: Option<StopTimetableServiceCapabilities>,
    /// What participants are allowed to see.
    #[serde(rename = "StopTimetablePermissions", default, skip_serializing_if = "Option::is_none")]
    pub stop_timetable_permissions: Option<StopTimetablePermissions>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl StopTimetableCapabilitiesResponse {
    /// A successful answer describing the given capabilities.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        capabilities: StopTimetableServiceCapabilities,
    ) -> Self {
        Self {
            version: None,
            response_timestamp,
            request_message_ref: None,
            delegator_address: None,
            delegator_ref: None,
            status: Some(true),
            error_condition: None,
            stop_timetable_service_capabilities: Some(capabilities),
            stop_timetable_permissions: None,
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

impl SiriRoot for StopTimetableCapabilitiesResponse {
    const ELEMENT_NAME: &'static str = "StopTimetableCapabilitiesResponse";
}

/// What a Stop Timetable service can do.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StopTimetableServiceCapabilities {
    /// Interaction patterns and delivery mechanics common to all SIRI services.
    #[serde(rename = "GeneralInteraction", default, skip_serializing_if = "Option::is_none")]
    pub general_interaction: Option<GeneralInteractionCapability>,
    /// How messages are carried and whether they are compressed.
    #[serde(rename = "TransportDescription", default, skip_serializing_if = "Option::is_none")]
    pub transport_description: Option<TransportDescription>,
    /// Which criteria a requestor may narrow the visits by.
    #[serde(rename = "TopicFiltering", default, skip_serializing_if = "Option::is_none")]
    pub topic_filtering: Option<StopTimetableTopicFiltering>,
    /// Languages and coordinate format applied to requests.
    #[serde(rename = "RequestPolicy", default, skip_serializing_if = "Option::is_none")]
    pub request_policy: Option<StopTimetableRequestPolicy>,
    /// Whether and how requests are checked against per-participant permissions.
    #[serde(rename = "AccessControl", default, skip_serializing_if = "Option::is_none")]
    pub access_control: Option<MonitoringCapabilityAccessControl>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// Which criteria a requestor may narrow a stop's timetable by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StopTimetableTopicFiltering {
    /// Whether visits can be narrowed to one monitoring point.
    ///
    /// The schema fixes this to `true`: naming the monitoring point is how a
    /// stop-timetable request states its topic at all.
    #[serde(rename = "FilterByMonitoringRef")]
    pub filter_by_monitoring_ref: bool,
    /// Whether visits can be narrowed to a line.
    #[serde(rename = "FilterByLineRef")]
    pub filter_by_line_ref: bool,
    /// Whether visits can be narrowed to a direction.
    #[serde(rename = "FilterByDirectionRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_direction_ref: Option<bool>,
}

/// Languages and coordinate format a service applies to requests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopTimetableRequestPolicy {
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
}

impl StopTimetableRequestPolicy {
    /// A policy offering the given language and WGS 84 decimal degrees.
    pub fn in_language(national_language: impl Into<String>) -> Self {
        Self {
            national_language: vec![national_language.into()],
            translations: None,
            coordinate_format: CoordinateFormat::WgsDecimalDegrees(Empty::new()),
            use_references: None,
            use_names: None,
        }
    }
}

/// What participants are allowed to see of a Stop Timetable service.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StopTimetablePermissions {
    /// The edition of the permission set these entries belong to.
    #[serde(rename = "PermissionVersionRef", default, skip_serializing_if = "Option::is_none")]
    pub permission_version_ref: Option<PermissionVersionRef>,
    /// One entry per participant, plus optionally one covering everybody else.
    #[serde(rename = "StopTimetablePermission", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_timetable_permission: Vec<StopTimetableServicePermission>,
}

/// What one participant may see of a Stop Timetable service.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopTimetableServicePermission {
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

impl StopTimetableServicePermission {
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
