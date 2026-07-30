//! Checking that a service is alive: `CheckStatus` on demand, `Heartbeat` on a timer.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::framework::error_condition::{ErrorCondition, StatusError};
use crate::types::{
    Duration, EndpointAddress, Extensions, MessageQualifier, MessageRef, ParticipantRef,
};

/// A request for the current operational status of a service.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckStatusRequest {
    /// Version of SIRI the request conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// When the request was made.
    #[serde(rename = "RequestTimestamp")]
    pub request_timestamp: DateTime<FixedOffset>,
    /// Account the requestor authenticates as.
    #[serde(rename = "AccountId", default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    /// Secret authenticating the account.
    #[serde(rename = "AccountKey", default, skip_serializing_if = "Option::is_none")]
    pub account_key: Option<String>,
    /// Address to send the answer to.
    #[serde(rename = "Address", default, skip_serializing_if = "Option::is_none")]
    pub address: Option<EndpointAddress>,
    /// Who is asking.
    #[serde(rename = "RequestorRef")]
    pub requestor_ref: ParticipantRef,
    /// Identifier the requestor puts on this message.
    #[serde(rename = "MessageIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub message_identifier: Option<MessageQualifier>,
    /// Address of the participant this request is made on behalf of.
    #[serde(rename = "DelegatorAddress", default, skip_serializing_if = "Option::is_none")]
    pub delegator_address: Option<EndpointAddress>,
    /// Participant this request is made on behalf of.
    #[serde(rename = "DelegatorRef", default, skip_serializing_if = "Option::is_none")]
    pub delegator_ref: Option<ParticipantRef>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl CheckStatusRequest {
    /// A status request from `requestor_ref`.
    pub fn new(
        request_timestamp: DateTime<FixedOffset>,
        requestor_ref: impl Into<ParticipantRef>,
    ) -> Self {
        Self {
            version: None,
            request_timestamp,
            account_id: None,
            account_key: None,
            address: None,
            requestor_ref: requestor_ref.into(),
            message_identifier: None,
            delegator_address: None,
            delegator_ref: None,
            extensions: None,
        }
    }
}

/// The current operational status of a service.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckStatusResponse {
    /// When the response was made.
    #[serde(rename = "ResponseTimestamp")]
    pub response_timestamp: DateTime<FixedOffset>,
    /// Who answered.
    #[serde(rename = "ProducerRef", default, skip_serializing_if = "Option::is_none")]
    pub producer_ref: Option<ParticipantRef>,
    /// Address of the producer.
    #[serde(rename = "Address", default, skip_serializing_if = "Option::is_none")]
    pub address: Option<EndpointAddress>,
    /// Identifier the producer puts on this message.
    #[serde(rename = "ResponseMessageIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub response_message_identifier: Option<MessageQualifier>,
    /// The status request this answers.
    #[serde(rename = "RequestMessageRef", default, skip_serializing_if = "Option::is_none")]
    pub request_message_ref: Option<MessageRef>,
    /// Address of the participant the answer is given on behalf of.
    #[serde(rename = "DelegatorAddress", default, skip_serializing_if = "Option::is_none")]
    pub delegator_address: Option<EndpointAddress>,
    /// Participant the answer is given on behalf of.
    #[serde(rename = "DelegatorRef", default, skip_serializing_if = "Option::is_none")]
    pub delegator_ref: Option<ParticipantRef>,
    /// Whether the service is working normally.
    #[serde(rename = "Status", default, skip_serializing_if = "Option::is_none")]
    pub status: Option<bool>,
    /// Whether data is waiting to be fetched.
    #[serde(rename = "DataReady", default, skip_serializing_if = "Option::is_none")]
    pub data_ready: Option<bool>,
    /// What is wrong with the service.
    #[serde(rename = "ErrorCondition", default, skip_serializing_if = "Option::is_none")]
    pub error_condition: Option<ErrorCondition<StatusError>>,
    /// How long this answer holds.
    #[serde(rename = "ValidUntil", default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<FixedOffset>>,
    /// The shortest interval at which the producer will accept requests.
    #[serde(rename = "ShortestPossibleCycle", default, skip_serializing_if = "Option::is_none")]
    pub shortest_possible_cycle: Option<Duration>,
    /// When the producer's service last started.
    ///
    /// A restart invalidates the producer's subscriptions, so a consumer that sees
    /// this move forward re-subscribes.
    #[serde(rename = "ServiceStartedTime", default, skip_serializing_if = "Option::is_none")]
    pub service_started_time: Option<DateTime<FixedOffset>>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl CheckStatusResponse {
    /// A response saying the service is working normally.
    pub fn healthy(
        response_timestamp: DateTime<FixedOffset>,
        producer_ref: impl Into<ParticipantRef>,
        service_started_time: DateTime<FixedOffset>,
    ) -> Self {
        Self {
            response_timestamp,
            producer_ref: Some(producer_ref.into()),
            address: None,
            response_message_identifier: None,
            request_message_ref: None,
            delegator_address: None,
            delegator_ref: None,
            status: Some(true),
            data_ready: None,
            error_condition: None,
            valid_until: None,
            shortest_possible_cycle: None,
            service_started_time: Some(service_started_time),
            extensions: None,
        }
    }

    /// Whether the service reported itself as working.
    ///
    /// `Status` is optional in the schema and defaults to true.
    pub fn is_healthy(&self) -> bool {
        self.status.unwrap_or(true)
    }
}

/// A producer's unsolicited sign of life.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatNotification {
    /// When the heartbeat was sent.
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
    /// Who is alive.
    #[serde(rename = "ProducerRef", default, skip_serializing_if = "Option::is_none")]
    pub producer_ref: Option<ParticipantRef>,
    /// Identifier the producer puts on this message.
    #[serde(rename = "MessageIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub message_identifier: Option<MessageQualifier>,
    /// Address of the participant the heartbeat is sent on behalf of.
    #[serde(rename = "DelegatorAddress", default, skip_serializing_if = "Option::is_none")]
    pub delegator_address: Option<EndpointAddress>,
    /// Participant the heartbeat is sent on behalf of.
    #[serde(rename = "DelegatorRef", default, skip_serializing_if = "Option::is_none")]
    pub delegator_ref: Option<ParticipantRef>,
    /// Whether the service is working normally.
    #[serde(rename = "Status", default, skip_serializing_if = "Option::is_none")]
    pub status: Option<bool>,
    /// Whether data is waiting to be fetched.
    #[serde(rename = "DataReady", default, skip_serializing_if = "Option::is_none")]
    pub data_ready: Option<bool>,
    /// What is wrong with the service.
    #[serde(rename = "ErrorCondition", default, skip_serializing_if = "Option::is_none")]
    pub error_condition: Option<ErrorCondition<StatusError>>,
    /// How long this heartbeat holds; a consumer that sees it lapse re-checks.
    #[serde(rename = "ValidUntil", default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<FixedOffset>>,
    /// The shortest interval at which the producer will accept requests.
    #[serde(rename = "ShortestPossibleCycle", default, skip_serializing_if = "Option::is_none")]
    pub shortest_possible_cycle: Option<Duration>,
    /// When the producer's service last started.
    #[serde(rename = "ServiceStartedTime", default, skip_serializing_if = "Option::is_none")]
    pub service_started_time: Option<DateTime<FixedOffset>>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl HeartbeatNotification {
    /// A heartbeat saying the service is working normally.
    pub fn healthy(
        request_timestamp: DateTime<FixedOffset>,
        producer_ref: impl Into<ParticipantRef>,
        service_started_time: DateTime<FixedOffset>,
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
            status: Some(true),
            data_ready: None,
            error_condition: None,
            valid_until: None,
            shortest_possible_cycle: None,
            service_started_time: Some(service_started_time),
            extensions: None,
        }
    }
}
