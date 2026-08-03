//! What a Connection Monitoring service can do, and to whom.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::framework::{
    ConnectionCapabilityAccessControl, ConnectionServicePermission, CoordinateFormat,
    ErrorCondition, GeneralInteractionCapability, ServiceRequestError,
    SubscriptionPolicyCapability, TransportDescription,
};
use crate::types::{Duration, Empty, EndpointAddress, Extensions, MessageRef, ParticipantRef};
use crate::xml::SiriRoot;

/// What the Connection Monitoring service offers, and to whom.
///
/// Appears both inside a [`CapabilitiesResponse`](crate::framework::CapabilitiesResponse)
/// and, when only this one service is of interest, as a document in its own right.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionMonitoringCapabilitiesResponse {
    /// Version of SIRI-CM the response conforms to.
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
        rename = "ConnectionMonitoringServiceCapabilities",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub connection_monitoring_service_capabilities: Option<ConnectionMonitoringServiceCapabilities>,
    /// What participants are allowed to see.
    #[serde(
        rename = "ConnectionMonitoringPermissions",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub connection_monitoring_permissions: Option<ConnectionMonitoringPermissions>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl ConnectionMonitoringCapabilitiesResponse {
    /// A successful answer describing the given capabilities.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        capabilities: ConnectionMonitoringServiceCapabilities,
    ) -> Self {
        Self {
            version: None,
            response_timestamp,
            request_message_ref: None,
            delegator_address: None,
            delegator_ref: None,
            status: Some(true),
            error_condition: None,
            connection_monitoring_service_capabilities: Some(capabilities),
            connection_monitoring_permissions: None,
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

impl SiriRoot for ConnectionMonitoringCapabilitiesResponse {
    const ELEMENT_NAME: &'static str = "ConnectionMonitoringCapabilitiesResponse";
}

/// What a Connection Monitoring service can do.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ConnectionMonitoringServiceCapabilities {
    /// Interaction patterns and delivery mechanics common to all SIRI services.
    #[serde(rename = "GeneralInteraction", default, skip_serializing_if = "Option::is_none")]
    pub general_interaction: Option<GeneralInteractionCapability>,
    /// How messages are carried and whether they are compressed.
    #[serde(rename = "TransportDescription", default, skip_serializing_if = "Option::is_none")]
    pub transport_description: Option<TransportDescription>,
    /// Which criteria a requestor may narrow the connections by.
    #[serde(rename = "TopicFiltering", default, skip_serializing_if = "Option::is_none")]
    pub topic_filtering: Option<ConnectionMonitoringTopicFiltering>,
    /// Languages, coordinate format and scope applied to requests.
    #[serde(rename = "RequestPolicy", default, skip_serializing_if = "Option::is_none")]
    pub request_policy: Option<ConnectionMonitoringRequestPolicy>,
    /// What may be asked for when opening a subscription.
    #[serde(rename = "SubscriptionPolicy", default, skip_serializing_if = "Option::is_none")]
    pub subscription_policy: Option<SubscriptionPolicyCapability>,
    /// Whether and how requests are checked against per-participant permissions.
    #[serde(rename = "AccessControl", default, skip_serializing_if = "Option::is_none")]
    pub access_control: Option<ConnectionCapabilityAccessControl>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// Which criteria a requestor may narrow a link's connections by.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ConnectionMonitoringTopicFiltering {
    /// The look-ahead window applied when a request does not name one.
    #[serde(rename = "DefaultPreviewInterval", default, skip_serializing_if = "Option::is_none")]
    pub default_preview_interval: Option<Duration>,
    /// Whether connections can be narrowed to one connection link.
    #[serde(rename = "FilterByConnectionLinkRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_connection_link_ref: Option<bool>,
    /// Whether connections can be narrowed to named feeder journeys.
    ///
    /// The schema fixes this to `true`: naming the journeys is one of the two ways
    /// a connection-monitoring request states its topic.
    #[serde(rename = "FilterByJourney", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_journey: Option<bool>,
    /// Whether they can be narrowed to a window of arrival times instead.
    #[serde(rename = "FilterByTime", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_time: Option<bool>,
}

/// Languages, coordinate format and scope a service applies to requests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionMonitoringRequestPolicy {
    /// Languages the service can return texts in, most preferred first.
    #[serde(rename = "NationalLanguage")]
    pub national_language: Vec<String>,
    /// Whether one text can be returned in several languages at once.
    #[serde(rename = "Translations", default, skip_serializing_if = "Option::is_none")]
    pub translations: Option<bool>,
    /// How positions are written in responses.
    #[serde(rename = "$value")]
    pub coordinate_format: CoordinateFormat,
    /// Whether the service reports only feeders run by other operators.
    #[serde(rename = "ForeignJourneysOnly", default, skip_serializing_if = "Option::is_none")]
    pub foreign_journeys_only: Option<bool>,
}

impl ConnectionMonitoringRequestPolicy {
    /// A policy offering the given language and WGS 84 decimal degrees.
    pub fn in_language(national_language: impl Into<String>) -> Self {
        Self {
            national_language: vec![national_language.into()],
            translations: None,
            coordinate_format: CoordinateFormat::WgsDecimalDegrees(Empty::new()),
            foreign_journeys_only: None,
        }
    }
}

/// What participants are allowed to see of a Connection Monitoring service.
///
/// Unlike the other services' permission sets this one carries no version
/// reference; the schema does not build it on the common permissions structure.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ConnectionMonitoringPermissions {
    /// One entry per participant, plus optionally one covering everybody else.
    #[serde(
        rename = "ConnectionMonitoringPermission",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub connection_monitoring_permission: Vec<ConnectionServicePermission>,
}
