//! Asking for the messages on a producer's channels.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::types::{Extensions, MessageQualifier, ParticipantRef, SubscriptionQualifier};

use super::InfoChannelRef;

/// A request for the messages a producer is publishing.
///
/// Naming no channel asks for all of them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneralMessageRequest {
    /// Version of SIRI-GM the request conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// When the request was made.
    #[serde(rename = "RequestTimestamp")]
    pub request_timestamp: DateTime<FixedOffset>,
    /// Identifier the requestor puts on this request.
    #[serde(rename = "MessageIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub message_identifier: Option<MessageQualifier>,
    /// Only these channels.
    #[serde(rename = "InfoChannelRef", default, skip_serializing_if = "Vec::is_empty")]
    pub info_channel_ref: Vec<InfoChannelRef>,
    /// Languages the requestor would like texts in, most preferred first.
    #[serde(rename = "Language", default, skip_serializing_if = "Vec::is_empty")]
    pub language: Vec<String>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl GeneralMessageRequest {
    /// A request for every channel the producer publishes.
    pub fn new(request_timestamp: DateTime<FixedOffset>) -> Self {
        Self {
            version: None,
            request_timestamp,
            message_identifier: None,
            info_channel_ref: Vec::new(),
            language: Vec::new(),
            extensions: None,
        }
    }

    /// A request for the named channels only.
    pub fn on_channels(
        request_timestamp: DateTime<FixedOffset>,
        info_channel_ref: Vec<InfoChannelRef>,
    ) -> Self {
        Self {
            info_channel_ref,
            ..Self::new(request_timestamp)
        }
    }
}

/// A subscription to the messages a producer is publishing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneralMessageSubscriptionRequest {
    /// Who is subscribing, when different from the requestor of the enclosing message.
    #[serde(rename = "SubscriberRef", default, skip_serializing_if = "Option::is_none")]
    pub subscriber_ref: Option<ParticipantRef>,
    /// The subscriber's name for this subscription, quoted in every delivery.
    #[serde(rename = "SubscriptionIdentifier")]
    pub subscription_identifier: SubscriptionQualifier,
    /// When the subscription lapses unless renewed.
    #[serde(rename = "InitialTerminationTime")]
    pub initial_termination_time: DateTime<FixedOffset>,
    /// Whether this replaces an existing subscription with the same identifier.
    #[serde(rename = "SubscriptionRenewal", default, skip_serializing_if = "Option::is_none")]
    pub subscription_renewal: Option<bool>,
    /// What to subscribe to.
    #[serde(rename = "GeneralMessageRequest")]
    pub general_message_request: GeneralMessageRequest,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl GeneralMessageSubscriptionRequest {
    /// A subscription running until `initial_termination_time`.
    pub fn new(
        subscription_identifier: impl Into<SubscriptionQualifier>,
        initial_termination_time: DateTime<FixedOffset>,
        general_message_request: GeneralMessageRequest,
    ) -> Self {
        Self {
            subscriber_ref: None,
            subscription_identifier: subscription_identifier.into(),
            initial_termination_time,
            subscription_renewal: None,
            general_message_request,
            extensions: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timestamp() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2004-12-17T09:30:47-05:00").expect("valid timestamp")
    }

    #[test]
    fn a_request_naming_no_channel_asks_for_all_of_them() {
        let all = GeneralMessageRequest::new(timestamp());
        let xml = quick_xml::se::to_string_with_root("GeneralMessageRequest", &all)
            .expect("request serialises");
        assert!(!xml.contains("InfoChannelRef"), "{xml}");

        let narrowed =
            GeneralMessageRequest::on_channels(timestamp(), vec![InfoChannelRef::new("WARNING")]);
        let xml = quick_xml::se::to_string_with_root("GeneralMessageRequest", &narrowed)
            .expect("request serialises");
        assert!(xml.contains("<InfoChannelRef>WARNING</InfoChannelRef>"), "{xml}");

        let read: GeneralMessageRequest =
            quick_xml::de::from_str(&xml).expect("request round-trips");
        assert_eq!(read, narrowed);
    }
}
