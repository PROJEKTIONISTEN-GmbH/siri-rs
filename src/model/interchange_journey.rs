//! The journeys on either side of a connection.
//!
//! Both connection services describe a change between two journeys with the same
//! structure, [`InterchangeJourney`]: the *feeder* is the journey passengers arrive
//! on, the *distributor* the one they leave on. It is a cut-down journey — enough to
//! recognise the service and know when it is due, without the calling pattern the
//! timetable services carry.
//!
//! As elsewhere in the model, the schema's groups are inlined: a group contributes
//! its elements to the enclosing sequence rather than nesting them.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::{FirstOrLastJourney, VehicleModesOfTransport};
use crate::model::facility::{FacilityChange, FacilityCondition};
use crate::model::formation::FormationCondition;
use crate::model::journey::{
    Branding, BrandingRef, GroupOfLinesRef, JourneyPlaceRef, SimpleContact, ViaName,
};
use crate::model::reference::{
    BlockRef, ControlActionRef, CourseOfJourneyRef, DestinationRef, DirectionRef,
    FramedVehicleJourneyRef, JourneyPatternRef, LineRef, OperatorRef, ProductCategoryRef, RouteRef,
    ServiceFeatureRef, SituationRef, VehicleFeatureRef, VehicleJourneyRef, VehicleRef,
};
use crate::types::{Extensions, NaturalLanguagePlaceName, NaturalLanguageString};

/// One of the two journeys a connection joins.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterchangeJourney {
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
    /// Changes to how the train is put together.
    #[serde(rename = "FormationCondition", default, skip_serializing_if = "Vec::is_empty")]
    pub formation_condition: Vec<FormationCondition>,
    /// Facilities along the journey whose state has changed.
    #[serde(rename = "FacilityConditionElement", default, skip_serializing_if = "Vec::is_empty")]
    pub facility_condition_element: Vec<FacilityCondition>,
    /// The same, in the older spelling the schema still allows.
    #[serde(rename = "FacilityChangeElement", default, skip_serializing_if = "Option::is_none")]
    pub facility_change_element: Option<FacilityChange>,
    /// Situations that explain the state of the journey.
    #[serde(rename = "SituationRef", default, skip_serializing_if = "Vec::is_empty")]
    pub situation_ref: Vec<SituationRef>,
    /// The control action taken to manage the journey.
    #[serde(rename = "ControlActionRef", default, skip_serializing_if = "Option::is_none")]
    pub control_action_ref: Option<ControlActionRef>,
    /// The day's work the journey belongs to.
    #[serde(rename = "BlockRef", default, skip_serializing_if = "Option::is_none")]
    pub block_ref: Option<BlockRef>,
    /// The run of journeys within that work.
    #[serde(rename = "CourseOfJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub course_of_journey_ref: Option<CourseOfJourneyRef>,
    /// The timetabled journey this run realises.
    #[serde(rename = "VehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_journey_ref: Option<VehicleJourneyRef>,
    /// The vehicle running it.
    #[serde(rename = "VehicleRef", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_ref: Option<VehicleRef>,
    /// Further journeys the same vehicle is running at the same time.
    #[serde(rename = "AdditionalVehicleJourneyRef", default, skip_serializing_if = "Vec::is_empty")]
    pub additional_vehicle_journey_ref: Vec<FramedVehicleJourneyRef>,
    /// The driver on duty.
    #[serde(rename = "DriverRef", default, skip_serializing_if = "Option::is_none")]
    pub driver_ref: Option<String>,
    /// The driver's name.
    #[serde(rename = "DriverName", default, skip_serializing_if = "Option::is_none")]
    pub driver_name: Option<String>,
    /// Whether the journey is being tracked in real time.
    #[serde(rename = "Monitored", default, skip_serializing_if = "Option::is_none")]
    pub monitored: Option<bool>,
    /// When the journey is planned to leave the interchange stop.
    #[serde(rename = "AimedDepartureTime", default, skip_serializing_if = "Option::is_none")]
    pub aimed_departure_time: Option<DateTime<FixedOffset>>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl InterchangeJourney {
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
            formation_condition: Vec::new(),
            facility_condition_element: Vec::new(),
            facility_change_element: None,
            situation_ref: Vec::new(),
            control_action_ref: None,
            block_ref: None,
            course_of_journey_ref: None,
            vehicle_journey_ref: None,
            vehicle_ref: None,
            additional_vehicle_journey_ref: Vec::new(),
            driver_ref: None,
            driver_name: None,
            monitored: None,
            aimed_departure_time: None,
            extensions: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_feeder_writes_its_operational_detail_after_its_names_and_reads_back() {
        let journey = InterchangeJourney {
            published_line_name: vec![NaturalLanguageString::with_lang("EN", "Line 123")],
            block_ref: Some("12345".into()),
            vehicle_ref: Some("V987".into()),
            monitored: Some(true),
            aimed_departure_time: Some(
                DateTime::parse_from_rfc3339("2001-12-17T08:35:47-05:00").expect("valid timestamp"),
            ),
            ..InterchangeJourney::new("123", "OUT")
        };

        let xml = quick_xml::se::to_string_with_root("FeederJourney", &journey)
            .expect("journey serialises");
        let published = xml.find("<PublishedLineName").expect("the line name is written");
        let block = xml.find("<BlockRef>").expect("the block is written");
        let departure = xml.find("<AimedDepartureTime>").expect("the departure is written");
        assert!(published < block && block < departure, "{xml}");

        let read: InterchangeJourney = quick_xml::de::from_str(&xml).expect("journey round-trips");
        assert_eq!(read, journey);
    }
}
