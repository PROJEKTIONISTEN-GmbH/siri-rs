//! What a Control Action service can do, and to whom.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::framework::{
    CoordinateFormat, ErrorCondition, GeneralInteractionCapability, GeneralPermissions,
    LinePermissions, MonitoringCapabilityAccessControl, OperatorPermissions, PermissionScope,
    PermissionVersionRef, ServiceRequestError, SubscriptionPolicyCapability, TransportDescription,
};
use crate::types::{Duration, Empty, EndpointAddress, Extensions, MessageRef, ParticipantRef};
use crate::xml::SiriRoot;

/// What the Control Action service offers, and to whom.
///
/// Appears both inside a [`CapabilitiesResponse`](crate::framework::CapabilitiesResponse)
/// and, when only this one service is of interest, as a document in its own right.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlActionCapabilitiesResponse {
    /// Version of SIRI-CA the response conforms to.
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
        rename = "ControlActionServiceCapabilities",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub control_action_service_capabilities: Option<ControlActionServiceCapabilities>,
    /// What participants are allowed to see.
    #[serde(rename = "ControlActionPermissions", default, skip_serializing_if = "Option::is_none")]
    pub control_action_permissions: Option<ControlActionPermissions>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl ControlActionCapabilitiesResponse {
    /// A successful answer describing the given capabilities.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        capabilities: ControlActionServiceCapabilities,
    ) -> Self {
        Self {
            version: None,
            response_timestamp,
            request_message_ref: None,
            delegator_address: None,
            delegator_ref: None,
            status: Some(true),
            error_condition: None,
            control_action_service_capabilities: Some(capabilities),
            control_action_permissions: None,
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

impl SiriRoot for ControlActionCapabilitiesResponse {
    const ELEMENT_NAME: &'static str = "ControlActionCapabilitiesResponse";
}

/// What a Control Action service can do.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ControlActionServiceCapabilities {
    /// Interaction patterns and delivery mechanics common to all SIRI services.
    #[serde(rename = "GeneralInteraction", default, skip_serializing_if = "Option::is_none")]
    pub general_interaction: Option<GeneralInteractionCapability>,
    /// How messages are carried and whether they are compressed.
    #[serde(rename = "TransportDescription", default, skip_serializing_if = "Option::is_none")]
    pub transport_description: Option<TransportDescription>,
    /// Which criteria a requestor may narrow the actions by.
    #[serde(rename = "TopicFiltering", default, skip_serializing_if = "Option::is_none")]
    pub topic_filtering: Option<ControlActionTopicFiltering>,
    /// Languages, coordinate format and volume limits applied to requests.
    #[serde(rename = "RequestPolicy", default, skip_serializing_if = "Option::is_none")]
    pub request_policy: Option<ControlActionRequestPolicy>,
    /// What may be asked for when opening a subscription.
    #[serde(rename = "SubscriptionPolicy", default, skip_serializing_if = "Option::is_none")]
    pub subscription_policy: Option<SubscriptionPolicyCapability>,
    /// Whether and how requests are checked against per-participant permissions.
    #[serde(rename = "AccessControl", default, skip_serializing_if = "Option::is_none")]
    pub access_control: Option<MonitoringCapabilityAccessControl>,
    /// Optional content the responses may carry.
    #[serde(rename = "ResponseFeatures", default, skip_serializing_if = "Option::is_none")]
    pub response_features: Option<ControlActionResponseFeatures>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// Which criteria a requestor may narrow a producer's control actions by.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlActionTopicFiltering {
    /// The look-ahead window applied when a request does not name one.
    #[serde(rename = "DefaultPreviewInterval", default, skip_serializing_if = "Option::is_none")]
    pub default_preview_interval: Option<Duration>,
    /// Whether the look-ahead window may start at a stated time.
    #[serde(rename = "ByStartTime", default, skip_serializing_if = "Option::is_none")]
    pub by_start_time: Option<bool>,
    /// Whether actions can be narrowed to one mode of transport.
    #[serde(rename = "FilterByMode", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_mode: Option<bool>,
    /// Whether they can be narrowed to one network.
    #[serde(rename = "FilterByNetworkRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_network_ref: Option<bool>,
    /// Whether they can be narrowed to one line.
    #[serde(rename = "FilterByLineRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_line_ref: Option<bool>,
}

/// Languages, coordinate format and volume limits a service applies to requests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlActionRequestPolicy {
    /// Languages the service can return texts in, most preferred first.
    #[serde(rename = "NationalLanguage")]
    pub national_language: Vec<String>,
    /// Whether one text can be returned in several languages at once.
    #[serde(rename = "Translations", default, skip_serializing_if = "Option::is_none")]
    pub translations: Option<bool>,
    /// How positions are written in responses.
    #[serde(rename = "$value")]
    pub coordinate_format: CoordinateFormat,
    /// Whether a request may cap how many actions come back.
    #[serde(rename = "HasMaximumControlActions", default, skip_serializing_if = "Option::is_none")]
    pub has_maximum_control_actions: Option<bool>,
}

impl ControlActionRequestPolicy {
    /// A policy offering the given language and WGS 84 decimal degrees.
    pub fn in_language(national_language: impl Into<String>) -> Self {
        Self {
            national_language: vec![national_language.into()],
            translations: None,
            coordinate_format: CoordinateFormat::WgsDecimalDegrees(Empty::new()),
            has_maximum_control_actions: None,
        }
    }
}

/// Optional content a Control Action service's responses may carry.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlActionResponseFeatures {
    /// Whether the responses name the situations the actions belong to.
    #[serde(rename = "HasSituations", default, skip_serializing_if = "Option::is_none")]
    pub has_situations: Option<bool>,
    /// Whether they carry the messages exchanged with drivers.
    #[serde(rename = "HasDriverMessages", default, skip_serializing_if = "Option::is_none")]
    pub has_driver_messages: Option<bool>,
    /// Whether they carry vehicles detected by trackside equipment.
    #[serde(rename = "HasVehicleDetectings", default, skip_serializing_if = "Option::is_none")]
    pub has_vehicle_detectings: Option<bool>,
}

/// What participants are allowed to see of a Control Action service.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ControlActionPermissions {
    /// The edition of the permission set these entries belong to.
    #[serde(rename = "PermissionVersionRef", default, skip_serializing_if = "Option::is_none")]
    pub permission_version_ref: Option<PermissionVersionRef>,
    /// One entry per participant, plus optionally one covering everybody else.
    #[serde(rename = "ControlActionPermission", default, skip_serializing_if = "Vec::is_empty")]
    pub control_action_permission: Vec<ControlActionServicePermission>,
}

/// What one participant may see of a Control Action service.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlActionServicePermission {
    /// Who this entry applies to.
    #[serde(rename = "$value")]
    pub scope: PermissionScope,
    /// Which interaction patterns the participant may use.
    #[serde(rename = "GeneralCapabilities", default, skip_serializing_if = "Option::is_none")]
    pub general_capabilities: Option<GeneralPermissions>,
    /// Whose actions the participant may see.
    #[serde(rename = "OperatorPermissions")]
    pub operator_permissions: OperatorPermissions,
    /// Which lines' actions the participant may see.
    #[serde(rename = "LinePermissions")]
    pub line_permissions: LinePermissions,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl ControlActionServicePermission {
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
