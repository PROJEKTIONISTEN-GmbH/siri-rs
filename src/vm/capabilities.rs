//! What a Vehicle Monitoring service can do, and to whom.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::VehicleMonitoringDetail;
use crate::framework::{
    CoordinateFormat, ErrorCondition, GeneralInteractionCapability, GeneralPermissions,
    LinePermissions, OperatorPermissions, PermissionScope, PermissionVersionRef,
    ServiceRequestError, SubscriptionPolicyCapability, TransportDescription,
};
use crate::types::{Duration, EndpointAddress, Extensions, MessageRef, ParticipantRef};
use crate::xml::SiriRoot;

use super::VehicleMonitoringRef;

/// What the Vehicle Monitoring service offers, and to whom.
///
/// Appears both inside a [`CapabilitiesResponse`](crate::framework::CapabilitiesResponse)
/// and, when only this one service is of interest, as a document in its own right.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleMonitoringCapabilitiesResponse {
    /// Version of SIRI-VM the response conforms to.
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
        rename = "VehicleMonitoringServiceCapabilities",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub vehicle_monitoring_service_capabilities: Option<VehicleMonitoringServiceCapabilities>,
    /// What participants are allowed to see.
    #[serde(
        rename = "VehicleMonitoringPermissions",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub vehicle_monitoring_permissions: Option<VehicleMonitoringPermissions>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl VehicleMonitoringCapabilitiesResponse {
    /// A successful answer describing the given capabilities.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        capabilities: VehicleMonitoringServiceCapabilities,
    ) -> Self {
        Self {
            version: None,
            response_timestamp,
            request_message_ref: None,
            delegator_address: None,
            delegator_ref: None,
            status: Some(true),
            error_condition: None,
            vehicle_monitoring_service_capabilities: Some(capabilities),
            vehicle_monitoring_permissions: None,
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

impl SiriRoot for VehicleMonitoringCapabilitiesResponse {
    const ELEMENT_NAME: &'static str = "VehicleMonitoringCapabilitiesResponse";
}

/// What a Vehicle Monitoring service can do.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VehicleMonitoringServiceCapabilities {
    /// Interaction patterns and delivery mechanics common to all SIRI services.
    #[serde(rename = "GeneralInteraction", default, skip_serializing_if = "Option::is_none")]
    pub general_interaction: Option<GeneralInteractionCapability>,
    /// How messages are carried and whether they are compressed.
    #[serde(rename = "TransportDescription", default, skip_serializing_if = "Option::is_none")]
    pub transport_description: Option<TransportDescription>,
    /// Which criteria a requestor may narrow the vehicles by.
    #[serde(rename = "TopicFiltering", default, skip_serializing_if = "Option::is_none")]
    pub topic_filtering: Option<VehicleMonitoringTopicFiltering>,
    /// Languages, coordinate format and volume limits applied to requests.
    #[serde(rename = "RequestPolicy", default, skip_serializing_if = "Option::is_none")]
    pub request_policy: Option<VehicleMonitoringRequestPolicy>,
    /// What may be asked for when opening a subscription.
    #[serde(rename = "SubscriptionPolicy", default, skip_serializing_if = "Option::is_none")]
    pub subscription_policy: Option<SubscriptionPolicyCapability>,
    /// Whether and how requests are checked against per-participant permissions.
    #[serde(rename = "AccessControl", default, skip_serializing_if = "Option::is_none")]
    pub access_control: Option<VehicleMonitoringAccessControl>,
    /// Optional content the responses may carry.
    #[serde(rename = "ResponseFeatures", default, skip_serializing_if = "Option::is_none")]
    pub response_features: Option<VehicleMonitoringResponseFeatures>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// Which criteria a requestor may narrow a set of tracked vehicles by.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleMonitoringTopicFiltering {
    /// The look-ahead window applied when a request does not name one.
    #[serde(rename = "DefaultPreviewInterval")]
    pub default_preview_interval: Duration,
    /// Whether vehicles can be narrowed to one of the producer's monitoring services.
    ///
    /// The schema fixes this to `true`: naming the monitoring service is how a
    /// vehicle-monitoring request states its topic at all.
    #[serde(rename = "FilterByVehicleMonitoringRef")]
    pub filter_by_vehicle_monitoring_ref: bool,
    /// Whether vehicles can be narrowed to one vehicle.
    #[serde(rename = "FilterByVehicleRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_vehicle_ref: Option<bool>,
    /// Whether vehicles can be narrowed to a line.
    #[serde(rename = "FilterByLineRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_line_ref: Option<bool>,
    /// Whether vehicles can be narrowed to a direction.
    #[serde(rename = "FilterByDirectionRef", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_direction_ref: Option<bool>,
}

/// Languages, coordinate format and volume limits a service applies to requests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleMonitoringRequestPolicy {
    /// Languages the service can return texts in, most preferred first.
    #[serde(rename = "NationalLanguage")]
    pub national_language: Vec<String>,
    /// Whether one text can be returned in several languages at once.
    #[serde(rename = "Translations", default, skip_serializing_if = "Option::is_none")]
    pub translations: Option<bool>,
    /// How positions are written in responses.
    #[serde(rename = "$value")]
    pub coordinate_format: CoordinateFormat,
    /// Whether a request may ask for a particular level of detail.
    #[serde(rename = "HasDetailLevel", default, skip_serializing_if = "Option::is_none")]
    pub has_detail_level: Option<bool>,
    /// The level of detail applied when a request does not ask for one.
    #[serde(rename = "DefaultDetailLevel", default, skip_serializing_if = "Option::is_none")]
    pub default_detail_level: Option<VehicleMonitoringDetail>,
    /// Whether a request may cap how many vehicles come back.
    #[serde(rename = "HasMaximumVehicles", default, skip_serializing_if = "Option::is_none")]
    pub has_maximum_vehicles: Option<bool>,
    /// Whether a request may cap how many calls come back per vehicle.
    #[serde(rename = "HasMaximumNumberOfCalls", default, skip_serializing_if = "Option::is_none")]
    pub has_maximum_number_of_calls: Option<bool>,
    /// Whether that cap may be set for the calls still to come.
    #[serde(rename = "HasNumberOfOnwardsCalls", default, skip_serializing_if = "Option::is_none")]
    pub has_number_of_onwards_calls: Option<bool>,
    /// Whether that cap may be set for the calls already made.
    #[serde(rename = "HasNumberOfPreviousCalls", default, skip_serializing_if = "Option::is_none")]
    pub has_number_of_previous_calls: Option<bool>,
}

/// Whether and how a Vehicle Monitoring service checks requests against the
/// permissions of the participant making them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VehicleMonitoringAccessControl {
    /// Whether requests are checked against permissions at all.
    #[serde(rename = "RequestChecking")]
    pub request_checking: bool,
    /// Whether the operator a request names is checked against its permissions.
    #[serde(rename = "CheckOperatorRef", default, skip_serializing_if = "Option::is_none")]
    pub check_operator_ref: Option<bool>,
    /// Whether the line a request names is checked against its permissions.
    #[serde(rename = "CheckLineRef", default, skip_serializing_if = "Option::is_none")]
    pub check_line_ref: Option<bool>,
    /// Whether the monitoring service a request names is checked against its
    /// permissions.
    #[serde(rename = "CheckVehicleMonitoringRef", default, skip_serializing_if = "Option::is_none")]
    pub check_vehicle_monitoring_ref: Option<bool>,
}

/// Optional content a Vehicle Monitoring service's responses may carry.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VehicleMonitoringResponseFeatures {
    /// Whether the responses carry vehicle positions.
    #[serde(rename = "HasLocation", default, skip_serializing_if = "Option::is_none")]
    pub has_location: Option<bool>,
    /// Whether they carry the situations affecting the vehicles.
    #[serde(rename = "HasSituations", default, skip_serializing_if = "Option::is_none")]
    pub has_situations: Option<bool>,
}

/// What participants are allowed to see of a Vehicle Monitoring service.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VehicleMonitoringPermissions {
    /// The edition of the permission set these entries belong to.
    #[serde(rename = "PermissionVersionRef", default, skip_serializing_if = "Option::is_none")]
    pub permission_version_ref: Option<PermissionVersionRef>,
    /// One entry per participant, plus optionally one covering everybody else.
    #[serde(
        rename = "VehicleMonitoringPermission",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub vehicle_monitoring_permission: Vec<VehicleMonitoringServicePermission>,
}

/// What one participant may see of a Vehicle Monitoring service.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleMonitoringServicePermission {
    /// Who this entry applies to.
    #[serde(rename = "$value")]
    pub scope: PermissionScope,
    /// Which interaction patterns the participant may use.
    #[serde(rename = "GeneralCapabilities", default, skip_serializing_if = "Option::is_none")]
    pub general_capabilities: Option<GeneralPermissions>,
    /// Whose vehicles the participant may see.
    #[serde(rename = "OperatorPermissions")]
    pub operator_permissions: OperatorPermissions,
    /// Which lines' vehicles the participant may see.
    #[serde(rename = "LinePermissions")]
    pub line_permissions: LinePermissions,
    /// Which of the producer's monitoring services the participant may draw from.
    #[serde(rename = "VehicleMonitoringPermissions")]
    pub vehicle_monitoring_permissions: MonitoringPermissions,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl VehicleMonitoringServicePermission {
    /// A permission entry applying to every participant that has no entry of its own.
    pub fn for_all_participants() -> Self {
        Self {
            scope: PermissionScope::AllParticipants(crate::types::Empty::new()),
            general_capabilities: None,
            operator_permissions: OperatorPermissions::allow_all(),
            line_permissions: LinePermissions::allow_all(),
            vehicle_monitoring_permissions: MonitoringPermissions::allow_all(),
            extensions: None,
        }
    }
}

/// Which of a producer's monitoring services a participant may draw from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitoringPermissions {
    /// The entries, all of the same kind.
    #[serde(rename = "$value")]
    pub items: Vec<MonitoringPermissionItem>,
}

impl MonitoringPermissions {
    /// Permission covering every monitoring service the producer publishes.
    pub fn allow_all() -> Self {
        Self {
            items: vec![MonitoringPermissionItem::AllowAll(true)],
        }
    }

    /// Permission listed service by service.
    pub fn per_service(permissions: Vec<VehicleMonitorPermission>) -> Self {
        Self {
            items: permissions
                .into_iter()
                .map(MonitoringPermissionItem::VehicleMonitorPermission)
                .collect(),
        }
    }
}

/// One entry of a [`MonitoringPermissions`] list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonitoringPermissionItem {
    /// Whether every monitoring service the producer publishes is covered.
    AllowAll(bool),
    /// A decision about one named monitoring service.
    VehicleMonitorPermission(VehicleMonitorPermission),
}

/// Whether a participant may draw from one named monitoring service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VehicleMonitorPermission {
    /// Whether access is granted or withheld.
    #[serde(rename = "Allow")]
    pub allow: bool,
    /// The monitoring service the decision is about.
    #[serde(rename = "VehicleMonitoringRef")]
    pub vehicle_monitoring_ref: VehicleMonitoringRef,
}
