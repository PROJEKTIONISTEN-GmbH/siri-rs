//! Asking which feeder services are planned onto a connection.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::model::{ConnectionLinkRef, DirectionRef, LineRef};
use crate::types::{
    ClosedTimestampRange, Extensions, MessageQualifier, ParticipantRef, SubscriptionQualifier,
};

/// A request for the connections planned over one connection link.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionTimetableRequest {
    /// Version of SIRI-CT the request conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// When the request was made.
    #[serde(rename = "RequestTimestamp")]
    pub request_timestamp: DateTime<FixedOffset>,
    /// Identifier the requestor puts on this request.
    #[serde(rename = "MessageIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub message_identifier: Option<MessageQualifier>,
    /// The window of feeder arrivals asked about.
    #[serde(rename = "ArrivalWindow", default, skip_serializing_if = "Option::is_none")]
    pub arrival_window: Option<ClosedTimestampRange>,
    /// The connection link to report on.
    #[serde(rename = "ConnectionLinkRef")]
    pub connection_link_ref: ConnectionLinkRef,
    /// Only feeders on this line.
    #[serde(rename = "LineRef", default, skip_serializing_if = "Option::is_none")]
    pub line_ref: Option<LineRef>,
    /// Only feeders running in this direction.
    #[serde(rename = "DirectionRef", default, skip_serializing_if = "Option::is_none")]
    pub direction_ref: Option<DirectionRef>,
    /// Languages the requestor would like texts in, most preferred first.
    #[serde(rename = "Language", default, skip_serializing_if = "Vec::is_empty")]
    pub language: Vec<String>,
    /// Whether to include every translation of a text rather than only one.
    #[serde(rename = "IncludeTranslations", default, skip_serializing_if = "Option::is_none")]
    pub include_translations: Option<bool>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl ConnectionTimetableRequest {
    /// An unfiltered request for the connections over one connection link.
    pub fn over_link(
        request_timestamp: DateTime<FixedOffset>,
        connection_link_ref: impl Into<ConnectionLinkRef>,
    ) -> Self {
        Self {
            version: None,
            request_timestamp,
            message_identifier: None,
            arrival_window: None,
            connection_link_ref: connection_link_ref.into(),
            line_ref: None,
            direction_ref: None,
            language: Vec::new(),
            include_translations: None,
            extensions: None,
        }
    }

    /// The same request narrowed to feeder arrivals within the given window.
    pub fn within(mut self, arrival_window: ClosedTimestampRange) -> Self {
        self.arrival_window = Some(arrival_window);
        self
    }
}

/// A subscription to the connections planned over a connection link.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionTimetableSubscriptionRequest {
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
    #[serde(rename = "ConnectionTimetableRequest")]
    pub connection_timetable_request: ConnectionTimetableRequest,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl ConnectionTimetableSubscriptionRequest {
    /// A subscription running until `initial_termination_time`.
    pub fn new(
        subscription_identifier: impl Into<SubscriptionQualifier>,
        initial_termination_time: DateTime<FixedOffset>,
        connection_timetable_request: ConnectionTimetableRequest,
    ) -> Self {
        Self {
            subscriber_ref: None,
            subscription_identifier: subscription_identifier.into(),
            initial_termination_time,
            subscription_renewal: None,
            connection_timetable_request,
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
    fn the_window_is_written_before_the_link_it_is_about() {
        let request = ConnectionTimetableRequest {
            line_ref: Some("LINE77".into()),
            ..ConnectionTimetableRequest::over_link(timestamp(), "EH00001")
                .within(ClosedTimestampRange::between(timestamp(), timestamp()))
        };

        let xml = quick_xml::se::to_string_with_root("ConnectionTimetableRequest", &request)
            .expect("request serialises");
        let window = xml.find("<ArrivalWindow>").expect("the window is written");
        let link = xml.find("<ConnectionLinkRef>").expect("the link is written");
        let line = xml.find("<LineRef>").expect("the line is written");
        assert!(window < link && link < line, "{xml}");

        let read: ConnectionTimetableRequest =
            quick_xml::de::from_str(&xml).expect("request round-trips");
        assert_eq!(read, request);
    }
}
