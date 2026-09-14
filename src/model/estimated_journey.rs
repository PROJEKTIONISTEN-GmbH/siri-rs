//! The real-time view of a journey: what is now expected to happen, and what did.
//!
//! An [`EstimatedVehicleJourney`] is a running journey as the operator currently
//! sees it. It restates only what the timetable cannot say: which calls have already
//! happened ([`RecordedCall`]) and what is now expected at the ones still to come
//! ([`EstimatedCall`]), together with the journey's own alterations — a run added
//! today, a run cancelled, a stop skipped.
//!
//! The schema builds these out of groups shared with the planned and monitored
//! views; a group contributes its elements to the enclosing sequence rather than
//! nesting them, so they appear here as runs of fields rather than as members.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::{
    ArrivalBoardingActivity, CallStatus, DepartureBoardingActivity, FirstOrLastJourney,
    InterchangeStatus, Occupancy, PredictionInaccurateReason, ProgressRate, QualityIndex,
    VehicleModesOfTransport, VehicleStatus,
};
use crate::model::call::StopAssignment;
use crate::model::facility::{FacilityChange, FacilityCondition};
use crate::model::formation::{
    CompoundTrains, FormationAssignment, FormationCondition, PassengerCapacity, TrainElements,
    Trains, VehicleOccupancy,
};
use crate::model::journey::{
    Branding, BrandingRef, ConnectingJourneyRef, DatedVehicleJourneyIndirectRef, GroupOfLinesRef,
    JourneyParts, JourneyPlaceRef, JourneyRelations, PredictionQuality, SimpleContact,
    TrainBlockPart, TrainNumbers, ViaName,
};
use crate::model::location::Location;
use crate::model::reference::{
    BlockRef, ConnectionLinkRef, CourseOfJourneyRef, ControlActionRef, DatedVehicleJourneyRef,
    DestinationRef, DirectionRef, FramedVehicleJourneyRef, InterchangeRef, JourneyPatternRef,
    LineRef, OperatorRef, ProductCategoryRef, RouteRef, ServiceFeatureRef, SituationRef,
    StopPointRef, VehicleFeatureRef, VehicleJourneyRef, VehicleRef,
};
use crate::types::{
    Duration, Empty, Extensions, NaturalLanguagePlaceName, NaturalLanguageString,
};

/// A journey as the operator now expects it to run.
///
/// The journey is identified either by an existing timetabled run — through
/// `framed_vehicle_journey_ref` or `dated_vehicle_journey_ref` — or, for a run that
/// is not in any timetable, by an `estimated_vehicle_journey_code` of its own.
/// [`EstimatedVehicleJourney::identity`] reports which was used and
/// [`EstimatedVehicleJourney::alteration`] whether the journey is an addition or a
/// cancellation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EstimatedVehicleJourney {
    /// When the producer last knew this to be true.
    #[serde(rename = "RecordedAtTime", default, skip_serializing_if = "Option::is_none")]
    pub recorded_at_time: Option<DateTime<FixedOffset>>,
    /// The line the journey runs on.
    #[serde(rename = "LineRef")]
    pub line_ref: LineRef,
    /// The direction it runs in.
    #[serde(rename = "DirectionRef")]
    pub direction_ref: DirectionRef,
    /// The timetabled journey on its operational day.
    #[serde(rename = "FramedVehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub framed_vehicle_journey_ref: Option<FramedVehicleJourneyRef>,
    /// The timetabled journey, when the operational day is understood.
    #[serde(rename = "DatedVehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub dated_vehicle_journey_ref: Option<DatedVehicleJourneyRef>,
    /// The timetabled journey, named by where and when it runs.
    #[serde(rename = "DatedVehicleJourneyIndirectRef", default, skip_serializing_if = "Option::is_none")]
    pub dated_vehicle_journey_indirect_ref: Option<DatedVehicleJourneyIndirectRef>,
    /// The producer's own identifier for a journey that is in no timetable.
    #[serde(rename = "EstimatedVehicleJourneyCode", default, skip_serializing_if = "Option::is_none")]
    pub estimated_vehicle_journey_code: Option<String>,
    /// Whether this journey is being run in addition to the timetable.
    #[serde(rename = "ExtraJourney", default, skip_serializing_if = "Option::is_none")]
    pub extra_journey: Option<bool>,
    /// Whether the timetabled journey will not run.
    #[serde(rename = "Cancellation", default, skip_serializing_if = "Option::is_none")]
    pub cancellation: Option<bool>,
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
    /// The calls that have already happened.
    #[serde(rename = "RecordedCalls", default, skip_serializing_if = "Option::is_none")]
    pub recorded_calls: Option<RecordedCalls>,
    /// The calls still to come.
    #[serde(rename = "EstimatedCalls", default, skip_serializing_if = "Option::is_none")]
    pub estimated_calls: Option<EstimatedCalls>,
    /// Whether the calls above are the journey's whole stop sequence.
    #[serde(rename = "IsCompleteStopSequence", default, skip_serializing_if = "Option::is_none")]
    pub is_complete_stop_sequence: Option<bool>,
    /// Other journeys this one is joined to, split from or continues as.
    #[serde(rename = "JourneyRelations", default, skip_serializing_if = "Option::is_none")]
    pub journey_relations: Option<JourneyRelations>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// How an [`EstimatedVehicleJourney`] names the journey it is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum JourneyIdentity<'a> {
    /// A timetabled journey on a stated operational day.
    Framed(&'a FramedVehicleJourneyRef),
    /// A timetabled journey, the operational day being understood.
    Dated(&'a DatedVehicleJourneyRef),
    /// A journey that is in no timetable, under the producer's own code.
    Code(&'a str),
}

/// Whether an [`EstimatedVehicleJourney`] adds a run or takes one away.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum JourneyAlteration {
    /// The journey is being run in addition to the timetable.
    Extra,
    /// The timetabled journey will not run.
    Cancelled,
}

impl EstimatedVehicleJourney {
    /// A journey on the given line and direction, identified by a timetabled run.
    pub fn dated(
        line_ref: impl Into<LineRef>,
        direction_ref: impl Into<DirectionRef>,
        dated_vehicle_journey_ref: impl Into<DatedVehicleJourneyRef>,
    ) -> Self {
        Self {
            recorded_at_time: None,
            line_ref: line_ref.into(),
            direction_ref: direction_ref.into(),
            framed_vehicle_journey_ref: None,
            dated_vehicle_journey_ref: Some(dated_vehicle_journey_ref.into()),
            dated_vehicle_journey_indirect_ref: None,
            estimated_vehicle_journey_code: None,
            extra_journey: None,
            cancellation: None,
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
            origin_ref: None,
            origin_name: Vec::new(),
            origin_short_name: Vec::new(),
            destination_display_at_origin: Vec::new(),
            via: Vec::new(),
            destination_ref: None,
            destination_name: Vec::new(),
            destination_short_name: Vec::new(),
            origin_display_at_destination: Vec::new(),
            operator_ref: None,
            product_category_ref: None,
            service_feature_ref: Vec::new(),
            vehicle_feature_ref: Vec::new(),
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
            monitored: None,
            monitoring_error: Vec::new(),
            in_congestion: None,
            in_panic: None,
            prediction_inaccurate: None,
            prediction_inaccurate_reason: None,
            data_source: None,
            confidence_level: None,
            vehicle_location: None,
            location_recorded_at_time: None,
            bearing: None,
            progress_rate: None,
            velocity: None,
            engine_on: None,
            occupancy: None,
            delay: None,
            progress_status: Vec::new(),
            vehicle_status: None,
            train_block_part: Vec::new(),
            block_ref: None,
            course_of_journey_ref: None,
            vehicle_journey_ref: None,
            vehicle_ref: None,
            additional_vehicle_journey_ref: Vec::new(),
            driver_ref: None,
            driver_name: None,
            train_numbers: None,
            journey_parts: None,
            train_elements: None,
            trains: None,
            compound_trains: None,
            recorded_calls: None,
            estimated_calls: None,
            is_complete_stop_sequence: None,
            journey_relations: None,
            extensions: None,
        }
    }

    /// The same journey, cancelled.
    pub fn cancelled(mut self) -> Self {
        self.cancellation = Some(true);
        self
    }

    /// The same journey, carrying the given calls still to come.
    pub fn with_estimated_calls(mut self, estimated_call: Vec<EstimatedCall>) -> Self {
        self.estimated_calls = Some(EstimatedCalls { estimated_call });
        self
    }

    /// How the journey names the run it is about, or `None` when it names none.
    pub fn identity(&self) -> Option<JourneyIdentity<'_>> {
        self.framed_vehicle_journey_ref
            .as_ref()
            .map(JourneyIdentity::Framed)
            .or_else(|| {
                self.dated_vehicle_journey_ref
                    .as_ref()
                    .map(JourneyIdentity::Dated)
            })
            .or_else(|| {
                self.estimated_vehicle_journey_code
                    .as_deref()
                    .map(JourneyIdentity::Code)
            })
    }

    /// Whether the journey adds a run or takes one away, or `None` when it does
    /// neither and merely updates a timetabled run.
    pub fn alteration(&self) -> Option<JourneyAlteration> {
        if self.extra_journey == Some(true) {
            Some(JourneyAlteration::Extra)
        } else if self.cancellation == Some(true) {
            Some(JourneyAlteration::Cancelled)
        } else {
            None
        }
    }

    /// The calls still to come.
    pub fn estimated_calls(&self) -> &[EstimatedCall] {
        self.estimated_calls
            .as_ref()
            .map(|calls| calls.estimated_call.as_slice())
            .unwrap_or_default()
    }

    /// The calls that have already happened.
    pub fn recorded_calls(&self) -> &[RecordedCall] {
        self.recorded_calls
            .as_ref()
            .map(|calls| calls.recorded_call.as_slice())
            .unwrap_or_default()
    }
}

/// The calls of a journey that are still to come.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EstimatedCalls {
    /// The calls, at least one.
    #[serde(rename = "EstimatedCall")]
    pub estimated_call: Vec<EstimatedCall>,
}

/// The calls of a journey that have already happened.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RecordedCalls {
    /// The calls, at least one.
    #[serde(rename = "RecordedCall")]
    pub recorded_call: Vec<RecordedCall>,
}

/// A call still to come, with what is now expected to happen at it.
///
/// A call that is being skipped carries `cancellation`; one that is being made in
/// addition to the timetable carries `extra_call`.
/// [`EstimatedCall::alteration`] reports which.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EstimatedCall {
    /// The stop.
    #[serde(rename = "StopPointRef")]
    pub stop_point_ref: StopPointRef,
    /// Which visit to that stop this is, when the journey calls more than once.
    #[serde(rename = "VisitNumber", default, skip_serializing_if = "Option::is_none")]
    pub visit_number: Option<u64>,
    /// Where the stop comes in the journey, counting from one.
    #[serde(rename = "Order", default, skip_serializing_if = "Option::is_none")]
    pub order: Option<u64>,
    /// Names of the stop, one per language.
    #[serde(rename = "StopPointName", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_point_name: Vec<NaturalLanguageString>,
    /// Whether the call is being made in addition to the timetable.
    #[serde(rename = "ExtraCall", default, skip_serializing_if = "Option::is_none")]
    pub extra_call: Option<bool>,
    /// Whether the stop is being skipped.
    #[serde(rename = "Cancellation", default, skip_serializing_if = "Option::is_none")]
    pub cancellation: Option<bool>,
    /// Whether the predicted times should be treated as unreliable.
    #[serde(rename = "PredictionInaccurate", default, skip_serializing_if = "Option::is_none")]
    pub prediction_inaccurate: Option<bool>,
    /// Why they are unreliable.
    #[serde(rename = "PredictionInaccurateReason", default, skip_serializing_if = "Option::is_none")]
    pub prediction_inaccurate_reason: Option<PredictionInaccurateReason>,
    /// How full the vehicle is expected to be here.
    #[serde(rename = "Occupancy", default, skip_serializing_if = "Option::is_none")]
    pub occupancy: Option<Occupancy>,
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

/// Whether a call is being added or skipped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CallAlteration {
    /// The call is being made in addition to the timetable.
    Extra,
    /// The stop is being skipped.
    Cancelled,
}

impl EstimatedCall {
    /// A call at the given stop with nothing said about it yet.
    pub fn at(stop_point_ref: impl Into<StopPointRef>) -> Self {
        Self {
            stop_point_ref: stop_point_ref.into(),
            visit_number: None,
            order: None,
            stop_point_name: Vec::new(),
            extra_call: None,
            cancellation: None,
            prediction_inaccurate: None,
            prediction_inaccurate_reason: None,
            occupancy: None,
            timing_point: None,
            boarding_stretch: None,
            request_stop: None,
            origin_display: Vec::new(),
            destination_display: Vec::new(),
            call_note: Vec::new(),
            formation_condition: Vec::new(),
            facility_condition_element: Vec::new(),
            facility_change_element: None,
            situation_ref: Vec::new(),
            control_action_ref: None,
            aimed_arrival_time: None,
            expected_arrival_time: None,
            latest_expected_arrival_time: None,
            expected_arrival_prediction_quality: None,
            arrival_prediction_unknown: None,
            arrival_status: None,
            arrival_cancellation_reason: Vec::new(),
            arrival_proximity_text: Vec::new(),
            arrival_platform_name: Vec::new(),
            arrival_boarding_activity: None,
            arrival_stop_assignment: Vec::new(),
            arrival_formation_assignment: Vec::new(),
            arrival_orientation_relative_to_quay: Vec::new(),
            arrival_operator_refs: Vec::new(),
            aimed_departure_time: None,
            expected_departure_time: None,
            provisional_expected_departure_time: None,
            earliest_expected_departure_time: None,
            expected_departure_prediction_quality: None,
            departure_prediction_unknown: None,
            aimed_latest_passenger_access_time: None,
            expected_latest_passenger_access_time: None,
            departure_status: None,
            departure_cancellation_reason: Vec::new(),
            departure_proximity_text: Vec::new(),
            departure_platform_name: Vec::new(),
            departure_boarding_activity: None,
            departure_stop_assignment: Vec::new(),
            departure_formation_assignment: Vec::new(),
            departure_orientation_relative_to_quay: Vec::new(),
            expected_departure_occupancy: Vec::new(),
            expected_departure_capacities: Vec::new(),
            recorded_departure_occupancy: Vec::new(),
            recorded_departure_capacities: Vec::new(),
            departure_operator_refs: Vec::new(),
            aimed_headway_interval: None,
            expected_headway_interval: None,
            distance_from_stop: None,
            number_of_stops_away: None,
            extensions: None,
        }
    }

    /// The same call, skipped.
    pub fn cancelled(mut self) -> Self {
        self.cancellation = Some(true);
        self
    }

    /// Whether the call is being added or skipped, or `None` when it is neither.
    pub fn alteration(&self) -> Option<CallAlteration> {
        if self.extra_call == Some(true) {
            Some(CallAlteration::Extra)
        } else if self.cancellation == Some(true) {
            Some(CallAlteration::Cancelled)
        } else {
            None
        }
    }
}

/// A call that has already happened, with what actually did.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordedCall {
    /// The stop.
    #[serde(rename = "StopPointRef")]
    pub stop_point_ref: StopPointRef,
    /// Which visit to that stop this was, when the journey calls more than once.
    #[serde(rename = "VisitNumber", default, skip_serializing_if = "Option::is_none")]
    pub visit_number: Option<u64>,
    /// Where the stop came in the journey, counting from one.
    #[serde(rename = "Order", default, skip_serializing_if = "Option::is_none")]
    pub order: Option<u64>,
    /// Names of the stop, one per language.
    #[serde(rename = "StopPointName", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_point_name: Vec<NaturalLanguageString>,
    /// Whether the call was made in addition to the timetable.
    #[serde(rename = "ExtraCall", default, skip_serializing_if = "Option::is_none")]
    pub extra_call: Option<bool>,
    /// Whether the stop was skipped.
    #[serde(rename = "Cancellation", default, skip_serializing_if = "Option::is_none")]
    pub cancellation: Option<bool>,
    /// Whether the times recorded should be treated as unreliable.
    #[serde(rename = "PredictionInaccurate", default, skip_serializing_if = "Option::is_none")]
    pub prediction_inaccurate: Option<bool>,
    /// Why they are unreliable.
    #[serde(rename = "PredictionInaccurateReason", default, skip_serializing_if = "Option::is_none")]
    pub prediction_inaccurate_reason: Option<PredictionInaccurateReason>,
    /// How full the vehicle was here.
    #[serde(rename = "Occupancy", default, skip_serializing_if = "Option::is_none")]
    pub occupancy: Option<Occupancy>,
    /// Whether the stop is a timing point the timetable is measured against.
    #[serde(rename = "TimingPoint", default, skip_serializing_if = "Option::is_none")]
    pub timing_point: Option<bool>,
    /// Whether passengers could board anywhere along the stretch before this stop.
    #[serde(rename = "BoardingStretch", default, skip_serializing_if = "Option::is_none")]
    pub boarding_stretch: Option<bool>,
    /// Whether the vehicle called only because it was asked to.
    #[serde(rename = "RequestStop", default, skip_serializing_if = "Option::is_none")]
    pub request_stop: Option<bool>,
    /// What was shown as the origin from this stop on, one per language.
    #[serde(rename = "OriginDisplay", default, skip_serializing_if = "Vec::is_empty")]
    pub origin_display: Vec<NaturalLanguageString>,
    /// What was shown as the destination from this stop on, one per language.
    #[serde(rename = "DestinationDisplay", default, skip_serializing_if = "Vec::is_empty")]
    pub destination_display: Vec<NaturalLanguageString>,
    /// Notes about this call, one per language.
    #[serde(rename = "CallNote", default, skip_serializing_if = "Vec::is_empty")]
    pub call_note: Vec<NaturalLanguageString>,
    /// Changes to how the train was put together, taking effect here.
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
    /// When the vehicle was planned to arrive.
    #[serde(rename = "AimedArrivalTime", default, skip_serializing_if = "Option::is_none")]
    pub aimed_arrival_time: Option<DateTime<FixedOffset>>,
    /// When it was last expected to arrive.
    #[serde(rename = "ExpectedArrivalTime", default, skip_serializing_if = "Option::is_none")]
    pub expected_arrival_time: Option<DateTime<FixedOffset>>,
    /// When it did arrive.
    #[serde(rename = "ActualArrivalTime", default, skip_serializing_if = "Option::is_none")]
    pub actual_arrival_time: Option<DateTime<FixedOffset>>,
    /// How the arrival stood against the timetable.
    #[serde(rename = "ArrivalStatus", default, skip_serializing_if = "Option::is_none")]
    pub arrival_status: Option<CallStatus>,
    /// Why the arrival was cancelled, one per language.
    #[serde(rename = "ArrivalCancellationReason", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_cancellation_reason: Vec<NaturalLanguageString>,
    /// How near the vehicle was, in words for a display.
    #[serde(rename = "ArrivalProximityText", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_proximity_text: Vec<NaturalLanguageString>,
    /// The platform the vehicle arrived at, one name per language.
    #[serde(rename = "ArrivalPlatformName", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_platform_name: Vec<NaturalLanguageString>,
    /// Whether passengers could alight here.
    #[serde(rename = "ArrivalBoardingActivity", default, skip_serializing_if = "Option::is_none")]
    pub arrival_boarding_activity: Option<ArrivalBoardingActivity>,
    /// Where the vehicle stood on arrival.
    #[serde(rename = "ArrivalStopAssignment", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_stop_assignment: Vec<StopAssignment>,
    /// Where each part of the train stood on arrival.
    #[serde(rename = "ArrivalFormationAssignment", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_formation_assignment: Vec<FormationAssignment>,
    /// Which way round the vehicle stood on arrival, one text per language.
    #[serde(rename = "ArrivalOrientationRelativeToQuay", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_orientation_relative_to_quay: Vec<NaturalLanguageString>,
    /// The operators whose tickets were valid on arrival.
    #[serde(rename = "ArrivalOperatorRefs", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_operator_refs: Vec<OperatorRef>,
    /// When the vehicle was planned to leave.
    #[serde(rename = "AimedDepartureTime", default, skip_serializing_if = "Option::is_none")]
    pub aimed_departure_time: Option<DateTime<FixedOffset>>,
    /// When it was last expected to leave.
    #[serde(rename = "ExpectedDepartureTime", default, skip_serializing_if = "Option::is_none")]
    pub expected_departure_time: Option<DateTime<FixedOffset>>,
    /// The platform the vehicle left from, one name per language.
    ///
    /// The schema puts this element here, between the expected and the actual
    /// departure, only in a recorded call; everywhere else it comes after the
    /// departure status. The schema says so explicitly, and calls it an old mistake
    /// that backward compatibility forbids correcting.
    #[serde(rename = "DeparturePlatformName", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_platform_name: Vec<NaturalLanguageString>,
    /// When it did leave.
    #[serde(rename = "ActualDepartureTime", default, skip_serializing_if = "Option::is_none")]
    pub actual_departure_time: Option<DateTime<FixedOffset>>,
    /// How the departure stood against the timetable.
    #[serde(rename = "DepartureStatus", default, skip_serializing_if = "Option::is_none")]
    pub departure_status: Option<CallStatus>,
    /// Why the departure was cancelled, one per language.
    #[serde(rename = "DepartureCancellationReason", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_cancellation_reason: Vec<NaturalLanguageString>,
    /// How near departure was, in words for a display.
    #[serde(rename = "DepartureProximityText", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_proximity_text: Vec<NaturalLanguageString>,
    /// Whether passengers could board here.
    #[serde(rename = "DepartureBoardingActivity", default, skip_serializing_if = "Option::is_none")]
    pub departure_boarding_activity: Option<DepartureBoardingActivity>,
    /// Where the vehicle stood for departure.
    #[serde(rename = "DepartureStopAssignment", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_stop_assignment: Vec<StopAssignment>,
    /// Where each part of the train stood for departure.
    #[serde(rename = "DepartureFormationAssignment", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_formation_assignment: Vec<FormationAssignment>,
    /// Which way round the vehicle stood for departure, one text per language.
    #[serde(rename = "DepartureOrientationRelativeToQuay", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_orientation_relative_to_quay: Vec<NaturalLanguageString>,
    /// How full the vehicle was when it left.
    #[serde(rename = "RecordedDepartureOccupancy", default, skip_serializing_if = "Vec::is_empty")]
    pub recorded_departure_occupancy: Vec<VehicleOccupancy>,
    /// How many passengers the vehicle could take when it left.
    #[serde(rename = "RecordedDepartureCapacities", default, skip_serializing_if = "Vec::is_empty")]
    pub recorded_departure_capacities: Vec<PassengerCapacity>,
    /// The operators whose tickets were valid on departure.
    #[serde(rename = "DepartureOperatorRefs", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_operator_refs: Vec<OperatorRef>,
    /// The planned interval between vehicles on a headway service.
    #[serde(rename = "AimedHeadwayInterval", default, skip_serializing_if = "Option::is_none")]
    pub aimed_headway_interval: Option<Duration>,
    /// The interval that was last expected.
    #[serde(rename = "ExpectedHeadwayInterval", default, skip_serializing_if = "Option::is_none")]
    pub expected_headway_interval: Option<Duration>,
    /// The interval there actually was.
    #[serde(rename = "ActualHeadwayInterval", default, skip_serializing_if = "Option::is_none")]
    pub actual_headway_interval: Option<Duration>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// An interchange between two journeys, as it is now expected to work out.
///
/// The feeder is the journey passengers arrive on, the distributor the one they
/// leave on. `will_not_wait` and `will_wait` are the schema's two answers to the
/// only question that matters once the feeder is late.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EstimatedServiceJourneyInterchange {
    /// The planned interchange this updates.
    #[serde(rename = "InterchangeRef", default, skip_serializing_if = "Option::is_none")]
    pub interchange_ref: Option<InterchangeRef>,
    /// The producer's code for an interchange that is not separately identified.
    #[serde(rename = "InterchangeCode", default, skip_serializing_if = "Option::is_none")]
    pub interchange_code: Option<String>,
    /// The link passengers walk along between the two.
    #[serde(rename = "ConnectionLinkRef", default, skip_serializing_if = "Option::is_none")]
    pub connection_link_ref: Option<ConnectionLinkRef>,
    /// The journey passengers arrive on.
    #[serde(rename = "FeederJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub feeder_journey_ref: Option<ConnectingJourneyRef>,
    /// Where they arrive.
    #[serde(rename = "FeederArrivalStopRef", default, skip_serializing_if = "Option::is_none")]
    pub feeder_arrival_stop_ref: Option<StopPointRef>,
    /// Which visit to that stop the arrival is.
    #[serde(rename = "FeederVisitNumber", default, skip_serializing_if = "Option::is_none")]
    pub feeder_visit_number: Option<u64>,
    /// Where that stop comes in the feeder's journey.
    #[serde(rename = "FeederStopOrder", default, skip_serializing_if = "Option::is_none")]
    pub feeder_stop_order: Option<u64>,
    /// When the feeder is planned to arrive.
    #[serde(rename = "AimedArrivalTimeOfFeeder", default, skip_serializing_if = "Option::is_none")]
    pub aimed_arrival_time_of_feeder: Option<DateTime<FixedOffset>>,
    /// The journey passengers leave on.
    #[serde(rename = "DistributorJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub distributor_journey_ref: Option<ConnectingJourneyRef>,
    /// Where they leave from.
    #[serde(rename = "DistributorDepartureStopRef", default, skip_serializing_if = "Option::is_none")]
    pub distributor_departure_stop_ref: Option<StopPointRef>,
    /// Which visit to that stop the departure is.
    #[serde(rename = "DistributorVisitNumber", default, skip_serializing_if = "Option::is_none")]
    pub distributor_visit_number: Option<u64>,
    /// Where that stop comes in the distributor's journey.
    #[serde(rename = "DistributorStopOrder", default, skip_serializing_if = "Option::is_none")]
    pub distributor_stop_order: Option<u64>,
    /// When the distributor is planned to leave.
    #[serde(rename = "AimedDepartureTimeOfDistributor", default, skip_serializing_if = "Option::is_none")]
    pub aimed_departure_time_of_distributor: Option<DateTime<FixedOffset>>,
    /// Whether passengers may stay in their seats.
    #[serde(rename = "StaySeated", default, skip_serializing_if = "Option::is_none")]
    pub stay_seated: Option<bool>,
    /// Whether the connection is guaranteed to be held.
    #[serde(rename = "Guaranteed", default, skip_serializing_if = "Option::is_none")]
    pub guaranteed: Option<bool>,
    /// Whether the connection is shown to passengers.
    #[serde(rename = "Advertised", default, skip_serializing_if = "Option::is_none")]
    pub advertised: Option<bool>,
    /// How the interchange is being managed.
    #[serde(rename = "InterchangeStatus", default, skip_serializing_if = "Option::is_none")]
    pub interchange_status: Option<InterchangeStatus>,
    /// That the distributor will leave without the feeder's passengers.
    #[serde(rename = "WillNotWait", default, skip_serializing_if = "Option::is_none")]
    pub will_not_wait: Option<Empty>,
    /// That the distributor will be held, and until when.
    #[serde(rename = "WillWait", default, skip_serializing_if = "Option::is_none")]
    pub will_wait: Option<WillWait>,
    /// When the feeder is now expected to arrive.
    #[serde(rename = "ExpectedArrivalTimeOfFeeder", default, skip_serializing_if = "Option::is_none")]
    pub expected_arrival_time_of_feeder: Option<DateTime<FixedOffset>>,
    /// When the distributor is now expected to leave.
    #[serde(rename = "ExpectedDepartureTimeOfDistributor", default, skip_serializing_if = "Option::is_none")]
    pub expected_departure_time_of_distributor: Option<DateTime<FixedOffset>>,
    /// Whether the connection is being watched.
    #[serde(rename = "ConnectionMonitoring", default, skip_serializing_if = "Option::is_none")]
    pub connection_monitoring: Option<bool>,
    /// How long the distributor normally waits.
    #[serde(rename = "StandardWaitTime", default, skip_serializing_if = "Option::is_none")]
    pub standard_wait_time: Option<Duration>,
    /// The longest it will wait.
    #[serde(rename = "MaximumWaitTime", default, skip_serializing_if = "Option::is_none")]
    pub maximum_wait_time: Option<Duration>,
    /// The longest it will wait without a controller deciding.
    #[serde(rename = "MaximumAutomaticWaitTime", default, skip_serializing_if = "Option::is_none")]
    pub maximum_automatic_wait_time: Option<Duration>,
    /// How long the transfer normally takes.
    #[serde(rename = "StandardTransferTime", default, skip_serializing_if = "Option::is_none")]
    pub standard_transfer_time: Option<Duration>,
    /// The shortest time it can be made in.
    #[serde(rename = "MinimumTransferTime", default, skip_serializing_if = "Option::is_none")]
    pub minimum_transfer_time: Option<Duration>,
    /// The longest it may take.
    #[serde(rename = "MaximumTransferTime", default, skip_serializing_if = "Option::is_none")]
    pub maximum_transfer_time: Option<Duration>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// That a distributor will be held for a late feeder, and until when.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WillWait {
    /// The latest the distributor will leave.
    #[serde(rename = "WaitUntilTime")]
    pub wait_until_time: DateTime<FixedOffset>,
    /// Whether the driver has confirmed the wait.
    ///
    /// The schema offers this element under two spellings, one of them a
    /// misspelling it keeps for backward compatibility; both are read, and the
    /// corrected spelling is what gets written.
    #[serde(rename = "DriverHasAcknowledgedWillWait", alias = "DriverHasAcknowledgeWIllWait", default, skip_serializing_if = "Option::is_none")]
    pub driver_has_acknowledged_will_wait: Option<bool>,
}

impl WillWait {
    /// A distributor held until the given time.
    pub fn until(wait_until_time: DateTime<FixedOffset>) -> Self {
        Self {
            wait_until_time,
            driver_has_acknowledged_will_wait: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::reference::DataFrameRef;

    fn timestamp() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2001-12-17T09:30:47-05:00").expect("valid timestamp")
    }

    #[test]
    fn a_journey_reports_which_run_it_names() {
        let dated = EstimatedVehicleJourney::dated("LZ123", "INBOUND", "00008");
        assert!(matches!(dated.identity(), Some(JourneyIdentity::Dated(reference))
            if reference.as_str() == "00008"));

        let mut framed = dated.clone();
        framed.dated_vehicle_journey_ref = None;
        framed.framed_vehicle_journey_ref = Some(FramedVehicleJourneyRef {
            data_frame_ref: DataFrameRef::new("2001-12-17"),
            dated_vehicle_journey_ref: DatedVehicleJourneyRef::new("00008"),
        });
        assert!(matches!(framed.identity(), Some(JourneyIdentity::Framed(_))));

        let mut unplanned = dated;
        unplanned.dated_vehicle_journey_ref = None;
        unplanned.estimated_vehicle_journey_code = Some("EXTRA-1".to_owned());
        assert!(matches!(unplanned.identity(), Some(JourneyIdentity::Code("EXTRA-1"))));
    }

    #[test]
    fn a_journey_reports_whether_it_adds_a_run_or_takes_one_away() {
        let running = EstimatedVehicleJourney::dated("LZ123", "INBOUND", "00008");
        assert_eq!(running.alteration(), None);
        assert_eq!(
            running.clone().cancelled().alteration(),
            Some(JourneyAlteration::Cancelled)
        );

        let mut extra = running;
        extra.extra_journey = Some(true);
        assert_eq!(extra.alteration(), Some(JourneyAlteration::Extra));
    }

    #[test]
    fn a_call_writes_the_arrival_before_the_departure_and_reads_back() {
        let call = EstimatedCall {
            aimed_arrival_time: Some(timestamp()),
            expected_arrival_time: Some(timestamp()),
            arrival_platform_name: vec![NaturalLanguageString::with_lang("EN", "4")],
            aimed_departure_time: Some(timestamp()),
            expected_departure_time: Some(timestamp()),
            departure_platform_name: vec![NaturalLanguageString::with_lang("EN", "3")],
            ..EstimatedCall::at("00002")
        };

        let xml = quick_xml::se::to_string_with_root("EstimatedCall", &call)
            .expect("the call serialises");
        let arrival = xml.find("<ExpectedArrivalTime>").expect("arrival is written");
        let platform = xml.find("<ArrivalPlatformName").expect("platform is written");
        let departure = xml
            .find("<AimedDepartureTime>")
            .expect("departure is written");
        assert!(arrival < platform && platform < departure, "{xml}");

        let read: EstimatedCall = quick_xml::de::from_str(&xml).expect("the call round-trips");
        assert_eq!(read, call);
    }

    #[test]
    fn a_call_reports_whether_it_is_being_added_or_skipped() {
        let call = EstimatedCall::at("00003");
        assert_eq!(call.alteration(), None);
        assert_eq!(
            call.clone().cancelled().alteration(),
            Some(CallAlteration::Cancelled)
        );

        let mut extra = call;
        extra.extra_call = Some(true);
        assert_eq!(extra.alteration(), Some(CallAlteration::Extra));
    }
}
