//! Asking what is due at a stop, once or by subscription.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::{StopMonitoringDetail, StopVisitType};
use crate::model::{
    DestinationRef, DirectionRef, LineRef, MaximumNumberOfCalls, MonitoringRef, OperatorRef,
};
use crate::types::{
    Duration, Extensions, MessageQualifier, ParticipantRef, SubscriptionQualifier,
};

/// A request for what is due at one monitoring point.
///
/// The topic names the stop and narrows it by operator, line, direction or
/// destination; the policy fields cap how much comes back. Filters of different
/// kinds are combined with "and".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopMonitoringRequest {
    /// Version of SIRI-SM the request conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// When the request was made.
    #[serde(rename = "RequestTimestamp")]
    pub request_timestamp: DateTime<FixedOffset>,
    /// Identifier the requestor puts on this request.
    #[serde(rename = "MessageIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub message_identifier: Option<MessageQualifier>,
    /// How far ahead to look.
    #[serde(rename = "PreviewInterval", default, skip_serializing_if = "Option::is_none")]
    pub preview_interval: Option<Duration>,
    /// When that look-ahead window starts, if not now.
    #[serde(rename = "StartTime", default, skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime<FixedOffset>>,
    /// The monitoring point to report on.
    #[serde(rename = "MonitoringRef")]
    pub monitoring_ref: MonitoringRef,
    /// Only this operator's services.
    #[serde(rename = "OperatorRef", default, skip_serializing_if = "Option::is_none")]
    pub operator_ref: Option<OperatorRef>,
    /// Only this line.
    #[serde(rename = "LineRef", default, skip_serializing_if = "Option::is_none")]
    pub line_ref: Option<LineRef>,
    /// Only services running in this direction.
    #[serde(rename = "DirectionRef", default, skip_serializing_if = "Option::is_none")]
    pub direction_ref: Option<DirectionRef>,
    /// Only services bound for this destination.
    #[serde(rename = "DestinationRef", default, skip_serializing_if = "Option::is_none")]
    pub destination_ref: Option<DestinationRef>,
    /// Whether to report arrivals, departures or both.
    #[serde(rename = "StopVisitTypes", default, skip_serializing_if = "Option::is_none")]
    pub stop_visit_types: Option<StopVisitType>,
    /// Languages the requestor would like texts in, most preferred first.
    #[serde(rename = "Language", default, skip_serializing_if = "Vec::is_empty")]
    pub language: Vec<String>,
    /// Whether to include every translation of a text rather than only one.
    #[serde(rename = "IncludeTranslations", default, skip_serializing_if = "Option::is_none")]
    pub include_translations: Option<bool>,
    /// At most this many visits altogether.
    #[serde(rename = "MaximumStopVisits", default, skip_serializing_if = "Option::is_none")]
    pub maximum_stop_visits: Option<u64>,
    /// At least this many visits for each line, whatever the overall cap.
    #[serde(rename = "MinimumStopVisitsPerLine", default, skip_serializing_if = "Option::is_none")]
    pub minimum_stop_visits_per_line: Option<u64>,
    /// At least this many for each line and via point, instead of per line.
    #[serde(rename = "MinimumStopVisitsPerLineVia", default, skip_serializing_if = "Option::is_none")]
    pub minimum_stop_visits_per_line_via: Option<u64>,
    /// How long texts may be, for a display with a fixed width.
    #[serde(rename = "MaximumTextLength", default, skip_serializing_if = "Option::is_none")]
    pub maximum_text_length: Option<u64>,
    /// How much detail to give per visit.
    #[serde(rename = "StopMonitoringDetailLevel", default, skip_serializing_if = "Option::is_none")]
    pub stop_monitoring_detail_level: Option<StopMonitoringDetail>,
    /// Whether to include the situations affecting the services.
    #[serde(rename = "IncludeSituations", default, skip_serializing_if = "Option::is_none")]
    pub include_situations: Option<bool>,
    /// How many calls before and after this stop to include.
    #[serde(rename = "MaximumNumberOfCalls", default, skip_serializing_if = "Option::is_none")]
    pub maximum_number_of_calls: Option<MaximumNumberOfCalls>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl StopMonitoringRequest {
    /// An unfiltered request for everything due at one monitoring point.
    pub fn at_stop(
        request_timestamp: DateTime<FixedOffset>,
        monitoring_ref: impl Into<MonitoringRef>,
    ) -> Self {
        Self {
            version: None,
            request_timestamp,
            message_identifier: None,
            preview_interval: None,
            start_time: None,
            monitoring_ref: monitoring_ref.into(),
            operator_ref: None,
            line_ref: None,
            direction_ref: None,
            destination_ref: None,
            stop_visit_types: None,
            language: Vec::new(),
            include_translations: None,
            maximum_stop_visits: None,
            minimum_stop_visits_per_line: None,
            minimum_stop_visits_per_line_via: None,
            maximum_text_length: None,
            stop_monitoring_detail_level: None,
            include_situations: None,
            maximum_number_of_calls: None,
            extensions: None,
        }
    }
}

/// A request covering several monitoring points at once, each with its own filter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopMonitoringMultipleRequest {
    /// Version of SIRI-SM the request conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// When the request was made.
    #[serde(rename = "RequestTimestamp")]
    pub request_timestamp: DateTime<FixedOffset>,
    /// Identifier the requestor puts on this request.
    #[serde(rename = "MessageIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub message_identifier: Option<MessageQualifier>,
    /// One filter per monitoring point, at least one.
    ///
    /// The element name carries a spelling mistake that has been in the published
    /// schema since SIRI 2.0 and is kept here so documents stay valid.
    #[serde(rename = "StopMonitoringFIlter")]
    pub stop_monitoring_filter: Vec<StopMonitoringFilter>,
}

impl StopMonitoringMultipleRequest {
    /// A request carrying the given per-stop filters.
    pub fn new(
        request_timestamp: DateTime<FixedOffset>,
        stop_monitoring_filter: Vec<StopMonitoringFilter>,
    ) -> Self {
        Self {
            version: None,
            request_timestamp,
            message_identifier: None,
            stop_monitoring_filter,
        }
    }
}

/// One monitoring point within a [`StopMonitoringMultipleRequest`], with its filters.
///
/// The fields are those of a [`StopMonitoringRequest`] without the request's own
/// timestamp and identifier, which the enclosing request carries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopMonitoringFilter {
    /// How far ahead to look.
    #[serde(rename = "PreviewInterval", default, skip_serializing_if = "Option::is_none")]
    pub preview_interval: Option<Duration>,
    /// When that look-ahead window starts, if not now.
    #[serde(rename = "StartTime", default, skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime<FixedOffset>>,
    /// The monitoring point to report on.
    #[serde(rename = "MonitoringRef")]
    pub monitoring_ref: MonitoringRef,
    /// Only this operator's services.
    #[serde(rename = "OperatorRef", default, skip_serializing_if = "Option::is_none")]
    pub operator_ref: Option<OperatorRef>,
    /// Only this line.
    #[serde(rename = "LineRef", default, skip_serializing_if = "Option::is_none")]
    pub line_ref: Option<LineRef>,
    /// Only services running in this direction.
    #[serde(rename = "DirectionRef", default, skip_serializing_if = "Option::is_none")]
    pub direction_ref: Option<DirectionRef>,
    /// Only services bound for this destination.
    #[serde(rename = "DestinationRef", default, skip_serializing_if = "Option::is_none")]
    pub destination_ref: Option<DestinationRef>,
    /// Whether to report arrivals, departures or both.
    #[serde(rename = "StopVisitTypes", default, skip_serializing_if = "Option::is_none")]
    pub stop_visit_types: Option<StopVisitType>,
    /// Languages the requestor would like texts in, most preferred first.
    #[serde(rename = "Language", default, skip_serializing_if = "Vec::is_empty")]
    pub language: Vec<String>,
    /// Whether to include every translation of a text rather than only one.
    #[serde(rename = "IncludeTranslations", default, skip_serializing_if = "Option::is_none")]
    pub include_translations: Option<bool>,
    /// At most this many visits altogether.
    #[serde(rename = "MaximumStopVisits", default, skip_serializing_if = "Option::is_none")]
    pub maximum_stop_visits: Option<u64>,
    /// At least this many visits for each line, whatever the overall cap.
    #[serde(rename = "MinimumStopVisitsPerLine", default, skip_serializing_if = "Option::is_none")]
    pub minimum_stop_visits_per_line: Option<u64>,
    /// At least this many for each line and via point, instead of per line.
    #[serde(rename = "MinimumStopVisitsPerLineVia", default, skip_serializing_if = "Option::is_none")]
    pub minimum_stop_visits_per_line_via: Option<u64>,
    /// How long texts may be, for a display with a fixed width.
    #[serde(rename = "MaximumTextLength", default, skip_serializing_if = "Option::is_none")]
    pub maximum_text_length: Option<u64>,
    /// How much detail to give per visit.
    #[serde(rename = "StopMonitoringDetailLevel", default, skip_serializing_if = "Option::is_none")]
    pub stop_monitoring_detail_level: Option<StopMonitoringDetail>,
    /// Whether to include the situations affecting the services.
    #[serde(rename = "IncludeSituations", default, skip_serializing_if = "Option::is_none")]
    pub include_situations: Option<bool>,
    /// How many calls before and after this stop to include.
    #[serde(rename = "MaximumNumberOfCalls", default, skip_serializing_if = "Option::is_none")]
    pub maximum_number_of_calls: Option<MaximumNumberOfCalls>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl StopMonitoringFilter {
    /// An unfiltered filter for one monitoring point.
    pub fn at_stop(monitoring_ref: impl Into<MonitoringRef>) -> Self {
        Self {
            preview_interval: None,
            start_time: None,
            monitoring_ref: monitoring_ref.into(),
            operator_ref: None,
            line_ref: None,
            direction_ref: None,
            destination_ref: None,
            stop_visit_types: None,
            language: Vec::new(),
            include_translations: None,
            maximum_stop_visits: None,
            minimum_stop_visits_per_line: None,
            minimum_stop_visits_per_line_via: None,
            maximum_text_length: None,
            stop_monitoring_detail_level: None,
            include_situations: None,
            maximum_number_of_calls: None,
            extensions: None,
        }
    }
}

/// A subscription to what is due at a monitoring point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopMonitoringSubscriptionRequest {
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
    #[serde(rename = "StopMonitoringRequest")]
    pub stop_monitoring_request: StopMonitoringRequest,
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

impl StopMonitoringSubscriptionRequest {
    /// A subscription running until `initial_termination_time`.
    pub fn new(
        subscription_identifier: impl Into<SubscriptionQualifier>,
        initial_termination_time: DateTime<FixedOffset>,
        stop_monitoring_request: StopMonitoringRequest,
    ) -> Self {
        Self {
            subscriber_ref: None,
            subscription_identifier: subscription_identifier.into(),
            initial_termination_time,
            subscription_renewal: None,
            stop_monitoring_request,
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
    fn the_topic_is_written_before_the_policy() {
        let request = StopMonitoringRequest {
            preview_interval: Some(Duration::parse("PT10M").expect("valid duration")),
            line_ref: Some("LINE77".into()),
            stop_visit_types: Some(StopVisitType::All),
            maximum_stop_visits: Some(7),
            stop_monitoring_detail_level: Some(StopMonitoringDetail::Calls),
            ..StopMonitoringRequest::at_stop(timestamp(), "EH00001")
        };

        let xml = quick_xml::se::to_string_with_root("StopMonitoringRequest", &request)
            .expect("request serialises");
        let preview = xml.find("<PreviewInterval>").expect("the window is written");
        let monitoring = xml.find("<MonitoringRef>").expect("the topic is written");
        let visit_types = xml.find("<StopVisitTypes>").expect("the visit types are written");
        let maximum = xml.find("<MaximumStopVisits>").expect("the policy is written");
        assert!(preview < monitoring && monitoring < visit_types && visit_types < maximum, "{xml}");

        let read: StopMonitoringRequest =
            quick_xml::de::from_str(&xml).expect("request round-trips");
        assert_eq!(read, request);
    }

    #[test]
    fn a_multiple_request_keeps_one_filter_per_stop() {
        let request = StopMonitoringMultipleRequest::new(
            timestamp(),
            vec![
                StopMonitoringFilter::at_stop("EH00001"),
                StopMonitoringFilter {
                    maximum_stop_visits: Some(3),
                    ..StopMonitoringFilter::at_stop("EH00002")
                },
            ],
        );

        let xml = quick_xml::se::to_string_with_root("StopMonitoringMultipleRequest", &request)
            .expect("request serialises");
        assert_eq!(xml.matches("<StopMonitoringFIlter>").count(), 2, "{xml}");

        let read: StopMonitoringMultipleRequest =
            quick_xml::de::from_str(&xml).expect("request round-trips");
        assert_eq!(read, request);
    }
}
