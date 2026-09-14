//! Asking whether a connection will be made.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::ConnectionMonitoringDetail;
use crate::model::{ConnectionLinkRef, DatedVehicleJourneyRef, DirectionRef, LineRef};
use crate::types::{
    Duration, Extensions, MessageQualifier, ParticipantRef, SubscriptionQualifier,
};

/// A request for the connections being watched over one connection link.
///
/// The link alone does not say which connections are meant, so the request narrows
/// it either to named feeder journeys or to a line and a window of arrival times;
/// use [`ConnectionMonitoringRequest::scope`] to see which.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionMonitoringRequest {
    /// Version of SIRI-CM the request conforms to.
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
    /// The connection link to report on.
    #[serde(rename = "ConnectionLinkRef")]
    pub connection_link_ref: ConnectionLinkRef,
    /// Only feeders on one line, arriving within a window.
    #[serde(rename = "ConnectingTimeFilter", default, skip_serializing_if = "Option::is_none")]
    pub connecting_time_filter: Option<ConnectingTimeFilter>,
    /// Only these named feeder journeys, instead of a line and a window.
    #[serde(rename = "ConnectingJourneyFilter", default, skip_serializing_if = "Vec::is_empty")]
    pub connecting_journey_filter: Vec<ConnectingJourneyFilter>,
    /// Languages the requestor would like texts in, most preferred first.
    #[serde(rename = "Language", default, skip_serializing_if = "Vec::is_empty")]
    pub language: Vec<String>,
    /// Whether to include every translation of a text rather than only one.
    #[serde(rename = "IncludeTranslations", default, skip_serializing_if = "Option::is_none")]
    pub include_translations: Option<bool>,
    /// How much detail to give per connection.
    #[serde(
        rename = "ConnectionMonitoringDetailLevel",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub connection_monitoring_detail_level: Option<ConnectionMonitoringDetail>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// Which connections a [`ConnectionMonitoringRequest`] is narrowed to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConnectionScope<'a> {
    /// One line's feeders, arriving within a window.
    Time(&'a ConnectingTimeFilter),
    /// Named feeder journeys.
    Journeys(&'a [ConnectingJourneyFilter]),
}

impl ConnectionMonitoringRequest {
    /// A request for the named feeder journeys over one connection link.
    pub fn for_journeys(
        request_timestamp: DateTime<FixedOffset>,
        connection_link_ref: impl Into<ConnectionLinkRef>,
        connecting_journey_filter: Vec<ConnectingJourneyFilter>,
    ) -> Self {
        Self {
            version: None,
            request_timestamp,
            message_identifier: None,
            preview_interval: None,
            connection_link_ref: connection_link_ref.into(),
            connecting_time_filter: None,
            connecting_journey_filter,
            language: Vec::new(),
            include_translations: None,
            connection_monitoring_detail_level: None,
            extensions: None,
        }
    }

    /// A request for one line's feeders over one connection link.
    pub fn for_line(
        request_timestamp: DateTime<FixedOffset>,
        connection_link_ref: impl Into<ConnectionLinkRef>,
        connecting_time_filter: ConnectingTimeFilter,
    ) -> Self {
        Self {
            connecting_time_filter: Some(connecting_time_filter),
            ..Self::for_journeys(request_timestamp, connection_link_ref, Vec::new())
        }
    }

    /// Which alternative of the schema's choice this request carries, or `None`
    /// when it carries neither.
    pub fn scope(&self) -> Option<ConnectionScope<'_>> {
        self.connecting_time_filter
            .as_ref()
            .map(ConnectionScope::Time)
            .or_else(|| {
                (!self.connecting_journey_filter.is_empty())
                    .then_some(ConnectionScope::Journeys(&self.connecting_journey_filter))
            })
    }
}

/// One named feeder journey a request is interested in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectingJourneyFilter {
    /// The feeder journey, on the operational day the request is about.
    #[serde(rename = "DatedVehicleJourneyRef")]
    pub dated_vehicle_journey_ref: DatedVehicleJourneyRef,
    /// Which visit to the interchange stop this is.
    #[serde(rename = "VisitNumber", default, skip_serializing_if = "Option::is_none")]
    pub visit_number: Option<u64>,
    /// When the timetable has the feeder arriving.
    #[serde(rename = "TimetabledArrivalTime")]
    pub timetabled_arrival_time: DateTime<FixedOffset>,
}

impl ConnectingJourneyFilter {
    /// A filter for one feeder journey due at `timetabled_arrival_time`.
    pub fn new(
        dated_vehicle_journey_ref: impl Into<DatedVehicleJourneyRef>,
        timetabled_arrival_time: DateTime<FixedOffset>,
    ) -> Self {
        Self {
            dated_vehicle_journey_ref: dated_vehicle_journey_ref.into(),
            visit_number: None,
            timetabled_arrival_time,
        }
    }
}

/// One line's feeders, narrowed to those arriving within a window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectingTimeFilter {
    /// The line the feeders run on.
    #[serde(rename = "LineRef")]
    pub line_ref: LineRef,
    /// The direction they run in.
    #[serde(rename = "DirectionRef")]
    pub direction_ref: DirectionRef,
    /// The earliest arrival the requestor is interested in.
    #[serde(rename = "EarliestArrivalTime", default, skip_serializing_if = "Option::is_none")]
    pub earliest_arrival_time: Option<DateTime<FixedOffset>>,
    /// The latest arrival the requestor is interested in.
    #[serde(rename = "LatestArrivalTime", default, skip_serializing_if = "Option::is_none")]
    pub latest_arrival_time: Option<DateTime<FixedOffset>>,
}

impl ConnectingTimeFilter {
    /// A filter for one line and direction, over the whole preview interval.
    pub fn new(line_ref: impl Into<LineRef>, direction_ref: impl Into<DirectionRef>) -> Self {
        Self {
            line_ref: line_ref.into(),
            direction_ref: direction_ref.into(),
            earliest_arrival_time: None,
            latest_arrival_time: None,
        }
    }
}

/// A subscription to the connections being watched over a connection link.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionMonitoringSubscriptionRequest {
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
    #[serde(rename = "ConnectionMonitoringRequest")]
    pub connection_monitoring_request: ConnectionMonitoringRequest,
    /// How large a change has to be before it is worth a delivery.
    #[serde(rename = "ChangeBeforeUpdates", default, skip_serializing_if = "Option::is_none")]
    pub change_before_updates: Option<Duration>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl ConnectionMonitoringSubscriptionRequest {
    /// A subscription running until `initial_termination_time`.
    pub fn new(
        subscription_identifier: impl Into<SubscriptionQualifier>,
        initial_termination_time: DateTime<FixedOffset>,
        connection_monitoring_request: ConnectionMonitoringRequest,
    ) -> Self {
        Self {
            subscriber_ref: None,
            subscription_identifier: subscription_identifier.into(),
            initial_termination_time,
            subscription_renewal: None,
            connection_monitoring_request,
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
    fn a_request_reports_whether_it_narrows_by_journey_or_by_time() {
        let by_journey = ConnectionMonitoringRequest::for_journeys(
            timestamp(),
            "EH00001",
            vec![ConnectingJourneyFilter::new("ABC56789", timestamp())],
        );
        assert!(matches!(by_journey.scope(), Some(ConnectionScope::Journeys(f)) if f.len() == 1));

        let by_time = ConnectionMonitoringRequest::for_line(
            timestamp(),
            "EH00002",
            ConnectingTimeFilter::new("LINE77", "OUT"),
        );
        assert!(matches!(by_time.scope(), Some(ConnectionScope::Time(f)) if f.line_ref.as_str() == "LINE77"));

        let unscoped = ConnectionMonitoringRequest::for_journeys(timestamp(), "EH0", Vec::new());
        assert_eq!(unscoped.scope(), None);
    }

    #[test]
    fn the_topic_is_written_before_the_policy() {
        let request = ConnectionMonitoringRequest {
            preview_interval: Some(Duration::parse("PT10M").expect("valid duration")),
            connection_monitoring_detail_level: Some(ConnectionMonitoringDetail::Full),
            ..ConnectionMonitoringRequest::for_line(
                timestamp(),
                "EH00002",
                ConnectingTimeFilter::new("LINE77", "OUT"),
            )
        };

        let xml = quick_xml::se::to_string_with_root("ConnectionMonitoringRequest", &request)
            .expect("request serialises");
        let preview = xml.find("<PreviewInterval>").expect("the window is written");
        let link = xml.find("<ConnectionLinkRef>").expect("the link is written");
        let filter = xml.find("<ConnectingTimeFilter>").expect("the filter is written");
        let detail = xml
            .find("<ConnectionMonitoringDetailLevel>")
            .expect("the policy is written");
        assert!(preview < link && link < filter && filter < detail, "{xml}");

        let read: ConnectionMonitoringRequest =
            quick_xml::de::from_str(&xml).expect("request round-trips");
        assert_eq!(read, request);
    }
}
