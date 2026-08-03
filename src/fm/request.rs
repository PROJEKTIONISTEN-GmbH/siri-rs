//! Asking about the state of passenger facilities.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::model::{
    ConnectionLinkRef, FacilityRef, FeatureRef, FramedVehicleJourneyRef, InterchangeRef, LineRef,
    SiteRef, StopPlaceComponentRef, StopPlaceRef, StopPointRef, UserNeed, VehicleJourneyRef,
    VehicleRef,
};
use crate::types::{
    Duration, Extensions, MessageQualifier, ParticipantRef, SubscriptionQualifier,
};

/// A request for the state of the facilities at a place or on a service.
///
/// Every field of the topic narrows the answer; a request with none asks for
/// everything the producer watches.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FacilityMonitoringRequest {
    /// Version of SIRI-FM the request conforms to.
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
    /// Only these facilities.
    #[serde(rename = "FacilityRef", default, skip_serializing_if = "Vec::is_empty")]
    pub facility_ref: Vec<FacilityRef>,
    /// Only facilities offering these features.
    #[serde(rename = "FeatureRef", default, skip_serializing_if = "Vec::is_empty")]
    pub feature_ref: Vec<FeatureRef>,
    /// Only facilities serving this line.
    #[serde(rename = "LineRef", default, skip_serializing_if = "Option::is_none")]
    pub line_ref: Option<LineRef>,
    /// Only facilities at this stop.
    #[serde(rename = "StopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_point_ref: Option<StopPointRef>,
    /// Only facilities on this connection link.
    #[serde(rename = "ConnectionLinkRef", default, skip_serializing_if = "Option::is_none")]
    pub connection_link_ref: Option<ConnectionLinkRef>,
    /// Only facilities serving this journey on its operational day.
    #[serde(rename = "FramedVehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub framed_vehicle_journey_ref: Option<FramedVehicleJourneyRef>,
    /// Only facilities serving this journey, the operational day being understood.
    #[serde(rename = "VehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_journey_ref: Option<VehicleJourneyRef>,
    /// Only facilities serving this interchange.
    #[serde(rename = "InterchangeRef", default, skip_serializing_if = "Option::is_none")]
    pub interchange_ref: Option<InterchangeRef>,
    /// Only facilities aboard this vehicle.
    #[serde(rename = "VehicleRef", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_ref: Option<VehicleRef>,
    /// Only facilities in this stop place.
    #[serde(rename = "StopPlaceRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_place_ref: Option<StopPlaceRef>,
    /// Only facilities in this part of that stop place.
    #[serde(rename = "StopPlaceComponentRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_place_component_ref: Option<StopPlaceComponentRef>,
    /// Only facilities at this site.
    #[serde(rename = "SiteRef", default, skip_serializing_if = "Option::is_none")]
    pub site_ref: Option<SiteRef>,
    /// Only facilities that bear on these passenger needs.
    #[serde(rename = "AccessibilityNeedsFilter", default, skip_serializing_if = "Vec::is_empty")]
    pub accessibility_needs_filter: Vec<AccessibilityNeedsFilter>,
    /// Languages the requestor would like texts in, most preferred first.
    #[serde(rename = "Language", default, skip_serializing_if = "Vec::is_empty")]
    pub language: Vec<String>,
    /// Whether to include every translation of a text rather than only one.
    #[serde(rename = "IncludeTranslations", default, skip_serializing_if = "Option::is_none")]
    pub include_translations: Option<bool>,
    /// At most this many facility conditions.
    #[serde(
        rename = "MaximumNumberOfFacilityConditions",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub maximum_number_of_facility_conditions: Option<u64>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl FacilityMonitoringRequest {
    /// An unfiltered request for everything the producer watches.
    pub fn new(request_timestamp: DateTime<FixedOffset>) -> Self {
        Self {
            version: None,
            request_timestamp,
            message_identifier: None,
            preview_interval: None,
            start_time: None,
            facility_ref: Vec::new(),
            feature_ref: Vec::new(),
            line_ref: None,
            stop_point_ref: None,
            connection_link_ref: None,
            framed_vehicle_journey_ref: None,
            vehicle_journey_ref: None,
            interchange_ref: None,
            vehicle_ref: None,
            stop_place_ref: None,
            stop_place_component_ref: None,
            site_ref: None,
            accessibility_needs_filter: Vec::new(),
            language: Vec::new(),
            include_translations: None,
            maximum_number_of_facility_conditions: None,
            extensions: None,
        }
    }

    /// A request for the facilities at one stop.
    pub fn at_stop(
        request_timestamp: DateTime<FixedOffset>,
        stop_point_ref: impl Into<StopPointRef>,
    ) -> Self {
        Self {
            stop_point_ref: Some(stop_point_ref.into()),
            ..Self::new(request_timestamp)
        }
    }
}

/// The passenger needs a request cares about.
///
/// A consumer serving wheelchair users, say, asks only for the facilities whose
/// state bears on that need rather than for every broken ticket machine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessibilityNeedsFilter {
    /// The needs, at least one.
    #[serde(rename = "UserNeed")]
    pub user_need: Vec<UserNeed>,
}

impl AccessibilityNeedsFilter {
    /// A filter for the given needs.
    pub fn new(user_need: Vec<UserNeed>) -> Self {
        Self { user_need }
    }
}

/// A subscription to the state of passenger facilities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FacilityMonitoringSubscriptionRequest {
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
    #[serde(rename = "FacilityMonitoringRequest")]
    pub facility_monitoring_request: FacilityMonitoringRequest,
    /// Whether to send only what has changed rather than the full set each time.
    #[serde(rename = "IncrementalUpdates", default, skip_serializing_if = "Option::is_none")]
    pub incremental_updates: Option<bool>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl FacilityMonitoringSubscriptionRequest {
    /// A subscription running until `initial_termination_time`.
    pub fn new(
        subscription_identifier: impl Into<SubscriptionQualifier>,
        initial_termination_time: DateTime<FixedOffset>,
        facility_monitoring_request: FacilityMonitoringRequest,
    ) -> Self {
        Self {
            subscriber_ref: None,
            subscription_identifier: subscription_identifier.into(),
            initial_termination_time,
            subscription_renewal: None,
            facility_monitoring_request,
            incremental_updates: None,
            extensions: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enumerations::Mobility;
    use crate::model::UserNeedKind;

    fn timestamp() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2004-12-17T09:30:47-05:00").expect("valid timestamp")
    }

    #[test]
    fn the_topic_is_written_before_the_policy() {
        let request = FacilityMonitoringRequest {
            preview_interval: Some(Duration::parse("P10M").expect("valid duration")),
            maximum_number_of_facility_conditions: Some(20),
            ..FacilityMonitoringRequest::at_stop(timestamp(), "PLACE98765")
        };

        let xml = quick_xml::se::to_string_with_root("FacilityMonitoringRequest", &request)
            .expect("request serialises");
        let preview = xml.find("<PreviewInterval>").expect("the window is written");
        let stop = xml.find("<StopPointRef>").expect("the stop is written");
        let maximum = xml
            .find("<MaximumNumberOfFacilityConditions>")
            .expect("the policy is written");
        assert!(preview < stop && stop < maximum, "{xml}");

        let read: FacilityMonitoringRequest =
            quick_xml::de::from_str(&xml).expect("request round-trips");
        assert_eq!(read, request);
    }

    #[test]
    fn a_needs_filter_writes_its_needs_as_annex_content() {
        let request = FacilityMonitoringRequest {
            accessibility_needs_filter: vec![AccessibilityNeedsFilter::new(vec![UserNeed {
                excluded: Some(true),
                ..UserNeed::new(UserNeedKind::MobilityNeed(Mobility::Wheelchair))
            }])],
            ..FacilityMonitoringRequest::new(timestamp())
        };

        let xml = quick_xml::se::to_string_with_root("FacilityMonitoringRequest", &request)
            .expect("request serialises");
        assert!(xml.contains("<acsb:MobilityNeed>wheelchair</acsb:MobilityNeed>"), "{xml}");

        let read: FacilityMonitoringRequest =
            quick_xml::de::from_str(&xml).expect("request round-trips");
        assert_eq!(read, request);
    }
}
