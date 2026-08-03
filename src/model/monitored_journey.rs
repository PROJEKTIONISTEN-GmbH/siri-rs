//! The tracked view of a journey: where the vehicle is and what it has done.
//!
//! A [`MonitoredVehicleJourney`] is what a vehicle-tracking system knows about a
//! run in progress — its position, how it is getting on, the stop it is at now
//! ([`MonitoredCall`]), the ones behind it ([`PreviousCall`]) and the ones ahead
//! ([`OnwardCall`]).
//!
//! As in the estimated view, the schema's groups are inlined: a group contributes
//! its elements to the enclosing sequence rather than nesting them.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::{
    ArrivalBoardingActivity, CallStatus, DepartureBoardingActivity, FirstOrLastJourney, Occupancy,
    PredictionInaccurateReason, ProgressRate, QualityIndex, VehicleModesOfTransport, VehicleStatus,
};
use crate::model::call::StopAssignment;
use crate::model::facility::{FacilityChange, FacilityCondition};
use crate::model::formation::{
    CompoundTrains, FormationAssignment, FormationCondition, PassengerCapacity, TrainElements,
    Trains, VehicleOccupancy,
};
use crate::model::journey::{
    Branding, BrandingRef, GroupOfLinesRef, JourneyParts, JourneyPlaceRef, PredictionQuality,
    SimpleContact, TrainBlockPart, TrainNumbers, ViaName,
};
use crate::model::location::Location;
use crate::model::reference::{
    BlockRef, ControlActionRef, CourseOfJourneyRef, DestinationRef, DirectionRef,
    FramedVehicleJourneyRef, JourneyPatternRef, LineRef, OperatorRef, ProductCategoryRef, RouteRef,
    ServiceFeatureRef, SituationRef, StopPointRef, VehicleFeatureRef, VehicleJourneyRef, VehicleRef,
};
use crate::types::{Duration, Empty, Extensions, NaturalLanguagePlaceName, NaturalLanguageString};

/// A journey as a tracking system currently sees it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MonitoredVehicleJourney {
    /// The line the journey runs on.
    #[serde(rename = "LineRef", default, skip_serializing_if = "Option::is_none")]
    pub line_ref: Option<LineRef>,
    /// The direction it runs in.
    #[serde(rename = "DirectionRef", default, skip_serializing_if = "Option::is_none")]
    pub direction_ref: Option<DirectionRef>,
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
    /// Whether the journey is being tracked in real time.
    #[serde(rename = "Monitored", default, skip_serializing_if = "Option::is_none")]
    pub monitored: Option<bool>,
    /// Why tracking is not working, in the producer's own codes.
    #[serde(
        rename = "MonitoringError",
        default,
        with = "crate::xml::token_list::space_separated",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub monitoring_error: Vec<String>,
    /// Whether the vehicle is caught in traffic.
    #[serde(rename = "InCongestion", default, skip_serializing_if = "Option::is_none")]
    pub in_congestion: Option<bool>,
    /// Whether the crew has raised an alarm.
    #[serde(rename = "InPanic", default, skip_serializing_if = "Option::is_none")]
    pub in_panic: Option<bool>,
    /// Whether the predicted times should be treated as unreliable.
    #[serde(rename = "PredictionInaccurate", default, skip_serializing_if = "Option::is_none")]
    pub prediction_inaccurate: Option<bool>,
    /// Why they are unreliable.
    #[serde(rename = "PredictionInaccurateReason", default, skip_serializing_if = "Option::is_none")]
    pub prediction_inaccurate_reason: Option<PredictionInaccurateReason>,
    /// Which system the real-time data came from.
    #[serde(rename = "DataSource", default, skip_serializing_if = "Option::is_none")]
    pub data_source: Option<String>,
    /// How much confidence to put in the data.
    #[serde(rename = "ConfidenceLevel", default, skip_serializing_if = "Option::is_none")]
    pub confidence_level: Option<QualityIndex>,
    /// Where the vehicle is.
    #[serde(rename = "VehicleLocation", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_location: Option<Location>,
    /// When the position was taken.
    #[serde(rename = "LocationRecordedAtTime", default, skip_serializing_if = "Option::is_none")]
    pub location_recorded_at_time: Option<DateTime<FixedOffset>>,
    /// Which way the vehicle is pointing, in degrees from north.
    #[serde(rename = "Bearing", default, skip_serializing_if = "Option::is_none")]
    pub bearing: Option<f64>,
    /// How well the vehicle is getting on.
    #[serde(rename = "ProgressRate", default, skip_serializing_if = "Option::is_none")]
    pub progress_rate: Option<ProgressRate>,
    /// How fast it is going, in metres per second.
    #[serde(rename = "Velocity", default, skip_serializing_if = "Option::is_none")]
    pub velocity: Option<u64>,
    /// Whether the engine is running.
    #[serde(rename = "EngineOn", default, skip_serializing_if = "Option::is_none")]
    pub engine_on: Option<bool>,
    /// How full the vehicle is.
    #[serde(rename = "Occupancy", default, skip_serializing_if = "Option::is_none")]
    pub occupancy: Option<Occupancy>,
    /// How far behind the timetable the journey is running.
    #[serde(rename = "Delay", default, skip_serializing_if = "Option::is_none")]
    pub delay: Option<Duration>,
    /// How the journey is getting on, in the producer's own words.
    #[serde(rename = "ProgressStatus", default, skip_serializing_if = "Vec::is_empty")]
    pub progress_status: Vec<NaturalLanguageString>,
    /// Where the vehicle is in its day's work.
    #[serde(rename = "VehicleStatus", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_status: Option<VehicleStatus>,
    /// The parts the train is formed of, and where each sits.
    #[serde(rename = "TrainBlockPart", default, skip_serializing_if = "Vec::is_empty")]
    pub train_block_part: Vec<TrainBlockPart>,
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
    /// The train numbers the journey runs under.
    #[serde(rename = "TrainNumbers", default, skip_serializing_if = "Option::is_none")]
    pub train_numbers: Option<TrainNumbers>,
    /// The parts the journey is split into.
    #[serde(rename = "JourneyParts", default, skip_serializing_if = "Option::is_none")]
    pub journey_parts: Option<JourneyParts>,
    /// The carriages and engines the train is made of.
    #[serde(rename = "TrainElements", default, skip_serializing_if = "Option::is_none")]
    pub train_elements: Option<TrainElements>,
    /// The trains those elements are coupled into.
    #[serde(rename = "Trains", default, skip_serializing_if = "Option::is_none")]
    pub trains: Option<Trains>,
    /// The compound trains those trains are joined into.
    #[serde(rename = "CompoundTrains", default, skip_serializing_if = "Option::is_none")]
    pub compound_trains: Option<CompoundTrains>,
    /// The stops the vehicle has already served.
    #[serde(rename = "PreviousCalls", default, skip_serializing_if = "Option::is_none")]
    pub previous_calls: Option<PreviousCalls>,
    /// The stop the vehicle is at or heading for.
    #[serde(rename = "MonitoredCall", default, skip_serializing_if = "Option::is_none")]
    pub monitored_call: Option<MonitoredCall>,
    /// The stops still ahead of it.
    #[serde(rename = "OnwardCalls", default, skip_serializing_if = "Option::is_none")]
    pub onward_calls: Option<OnwardCalls>,
    /// Whether the calls above are the journey's whole stop sequence.
    #[serde(rename = "IsCompleteStopSequence", default, skip_serializing_if = "Option::is_none")]
    pub is_complete_stop_sequence: Option<bool>,
}

impl MonitoredVehicleJourney {
    /// A journey on the given line, with nothing else said about it yet.
    pub fn on_line(line_ref: impl Into<LineRef>) -> Self {
        Self {
            line_ref: Some(line_ref.into()),
            ..Self::default()
        }
    }

    /// The stops the vehicle has already served.
    pub fn previous_calls(&self) -> &[PreviousCall] {
        self.previous_calls
            .as_ref()
            .map(|calls| calls.previous_call.as_slice())
            .unwrap_or_default()
    }

    /// The stops still ahead of the vehicle.
    pub fn onward_calls(&self) -> &[OnwardCall] {
        self.onward_calls
            .as_ref()
            .map(|calls| calls.onward_call.as_slice())
            .unwrap_or_default()
    }
}

/// The stops a vehicle has already served on the current journey.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PreviousCalls {
    /// The calls, at least one.
    #[serde(rename = "PreviousCall")]
    pub previous_call: Vec<PreviousCall>,
}

/// The stops a vehicle has still to serve on the current journey.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OnwardCalls {
    /// The calls, at least one.
    #[serde(rename = "OnwardCall")]
    pub onward_call: Vec<OnwardCall>,
}

/// A stop the vehicle has already served, and when it did.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PreviousCall {
    /// The stop.
    #[serde(rename = "StopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_point_ref: Option<StopPointRef>,
    /// Which visit to that stop this was, when the journey calls more than once.
    #[serde(rename = "VisitNumber", default, skip_serializing_if = "Option::is_none")]
    pub visit_number: Option<u64>,
    /// Where the stop came in the journey, counting from one.
    #[serde(rename = "Order", default, skip_serializing_if = "Option::is_none")]
    pub order: Option<u64>,
    /// Names of the stop, one per language.
    #[serde(rename = "StopPointName", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_point_name: Vec<NaturalLanguageString>,
    /// Whether the vehicle is standing at the stop.
    #[serde(rename = "VehicleAtStop", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_at_stop: Option<bool>,
    /// When the vehicle was planned to arrive.
    #[serde(rename = "AimedArrivalTime", default, skip_serializing_if = "Option::is_none")]
    pub aimed_arrival_time: Option<DateTime<FixedOffset>>,
    /// When it did arrive.
    #[serde(rename = "ActualArrivalTime", default, skip_serializing_if = "Option::is_none")]
    pub actual_arrival_time: Option<DateTime<FixedOffset>>,
    /// When it was expected to arrive, if the arrival was not recorded.
    #[serde(rename = "ExpectedArrivalTime", default, skip_serializing_if = "Option::is_none")]
    pub expected_arrival_time: Option<DateTime<FixedOffset>>,
    /// When the vehicle was planned to leave.
    #[serde(rename = "AimedDepartureTime", default, skip_serializing_if = "Option::is_none")]
    pub aimed_departure_time: Option<DateTime<FixedOffset>>,
    /// When it did leave.
    #[serde(rename = "ActualDepartureTime", default, skip_serializing_if = "Option::is_none")]
    pub actual_departure_time: Option<DateTime<FixedOffset>>,
    /// When it was expected to leave, if the departure was not recorded.
    #[serde(rename = "ExpectedDepartureTime", default, skip_serializing_if = "Option::is_none")]
    pub expected_departure_time: Option<DateTime<FixedOffset>>,
    /// How full the vehicle was when it left.
    #[serde(rename = "RecordedDepartureOccupancy", default, skip_serializing_if = "Vec::is_empty")]
    pub recorded_departure_occupancy: Vec<VehicleOccupancy>,
    /// How many passengers the vehicle could take when it left.
    #[serde(rename = "RecordedDepartureCapacities", default, skip_serializing_if = "Vec::is_empty")]
    pub recorded_departure_capacities: Vec<PassengerCapacity>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// The stop a vehicle is at, or is on its way to.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MonitoredCall {
    /// The stop.
    #[serde(rename = "StopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_point_ref: Option<StopPointRef>,
    /// Which visit to that stop this is, when the journey calls more than once.
    #[serde(rename = "VisitNumber", default, skip_serializing_if = "Option::is_none")]
    pub visit_number: Option<u64>,
    /// Where the stop comes in the journey, counting from one.
    #[serde(rename = "Order", default, skip_serializing_if = "Option::is_none")]
    pub order: Option<u64>,
    /// Names of the stop, one per language.
    #[serde(rename = "StopPointName", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_point_name: Vec<NaturalLanguageString>,
    /// Whether the vehicle is standing at the stop.
    #[serde(rename = "VehicleAtStop", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_at_stop: Option<bool>,
    /// Where the vehicle is standing, when that is more precise than the stop.
    #[serde(rename = "VehicleLocationAtStop", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_location_at_stop: Option<Location>,
    /// Whether the train changes direction here.
    #[serde(rename = "ReversesAtStop", default, skip_serializing_if = "Option::is_none")]
    pub reverses_at_stop: Option<bool>,
    /// Whether the train runs through the platform without stopping.
    #[serde(rename = "PlatformTraversal", default, skip_serializing_if = "Option::is_none")]
    pub platform_traversal: Option<bool>,
    /// The state of the signal governing the approach, in the operator's own codes.
    #[serde(rename = "SignalStatus", default, skip_serializing_if = "Option::is_none")]
    pub signal_status: Option<String>,
    /// Whether the stop is a timing point the timetable is measured against.
    #[serde(rename = "TimingPoint", default, skip_serializing_if = "Option::is_none")]
    pub timing_point: Option<bool>,
    /// Whether passengers may board anywhere along the stretch before this stop.
    #[serde(rename = "BoardingStretch", default, skip_serializing_if = "Option::is_none")]
    pub boarding_stretch: Option<bool>,
    /// Whether the vehicle calls only when asked to.
    #[serde(rename = "RequestStop", default, skip_serializing_if = "Option::is_none")]
    pub request_stop: Option<bool>,
    /// What is shown as the origin from this stop on, one per language.
    #[serde(rename = "OriginDisplay", default, skip_serializing_if = "Vec::is_empty")]
    pub origin_display: Vec<NaturalLanguageString>,
    /// What is shown as the destination from this stop on, one per language.
    #[serde(rename = "DestinationDisplay", default, skip_serializing_if = "Vec::is_empty")]
    pub destination_display: Vec<NaturalLanguageString>,
    /// Notes about this call, one per language.
    #[serde(rename = "CallNote", default, skip_serializing_if = "Vec::is_empty")]
    pub call_note: Vec<NaturalLanguageString>,
    /// Changes to how the train is put together, taking effect here.
    #[serde(rename = "FormationCondition", default, skip_serializing_if = "Vec::is_empty")]
    pub formation_condition: Vec<FormationCondition>,
    /// Facilities at this stop whose state has changed.
    #[serde(rename = "FacilityConditionElement", default, skip_serializing_if = "Vec::is_empty")]
    pub facility_condition_element: Vec<FacilityCondition>,
    /// The same, in the older spelling the schema still allows.
    #[serde(rename = "FacilityChangeElement", default, skip_serializing_if = "Option::is_none")]
    pub facility_change_element: Option<FacilityChange>,
    /// Situations that explain the state of this call.
    #[serde(rename = "SituationRef", default, skip_serializing_if = "Vec::is_empty")]
    pub situation_ref: Vec<SituationRef>,
    /// The control action taken at this call.
    #[serde(rename = "ControlActionRef", default, skip_serializing_if = "Option::is_none")]
    pub control_action_ref: Option<ControlActionRef>,
    /// When the vehicle is planned to arrive.
    #[serde(rename = "AimedArrivalTime", default, skip_serializing_if = "Option::is_none")]
    pub aimed_arrival_time: Option<DateTime<FixedOffset>>,
    /// When it did arrive.
    #[serde(rename = "ActualArrivalTime", default, skip_serializing_if = "Option::is_none")]
    pub actual_arrival_time: Option<DateTime<FixedOffset>>,
    /// When it is now expected to arrive.
    #[serde(rename = "ExpectedArrivalTime", default, skip_serializing_if = "Option::is_none")]
    pub expected_arrival_time: Option<DateTime<FixedOffset>>,
    /// The latest it could still arrive.
    #[serde(rename = "LatestExpectedArrivalTime", default, skip_serializing_if = "Option::is_none")]
    pub latest_expected_arrival_time: Option<DateTime<FixedOffset>>,
    /// How the arrival stands against the timetable.
    #[serde(rename = "ArrivalStatus", default, skip_serializing_if = "Option::is_none")]
    pub arrival_status: Option<CallStatus>,
    /// Why the arrival was cancelled, one per language.
    #[serde(rename = "ArrivalCancellationReason", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_cancellation_reason: Vec<NaturalLanguageString>,
    /// How near the vehicle is, in words for a display.
    #[serde(rename = "ArrivalProximityText", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_proximity_text: Vec<NaturalLanguageString>,
    /// The platform the vehicle arrives at, one name per language.
    #[serde(rename = "ArrivalPlatformName", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_platform_name: Vec<NaturalLanguageString>,
    /// Whether passengers may alight here.
    #[serde(rename = "ArrivalBoardingActivity", default, skip_serializing_if = "Option::is_none")]
    pub arrival_boarding_activity: Option<ArrivalBoardingActivity>,
    /// Where the vehicle stands on arrival.
    #[serde(rename = "ArrivalStopAssignment", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_stop_assignment: Vec<StopAssignment>,
    /// Where each part of the train stands on arrival.
    #[serde(rename = "ArrivalFormationAssignment", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_formation_assignment: Vec<FormationAssignment>,
    /// Which way round the vehicle stands on arrival, one text per language.
    #[serde(rename = "ArrivalOrientationRelativeToQuay", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_orientation_relative_to_quay: Vec<NaturalLanguageString>,
    /// The operators whose tickets are valid on arrival.
    #[serde(rename = "ArrivalOperatorRefs", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_operator_refs: Vec<OperatorRef>,
    /// When the vehicle is planned to leave.
    #[serde(rename = "AimedDepartureTime", default, skip_serializing_if = "Option::is_none")]
    pub aimed_departure_time: Option<DateTime<FixedOffset>>,
    /// When it did leave.
    #[serde(rename = "ActualDepartureTime", default, skip_serializing_if = "Option::is_none")]
    pub actual_departure_time: Option<DateTime<FixedOffset>>,
    /// When it is now expected to leave.
    #[serde(rename = "ExpectedDepartureTime", default, skip_serializing_if = "Option::is_none")]
    pub expected_departure_time: Option<DateTime<FixedOffset>>,
    /// A working estimate of the departure, pending a firmer one.
    #[serde(rename = "ProvisionalExpectedDepartureTime", default, skip_serializing_if = "Option::is_none")]
    pub provisional_expected_departure_time: Option<DateTime<FixedOffset>>,
    /// The earliest the vehicle will leave, whatever else happens.
    #[serde(rename = "EarliestExpectedDepartureTime", default, skip_serializing_if = "Option::is_none")]
    pub earliest_expected_departure_time: Option<DateTime<FixedOffset>>,
    /// How much confidence to put in the expected departure.
    #[serde(rename = "ExpectedDeparturePredictionQuality", default, skip_serializing_if = "Option::is_none")]
    pub expected_departure_prediction_quality: Option<PredictionQuality>,
    /// When passengers must be aboard by, as planned.
    #[serde(rename = "AimedLatestPassengerAccessTime", default, skip_serializing_if = "Option::is_none")]
    pub aimed_latest_passenger_access_time: Option<DateTime<FixedOffset>>,
    /// When passengers must be aboard by, as now expected.
    #[serde(rename = "ExpectedLatestPassengerAccessTime", default, skip_serializing_if = "Option::is_none")]
    pub expected_latest_passenger_access_time: Option<DateTime<FixedOffset>>,
    /// How the departure stands against the timetable.
    #[serde(rename = "DepartureStatus", default, skip_serializing_if = "Option::is_none")]
    pub departure_status: Option<CallStatus>,
    /// Why the departure was cancelled, one per language.
    #[serde(rename = "DepartureCancellationReason", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_cancellation_reason: Vec<NaturalLanguageString>,
    /// How near departure is, in words for a display.
    #[serde(rename = "DepartureProximityText", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_proximity_text: Vec<NaturalLanguageString>,
    /// The platform the vehicle leaves from, one name per language.
    #[serde(rename = "DeparturePlatformName", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_platform_name: Vec<NaturalLanguageString>,
    /// Whether passengers may board here.
    #[serde(rename = "DepartureBoardingActivity", default, skip_serializing_if = "Option::is_none")]
    pub departure_boarding_activity: Option<DepartureBoardingActivity>,
    /// Where the vehicle stands for departure.
    #[serde(rename = "DepartureStopAssignment", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_stop_assignment: Vec<StopAssignment>,
    /// Where each part of the train stands for departure.
    #[serde(rename = "DepartureFormationAssignment", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_formation_assignment: Vec<FormationAssignment>,
    /// Which way round the vehicle stands for departure, one text per language.
    #[serde(rename = "DepartureOrientationRelativeToQuay", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_orientation_relative_to_quay: Vec<NaturalLanguageString>,
    /// How full the vehicle is expected to be when it leaves.
    #[serde(rename = "ExpectedDepartureOccupancy", default, skip_serializing_if = "Vec::is_empty")]
    pub expected_departure_occupancy: Vec<VehicleOccupancy>,
    /// How many passengers the vehicle is expected to be able to take on departure.
    #[serde(rename = "ExpectedDepartureCapacities", default, skip_serializing_if = "Vec::is_empty")]
    pub expected_departure_capacities: Vec<PassengerCapacity>,
    /// How full the vehicle was when it left, instead of the expected figures.
    #[serde(rename = "RecordedDepartureOccupancy", default, skip_serializing_if = "Vec::is_empty")]
    pub recorded_departure_occupancy: Vec<VehicleOccupancy>,
    /// How many passengers the vehicle could take when it left.
    #[serde(rename = "RecordedDepartureCapacities", default, skip_serializing_if = "Vec::is_empty")]
    pub recorded_departure_capacities: Vec<PassengerCapacity>,
    /// The operators whose tickets are valid on departure.
    #[serde(rename = "DepartureOperatorRefs", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_operator_refs: Vec<OperatorRef>,
    /// The planned interval between vehicles on a headway service.
    #[serde(rename = "AimedHeadwayInterval", default, skip_serializing_if = "Option::is_none")]
    pub aimed_headway_interval: Option<Duration>,
    /// The interval now expected.
    #[serde(rename = "ExpectedHeadwayInterval", default, skip_serializing_if = "Option::is_none")]
    pub expected_headway_interval: Option<Duration>,
    /// How far away the vehicle still is, in metres.
    #[serde(rename = "DistanceFromStop", default, skip_serializing_if = "Option::is_none")]
    pub distance_from_stop: Option<u64>,
    /// How many stops away it still is.
    #[serde(rename = "NumberOfStopsAway", default, skip_serializing_if = "Option::is_none")]
    pub number_of_stops_away: Option<u64>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// A stop still ahead of the vehicle, and what is expected there.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OnwardCall {
    /// The stop.
    #[serde(rename = "StopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_point_ref: Option<StopPointRef>,
    /// Which visit to that stop this is, when the journey calls more than once.
    #[serde(rename = "VisitNumber", default, skip_serializing_if = "Option::is_none")]
    pub visit_number: Option<u64>,
    /// Where the stop comes in the journey, counting from one.
    #[serde(rename = "Order", default, skip_serializing_if = "Option::is_none")]
    pub order: Option<u64>,
    /// Names of the stop, one per language.
    #[serde(rename = "StopPointName", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_point_name: Vec<NaturalLanguageString>,
    /// Whether the vehicle is standing at the stop.
    #[serde(rename = "VehicleAtStop", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_at_stop: Option<bool>,
    /// Whether the stop is a timing point the timetable is measured against.
    #[serde(rename = "TimingPoint", default, skip_serializing_if = "Option::is_none")]
    pub timing_point: Option<bool>,
    /// When the vehicle is planned to arrive.
    #[serde(rename = "AimedArrivalTime", default, skip_serializing_if = "Option::is_none")]
    pub aimed_arrival_time: Option<DateTime<FixedOffset>>,
    /// When it is now expected to arrive.
    #[serde(rename = "ExpectedArrivalTime", default, skip_serializing_if = "Option::is_none")]
    pub expected_arrival_time: Option<DateTime<FixedOffset>>,
    /// The latest it could still arrive.
    #[serde(rename = "LatestExpectedArrivalTime", default, skip_serializing_if = "Option::is_none")]
    pub latest_expected_arrival_time: Option<DateTime<FixedOffset>>,
    /// How much confidence to put in the expected arrival.
    #[serde(rename = "ExpectedArrivalPredictionQuality", default, skip_serializing_if = "Option::is_none")]
    pub expected_arrival_prediction_quality: Option<PredictionQuality>,
    /// That the arrival cannot be predicted at all, instead of the times above.
    #[serde(rename = "ArrivalPredictionUnknown", default, skip_serializing_if = "Option::is_none")]
    pub arrival_prediction_unknown: Option<Empty>,
    /// How the arrival stands against the timetable.
    #[serde(rename = "ArrivalStatus", default, skip_serializing_if = "Option::is_none")]
    pub arrival_status: Option<CallStatus>,
    /// Why the arrival was cancelled, one per language.
    #[serde(rename = "ArrivalCancellationReason", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_cancellation_reason: Vec<NaturalLanguageString>,
    /// How near the vehicle is, in words for a display.
    #[serde(rename = "ArrivalProximityText", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_proximity_text: Vec<NaturalLanguageString>,
    /// The platform the vehicle arrives at, one name per language.
    #[serde(rename = "ArrivalPlatformName", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_platform_name: Vec<NaturalLanguageString>,
    /// Whether passengers may alight here.
    #[serde(rename = "ArrivalBoardingActivity", default, skip_serializing_if = "Option::is_none")]
    pub arrival_boarding_activity: Option<ArrivalBoardingActivity>,
    /// Where the vehicle stands on arrival.
    #[serde(rename = "ArrivalStopAssignment", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_stop_assignment: Vec<StopAssignment>,
    /// Where each part of the train stands on arrival.
    #[serde(rename = "ArrivalFormationAssignment", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_formation_assignment: Vec<FormationAssignment>,
    /// Which way round the vehicle stands on arrival, one text per language.
    #[serde(rename = "ArrivalOrientationRelativeToQuay", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_orientation_relative_to_quay: Vec<NaturalLanguageString>,
    /// The operators whose tickets are valid on arrival.
    #[serde(rename = "ArrivalOperatorRefs", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_operator_refs: Vec<OperatorRef>,
    /// When the vehicle is planned to leave.
    #[serde(rename = "AimedDepartureTime", default, skip_serializing_if = "Option::is_none")]
    pub aimed_departure_time: Option<DateTime<FixedOffset>>,
    /// When it is now expected to leave.
    #[serde(rename = "ExpectedDepartureTime", default, skip_serializing_if = "Option::is_none")]
    pub expected_departure_time: Option<DateTime<FixedOffset>>,
    /// A working estimate of the departure, pending a firmer one.
    #[serde(rename = "ProvisionalExpectedDepartureTime", default, skip_serializing_if = "Option::is_none")]
    pub provisional_expected_departure_time: Option<DateTime<FixedOffset>>,
    /// The earliest the vehicle will leave, whatever else happens.
    #[serde(rename = "EarliestExpectedDepartureTime", default, skip_serializing_if = "Option::is_none")]
    pub earliest_expected_departure_time: Option<DateTime<FixedOffset>>,
    /// How much confidence to put in the expected departure.
    #[serde(rename = "ExpectedDeparturePredictionQuality", default, skip_serializing_if = "Option::is_none")]
    pub expected_departure_prediction_quality: Option<PredictionQuality>,
    /// That the departure cannot be predicted at all, instead of the times above.
    #[serde(rename = "DeparturePredictionUnknown", default, skip_serializing_if = "Option::is_none")]
    pub departure_prediction_unknown: Option<Empty>,
    /// When passengers must be aboard by, as planned.
    #[serde(rename = "AimedLatestPassengerAccessTime", default, skip_serializing_if = "Option::is_none")]
    pub aimed_latest_passenger_access_time: Option<DateTime<FixedOffset>>,
    /// When passengers must be aboard by, as now expected.
    #[serde(rename = "ExpectedLatestPassengerAccessTime", default, skip_serializing_if = "Option::is_none")]
    pub expected_latest_passenger_access_time: Option<DateTime<FixedOffset>>,
    /// How the departure stands against the timetable.
    #[serde(rename = "DepartureStatus", default, skip_serializing_if = "Option::is_none")]
    pub departure_status: Option<CallStatus>,
    /// Why the departure was cancelled, one per language.
    #[serde(rename = "DepartureCancellationReason", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_cancellation_reason: Vec<NaturalLanguageString>,
    /// How near departure is, in words for a display.
    #[serde(rename = "DepartureProximityText", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_proximity_text: Vec<NaturalLanguageString>,
    /// The platform the vehicle leaves from, one name per language.
    #[serde(rename = "DeparturePlatformName", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_platform_name: Vec<NaturalLanguageString>,
    /// Whether passengers may board here.
    #[serde(rename = "DepartureBoardingActivity", default, skip_serializing_if = "Option::is_none")]
    pub departure_boarding_activity: Option<DepartureBoardingActivity>,
    /// Where the vehicle stands for departure.
    #[serde(rename = "DepartureStopAssignment", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_stop_assignment: Vec<StopAssignment>,
    /// Where each part of the train stands for departure.
    #[serde(rename = "DepartureFormationAssignment", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_formation_assignment: Vec<FormationAssignment>,
    /// Which way round the vehicle stands for departure, one text per language.
    #[serde(rename = "DepartureOrientationRelativeToQuay", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_orientation_relative_to_quay: Vec<NaturalLanguageString>,
    /// How full the vehicle is expected to be when it leaves.
    #[serde(rename = "ExpectedDepartureOccupancy", default, skip_serializing_if = "Vec::is_empty")]
    pub expected_departure_occupancy: Vec<VehicleOccupancy>,
    /// How many passengers the vehicle is expected to be able to take on departure.
    #[serde(rename = "ExpectedDepartureCapacities", default, skip_serializing_if = "Vec::is_empty")]
    pub expected_departure_capacities: Vec<PassengerCapacity>,
    /// How full the vehicle was when it left, instead of the expected figures.
    #[serde(rename = "RecordedDepartureOccupancy", default, skip_serializing_if = "Vec::is_empty")]
    pub recorded_departure_occupancy: Vec<VehicleOccupancy>,
    /// How many passengers the vehicle could take when it left.
    #[serde(rename = "RecordedDepartureCapacities", default, skip_serializing_if = "Vec::is_empty")]
    pub recorded_departure_capacities: Vec<PassengerCapacity>,
    /// The operators whose tickets are valid on departure.
    #[serde(rename = "DepartureOperatorRefs", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_operator_refs: Vec<OperatorRef>,
    /// The planned interval between vehicles on a headway service.
    #[serde(rename = "AimedHeadwayInterval", default, skip_serializing_if = "Option::is_none")]
    pub aimed_headway_interval: Option<Duration>,
    /// The interval now expected.
    #[serde(rename = "ExpectedHeadwayInterval", default, skip_serializing_if = "Option::is_none")]
    pub expected_headway_interval: Option<Duration>,
    /// How far away the vehicle still is, in metres.
    #[serde(rename = "DistanceFromStop", default, skip_serializing_if = "Option::is_none")]
    pub distance_from_stop: Option<u64>,
    /// How many stops away it still is.
    #[serde(rename = "NumberOfStopsAway", default, skip_serializing_if = "Option::is_none")]
    pub number_of_stops_away: Option<u64>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl OnwardCall {
    /// A call at the given stop with nothing said about it yet.
    pub fn at(stop_point_ref: impl Into<StopPointRef>) -> Self {
        Self {
            stop_point_ref: Some(stop_point_ref.into()),
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::location::Location;

    fn timestamp() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2004-12-17T09:30:47-05:00").expect("valid timestamp")
    }

    #[test]
    fn a_journey_writes_its_progress_after_its_identity_and_reads_back() {
        let journey = MonitoredVehicleJourney {
            monitored: Some(true),
            vehicle_location: Some(Location::wgs84(0.1, 53.55)),
            bearing: Some(123.0),
            delay: Some(Duration::parse("PT2M").expect("valid duration")),
            vehicle_ref: Some("VEH987654".into()),
            onward_calls: Some(OnwardCalls {
                onward_call: vec![OnwardCall {
                    aimed_arrival_time: Some(timestamp()),
                    expected_arrival_time: Some(timestamp()),
                    ..OnwardCall::at("HLTST012")
                }],
            }),
            ..MonitoredVehicleJourney::on_line("Line123")
        };

        let xml = quick_xml::se::to_string_with_root("MonitoredVehicleJourney", &journey)
            .expect("the journey serialises");
        let line = xml.find("<LineRef>").expect("the line is written");
        let location = xml.find("<VehicleLocation>").expect("the position is written");
        let vehicle = xml.find("<VehicleRef>").expect("the vehicle is written");
        let onward = xml.find("<OnwardCalls>").expect("the calls are written");
        assert!(line < location && location < vehicle && vehicle < onward, "{xml}");

        let read: MonitoredVehicleJourney =
            quick_xml::de::from_str(&xml).expect("the journey round-trips");
        assert_eq!(read, journey);
        assert_eq!(read.onward_calls().len(), 1);
        assert!(read.previous_calls().is_empty());
    }
}
