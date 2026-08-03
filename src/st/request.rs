//! Asking what the timetable says will call at a stop.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::model::{DirectionRef, LineRef, MonitoringRef};
use crate::types::{
    ClosedTimestampRange, Duration, Extensions, MessageQualifier, ParticipantRef,
    SubscriptionQualifier,
};

/// A request for the timetable at one monitoring point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopTimetableRequest {
    /// Version of SIRI-ST the request conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// When the request was made.
    #[serde(rename = "RequestTimestamp")]
    pub request_timestamp: DateTime<FixedOffset>,
    /// Identifier the requestor puts on this request.
    #[serde(rename = "MessageIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub message_identifier: Option<MessageQualifier>,
    /// The window of departures asked about.
    #[serde(rename = "DepartureWindow", default, skip_serializing_if = "Option::is_none")]
    pub departure_window: Option<ClosedTimestampRange>,
    /// The monitoring point to report on.
    #[serde(rename = "MonitoringRef")]
    pub monitoring_ref: MonitoringRef,
    /// Only this line.
    #[serde(rename = "LineRef", default, skip_serializing_if = "Option::is_none")]
    pub line_ref: Option<LineRef>,
    /// Only services running in this direction.
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

impl StopTimetableRequest {
    /// An unfiltered request for the timetable at one monitoring point.
    pub fn at_stop(
        request_timestamp: DateTime<FixedOffset>,
        monitoring_ref: impl Into<MonitoringRef>,
    ) -> Self {
        Self {
            version: None,
            request_timestamp,
            message_identifier: None,
            departure_window: None,
            monitoring_ref: monitoring_ref.into(),
            line_ref: None,
            direction_ref: None,
            language: Vec::new(),
            include_translations: None,
            extensions: None,
        }
    }

    /// The same request narrowed to departures within the given window.
    pub fn within(mut self, departure_window: ClosedTimestampRange) -> Self {
        self.departure_window = Some(departure_window);
        self
    }
}

/// A subscription to the timetable at a monitoring point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopTimetableSubscriptionRequest {
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
    #[serde(rename = "StopTimetableRequest")]
    pub stop_timetable_request: StopTimetableRequest,
    /// Whether to send only what has changed rather than the full set each time.
    #[serde(rename = "IncrementalUpdates", default, skip_serializing_if = "Option::is_none")]
    pub incremental_updates: Option<bool>,
    /// How large a change has to be before it is worth a delivery.
    #[serde(rename = "ChangeBeforeUpdates", default, skip_serializing_if = "Option::is_none")]
    pub change_before_updates: Option<Duration>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl StopTimetableSubscriptionRequest {
    /// A subscription running until `initial_termination_time`.
    pub fn new(
        subscription_identifier: impl Into<SubscriptionQualifier>,
        initial_termination_time: DateTime<FixedOffset>,
        stop_timetable_request: StopTimetableRequest,
    ) -> Self {
        Self {
            subscriber_ref: None,
            subscription_identifier: subscription_identifier.into(),
            initial_termination_time,
            subscription_renewal: None,
            stop_timetable_request,
            incremental_updates: None,
            change_before_updates: None,
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
    fn the_window_is_written_before_the_stop_it_is_about() {
        let request = StopTimetableRequest {
            line_ref: Some("LINE77".into()),
            ..StopTimetableRequest::at_stop(timestamp(), "EH00001")
                .within(ClosedTimestampRange::between(timestamp(), timestamp()))
        };

        let xml = quick_xml::se::to_string_with_root("StopTimetableRequest", &request)
            .expect("request serialises");
        let window = xml.find("<DepartureWindow>").expect("the window is written");
        let monitoring = xml.find("<MonitoringRef>").expect("the stop is written");
        let line = xml.find("<LineRef>").expect("the line is written");
        assert!(window < monitoring && monitoring < line, "{xml}");

        let read: StopTimetableRequest =
            quick_xml::de::from_str(&xml).expect("request round-trips");
        assert_eq!(read, request);
    }
}
