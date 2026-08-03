//! Delivering free-form messages.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::framework::{ErrorCondition, ServiceRequestError};
use crate::model::SituationRef;
use crate::types::{
    AnyContent, Duration, EndpointAddress, Extensions, ItemIdentifier, ItemRef, MessageRef,
    ParticipantRef, SubscriptionFilterRef, SubscriptionRef,
};

use super::{InfoChannelRef, InfoMessageRef};

/// The messages a producer is publishing.
///
/// A delivery either answers a
/// [`GeneralMessageRequest`](crate::gm::GeneralMessageRequest) — in which case it
/// quotes the request's identifier — or satisfies a subscription, in which case it
/// quotes the subscription's.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneralMessageDelivery {
    /// Version of SIRI-GM the delivery conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// When the delivery was made.
    #[serde(rename = "ResponseTimestamp")]
    pub response_timestamp: DateTime<FixedOffset>,
    /// The request this delivery answers.
    #[serde(rename = "RequestMessageRef", default, skip_serializing_if = "Option::is_none")]
    pub request_message_ref: Option<MessageRef>,
    /// Who holds the subscription this delivery satisfies.
    #[serde(rename = "SubscriberRef", default, skip_serializing_if = "Option::is_none")]
    pub subscriber_ref: Option<ParticipantRef>,
    /// The shared filter the subscription uses.
    #[serde(rename = "SubscriptionFilterRef", default, skip_serializing_if = "Option::is_none")]
    pub subscription_filter_ref: Option<SubscriptionFilterRef>,
    /// The subscription this delivery satisfies.
    #[serde(rename = "SubscriptionRef", default, skip_serializing_if = "Option::is_none")]
    pub subscription_ref: Option<SubscriptionRef>,
    /// Address of the participant the data is delivered on behalf of.
    #[serde(rename = "DelegatorAddress", default, skip_serializing_if = "Option::is_none")]
    pub delegator_address: Option<EndpointAddress>,
    /// Participant the data is delivered on behalf of.
    #[serde(rename = "DelegatorRef", default, skip_serializing_if = "Option::is_none")]
    pub delegator_ref: Option<ParticipantRef>,
    /// Whether the request or subscription was processed successfully.
    #[serde(rename = "Status", default, skip_serializing_if = "Option::is_none")]
    pub status: Option<bool>,
    /// Why it could not be processed.
    #[serde(rename = "ErrorCondition", default, skip_serializing_if = "Option::is_none")]
    pub error_condition: Option<ErrorCondition<ServiceRequestError>>,
    /// How long this delivery holds.
    #[serde(rename = "ValidUntil", default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<FixedOffset>>,
    /// The shortest interval at which the producer will send updates.
    #[serde(rename = "ShortestPossibleCycle", default, skip_serializing_if = "Option::is_none")]
    pub shortest_possible_cycle: Option<Duration>,
    /// The language texts are in unless a message says otherwise.
    #[serde(rename = "DefaultLanguage", default, skip_serializing_if = "Option::is_none")]
    pub default_language: Option<String>,
    /// The messages themselves.
    #[serde(rename = "GeneralMessage", default, skip_serializing_if = "Vec::is_empty")]
    pub general_message: Vec<InfoMessage>,
    /// Messages the producer is withdrawing.
    #[serde(rename = "GeneralMessageCancellation", default, skip_serializing_if = "Vec::is_empty")]
    pub general_message_cancellation: Vec<InfoMessageCancellation>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl GeneralMessageDelivery {
    /// A delivery carrying the given messages.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        general_message: Vec<InfoMessage>,
    ) -> Self {
        Self {
            version: None,
            response_timestamp,
            request_message_ref: None,
            subscriber_ref: None,
            subscription_filter_ref: None,
            subscription_ref: None,
            delegator_address: None,
            delegator_ref: None,
            status: None,
            error_condition: None,
            valid_until: None,
            shortest_possible_cycle: None,
            default_language: None,
            general_message,
            general_message_cancellation: Vec::new(),
            extensions: None,
        }
    }
}

/// One message on one channel.
///
/// The body is whatever the two ends have agreed, so it is kept as the subtree it
/// was written as; `format_ref` names the vocabulary it is written in.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InfoMessage {
    /// Which vocabulary the content is written in.
    #[serde(rename = "@formatRef", default, skip_serializing_if = "Option::is_none")]
    pub format_ref: Option<String>,
    /// When the producer last knew this to be true.
    #[serde(rename = "RecordedAtTime")]
    pub recorded_at_time: DateTime<FixedOffset>,
    /// The producer's identifier for this record, so that it can be superseded.
    #[serde(rename = "ItemIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub item_identifier: Option<ItemIdentifier>,
    /// The message's own identifier, which a revision or a withdrawal quotes.
    #[serde(rename = "InfoMessageIdentifier")]
    pub info_message_identifier: InfoMessageRef,
    /// Which revision of that message this is.
    #[serde(rename = "InfoMessageVersion", default, skip_serializing_if = "Option::is_none")]
    pub info_message_version: Option<u64>,
    /// The channel it is published on.
    #[serde(rename = "InfoChannelRef", default, skip_serializing_if = "Option::is_none")]
    pub info_channel_ref: Option<InfoChannelRef>,
    /// How long the message may be shown.
    #[serde(rename = "ValidUntilTime", default, skip_serializing_if = "Option::is_none")]
    pub valid_until_time: Option<DateTime<FixedOffset>>,
    /// The situation the message is about.
    #[serde(rename = "SituationRef", default, skip_serializing_if = "Option::is_none")]
    pub situation_ref: Option<SituationRef>,
    /// The message itself.
    #[serde(rename = "Content")]
    pub content: AnyContent,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl InfoMessage {
    /// A message carrying the given content.
    pub fn new(
        recorded_at_time: DateTime<FixedOffset>,
        info_message_identifier: impl Into<InfoMessageRef>,
        content: AnyContent,
    ) -> Self {
        Self {
            format_ref: None,
            recorded_at_time,
            item_identifier: None,
            info_message_identifier: info_message_identifier.into(),
            info_message_version: None,
            info_channel_ref: None,
            valid_until_time: None,
            situation_ref: None,
            content,
            extensions: None,
        }
    }
}

/// A message the producer is withdrawing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InfoMessageCancellation {
    /// When the withdrawal was decided.
    #[serde(rename = "RecordedAtTime")]
    pub recorded_at_time: DateTime<FixedOffset>,
    /// The record being withdrawn.
    #[serde(rename = "ItemRef", default, skip_serializing_if = "Option::is_none")]
    pub item_ref: Option<ItemRef>,
    /// The message being withdrawn.
    #[serde(rename = "InfoMessageIdentifier")]
    pub info_message_identifier: InfoMessageRef,
    /// The channel it was published on.
    #[serde(rename = "InfoChannelRef", default, skip_serializing_if = "Option::is_none")]
    pub info_channel_ref: Option<InfoChannelRef>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl InfoMessageCancellation {
    /// A withdrawal of the message with the given identifier.
    pub fn new(
        recorded_at_time: DateTime<FixedOffset>,
        info_message_identifier: impl Into<InfoMessageRef>,
    ) -> Self {
        Self {
            recorded_at_time,
            item_ref: None,
            info_message_identifier: info_message_identifier.into(),
            info_channel_ref: None,
            extensions: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timestamp() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2001-12-17T09:30:47-05:00").expect("valid timestamp")
    }

    #[test]
    fn a_message_keeps_whatever_content_it_was_given() {
        let mut delivery = GeneralMessageDelivery::new(
            timestamp(),
            vec![InfoMessage {
                info_channel_ref: Some(InfoChannelRef::new("WARNINGS")),
                ..InfoMessage::new(
                    timestamp(),
                    "00034567",
                    AnyContent::text("Beware the Ides of March"),
                )
            }],
        );
        delivery
            .general_message_cancellation
            .push(InfoMessageCancellation::new(timestamp(), "00034564"));

        let xml = quick_xml::se::to_string_with_root("GeneralMessageDelivery", &delivery)
            .expect("delivery serialises");
        assert!(xml.contains("<Content>Beware the Ides of March</Content>"), "{xml}");

        let read: GeneralMessageDelivery =
            quick_xml::de::from_str(&xml).expect("delivery round-trips");
        assert_eq!(read, delivery);
    }
}
