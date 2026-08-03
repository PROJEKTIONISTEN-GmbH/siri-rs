//! Delivering the timetable at a stop.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::VehicleModesOfTransport;
use crate::framework::{ErrorCondition, ServiceRequestError};
use crate::model::{
    Branding, BrandingRef, DirectionRef, FramedVehicleJourneyRef, GroupOfLinesRef,
    JourneyPatternRef, LineRef, MonitoringRef, RouteRef, TargetedVehicleJourney,
};
use crate::types::{
    Duration, EndpointAddress, Extensions, ItemIdentifier, ItemRef, MessageRef,
    NaturalLanguageString, ParticipantRef, SubscriptionFilterRef, SubscriptionRef,
};

/// The timetable at one monitoring point.
///
/// A delivery either answers a
/// [`StopTimetableRequest`](crate::st::StopTimetableRequest) — in which case it
/// quotes the request's identifier — or satisfies a subscription, in which case it
/// quotes the subscription's.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopTimetableDelivery {
    /// Version of SIRI-ST the delivery conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// When the delivery was made.
    #[serde(rename = "ResponseTimestamp")]
    pub response_timestamp: DateTime<FixedOffset>,
    /// The request this delivery answers.
    #[serde(rename = "RequestMessageRef", default, skip_serializing_if = "Option::is_none")]
    pub request_message_ref: Option<MessageRef>,
    /// Who holds the subscription this delivery satisfies.
    #[serde(rename = "SubscriberRef", default, skip_serializing_if = "Option::is_none")]
    pub subscriber_ref: Option<ParticipantRef>,
    /// The shared filter the subscription uses.
    #[serde(rename = "SubscriptionFilterRef", default, skip_serializing_if = "Option::is_none")]
    pub subscription_filter_ref: Option<SubscriptionFilterRef>,
    /// The subscription this delivery satisfies.
    #[serde(rename = "SubscriptionRef", default, skip_serializing_if = "Option::is_none")]
    pub subscription_ref: Option<SubscriptionRef>,
    /// Address of the participant the data is delivered on behalf of.
    #[serde(rename = "DelegatorAddress", default, skip_serializing_if = "Option::is_none")]
    pub delegator_address: Option<EndpointAddress>,
    /// Participant the data is delivered on behalf of.
    #[serde(rename = "DelegatorRef", default, skip_serializing_if = "Option::is_none")]
    pub delegator_ref: Option<ParticipantRef>,
    /// Whether the request or subscription was processed successfully.
    #[serde(rename = "Status", default, skip_serializing_if = "Option::is_none")]
    pub status: Option<bool>,
    /// Why it could not be processed.
    #[serde(rename = "ErrorCondition", default, skip_serializing_if = "Option::is_none")]
    pub error_condition: Option<ErrorCondition<ServiceRequestError>>,
    /// How long this delivery holds.
    #[serde(rename = "ValidUntil", default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<FixedOffset>>,
    /// The shortest interval at which the producer will send updates.
    #[serde(rename = "ShortestPossibleCycle", default, skip_serializing_if = "Option::is_none")]
    pub shortest_possible_cycle: Option<Duration>,
    /// The language texts are in unless a visit says otherwise.
    #[serde(rename = "DefaultLanguage", default, skip_serializing_if = "Option::is_none")]
    pub default_language: Option<String>,
    /// The timetabled visits.
    #[serde(rename = "TimetabledStopVisit", default, skip_serializing_if = "Vec::is_empty")]
    pub timetabled_stop_visit: Vec<TimetabledStopVisit>,
    /// Visits the producer previously published and is now withdrawing.
    #[serde(rename = "TimetabledStopVisitCancellation", default, skip_serializing_if = "Vec::is_empty")]
    pub timetabled_stop_visit_cancellation: Vec<TimetabledStopVisitCancellation>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl StopTimetableDelivery {
    /// A delivery carrying the given timetabled visits.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        timetabled_stop_visit: Vec<TimetabledStopVisit>,
    ) -> Self {
        Self {
            version: None,
            response_timestamp,
            request_message_ref: None,
            subscriber_ref: None,
            subscription_filter_ref: None,
            subscription_ref: None,
            delegator_address: None,
            delegator_ref: None,
            status: None,
            error_condition: None,
            valid_until: None,
            shortest_possible_cycle: None,
            default_language: None,
            timetabled_stop_visit,
            timetabled_stop_visit_cancellation: Vec::new(),
            extensions: None,
        }
    }
}

/// One service the timetable says will call at a monitoring point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimetabledStopVisit {
    /// When the producer last knew this to be true.
    #[serde(rename = "RecordedAtTime")]
    pub recorded_at_time: DateTime<FixedOffset>,
    /// The producer's identifier for this record, so that it can be superseded.
    #[serde(rename = "ItemIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub item_identifier: Option<ItemIdentifier>,
    /// The monitoring point the visit is at.
    #[serde(rename = "MonitoringRef")]
    pub monitoring_ref: MonitoringRef,
    /// The journey making the visit, as the timetable has it.
    #[serde(rename = "TargetedVehicleJourney")]
    pub targeted_vehicle_journey: TargetedVehicleJourney,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl TimetabledStopVisit {
    /// A visit by the given journey at the given monitoring point.
    pub fn new(
        recorded_at_time: DateTime<FixedOffset>,
        monitoring_ref: impl Into<MonitoringRef>,
        targeted_vehicle_journey: TargetedVehicleJourney,
    ) -> Self {
        Self {
            recorded_at_time,
            item_identifier: None,
            monitoring_ref: monitoring_ref.into(),
            targeted_vehicle_journey,
            extensions: None,
        }
    }
}

/// A timetabled visit the producer is withdrawing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimetabledStopVisitCancellation {
    /// When the withdrawal was decided.
    #[serde(rename = "RecordedAtTime")]
    pub recorded_at_time: DateTime<FixedOffset>,
    /// The record being withdrawn.
    #[serde(rename = "ItemRef", default, skip_serializing_if = "Option::is_none")]
    pub item_ref: Option<ItemRef>,
    /// The monitoring point the withdrawn visit was at.
    #[serde(rename = "MonitoringRef")]
    pub monitoring_ref: MonitoringRef,
    /// Which visit to that point it was, when the journey calls more than once.
    #[serde(rename = "VisitNumber", default, skip_serializing_if = "Option::is_none")]
    pub visit_number: Option<u64>,
    /// The line the withdrawn visit was on.
    #[serde(rename = "LineRef")]
    pub line_ref: LineRef,
    /// The direction it ran in.
    #[serde(rename = "DirectionRef")]
    pub direction_ref: DirectionRef,
    /// The timetabled journey on its operational day.
    #[serde(rename = "FramedVehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub framed_vehicle_journey_ref: Option<FramedVehicleJourneyRef>,
    /// The journey pattern the journey follows.
    #[serde(rename = "JourneyPatternRef", default, skip_serializing_if = "Option::is_none")]
    pub journey_pattern_ref: Option<JourneyPatternRef>,
    /// The journey pattern's name.
    #[serde(rename = "JourneyPatternName", default, skip_serializing_if = "Option::is_none")]
    pub journey_pattern_name: Option<NaturalLanguageString>,
    /// The modes of transport it uses.
    #[serde(
        rename = "VehicleMode",
        default,
        deserialize_with = "crate::xml::token_list::deserialize",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub vehicle_mode: Vec<VehicleModesOfTransport>,
    /// The route it follows.
    #[serde(rename = "RouteRef", default, skip_serializing_if = "Option::is_none")]
    pub route_ref: Option<RouteRef>,
    /// The line's name as shown to passengers, one per language.
    #[serde(rename = "PublishedLineName", default, skip_serializing_if = "Vec::is_empty")]
    pub published_line_name: Vec<NaturalLanguageString>,
    /// The group of lines the line is marketed within.
    #[serde(rename = "GroupOfLinesRef", default, skip_serializing_if = "Option::is_none")]
    pub group_of_lines_ref: Option<GroupOfLinesRef>,
    /// The direction's name as shown to passengers, one per language.
    #[serde(rename = "DirectionName", default, skip_serializing_if = "Vec::is_empty")]
    pub direction_name: Vec<NaturalLanguageString>,
    /// The line as another operator's network identifies it.
    #[serde(rename = "ExternalLineRef", default, skip_serializing_if = "Option::is_none")]
    pub external_line_ref: Option<LineRef>,
    /// The brand the service is presented under, stated elsewhere.
    #[serde(rename = "BrandingRef", default, skip_serializing_if = "Option::is_none")]
    pub branding_ref: Option<BrandingRef>,
    /// The brand the service is presented under, stated here.
    #[serde(rename = "Branding", default, skip_serializing_if = "Option::is_none")]
    pub branding: Option<Branding>,
    /// Why the visit is being withdrawn, one text per language.
    #[serde(rename = "Reason", default, skip_serializing_if = "Vec::is_empty")]
    pub reason: Vec<NaturalLanguageString>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl TimetabledStopVisitCancellation {
    /// A withdrawal of the visit the given journey was to make at the given point.
    pub fn new(
        recorded_at_time: DateTime<FixedOffset>,
        monitoring_ref: impl Into<MonitoringRef>,
        line_ref: impl Into<LineRef>,
        direction_ref: impl Into<DirectionRef>,
    ) -> Self {
        Self {
            recorded_at_time,
            item_ref: None,
            monitoring_ref: monitoring_ref.into(),
            visit_number: None,
            line_ref: line_ref.into(),
            direction_ref: direction_ref.into(),
            framed_vehicle_journey_ref: None,
            journey_pattern_ref: None,
            journey_pattern_name: None,
            vehicle_mode: Vec::new(),
            route_ref: None,
            published_line_name: Vec::new(),
            group_of_lines_ref: None,
            direction_name: Vec::new(),
            external_line_ref: None,
            branding_ref: None,
            branding: None,
            reason: Vec::new(),
            extensions: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::TargetedCall;

    fn timestamp() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2004-12-17T09:25:46-05:00").expect("valid timestamp")
    }

    #[test]
    fn a_delivery_writes_its_visits_before_its_cancellations_and_reads_back() {
        let mut delivery = StopTimetableDelivery::new(
            timestamp(),
            vec![TimetabledStopVisit::new(
                timestamp(),
                "HLTST011",
                TargetedVehicleJourney {
                    targeted_call: Some(TargetedCall::new(1)),
                    ..TargetedVehicleJourney::new("Line123", "Out")
                },
            )],
        );
        delivery
            .timetabled_stop_visit_cancellation
            .push(TimetabledStopVisitCancellation::new(
                timestamp(),
                "HLTST011",
                "Line123",
                "Out",
            ));

        let xml = quick_xml::se::to_string_with_root("StopTimetableDelivery", &delivery)
            .expect("delivery serialises");
        let visit = xml.find("<TimetabledStopVisit>").expect("the visit is written");
        let cancellation = xml
            .find("<TimetabledStopVisitCancellation>")
            .expect("the cancellation is written");
        assert!(visit < cancellation, "{xml}");

        let read: StopTimetableDelivery =
            quick_xml::de::from_str(&xml).expect("delivery round-trips");
        assert_eq!(read, delivery);
    }
}
