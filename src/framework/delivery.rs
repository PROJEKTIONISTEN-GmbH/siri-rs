//! Fetched delivery: announcing that data is ready, asking for it, acknowledging it.
//!
//! A producer that does not push deliveries sends a [`DataReadyNotification`]
//! instead; the consumer answers with a [`DataReadyAcknowledgement`] and collects
//! the data with a [`DataSupplyRequest`].

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::framework::error_condition::{AcknowledgementError, ErrorCondition};
use crate::types::{EndpointAddress, MessageQualifier, MessageRef, ParticipantRef};

/// A producer telling a consumer that data is waiting to be fetched.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataReadyNotification {
    /// When the notification was made.
    #[serde(rename = "RequestTimestamp")]
    pub request_timestamp: DateTime<FixedOffset>,
    /// Account the producer authenticates as.
    #[serde(rename = "AccountId", default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    /// Secret authenticating the account.
    #[serde(rename = "AccountKey", default, skip_serializing_if = "Option::is_none")]
    pub account_key: Option<String>,
    /// Address of the producer.
    #[serde(rename = "Address", default, skip_serializing_if = "Option::is_none")]
    pub address: Option<EndpointAddress>,
    /// Who has data ready.
    #[serde(rename = "ProducerRef", default, skip_serializing_if = "Option::is_none")]
    pub producer_ref: Option<ParticipantRef>,
    /// Identifier the producer puts on this message, quoted back in the fetch.
    #[serde(rename = "MessageIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub message_identifier: Option<MessageQualifier>,
    /// Address of the participant the notification is sent on behalf of.
    #[serde(rename = "DelegatorAddress", default, skip_serializing_if = "Option::is_none")]
    pub delegator_address: Option<EndpointAddress>,
    /// Participant the notification is sent on behalf of.
    #[serde(rename = "DelegatorRef", default, skip_serializing_if = "Option::is_none")]
    pub delegator_ref: Option<ParticipantRef>,
}

impl DataReadyNotification {
    /// A notification from `producer_ref`.
    pub fn new(
        request_timestamp: DateTime<FixedOffset>,
        producer_ref: impl Into<ParticipantRef>,
    ) -> Self {
        Self {
            request_timestamp,
            account_id: None,
            account_key: None,
            address: None,
            producer_ref: Some(producer_ref.into()),
            message_identifier: None,
            delegator_address: None,
            delegator_ref: None,
        }
    }
}

/// A consumer acknowledging a [`DataReadyNotification`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataReadyAcknowledgement {
    /// When the acknowledgement was made.
    #[serde(rename = "ResponseTimestamp")]
    pub response_timestamp: DateTime<FixedOffset>,
    /// Who acknowledges.
    #[serde(rename = "ConsumerRef", default, skip_serializing_if = "Option::is_none")]
    pub consumer_ref: Option<ParticipantRef>,
    /// The notification being acknowledged.
    #[serde(rename = "RequestMessageRef", default, skip_serializing_if = "Option::is_none")]
    pub request_message_ref: Option<MessageRef>,
    /// Address of the participant the acknowledgement is sent on behalf of.
    #[serde(rename = "DelegatorAddress", default, skip_serializing_if = "Option::is_none")]
    pub delegator_address: Option<EndpointAddress>,
    /// Participant the acknowledgement is sent on behalf of.
    #[serde(rename = "DelegatorRef", default, skip_serializing_if = "Option::is_none")]
    pub delegator_ref: Option<ParticipantRef>,
    /// Whether the notification was accepted.
    #[serde(rename = "Status", default, skip_serializing_if = "Option::is_none")]
    pub status: Option<bool>,
    /// Why the notification was not accepted.
    #[serde(rename = "ErrorCondition", default, skip_serializing_if = "Option::is_none")]
    pub error_condition: Option<ErrorCondition<AcknowledgementError>>,
}

impl DataReadyAcknowledgement {
    /// An acknowledgement saying the notification was accepted.
    pub fn accepted(
        response_timestamp: DateTime<FixedOffset>,
        consumer_ref: impl Into<ParticipantRef>,
    ) -> Self {
        Self {
            response_timestamp,
            consumer_ref: Some(consumer_ref.into()),
            request_message_ref: None,
            delegator_address: None,
            delegator_ref: None,
            status: Some(true),
            error_condition: None,
        }
    }
}

/// A consumer fetching the data a producer announced.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataSupplyRequest {
    /// When the request was made.
    #[serde(rename = "RequestTimestamp")]
    pub request_timestamp: DateTime<FixedOffset>,
    /// Account the consumer authenticates as.
    #[serde(rename = "AccountId", default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    /// Secret authenticating the account.
    #[serde(rename = "AccountKey", default, skip_serializing_if = "Option::is_none")]
    pub account_key: Option<String>,
    /// Address of the consumer.
    #[serde(rename = "Address", default, skip_serializing_if = "Option::is_none")]
    pub address: Option<EndpointAddress>,
    /// Who is fetching.
    #[serde(rename = "ConsumerRef", default, skip_serializing_if = "Option::is_none")]
    pub consumer_ref: Option<ParticipantRef>,
    /// Identifier the consumer puts on this message.
    #[serde(rename = "MessageIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub message_identifier: Option<MessageQualifier>,
    /// Address of the participant the fetch is made on behalf of.
    #[serde(rename = "DelegatorAddress", default, skip_serializing_if = "Option::is_none")]
    pub delegator_address: Option<EndpointAddress>,
    /// Participant the fetch is made on behalf of.
    #[serde(rename = "DelegatorRef", default, skip_serializing_if = "Option::is_none")]
    pub delegator_ref: Option<ParticipantRef>,
    /// The notification whose data is being fetched.
    #[serde(rename = "NotificationRef", default, skip_serializing_if = "Option::is_none")]
    pub notification_ref: Option<MessageRef>,
    /// Whether to send everything currently held rather than only what is new.
    #[serde(rename = "AllData", default, skip_serializing_if = "Option::is_none")]
    pub all_data: Option<bool>,
}

impl DataSupplyRequest {
    /// A fetch by `consumer_ref` of whatever the producer holds for it.
    ///
    /// A fetch that answers one particular announcement names it in
    /// `notification_ref`; one that does not leaves the element out.
    pub fn new(
        request_timestamp: DateTime<FixedOffset>,
        consumer_ref: impl Into<ParticipantRef>,
    ) -> Self {
        Self {
            request_timestamp,
            account_id: None,
            account_key: None,
            address: None,
            consumer_ref: Some(consumer_ref.into()),
            message_identifier: None,
            delegator_address: None,
            delegator_ref: None,
            notification_ref: None,
            all_data: None,
        }
    }
}

/// A consumer acknowledging a delivery.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataReceivedAcknowledgement {
    /// When the acknowledgement was made.
    #[serde(rename = "ResponseTimestamp")]
    pub response_timestamp: DateTime<FixedOffset>,
    /// Who acknowledges.
    #[serde(rename = "ConsumerRef", default, skip_serializing_if = "Option::is_none")]
    pub consumer_ref: Option<ParticipantRef>,
    /// The delivery being acknowledged.
    #[serde(rename = "RequestMessageRef", default, skip_serializing_if = "Option::is_none")]
    pub request_message_ref: Option<MessageRef>,
    /// Address of the participant the acknowledgement is sent on behalf of.
    #[serde(rename = "DelegatorAddress", default, skip_serializing_if = "Option::is_none")]
    pub delegator_address: Option<EndpointAddress>,
    /// Participant the acknowledgement is sent on behalf of.
    #[serde(rename = "DelegatorRef", default, skip_serializing_if = "Option::is_none")]
    pub delegator_ref: Option<ParticipantRef>,
    /// Whether the delivery was processed successfully.
    #[serde(rename = "Status", default, skip_serializing_if = "Option::is_none")]
    pub status: Option<bool>,
    /// Why the delivery could not be processed.
    #[serde(rename = "ErrorCondition", default, skip_serializing_if = "Option::is_none")]
    pub error_condition: Option<ErrorCondition<AcknowledgementError>>,
}

impl DataReceivedAcknowledgement {
    /// An acknowledgement saying the delivery was processed successfully.
    pub fn accepted(
        response_timestamp: DateTime<FixedOffset>,
        consumer_ref: impl Into<ParticipantRef>,
    ) -> Self {
        Self {
            response_timestamp,
            consumer_ref: Some(consumer_ref.into()),
            request_message_ref: None,
            delegator_address: None,
            delegator_ref: None,
            status: Some(true),
            error_condition: None,
        }
    }
}
