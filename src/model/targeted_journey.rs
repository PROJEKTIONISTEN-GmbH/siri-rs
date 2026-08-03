//! The timetabled view of a journey as one stop sees it.
//!
//! A [`TargetedVehicleJourney`] is what a departure board would show before the day
//! begins: the journey's identity and the single [`TargetedCall`] it is to make at
//! the stop being asked about. It carries aimed times only — what is actually
//! happening belongs to the estimated and monitored views.
//!
//! As in those, the schema's groups are inlined: a group contributes its elements
//! to the enclosing sequence rather than nesting them.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::{
    ArrivalBoardingActivity, DepartureBoardingActivity, FirstOrLastJourney, VehicleModesOfTransport,
};
use crate::model::call::PlannedStopAssignment;
use crate::model::formation::FormationAssignment;
use crate::model::journey::{
    Branding, BrandingRef, GroupOfLinesRef, JourneyPlaceRef, SimpleContact, ViaName,
};
use crate::model::reference::{
    DestinationRef, DirectionRef, FramedVehicleJourneyRef, JourneyPatternRef, LineRef, OperatorRef,
    ProductCategoryRef, RouteRef, ServiceFeatureRef, StopPointRef, VehicleFeatureRef,
};
use crate::types::{Duration, NaturalLanguagePlaceName, NaturalLanguageString};

/// A timetabled journey, together with the call it makes at the stop in question.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TargetedVehicleJourney {
    /// The line the journey runs on.
    #[serde(rename = "LineRef")]
    pub line_ref: LineRef,
    /// The direction it runs in.
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
    /// The modes of transport the journey uses.
    #[serde(
        rename = "VehicleMode",
        default,
        deserialize_with = "crate::xml::token_list::deserialize",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub vehicle_mode: Vec<VehicleModesOfTransport>,
    /// The route the journey follows.
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
    /// The operator running the journey.
    #[serde(rename = "OperatorRef", default, skip_serializing_if = "Option::is_none")]
    pub operator_ref: Option<OperatorRef>,
    /// The commercial category the service belongs to.
    #[serde(rename = "ProductCategoryRef", default, skip_serializing_if = "Option::is_none")]
    pub product_category_ref: Option<ProductCategoryRef>,
    /// Properties of the service, e.g. that cycles may be carried.
    #[serde(rename = "ServiceFeatureRef", default, skip_serializing_if = "Vec::is_empty")]
    pub service_feature_ref: Vec<ServiceFeatureRef>,
    /// Properties of the vehicle, e.g. that it has a low floor.
    #[serde(rename = "VehicleFeatureRef", default, skip_serializing_if = "Vec::is_empty")]
    pub vehicle_feature_ref: Vec<VehicleFeatureRef>,
    /// Where the journey starts.
    #[serde(rename = "OriginRef", default, skip_serializing_if = "Option::is_none")]
    pub origin_ref: Option<JourneyPlaceRef>,
    /// Names of the origin, one per language.
    #[serde(rename = "OriginName", default, skip_serializing_if = "Vec::is_empty")]
    pub origin_name: Vec<NaturalLanguagePlaceName>,
    /// Shorter names of the origin, one per language.
    #[serde(rename = "OriginShortName", default, skip_serializing_if = "Vec::is_empty")]
    pub origin_short_name: Vec<NaturalLanguagePlaceName>,
    /// What is shown at the origin as the destination, one per language.
    #[serde(rename = "DestinationDisplayAtOrigin", default, skip_serializing_if = "Vec::is_empty")]
    pub destination_display_at_origin: Vec<NaturalLanguagePlaceName>,
    /// Places passed through that tell this journey apart from similar ones.
    #[serde(rename = "Via", default, skip_serializing_if = "Vec::is_empty")]
    pub via: Vec<ViaName>,
    /// Where the journey ends.
    #[serde(rename = "DestinationRef", default, skip_serializing_if = "Option::is_none")]
    pub destination_ref: Option<DestinationRef>,
    /// Names of the destination, one per language.
    #[serde(rename = "DestinationName", default, skip_serializing_if = "Vec::is_empty")]
    pub destination_name: Vec<NaturalLanguageString>,
    /// Shorter names of the destination, one per language.
    #[serde(rename = "DestinationShortName", default, skip_serializing_if = "Vec::is_empty")]
    pub destination_short_name: Vec<NaturalLanguagePlaceName>,
    /// What is shown at the destination as the origin, one per language.
    #[serde(rename = "OriginDisplayAtDestination", default, skip_serializing_if = "Vec::is_empty")]
    pub origin_display_at_destination: Vec<NaturalLanguagePlaceName>,
    /// The journey's own name, one per language.
    #[serde(rename = "VehicleJourneyName", default, skip_serializing_if = "Vec::is_empty")]
    pub vehicle_journey_name: Vec<NaturalLanguageString>,
    /// Notes about the journey, one per language.
    #[serde(rename = "JourneyNote", default, skip_serializing_if = "Vec::is_empty")]
    pub journey_note: Vec<NaturalLanguageString>,
    /// How passengers can reach the operator.
    #[serde(rename = "PublicContact", default, skip_serializing_if = "Option::is_none")]
    pub public_contact: Option<SimpleContact>,
    /// How staff can reach the operator's control room.
    #[serde(rename = "OperationsContact", default, skip_serializing_if = "Option::is_none")]
    pub operations_contact: Option<SimpleContact>,
    /// Whether the service runs to a headway rather than to fixed times.
    #[serde(rename = "HeadwayService", default, skip_serializing_if = "Option::is_none")]
    pub headway_service: Option<bool>,
    /// When the journey is planned to leave its origin.
    #[serde(rename = "OriginAimedDepartureTime", default, skip_serializing_if = "Option::is_none")]
    pub origin_aimed_departure_time: Option<DateTime<FixedOffset>>,
    /// When it is planned to reach its destination.
    #[serde(rename = "DestinationAimedArrivalTime", default, skip_serializing_if = "Option::is_none")]
    pub destination_aimed_arrival_time: Option<DateTime<FixedOffset>>,
    /// Whether this is the first or the last journey of the day on the line.
    #[serde(rename = "FirstOrLastJourney", default, skip_serializing_if = "Option::is_none")]
    pub first_or_last_journey: Option<FirstOrLastJourney>,
    /// The call the journey makes at the stop being asked about.
    #[serde(rename = "TargetedCall", default, skip_serializing_if = "Option::is_none")]
    pub targeted_call: Option<TargetedCall>,
}

impl TargetedVehicleJourney {
    /// A journey on the given line and direction, with nothing else said about it yet.
    pub fn new(line_ref: impl Into<LineRef>, direction_ref: impl Into<DirectionRef>) -> Self {
        Self {
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
            operator_ref: None,
            product_category_ref: None,
            service_feature_ref: Vec::new(),
            vehicle_feature_ref: Vec::new(),
            origin_ref: None,
            origin_name: Vec::new(),
            origin_short_name: Vec::new(),
            destination_display_at_origin: Vec::new(),
            via: Vec::new(),
            destination_ref: None,
            destination_name: Vec::new(),
            destination_short_name: Vec::new(),
            origin_display_at_destination: Vec::new(),
            vehicle_journey_name: Vec::new(),
            journey_note: Vec::new(),
            public_contact: None,
            operations_contact: None,
            headway_service: None,
            origin_aimed_departure_time: None,
            destination_aimed_arrival_time: None,
            first_or_last_journey: None,
            targeted_call: None,
        }
    }
}

/// One stop a timetabled journey is to make, with its aimed times.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TargetedCall {
    /// The stop.
    #[serde(rename = "StopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_point_ref: Option<StopPointRef>,
    /// Which visit to that stop this is, when the journey calls more than once.
    #[serde(rename = "VisitNumber")]
    pub visit_number: u64,
    /// Where the stop comes in the journey, counting from one.
    #[serde(rename = "Order", default, skip_serializing_if = "Option::is_none")]
    pub order: Option<u64>,
    /// Whether the stop is a timing point the timetable is measured against.
    #[serde(rename = "TimingPoint", default, skip_serializing_if = "Option::is_none")]
    pub timing_point: Option<bool>,
    /// The operator running the journey, when it differs from stop to stop.
    #[serde(rename = "OperatorRef", default, skip_serializing_if = "Option::is_none")]
    pub operator_ref: Option<OperatorRef>,
    /// The commercial category the service belongs to.
    #[serde(rename = "ProductCategoryRef", default, skip_serializing_if = "Option::is_none")]
    pub product_category_ref: Option<ProductCategoryRef>,
    /// Properties of the service from this stop on.
    #[serde(rename = "ServiceFeatureRef", default, skip_serializing_if = "Vec::is_empty")]
    pub service_feature_ref: Vec<ServiceFeatureRef>,
    /// Properties of the vehicle serving this stop.
    #[serde(rename = "VehicleFeatureRef", default, skip_serializing_if = "Vec::is_empty")]
    pub vehicle_feature_ref: Vec<VehicleFeatureRef>,
    /// When the vehicle is planned to arrive.
    #[serde(rename = "AimedArrivalTime", default, skip_serializing_if = "Option::is_none")]
    pub aimed_arrival_time: Option<DateTime<FixedOffset>>,
    /// The platform the vehicle is planned to arrive at, one name per language.
    #[serde(rename = "ArrivalPlatformName", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_platform_name: Vec<NaturalLanguageString>,
    /// Whether passengers may alight here.
    #[serde(rename = "ArrivalBoardingActivity", default, skip_serializing_if = "Option::is_none")]
    pub arrival_boarding_activity: Option<ArrivalBoardingActivity>,
    /// Where the vehicle is planned to stand on arrival.
    #[serde(rename = "ArrivalStopAssignment", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_stop_assignment: Vec<PlannedStopAssignment>,
    /// Where each part of the train is planned to stand on arrival.
    #[serde(rename = "ArrivalFormationAssignment", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_formation_assignment: Vec<FormationAssignment>,
    /// The operators whose tickets are valid on arrival.
    #[serde(rename = "ArrivalOperatorRefs", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_operator_refs: Vec<OperatorRef>,
    /// When the vehicle is planned to leave.
    #[serde(rename = "AimedDepartureTime", default, skip_serializing_if = "Option::is_none")]
    pub aimed_departure_time: Option<DateTime<FixedOffset>>,
    /// The platform the vehicle is planned to leave from, one name per language.
    #[serde(rename = "DeparturePlatformName", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_platform_name: Vec<NaturalLanguageString>,
    /// Whether passengers may board here.
    #[serde(rename = "DepartureBoardingActivity", default, skip_serializing_if = "Option::is_none")]
    pub departure_boarding_activity: Option<DepartureBoardingActivity>,
    /// Where the vehicle is planned to stand for departure.
    #[serde(rename = "DepartureStopAssignment", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_stop_assignment: Vec<PlannedStopAssignment>,
    /// Where each part of the train is planned to stand for departure.
    #[serde(rename = "DepartureFormationAssignment", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_formation_assignment: Vec<FormationAssignment>,
    /// The operators whose tickets are valid on departure.
    #[serde(rename = "DepartureOperatorRefs", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_operator_refs: Vec<OperatorRef>,
    /// When passengers must be aboard by.
    #[serde(rename = "AimedLatestPassengerAccessTime", default, skip_serializing_if = "Option::is_none")]
    pub aimed_latest_passenger_access_time: Option<DateTime<FixedOffset>>,
    /// The planned interval between vehicles on a headway service.
    #[serde(rename = "AimedHeadwayInterval", default, skip_serializing_if = "Option::is_none")]
    pub aimed_headway_interval: Option<Duration>,
}

impl TargetedCall {
    /// The journey's `visit_number`th call, with nothing said about it yet.
    pub fn new(visit_number: u64) -> Self {
        Self {
            stop_point_ref: None,
            visit_number,
            order: None,
            timing_point: None,
            operator_ref: None,
            product_category_ref: None,
            service_feature_ref: Vec::new(),
            vehicle_feature_ref: Vec::new(),
            aimed_arrival_time: None,
            arrival_platform_name: Vec::new(),
            arrival_boarding_activity: None,
            arrival_stop_assignment: Vec::new(),
            arrival_formation_assignment: Vec::new(),
            arrival_operator_refs: Vec::new(),
            aimed_departure_time: None,
            departure_platform_name: Vec::new(),
            departure_boarding_activity: None,
            departure_stop_assignment: Vec::new(),
            departure_formation_assignment: Vec::new(),
            departure_operator_refs: Vec::new(),
            aimed_latest_passenger_access_time: None,
            aimed_headway_interval: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::reference::{DataFrameRef, DatedVehicleJourneyRef};

    fn timestamp() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2004-12-17T09:40:46-05:00").expect("valid timestamp")
    }

    #[test]
    fn a_journey_writes_its_identity_before_its_names_and_its_call_last() {
        let journey = TargetedVehicleJourney {
            framed_vehicle_journey_ref: Some(FramedVehicleJourneyRef {
                data_frame_ref: DataFrameRef::new("2004-12-17"),
                dated_vehicle_journey_ref: DatedVehicleJourneyRef::new("Outbound"),
            }),
            published_line_name: vec![NaturalLanguageString::with_lang("EN", "123")],
            destination_name: vec![NaturalLanguageString::with_lang("EN", "Paradise Park")],
            targeted_call: Some(TargetedCall {
                aimed_arrival_time: Some(timestamp()),
                ..TargetedCall::new(1)
            }),
            ..TargetedVehicleJourney::new("Line123", "Out")
        };

        let xml = quick_xml::se::to_string_with_root("TargetedVehicleJourney", &journey)
            .expect("journey serialises");
        let line = xml.find("<LineRef>").expect("the line is written");
        let published = xml.find("<PublishedLineName").expect("the line name is written");
        let destination = xml.find("<DestinationName").expect("the destination is written");
        let call = xml.find("<TargetedCall>").expect("the call is written");
        assert!(line < published && published < destination && destination < call, "{xml}");

        let read: TargetedVehicleJourney =
            quick_xml::de::from_str(&xml).expect("journey round-trips");
        assert_eq!(read, journey);
    }
}
