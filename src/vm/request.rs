//! Asking where the vehicles are, once or by subscription.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::VehicleMonitoringDetail;
use crate::model::{DirectionRef, LineRef, VehicleRef};
use crate::types::{Duration, Extensions, MessageQualifier, ParticipantRef, SubscriptionQualifier};

use super::VehicleMonitoringRef;

/// A request for the vehicles a producer is tracking.
///
/// The topic is either a vehicle-monitoring service the producer publishes under, a
/// single vehicle, or a line; the policy fields cap how much comes back.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleMonitoringRequest {
    /// Version of SIRI-VM the request conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// When the request was made.
    #[serde(rename = "RequestTimestamp")]
    pub request_timestamp: DateTime<FixedOffset>,
    /// Identifier the requestor puts on this request.
    #[serde(rename = "MessageIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub message_identifier: Option<MessageQualifier>,
    /// The producer's vehicle-monitoring service to draw from.
    #[serde(rename = "VehicleMonitoringRef", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_monitoring_ref: Option<VehicleMonitoringRef>,
    /// Only this vehicle.
    #[serde(rename = "VehicleRef", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_ref: Option<VehicleRef>,
    /// Only vehicles on this line, instead of a single vehicle.
    #[serde(rename = "LineRef", default, skip_serializing_if = "Option::is_none")]
    pub line_ref: Option<LineRef>,
    /// Only vehicles running in this direction.
    #[serde(rename = "DirectionRef", default, skip_serializing_if = "Option::is_none")]
    pub direction_ref: Option<DirectionRef>,
    /// Languages the requestor would like texts in, most preferred first.
    #[serde(rename = "Language", default, skip_serializing_if = "Vec::is_empty")]
    pub language: Vec<String>,
    /// Whether to include every translation of a text rather than only one.
    #[serde(rename = "IncludeTranslations", default, skip_serializing_if = "Option::is_none")]
    pub include_translations: Option<bool>,
    /// At most this many vehicles.
    #[serde(rename = "MaximumVehicles", default, skip_serializing_if = "Option::is_none")]
    pub maximum_vehicles: Option<u64>,
    /// How much detail to give per vehicle.
    #[serde(rename = "VehicleMonitoringDetailLevel", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_monitoring_detail_level: Option<VehicleMonitoringDetail>,
    /// How many calls before and after the current one to include.
    #[serde(rename = "MaximumNumberOfCalls", default, skip_serializing_if = "Option::is_none")]
    pub maximum_number_of_calls: Option<MaximumNumberOfCalls>,
    /// Whether to include the situations affecting the vehicles.
    #[serde(rename = "IncludeSituations", default, skip_serializing_if = "Option::is_none")]
    pub include_situations: Option<bool>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// Which vehicles a [`VehicleMonitoringRequest`] is narrowed to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitoredSubject<'a> {
    /// One named vehicle.
    Vehicle(&'a VehicleRef),
    /// Every vehicle on one line.
    Line(&'a LineRef),
}

impl VehicleMonitoringRequest {
    /// An unfiltered request for everything the producer is tracking.
    pub fn new(request_timestamp: DateTime<FixedOffset>) -> Self {
        Self {
            version: None,
            request_timestamp,
            message_identifier: None,
            vehicle_monitoring_ref: None,
            vehicle_ref: None,
            line_ref: None,
            direction_ref: None,
            language: Vec::new(),
            include_translations: None,
            maximum_vehicles: None,
            vehicle_monitoring_detail_level: None,
            maximum_number_of_calls: None,
            include_situations: None,
            extensions: None,
        }
    }

    /// A request for one named vehicle.
    pub fn for_vehicle(
        request_timestamp: DateTime<FixedOffset>,
        vehicle_ref: impl Into<VehicleRef>,
    ) -> Self {
        Self {
            vehicle_ref: Some(vehicle_ref.into()),
            ..Self::new(request_timestamp)
        }
    }

    /// A request for every vehicle on one line.
    pub fn for_line(
        request_timestamp: DateTime<FixedOffset>,
        line_ref: impl Into<LineRef>,
    ) -> Self {
        Self {
            line_ref: Some(line_ref.into()),
            ..Self::new(request_timestamp)
        }
    }

    /// Which alternative of the schema's choice this request carries, or `None`
    /// when it narrows to neither a vehicle nor a line.
    pub fn subject(&self) -> Option<MonitoredSubject<'_>> {
        self.vehicle_ref
            .as_ref()
            .map(MonitoredSubject::Vehicle)
            .or_else(|| self.line_ref.as_ref().map(MonitoredSubject::Line))
    }
}

/// How many calls before and after the current one a request asks for.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaximumNumberOfCalls {
    /// At most this many stops already served.
    #[serde(rename = "Previous", default, skip_serializing_if = "Option::is_none")]
    pub previous: Option<u64>,
    /// At most this many stops still to come.
    #[serde(rename = "Onwards", default, skip_serializing_if = "Option::is_none")]
    pub onwards: Option<u64>,
}

/// A subscription to the vehicles a producer is tracking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleMonitoringSubscriptionRequest {
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
    #[serde(rename = "VehicleMonitoringRequest")]
    pub vehicle_monitoring_request: VehicleMonitoringRequest,
    /// Whether to send only what has changed rather than the full set each time.
    #[serde(rename = "IncrementalUpdates", default, skip_serializing_if = "Option::is_none")]
    pub incremental_updates: Option<bool>,
    /// How large a change has to be before it is worth a delivery.
    #[serde(rename = "ChangeBeforeUpdates", default, skip_serializing_if = "Option::is_none")]
    pub change_before_updates: Option<Duration>,
    /// How often to send an update regardless of change, instead of a threshold.
    #[serde(rename = "UpdateInterval", default, skip_serializing_if = "Option::is_none")]
    pub update_interval: Option<Duration>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl VehicleMonitoringSubscriptionRequest {
    /// A subscription running until `initial_termination_time`.
    pub fn new(
        subscription_identifier: impl Into<SubscriptionQualifier>,
        initial_termination_time: DateTime<FixedOffset>,
        vehicle_monitoring_request: VehicleMonitoringRequest,
    ) -> Self {
        Self {
            subscriber_ref: None,
            subscription_identifier: subscription_identifier.into(),
            initial_termination_time,
            subscription_renewal: None,
            vehicle_monitoring_request,
            incremental_updates: None,
            change_before_updates: None,
            update_interval: None,
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
    fn a_request_reports_whether_it_narrows_to_a_vehicle_or_a_line() {
        let vehicle = VehicleMonitoringRequest::for_vehicle(timestamp(), "VEH154");
        assert!(matches!(vehicle.subject(), Some(MonitoredSubject::Vehicle(v)) if v.as_str() == "VEH154"));

        let line = VehicleMonitoringRequest::for_line(timestamp(), "10");
        assert!(matches!(line.subject(), Some(MonitoredSubject::Line(l)) if l.as_str() == "10"));

        assert_eq!(VehicleMonitoringRequest::new(timestamp()).subject(), None);
    }

    #[test]
    fn the_topic_is_written_before_the_policy() {
        let mut request = VehicleMonitoringRequest::for_vehicle(timestamp(), "VEH154");
        request.vehicle_monitoring_ref = Some(VehicleMonitoringRef::new("VEHPT55"));
        request.direction_ref = Some(crate::model::DirectionRef::new("Out"));
        request.vehicle_monitoring_detail_level = Some(VehicleMonitoringDetail::Normal);
        request.maximum_vehicles = Some(20);

        let xml = quick_xml::se::to_string_with_root("VehicleMonitoringRequest", &request)
            .expect("request serialises");
        let monitoring = xml.find("<VehicleMonitoringRef>").expect("topic is written");
        let vehicle = xml.find("<VehicleRef>").expect("vehicle is written");
        let direction = xml.find("<DirectionRef>").expect("direction is written");
        let maximum = xml.find("<MaximumVehicles>").expect("policy is written");
        assert!(monitoring < vehicle && vehicle < direction && direction < maximum, "{xml}");

        let read: VehicleMonitoringRequest =
            quick_xml::de::from_str(&xml).expect("request round-trips");
        assert_eq!(read, request);
    }

    #[test]
    fn a_subscription_carries_the_request_it_subscribes_to() {
        let subscription = VehicleMonitoringSubscriptionRequest::new(
            "00000456",
            timestamp(),
            VehicleMonitoringRequest::new(timestamp()),
        );
        let xml =
            quick_xml::se::to_string_with_root("VehicleMonitoringSubscriptionRequest", &subscription)
                .expect("subscription serialises");
        assert!(xml.contains("<SubscriptionIdentifier>00000456</SubscriptionIdentifier>"), "{xml}");

        let read: VehicleMonitoringSubscriptionRequest =
            quick_xml::de::from_str(&xml).expect("subscription round-trips");
        assert_eq!(read, subscription);
    }
}
