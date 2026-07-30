//! Asking for a real-time timetable, once or by subscription.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::{EstimatedTimetableDetail, VehicleModesOfTransport};
use crate::model::{
    LineDirection, OperatorRef, ProductCategoryRef, RequestedLines, StopPointRef, VersionRef,
};
use crate::types::{
    Duration, Extensions, MessageQualifier, ParticipantRef, SubscriptionQualifier,
};

/// A request for the real-time timetable a producer holds.
///
/// Every field beyond the timestamp narrows the answer; a request with no filters
/// asks for everything the producer is willing to publish. Filters of different
/// kinds are combined with "and".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EstimatedTimetableRequest {
    /// Version of SIRI-ET the request conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// When the request was made.
    #[serde(rename = "RequestTimestamp")]
    pub request_timestamp: DateTime<FixedOffset>,
    /// Identifier the requestor puts on this request.
    #[serde(rename = "MessageIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub message_identifier: Option<MessageQualifier>,
    /// How far ahead to include departures.
    #[serde(rename = "PreviewInterval", default, skip_serializing_if = "Option::is_none")]
    pub preview_interval: Option<Duration>,
    /// Only journeys from this edition of the timetable.
    #[serde(rename = "TimetableVersionRef", default, skip_serializing_if = "Option::is_none")]
    pub timetable_version_ref: Option<VersionRef>,
    /// Only journeys run by these operators.
    #[serde(rename = "OperatorRef", default, skip_serializing_if = "Vec::is_empty")]
    pub operator_ref: Vec<OperatorRef>,
    /// Only journeys on these lines, each optionally narrowed to one direction.
    #[serde(rename = "Lines", default, skip_serializing_if = "Option::is_none")]
    pub lines: Option<RequestedLines>,
    /// Only journeys using these modes of transport.
    #[serde(
        rename = "VehicleMode",
        default,
        deserialize_with = "crate::xml::token_list::deserialize",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub vehicle_mode: Vec<VehicleModesOfTransport>,
    /// Only journeys in these commercial categories.
    #[serde(rename = "ProductCategoryRef", default, skip_serializing_if = "Vec::is_empty")]
    pub product_category_ref: Vec<ProductCategoryRef>,
    /// Only journeys calling at these stops.
    #[serde(rename = "StopPointRef", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_point_ref: Vec<StopPointRef>,
    /// Languages the requestor would like texts in, most preferred first.
    #[serde(rename = "Language", default, skip_serializing_if = "Vec::is_empty")]
    pub language: Vec<String>,
    /// Whether to include every translation of a text rather than only one.
    #[serde(rename = "IncludeTranslations", default, skip_serializing_if = "Option::is_none")]
    pub include_translations: Option<bool>,
    /// Whether to include the interchanges planned around the journeys.
    #[serde(rename = "IncludeInterchanges", default, skip_serializing_if = "Option::is_none")]
    pub include_interchanges: Option<bool>,
    /// Whether to include the relations to journeys joined, split or continued.
    #[serde(rename = "IncludeJourneyRelations", default, skip_serializing_if = "Option::is_none")]
    pub include_journey_relations: Option<bool>,
    /// Whether to include how the trains are put together.
    #[serde(rename = "IncludeTrainFormations", default, skip_serializing_if = "Option::is_none")]
    pub include_train_formations: Option<bool>,
    /// How much detail to give per journey.
    #[serde(rename = "EstimatedTimetableDetailLevel", default, skip_serializing_if = "Option::is_none")]
    pub estimated_timetable_detail_level: Option<EstimatedTimetableDetail>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl EstimatedTimetableRequest {
    /// An unfiltered request for everything the producer publishes.
    pub fn new(request_timestamp: DateTime<FixedOffset>) -> Self {
        Self {
            version: None,
            request_timestamp,
            message_identifier: None,
            preview_interval: None,
            timetable_version_ref: None,
            operator_ref: Vec::new(),
            lines: None,
            vehicle_mode: Vec::new(),
            product_category_ref: Vec::new(),
            stop_point_ref: Vec::new(),
            language: Vec::new(),
            include_translations: None,
            include_interchanges: None,
            include_journey_relations: None,
            include_train_formations: None,
            estimated_timetable_detail_level: None,
            extensions: None,
        }
    }

    /// The same request, narrowed to the given lines and directions.
    pub fn for_lines(mut self, line_direction: Vec<LineDirection>) -> Self {
        self.lines = Some(RequestedLines { line_direction });
        self
    }

    /// The lines this request filters on.
    pub fn line_directions(&self) -> &[LineDirection] {
        self.lines
            .as_ref()
            .map(|lines| lines.line_direction.as_slice())
            .unwrap_or_default()
    }
}

/// A subscription to the real-time timetable a producer holds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EstimatedTimetableSubscriptionRequest {
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
    #[serde(rename = "EstimatedTimetableRequest")]
    pub estimated_timetable_request: EstimatedTimetableRequest,
    /// Whether to send only what has changed rather than the full set each time.
    #[serde(rename = "IncrementalUpdates", default, skip_serializing_if = "Option::is_none")]
    pub incremental_updates: Option<bool>,
    /// How large a change has to be before it is worth a delivery.
    #[serde(rename = "ChangeBeforeUpdates", default, skip_serializing_if = "Option::is_none")]
    pub change_before_updates: Option<Duration>,
    /// Whether to leave out updates that only restate calls already made.
    #[serde(rename = "SkipRecordedCallUpdates", default, skip_serializing_if = "Option::is_none")]
    pub skip_recorded_call_updates: Option<bool>,
    /// Whether to send *only* the updates to calls already made.
    #[serde(rename = "IncludeOnlyRecordedCallUpdates", default, skip_serializing_if = "Option::is_none")]
    pub include_only_recorded_call_updates: Option<bool>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl EstimatedTimetableSubscriptionRequest {
    /// A subscription running until `initial_termination_time`.
    pub fn new(
        subscription_identifier: impl Into<SubscriptionQualifier>,
        initial_termination_time: DateTime<FixedOffset>,
        estimated_timetable_request: EstimatedTimetableRequest,
    ) -> Self {
        Self {
            subscriber_ref: None,
            subscription_identifier: subscription_identifier.into(),
            initial_termination_time,
            subscription_renewal: None,
            estimated_timetable_request,
            incremental_updates: None,
            change_before_updates: None,
            skip_recorded_call_updates: None,
            include_only_recorded_call_updates: None,
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
    fn an_unfiltered_request_writes_only_its_timestamp() {
        let request = EstimatedTimetableRequest::new(timestamp());
        let xml = quick_xml::se::to_string_with_root("EstimatedTimetableRequest", &request)
            .expect("request serialises");
        assert_eq!(
            xml,
            "<EstimatedTimetableRequest><RequestTimestamp>2004-12-17T09:30:47-05:00\
             </RequestTimestamp></EstimatedTimetableRequest>"
        );
    }

    #[test]
    fn filters_are_written_in_schema_order() {
        let mut request = EstimatedTimetableRequest::new(timestamp())
            .for_lines(vec![LineDirection::with_direction("123", "INBOUND")]);
        request.preview_interval = Some(Duration::parse("PT30M").expect("valid duration"));
        request.operator_ref = vec![OperatorRef::new("SMOOTH")];
        request.estimated_timetable_detail_level = Some(EstimatedTimetableDetail::Calls);

        let xml = quick_xml::se::to_string_with_root("EstimatedTimetableRequest", &request)
            .expect("request serialises");
        let preview = xml.find("<PreviewInterval>").expect("preview is written");
        let operator = xml.find("<OperatorRef>").expect("operator is written");
        let lines = xml.find("<Lines>").expect("lines are written");
        let detail = xml
            .find("<EstimatedTimetableDetailLevel>")
            .expect("detail level is written");
        assert!(preview < operator && operator < lines && lines < detail, "{xml}");

        let read: EstimatedTimetableRequest =
            quick_xml::de::from_str(&xml).expect("request round-trips");
        assert_eq!(read, request);
        assert_eq!(read.line_directions().len(), 1);
    }

    #[test]
    fn a_subscription_carries_the_request_it_subscribes_to() {
        let subscription = EstimatedTimetableSubscriptionRequest::new(
            "000765",
            timestamp(),
            EstimatedTimetableRequest::new(timestamp()),
        );
        let xml = quick_xml::se::to_string_with_root(
            "EstimatedTimetableSubscriptionRequest",
            &subscription,
        )
        .expect("subscription serialises");
        assert!(xml.contains("<SubscriptionIdentifier>000765</SubscriptionIdentifier>"), "{xml}");
        assert!(xml.contains("<EstimatedTimetableRequest>"), "{xml}");

        let read: EstimatedTimetableSubscriptionRequest =
            quick_xml::de::from_str(&xml).expect("subscription round-trips");
        assert_eq!(read, subscription);
    }
}
