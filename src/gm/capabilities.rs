//! What a General Message service can do, and to whom.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::framework::{
    CapabilityRequestPolicy, ErrorCondition, GeneralInteractionCapability, GeneralPermissions,
    PermissionScope, PermissionVersionRef, ServiceRequestError, TransportDescription,
};
use crate::types::{
    DefaultedBoolean, Duration, Empty, EndpointAddress, Extensions, MessageRef, ParticipantRef,
};
use crate::xml::SiriRoot;

use super::InfoChannelRef;

/// What the General Message service offers, and to whom.
///
/// Appears both inside a [`CapabilitiesResponse`](crate::framework::CapabilitiesResponse)
/// and, when only this one service is of interest, as a document in its own right.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneralMessageCapabilitiesResponse {
    /// Version of SIRI-GM the response conforms to.
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
        rename = "GeneralMessageServiceCapabilities",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub general_message_service_capabilities: Option<GeneralMessageServiceCapabilities>,
    /// What participants are allowed to see.
    #[serde(rename = "GeneralMessagePermissions", default, skip_serializing_if = "Option::is_none")]
    pub general_message_permissions: Option<GeneralMessagePermissions>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl GeneralMessageCapabilitiesResponse {
    /// A successful answer describing the given capabilities.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        capabilities: GeneralMessageServiceCapabilities,
    ) -> Self {
        Self {
            version: None,
            response_timestamp,
            request_message_ref: None,
            delegator_address: None,
            delegator_ref: None,
            status: Some(true),
            error_condition: None,
            general_message_service_capabilities: Some(capabilities),
            general_message_permissions: None,
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

impl SiriRoot for GeneralMessageCapabilitiesResponse {
    const ELEMENT_NAME: &'static str = "GeneralMessageCapabilitiesResponse";
}

/// What a General Message service can do.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GeneralMessageServiceCapabilities {
    /// Interaction patterns and delivery mechanics common to all SIRI services.
    #[serde(rename = "GeneralInteraction", default, skip_serializing_if = "Option::is_none")]
    pub general_interaction: Option<GeneralInteractionCapability>,
    /// How messages are carried and whether they are compressed.
    #[serde(rename = "TransportDescription", default, skip_serializing_if = "Option::is_none")]
    pub transport_description: Option<TransportDescription>,
    /// Which criteria a requestor may narrow the messages by.
    #[serde(rename = "TopicFiltering", default, skip_serializing_if = "Option::is_none")]
    pub topic_filtering: Option<GeneralMessageTopicFiltering>,
    /// Languages and coordinate format applied to requests.
    #[serde(rename = "RequestPolicy", default, skip_serializing_if = "Option::is_none")]
    pub request_policy: Option<CapabilityRequestPolicy>,
    /// Whether and how requests are checked against per-participant permissions.
    #[serde(rename = "AccessControl", default, skip_serializing_if = "Option::is_none")]
    pub access_control: Option<GeneralMessageAccessControl>,
}

/// Which criteria a requestor may narrow a producer's messages by.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneralMessageTopicFiltering {
    /// The look-ahead window applied when a request does not name one.
    #[serde(rename = "DefaultPreviewInterval")]
    pub default_preview_interval: Duration,
    /// Whether messages can be narrowed to named channels.
    #[serde(rename = "FilterByInfoChannel", default, skip_serializing_if = "Option::is_none")]
    pub filter_by_info_channel: Option<bool>,
}

/// Whether and how a General Message service checks requests against the
/// permissions of the participant making them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneralMessageAccessControl {
    /// Whether requests are checked against permissions at all.
    #[serde(rename = "RequestChecking")]
    pub request_checking: bool,
    /// Whether the channel a request names is checked against its permissions.
    #[serde(rename = "CheckInfoChannelRef")]
    pub check_info_channel_ref: bool,
}

/// What participants are allowed to see of a General Message service.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GeneralMessagePermissions {
    /// The edition of the permission set these entries belong to.
    #[serde(rename = "PermissionVersionRef", default, skip_serializing_if = "Option::is_none")]
    pub permission_version_ref: Option<PermissionVersionRef>,
    /// One entry per participant, plus optionally one covering everybody else.
    #[serde(rename = "GeneralMessagePermission", default, skip_serializing_if = "Vec::is_empty")]
    pub general_message_permission: Vec<GeneralMessageServicePermission>,
}

/// What one participant may see of a General Message service.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneralMessageServicePermission {
    /// Who this entry applies to.
    #[serde(rename = "$value")]
    pub scope: PermissionScope,
    /// Which interaction patterns the participant may use.
    #[serde(rename = "GeneralCapabilities", default, skip_serializing_if = "Option::is_none")]
    pub general_capabilities: Option<GeneralPermissions>,
    /// Which channels the participant may read.
    #[serde(rename = "InfoChannelPermissions")]
    pub info_channel_permissions: InfoChannelPermissions,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl GeneralMessageServicePermission {
    /// A permission entry applying to every participant that has no entry of its own.
    pub fn for_all_participants() -> Self {
        Self {
            scope: PermissionScope::AllParticipants(Empty::new()),
            general_capabilities: None,
            info_channel_permissions: InfoChannelPermissions::allow_all(),
            extensions: None,
        }
    }
}

/// Which channels a participant may read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InfoChannelPermissions {
    /// The entries, all of the same kind.
    #[serde(rename = "$value")]
    pub items: Vec<InfoChannelPermissionItem>,
}

impl InfoChannelPermissions {
    /// Permission covering every channel the producer publishes.
    pub fn allow_all() -> Self {
        Self {
            items: vec![InfoChannelPermissionItem::AllowAll(true)],
        }
    }

    /// Permission listed channel by channel.
    pub fn per_channel(permissions: Vec<InfoChannelPermission>) -> Self {
        Self {
            items: permissions
                .into_iter()
                .map(InfoChannelPermissionItem::InfoChannelPermission)
                .collect(),
        }
    }
}

/// One entry of an [`InfoChannelPermissions`] list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InfoChannelPermissionItem {
    /// Whether every channel the producer publishes is covered.
    AllowAll(bool),
    /// A decision about one named channel.
    InfoChannelPermission(InfoChannelPermission),
}

/// Whether a participant may read one named channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InfoChannelPermission {
    /// Whether access is granted or withheld; an empty element grants it.
    #[serde(rename = "Allow")]
    pub allow: DefaultedBoolean,
    /// The channel the decision is about.
    #[serde(rename = "InfoChannelRef")]
    pub info_channel_ref: InfoChannelRef,
}
