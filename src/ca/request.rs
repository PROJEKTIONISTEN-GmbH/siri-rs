//! Asking what a control room has decided, once or by subscription.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::model::{LineRef, OperationalUnitRef, OperatorRef, RequestedLines};
use crate::types::{
    Duration, Extensions, MessageQualifier, ParticipantRef, SubscriptionQualifier,
};

/// A request for the control actions a producer holds.
///
/// The topic fields narrow which actions come back — by operator, by part of the
/// organisation, by network or by line — and the policy fields cap how much comes
/// back at once. Filters of different kinds are combined with "and".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlActionRequest {
    /// Version of SIRI-CA the request conforms to.
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
    /// Only this operator's actions.
    #[serde(rename = "OperatorRef", default, skip_serializing_if = "Option::is_none")]
    pub operator_ref: Option<OperatorRef>,
    /// Only actions taken by these parts of the organisation.
    #[serde(rename = "OperationalUnitRef", default, skip_serializing_if = "Vec::is_empty")]
    pub operational_unit_ref: Vec<OperationalUnitRef>,
    /// Only actions on this network.
    #[serde(rename = "NetworkRef", default, skip_serializing_if = "Option::is_none")]
    pub network_ref: Option<OperatorRef>,
    /// Only actions on these lines.
    #[serde(rename = "LineRef", default, skip_serializing_if = "Vec::is_empty")]
    pub line_ref: Vec<LineRef>,
    /// Only actions on these lines, each narrowed to one direction.
    #[serde(rename = "Lines", default, skip_serializing_if = "Option::is_none")]
    pub lines: Option<RequestedLines>,
    /// Whether to include the messages exchanged with drivers.
    #[serde(rename = "IncludeDriverMessages", default, skip_serializing_if = "Option::is_none")]
    pub include_driver_messages: Option<bool>,
    /// Whether to include vehicles detected by trackside equipment.
    #[serde(rename = "IncludeVehicleDetectings", default, skip_serializing_if = "Option::is_none")]
    pub include_vehicle_detectings: Option<bool>,
    /// Languages the requestor would like texts in, most preferred first.
    #[serde(rename = "Language", default, skip_serializing_if = "Vec::is_empty")]
    pub language: Vec<String>,
    /// Whether to include every translation of a text rather than only one.
    #[serde(rename = "IncludeTranslations", default, skip_serializing_if = "Option::is_none")]
    pub include_translations: Option<bool>,
    /// Whether to include the situations the actions belong to.
    #[serde(rename = "IncludeSituations", default, skip_serializing_if = "Option::is_none")]
    pub include_situations: Option<bool>,
    /// At most this many actions.
    #[serde(
        rename = "MaximumNumberOfControlActions",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub maximum_number_of_control_actions: Option<u64>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl ControlActionRequest {
    /// An unfiltered request for everything the producer publishes.
    pub fn new(request_timestamp: DateTime<FixedOffset>) -> Self {
        Self {
            version: None,
            request_timestamp,
            message_identifier: None,
            preview_interval: None,
            start_time: None,
            operator_ref: None,
            operational_unit_ref: Vec::new(),
            network_ref: None,
            line_ref: Vec::new(),
            lines: None,
            include_driver_messages: None,
            include_vehicle_detectings: None,
            language: Vec::new(),
            include_translations: None,
            include_situations: None,
            maximum_number_of_control_actions: None,
            extensions: None,
        }
    }

    /// Which of the two ways of naming lines this request uses, if either.
    pub fn line_scope(&self) -> Option<LineScope<'_>> {
        if !self.line_ref.is_empty() {
            return Some(LineScope::Lines(&self.line_ref));
        }
        self.lines.as_ref().map(LineScope::Directions)
    }
}

/// How a [`ControlActionRequest`] names the lines it is interested in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineScope<'a> {
    /// Whole lines.
    Lines(&'a [LineRef]),
    /// Lines narrowed to one direction each.
    Directions(&'a RequestedLines),
}

/// A request covering several topics at once, each with its own filter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlActionMultipleRequest {
    /// Version of SIRI-CA the request conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// When the request was made.
    #[serde(rename = "RequestTimestamp")]
    pub request_timestamp: DateTime<FixedOffset>,
    /// Identifier the requestor puts on this request.
    #[serde(rename = "MessageIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub message_identifier: Option<MessageQualifier>,
    /// One filter per topic, at least one.
    #[serde(rename = "ControlActionFilter")]
    pub control_action_filter: Vec<ControlActionFilter>,
}

impl ControlActionMultipleRequest {
    /// A request carrying the given filters.
    pub fn new(
        request_timestamp: DateTime<FixedOffset>,
        control_action_filter: Vec<ControlActionFilter>,
    ) -> Self {
        Self {
            version: None,
            request_timestamp,
            message_identifier: None,
            control_action_filter,
        }
    }
}

/// One topic within a [`ControlActionMultipleRequest`], with its filters.
///
/// The fields are those of a [`ControlActionRequest`] without the request's own
/// timestamp and identifier, which the enclosing request carries.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ControlActionFilter {
    /// How far ahead to look.
    #[serde(rename = "PreviewInterval", default, skip_serializing_if = "Option::is_none")]
    pub preview_interval: Option<Duration>,
    /// When that look-ahead window starts, if not now.
    #[serde(rename = "StartTime", default, skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime<FixedOffset>>,
    /// Only this operator's actions.
    #[serde(rename = "OperatorRef", default, skip_serializing_if = "Option::is_none")]
    pub operator_ref: Option<OperatorRef>,
    /// Only actions taken by these parts of the organisation.
    #[serde(rename = "OperationalUnitRef", default, skip_serializing_if = "Vec::is_empty")]
    pub operational_unit_ref: Vec<OperationalUnitRef>,
    /// Only actions on this network.
    #[serde(rename = "NetworkRef", default, skip_serializing_if = "Option::is_none")]
    pub network_ref: Option<OperatorRef>,
    /// Only actions on these lines.
    #[serde(rename = "LineRef", default, skip_serializing_if = "Vec::is_empty")]
    pub line_ref: Vec<LineRef>,
    /// Only actions on these lines, each narrowed to one direction.
    #[serde(rename = "Lines", default, skip_serializing_if = "Option::is_none")]
    pub lines: Option<RequestedLines>,
    /// Whether to include the messages exchanged with drivers.
    #[serde(rename = "IncludeDriverMessages", default, skip_serializing_if = "Option::is_none")]
    pub include_driver_messages: Option<bool>,
    /// Whether to include vehicles detected by trackside equipment.
    #[serde(rename = "IncludeVehicleDetectings", default, skip_serializing_if = "Option::is_none")]
    pub include_vehicle_detectings: Option<bool>,
    /// Languages the requestor would like texts in, most preferred first.
    #[serde(rename = "Language", default, skip_serializing_if = "Vec::is_empty")]
    pub language: Vec<String>,
    /// Whether to include every translation of a text rather than only one.
    #[serde(rename = "IncludeTranslations", default, skip_serializing_if = "Option::is_none")]
    pub include_translations: Option<bool>,
    /// Whether to include the situations the actions belong to.
    #[serde(rename = "IncludeSituations", default, skip_serializing_if = "Option::is_none")]
    pub include_situations: Option<bool>,
    /// At most this many actions.
    #[serde(
        rename = "MaximumNumberOfControlActions",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub maximum_number_of_control_actions: Option<u64>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl ControlActionFilter {
    /// An unfiltered filter, narrowed to one operator.
    pub fn for_operator(operator_ref: impl Into<OperatorRef>) -> Self {
        Self {
            operator_ref: Some(operator_ref.into()),
            ..Self::default()
        }
    }
}

/// A subscription to what a control room decides.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlActionSubscriptionRequest {
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
    #[serde(rename = "ControlActionRequest")]
    pub control_action_request: ControlActionRequest,
    /// Whether to send only what has changed rather than the full set each time.
    #[serde(rename = "IncrementalUpdates", default, skip_serializing_if = "Option::is_none")]
    pub incremental_updates: Option<bool>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl ControlActionSubscriptionRequest {
    /// A subscription running until `initial_termination_time`.
    pub fn new(
        subscription_identifier: impl Into<SubscriptionQualifier>,
        initial_termination_time: DateTime<FixedOffset>,
        control_action_request: ControlActionRequest,
    ) -> Self {
        Self {
            subscriber_ref: None,
            subscription_identifier: subscription_identifier.into(),
            initial_termination_time,
            subscription_renewal: None,
            control_action_request,
            incremental_updates: None,
            extensions: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::LineDirection;

    fn timestamp() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2026-03-04T08:15:00+01:00").expect("valid timestamp")
    }

    #[test]
    fn the_topic_is_written_before_the_policy() {
        let request = ControlActionRequest {
            operator_ref: Some("USTRA".into()),
            line_ref: vec!["10".into()],
            include_driver_messages: Some(true),
            maximum_number_of_control_actions: Some(20),
            ..ControlActionRequest::new(timestamp())
        };

        let xml = quick_xml::se::to_string_with_root("ControlActionRequest", &request)
            .expect("request serialises");
        let operator = xml.find("<OperatorRef>").expect("the topic is written");
        let messages = xml
            .find("<IncludeDriverMessages>")
            .expect("the content choice is written");
        let maximum = xml
            .find("<MaximumNumberOfControlActions>")
            .expect("the policy is written");
        assert!(operator < messages && messages < maximum, "{xml}");

        let read: ControlActionRequest =
            quick_xml::de::from_str(&xml).expect("request round-trips");
        assert_eq!(read, request);
    }

    #[test]
    fn a_request_reports_how_it_names_the_lines_it_wants() {
        let whole_lines = ControlActionRequest {
            line_ref: vec!["10".into(), "17".into()],
            ..ControlActionRequest::new(timestamp())
        };
        assert!(matches!(
            whole_lines.line_scope(),
            Some(LineScope::Lines(lines)) if lines.len() == 2
        ));

        let one_direction = ControlActionRequest {
            lines: Some(RequestedLines {
                line_direction: vec![LineDirection::new("10")],
            }),
            ..ControlActionRequest::new(timestamp())
        };
        assert!(matches!(
            one_direction.line_scope(),
            Some(LineScope::Directions(_))
        ));
        assert_eq!(ControlActionRequest::new(timestamp()).line_scope(), None);
    }

    #[test]
    fn a_multiple_request_keeps_one_filter_per_topic() {
        let request = ControlActionMultipleRequest::new(
            timestamp(),
            vec![
                ControlActionFilter::for_operator("USTRA"),
                ControlActionFilter::for_operator("REGIOBUS"),
            ],
        );

        let xml = quick_xml::se::to_string_with_root("ControlActionMultipleRequest", &request)
            .expect("request serialises");
        assert_eq!(xml.matches("<ControlActionFilter>").count(), 2, "{xml}");
        assert_eq!(
            quick_xml::de::from_str::<ControlActionMultipleRequest>(&xml)
                .expect("request round-trips"),
            request
        );
    }
}
