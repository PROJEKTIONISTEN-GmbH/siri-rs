//! What an Estimated Timetable service can do, and to whom.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::framework::{
    CapabilityRequestPolicy, ConnectionCapabilityAccessControl, ConnectionServicePermission,
    ErrorCondition, GeneralInteractionCapability, PermissionVersionRef, ServiceRequestError,
    SubscriptionPolicyCapability, TransportDescription,
};
use crate::types::{Duration, EndpointAddress, Extensions, MessageRef, ParticipantRef};
use crate::xml::SiriRoot;

/// What the Estimated Timetable service offers, and to whom.
///
/// Appears both inside a [`CapabilitiesResponse`](crate::framework::CapabilitiesResponse)
/// and, when only this one service is of interest, as a document in its own right.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EstimatedTimetableCapabilitiesResponse {
    /// Version of SIRI-ET the response conforms to.
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
        rename = "EstimatedTimetableServiceCapabilities",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub estimated_timetable_service_capabilities: Option<EstimatedTimetableServiceCapabilities>,
    /// What participants are allowed to see.
    #[serde(
        rename = "EstimatedTimetablePermissions",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub estimated_timetable_permissions: Option<EstimatedTimetablePermissions>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl EstimatedTimetableCapabilitiesResponse {
    /// A successful answer describing the given capabilities.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        capabilities: EstimatedTimetableServiceCapabilities,
    ) -> Self {
        Self {
            version: None,
            response_timestamp,
            request_message_ref: None,
            delegator_address: None,
            delegator_ref: None,
            status: Some(true),
            error_condition: None,
            estimated_timetable_service_capabilities: Some(capabilities),
            estimated_timetable_permissions: None,
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

impl SiriRoot for EstimatedTimetableCapabilitiesResponse {
    const ELEMENT_NAME: &'static str = "EstimatedTimetableCapabilitiesResponse";
}

/// What an Estimated Timetable service can do.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EstimatedTimetableServiceCapabilities {
    /// Interaction patterns and delivery mechanics common to all SIRI services.
    #[serde(rename = "GeneralInteraction", default, skip_serializing_if = "Option::is_none")]
    pub general_interaction: Option<GeneralInteractionCapability>,
    /// How messages are carried and whether they are compressed.
    #[serde(rename = "TransportDescription", default, skip_serializing_if = "Option::is_none")]
    pub transport_description: Option<TransportDescription>,
    /// Which criteria a requestor may narrow the timetable by.
    #[serde(rename = "TopicFiltering", default, skip_serializing_if = "Option::is_none")]
    pub topic_filtering: Option<EstimatedTimetableTopicFiltering>,
    /// Languages and coordinate format applied to requests.
    #[serde(rename = "RequestPolicy", default, skip_serializing_if = "Option::is_none")]
    pub request_policy: Option<CapabilityRequestPolicy>,
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

/// Which criteria a requestor may narrow a real-time timetable by.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EstimatedTimetableTopicFiltering {
    /// The look-ahead window applied when a request does not name one.
    #[serde(rename = "DefaultPreviewInterval", default, skip_serializing_if = "Option::is_none")]
    pub default_preview_interval: Option<Duration>,
    /// Whether journeys can be narrowed to an operator.
    #[serde(rename = "FilterByOperatorRef")]
    pub filter_by_operator_ref: bool,
    /// Whether journeys can be narrowed to a line.
    #[serde(rename = "FilterByLineRef")]
    pub filter_by_line_ref: bool,
    /// Whether journeys can be narrowed to a mode of transport.
    #[serde(rename = "FilterByVehicleMode", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_vehicle_mode: Option<bool>,
    /// Whether journeys can be narrowed to a commercial category.
    #[serde(rename = "FilterByProductCategoryRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_product_category_ref: Option<bool>,
    /// Whether journeys can be narrowed to a scheduled stop point.
    #[serde(rename = "FilterByStopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_stop_point_ref: Option<bool>,
    /// Whether journeys can be narrowed to an edition of the timetable.
    #[serde(rename = "FilterByVersionRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_version_ref: Option<bool>,
}

impl EstimatedTimetableTopicFiltering {
    /// A service that filters by operator and by line and says nothing further.
    pub fn by_operator_and_line() -> Self {
        Self {
            default_preview_interval: None,
            filter_by_operator_ref: true,
            filter_by_line_ref: true,
            filter_by_vehicle_mode: None,
            filter_by_product_category_ref: None,
            filter_by_stop_point_ref: None,
            filter_by_version_ref: None,
        }
    }
}

/// What participants are allowed to see of an Estimated Timetable service.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EstimatedTimetablePermissions {
    /// The edition of the permission set these entries belong to.
    #[serde(rename = "PermissionVersionRef", default, skip_serializing_if = "Option::is_none")]
    pub permission_version_ref: Option<PermissionVersionRef>,
    /// One entry per participant, plus optionally one covering everybody else.
    #[serde(
        rename = "EstimatedTimetablePermission",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub estimated_timetable_permission: Vec<ConnectionServicePermission>,
}
