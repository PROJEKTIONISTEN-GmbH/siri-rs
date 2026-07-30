//! The planned view of a journey: what the timetable says will happen.
//!
//! A [`DatedVehicleJourney`] is one run on one operational day, with the stops it
//! is planned to make ([`DatedCall`]) and the connections planned around it. It
//! carries only aimed times: what is actually happening belongs to the estimated
//! and monitored views.
//!
//! As in those, the schema's groups are inlined: a group contributes its elements
//! to the enclosing sequence rather than nesting them.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::{
    ArrivalBoardingActivity, DepartureBoardingActivity, FirstOrLastJourney, VehicleModesOfTransport,
};
use crate::model::call::PlannedStopAssignment;
use crate::model::formation::{CompoundTrains, FormationAssignment, TrainElements, Trains};
use crate::model::journey::{
    Branding, BrandingRef, ConnectingJourneyRef, DatedVehicleJourneyIndirectRef, GroupOfLinesRef,
    JourneyParts, JourneyPlaceRef, JourneyRelations, SimpleContact, TrainBlockPart, TrainNumbers,
    ViaName,
};
use crate::model::reference::{
    BlockRef, ConnectionLinkRef, CourseOfJourneyRef, DatedVehicleJourneyRef, DestinationRef,
    FramedVehicleJourneyRef, InterchangeRef, JourneyPatternRef, LineRef, OperatorRef,
    ProductCategoryRef, RouteRef, ServiceFeatureRef, StopPointRef, VehicleFeatureRef,
    VehicleJourneyRef, VehicleRef,
};
use crate::types::{Duration, Extensions, NaturalLanguagePlaceName, NaturalLanguageString};

/// One planned run of a journey on one operational day.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DatedVehicleJourney {
    /// The producer's identifier for this run.
    #[serde(rename = "DatedVehicleJourneyCode", default, skip_serializing_if = "Option::is_none")]
    pub dated_vehicle_journey_code: Option<String>,
    /// The journey this run realises, on its operational day.
    #[serde(rename = "FramedVehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub framed_vehicle_journey_ref: Option<FramedVehicleJourneyRef>,
    /// The journey this run realises, the operational day being understood.
    #[serde(rename = "VehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_journey_ref: Option<VehicleJourneyRef>,
    /// Whether this run is being added to the timetable.
    #[serde(rename = "ExtraJourney", default, skip_serializing_if = "Option::is_none")]
    pub extra_journey: Option<bool>,
    /// Whether the timetabled run will not happen.
    #[serde(rename = "Cancellation", default, skip_serializing_if = "Option::is_none")]
    pub cancellation: Option<bool>,
    /// The journey pattern the run follows.
    #[serde(rename = "JourneyPatternRef", default, skip_serializing_if = "Option::is_none")]
    pub journey_pattern_ref: Option<JourneyPatternRef>,
    /// The journey pattern's name.
    #[serde(rename = "JourneyPatternName", default, skip_serializing_if = "Option::is_none")]
    pub journey_pattern_name: Option<NaturalLanguageString>,
    /// The modes of transport the run uses.
    #[serde(
        rename = "VehicleMode",
        default,
        deserialize_with = "crate::xml::token_list::deserialize",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub vehicle_mode: Vec<VehicleModesOfTransport>,
    /// The route the run follows.
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
    /// Where the run starts.
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
    /// Places passed through that tell this run apart from similar ones.
    #[serde(rename = "Via", default, skip_serializing_if = "Vec::is_empty")]
    pub via: Vec<ViaName>,
    /// Where the run ends.
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
    /// The operator running the service.
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
    /// The run's own name, one per language.
    #[serde(rename = "VehicleJourneyName", default, skip_serializing_if = "Vec::is_empty")]
    pub vehicle_journey_name: Vec<NaturalLanguageString>,
    /// Notes about the run, one per language.
    #[serde(rename = "JourneyNote", default, skip_serializing_if = "Vec::is_empty")]
    pub journey_note: Vec<NaturalLanguageString>,
    /// How passengers can reach the operator.
    #[serde(rename = "PublicContact", default, skip_serializing_if = "Option::is_none")]
    pub public_contact: Option<SimpleContact>,
    /// How staff can reach the operator's control room.
    #[serde(rename = "OperationsContact", default, skip_serializing_if = "Option::is_none")]
    pub operations_contact: Option<SimpleContact>,
    /// What is shown as the origin, one per language.
    #[serde(rename = "OriginDisplay", default, skip_serializing_if = "Vec::is_empty")]
    pub origin_display: Vec<NaturalLanguageString>,
    /// What is shown as the destination, one per language.
    #[serde(rename = "DestinationDisplay", default, skip_serializing_if = "Vec::is_empty")]
    pub destination_display: Vec<NaturalLanguageString>,
    /// Notes about the line, one per language.
    #[serde(rename = "LineNote", default, skip_serializing_if = "Vec::is_empty")]
    pub line_note: Vec<NaturalLanguagePlaceName>,
    /// Whether this is the first or the last run of the day on the line.
    #[serde(rename = "FirstOrLastJourney", default, skip_serializing_if = "Option::is_none")]
    pub first_or_last_journey: Option<FirstOrLastJourney>,
    /// Whether the service runs to a headway rather than to fixed times.
    #[serde(rename = "HeadwayService", default, skip_serializing_if = "Option::is_none")]
    pub headway_service: Option<bool>,
    /// Whether the run will be tracked in real time.
    #[serde(rename = "Monitored", default, skip_serializing_if = "Option::is_none")]
    pub monitored: Option<bool>,
    /// The parts the train is formed of, and where each sits.
    #[serde(rename = "TrainBlockPart", default, skip_serializing_if = "Vec::is_empty")]
    pub train_block_part: Vec<TrainBlockPart>,
    /// The day's work the run belongs to.
    #[serde(rename = "BlockRef", default, skip_serializing_if = "Option::is_none")]
    pub block_ref: Option<BlockRef>,
    /// The sequence of runs within that work.
    #[serde(rename = "CourseOfJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub course_of_journey_ref: Option<CourseOfJourneyRef>,
    /// The vehicle planned to run it.
    #[serde(rename = "VehicleRef", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_ref: Option<VehicleRef>,
    /// The train numbers the run is planned under.
    #[serde(rename = "TrainNumbers", default, skip_serializing_if = "Option::is_none")]
    pub train_numbers: Option<TrainNumbers>,
    /// The parts the run is split into.
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
    /// The stops the run is planned to make; the schema requires at least two.
    #[serde(rename = "DatedCalls")]
    pub dated_calls: DatedCalls,
    /// Other runs this one is joined to, split from or continues as.
    #[serde(rename = "JourneyRelations", default, skip_serializing_if = "Option::is_none")]
    pub journey_relations: Option<JourneyRelations>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl DatedVehicleJourney {
    /// A run under the given code, calling at the given stops.
    pub fn new(dated_vehicle_journey_code: impl Into<String>, dated_call: Vec<DatedCall>) -> Self {
        Self {
            dated_vehicle_journey_code: Some(dated_vehicle_journey_code.into()),
            dated_calls: DatedCalls { dated_call },
            ..Self::default()
        }
    }

    /// The stops the run is planned to make.
    pub fn dated_calls(&self) -> &[DatedCall] {
        &self.dated_calls.dated_call
    }
}

/// The stops a planned run is to make.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DatedCalls {
    /// The calls; the schema requires at least two.
    #[serde(rename = "DatedCall")]
    pub dated_call: Vec<DatedCall>,
}

/// One stop a planned run is to make, with its aimed times.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatedCall {
    /// The stop.
    #[serde(rename = "StopPointRef")]
    pub stop_point_ref: StopPointRef,
    /// Which visit to that stop this is, when the run calls more than once.
    #[serde(rename = "VisitNumber", default, skip_serializing_if = "Option::is_none")]
    pub visit_number: Option<u64>,
    /// Where the stop comes in the run, counting from one.
    #[serde(rename = "Order", default, skip_serializing_if = "Option::is_none")]
    pub order: Option<u64>,
    /// Names of the stop, one per language.
    #[serde(rename = "StopPointName", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_point_name: Vec<NaturalLanguageString>,
    /// Whether the call is being added to the timetable.
    #[serde(rename = "ExtraCall", default, skip_serializing_if = "Option::is_none")]
    pub extra_call: Option<bool>,
    /// Whether the stop is being skipped.
    #[serde(rename = "Cancellation", default, skip_serializing_if = "Option::is_none")]
    pub cancellation: Option<bool>,
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
    /// Connections onto other services planned from this call.
    #[serde(rename = "TargetedInterchange", default, skip_serializing_if = "Vec::is_empty")]
    pub targeted_interchange: Vec<TargetedInterchange>,
    /// Interchanges at which this call is the arrival passengers change from.
    #[serde(rename = "FromServiceJourneyInterchange", default, skip_serializing_if = "Vec::is_empty")]
    pub from_service_journey_interchange: Vec<FromServiceJourneyInterchange>,
    /// Interchanges at which this call is the departure passengers change onto.
    #[serde(rename = "ToServiceJourneyInterchange", default, skip_serializing_if = "Vec::is_empty")]
    pub to_service_journey_interchange: Vec<ToServiceJourneyInterchange>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl DatedCall {
    /// A call at the given stop with nothing said about it yet.
    pub fn at(stop_point_ref: impl Into<StopPointRef>) -> Self {
        Self {
            stop_point_ref: stop_point_ref.into(),
            visit_number: None,
            order: None,
            stop_point_name: Vec::new(),
            extra_call: None,
            cancellation: None,
            timing_point: None,
            boarding_stretch: None,
            request_stop: None,
            origin_display: Vec::new(),
            destination_display: Vec::new(),
            call_note: Vec::new(),
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
            targeted_interchange: Vec::new(),
            from_service_journey_interchange: Vec::new(),
            to_service_journey_interchange: Vec::new(),
            extensions: None,
        }
    }
}

/// A run that has been taken out of a timetable already published.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemovedDatedVehicleJourney {
    /// The run being removed.
    #[serde(rename = "FramedVehicleJourneyRef")]
    pub framed_vehicle_journey_ref: FramedVehicleJourneyRef,
    /// The same run, named by where and when it ran.
    #[serde(rename = "DatedVehicleJourneyIndirectRef", default, skip_serializing_if = "Option::is_none")]
    pub dated_vehicle_journey_indirect_ref: Option<DatedVehicleJourneyIndirectRef>,
    /// The train numbers it was planned under.
    #[serde(rename = "TrainNumbers", default, skip_serializing_if = "Option::is_none")]
    pub train_numbers: Option<TrainNumbers>,
    /// Why it was removed, in words.
    #[serde(rename = "ReasonForRemoval")]
    pub reason_for_removal: NaturalLanguageString,
}

/// A connection planned from one call onto a named onward service.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TargetedInterchange {
    /// The producer's code for the interchange.
    #[serde(rename = "InterchangeCode", default, skip_serializing_if = "Option::is_none")]
    pub interchange_code: Option<String>,
    /// The onward run passengers change onto.
    #[serde(rename = "DistributorVehicleJourneyRef")]
    pub distributor_vehicle_journey_ref: DatedVehicleJourneyRef,
    /// The link passengers walk along, stated elsewhere.
    #[serde(rename = "DistributorConnectionLinkRef", default, skip_serializing_if = "Option::is_none")]
    pub distributor_connection_link_ref: Option<ConnectionLinkRef>,
    /// The link passengers walk along, stated here.
    #[serde(rename = "DistributorConnectionLink", default, skip_serializing_if = "Option::is_none")]
    pub distributor_connection_link: Option<ContextualisedConnectionLink>,
    /// Which visit to the departure stop the onward service makes.
    #[serde(rename = "DistributorVisitNumber", default, skip_serializing_if = "Option::is_none")]
    pub distributor_visit_number: Option<u64>,
    /// Where that stop comes in the onward run.
    #[serde(rename = "DistributorOrder", default, skip_serializing_if = "Option::is_none")]
    pub distributor_order: Option<u64>,
    /// Whether passengers may stay in their seats.
    #[serde(rename = "StaySeated", default, skip_serializing_if = "Option::is_none")]
    pub stay_seated: Option<bool>,
    /// Whether the connection is guaranteed to be held.
    #[serde(rename = "Guaranteed", default, skip_serializing_if = "Option::is_none")]
    pub guaranteed: Option<bool>,
    /// Whether the connection is shown to passengers.
    #[serde(rename = "Advertised", default, skip_serializing_if = "Option::is_none")]
    pub advertised: Option<bool>,
    /// How long the onward service normally waits.
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
}

impl TargetedInterchange {
    /// A connection onto the given run, with nothing else said about it yet.
    pub fn onto(distributor_vehicle_journey_ref: impl Into<DatedVehicleJourneyRef>) -> Self {
        Self {
            interchange_code: None,
            distributor_vehicle_journey_ref: distributor_vehicle_journey_ref.into(),
            distributor_connection_link_ref: None,
            distributor_connection_link: None,
            distributor_visit_number: None,
            distributor_order: None,
            stay_seated: None,
            guaranteed: None,
            advertised: None,
            standard_wait_time: None,
            maximum_wait_time: None,
            maximum_automatic_wait_time: None,
            standard_transfer_time: None,
            minimum_transfer_time: None,
            maximum_transfer_time: None,
        }
    }
}

/// A walking link between two services, described where it is used.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ContextualisedConnectionLink {
    /// The producer's code for the link.
    #[serde(rename = "ConnectionLinkCode", default, skip_serializing_if = "Option::is_none")]
    pub connection_link_code: Option<String>,
    /// The stop the link runs within or from.
    #[serde(rename = "StopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_point_ref: Option<StopPointRef>,
    /// That stop's name.
    #[serde(rename = "StopPointName", default, skip_serializing_if = "Option::is_none")]
    pub stop_point_name: Option<NaturalLanguageString>,
    /// How long the walk normally takes.
    #[serde(rename = "DefaultDuration", default, skip_serializing_if = "Option::is_none")]
    pub default_duration: Option<Duration>,
    /// How long it takes someone who knows the way.
    #[serde(rename = "FrequentTravellerDuration", default, skip_serializing_if = "Option::is_none")]
    pub frequent_traveller_duration: Option<Duration>,
    /// How long it takes someone who does not.
    #[serde(rename = "OccasionalTravellerDuration", default, skip_serializing_if = "Option::is_none")]
    pub occasional_traveller_duration: Option<Duration>,
    /// How long it takes someone with impaired mobility.
    #[serde(rename = "ImpairedAccessDuration", default, skip_serializing_if = "Option::is_none")]
    pub impaired_access_duration: Option<Duration>,
}

/// A planned interchange between two runs, stated as part of a timetable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceJourneyInterchange {
    /// The producer's code for the interchange.
    #[serde(rename = "InterchangeCode", default, skip_serializing_if = "Option::is_none")]
    pub interchange_code: Option<String>,
    /// The link passengers walk along.
    #[serde(rename = "ConnectionLinkRef", default, skip_serializing_if = "Option::is_none")]
    pub connection_link_ref: Option<ConnectionLinkRef>,
    /// Whether the interchange is being added to the timetable.
    #[serde(rename = "ExtraInterchange", default, skip_serializing_if = "Option::is_none")]
    pub extra_interchange: Option<bool>,
    /// Whether the planned interchange will not happen.
    #[serde(rename = "Cancellation", default, skip_serializing_if = "Option::is_none")]
    pub cancellation: Option<bool>,
    /// The run passengers arrive on.
    #[serde(rename = "FeederRef")]
    pub feeder_ref: ConnectingJourneyRef,
    /// Where they arrive.
    #[serde(rename = "FeederArrivalStopRef")]
    pub feeder_arrival_stop_ref: StopPointRef,
    /// Which visit to that stop the arrival is.
    #[serde(rename = "FeederVisitNumber", default, skip_serializing_if = "Option::is_none")]
    pub feeder_visit_number: Option<u64>,
    /// Where that stop comes in the feeder's run.
    #[serde(rename = "FeederStopOrder", default, skip_serializing_if = "Option::is_none")]
    pub feeder_stop_order: Option<u64>,
    /// When the feeder is planned to arrive.
    #[serde(rename = "AimedArrivalTimeOfFeeder", default, skip_serializing_if = "Option::is_none")]
    pub aimed_arrival_time_of_feeder: Option<DateTime<FixedOffset>>,
    /// The run passengers leave on.
    #[serde(rename = "DistributorRef")]
    pub distributor_ref: ConnectingJourneyRef,
    /// Where they leave from.
    #[serde(rename = "DistributorDepartureStopRef")]
    pub distributor_departure_stop_ref: StopPointRef,
    /// Which visit to that stop the departure is.
    #[serde(rename = "DistributorVisitNumber", default, skip_serializing_if = "Option::is_none")]
    pub distributor_visit_number: Option<u64>,
    /// Where that stop comes in the distributor's run.
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

/// An interchange seen from the arriving side: the call is where passengers get off.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FromServiceJourneyInterchange {
    /// The producer's code for the interchange.
    #[serde(rename = "InterchangeCode", default, skip_serializing_if = "Option::is_none")]
    pub interchange_code: Option<String>,
    /// The link passengers walk along.
    #[serde(rename = "ConnectionLinkRef", default, skip_serializing_if = "Option::is_none")]
    pub connection_link_ref: Option<ConnectionLinkRef>,
    /// The run passengers arrive on.
    #[serde(rename = "FeederRef")]
    pub feeder_ref: ConnectingJourneyRef,
    /// Where they arrive.
    #[serde(rename = "FeederArrivalStopRef")]
    pub feeder_arrival_stop_ref: StopPointRef,
    /// Which visit to that stop the arrival is.
    #[serde(rename = "FeederVisitNumber", default, skip_serializing_if = "Option::is_none")]
    pub feeder_visit_number: Option<u64>,
    /// Where that stop comes in the feeder's run.
    #[serde(rename = "FeederStopOrder", default, skip_serializing_if = "Option::is_none")]
    pub feeder_stop_order: Option<u64>,
    /// When the feeder is planned to arrive.
    #[serde(rename = "AimedArrivalTimeOfFeeder", default, skip_serializing_if = "Option::is_none")]
    pub aimed_arrival_time_of_feeder: Option<DateTime<FixedOffset>>,
    /// Where the onward stop comes in the onward run.
    #[serde(rename = "DistributorStopOrder", default, skip_serializing_if = "Option::is_none")]
    pub distributor_stop_order: Option<u64>,
    /// When the onward service is planned to leave.
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
    /// How long the onward service normally waits.
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

/// An interchange seen from the onward side: the call is where passengers get on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToServiceJourneyInterchange {
    /// The producer's code for the interchange.
    #[serde(rename = "InterchangeCode", default, skip_serializing_if = "Option::is_none")]
    pub interchange_code: Option<String>,
    /// The link passengers walk along.
    #[serde(rename = "ConnectionLinkRef", default, skip_serializing_if = "Option::is_none")]
    pub connection_link_ref: Option<ConnectionLinkRef>,
    /// Where the arriving stop comes in the feeder's run.
    #[serde(rename = "FeederStopOrder", default, skip_serializing_if = "Option::is_none")]
    pub feeder_stop_order: Option<u64>,
    /// When the feeder is planned to arrive.
    #[serde(rename = "AimedArrivalTimeOfFeeder", default, skip_serializing_if = "Option::is_none")]
    pub aimed_arrival_time_of_feeder: Option<DateTime<FixedOffset>>,
    /// The run passengers leave on.
    #[serde(rename = "DistributorRef")]
    pub distributor_ref: ConnectingJourneyRef,
    /// Where they leave from.
    #[serde(rename = "DistributorDepartureStopRef")]
    pub distributor_departure_stop_ref: StopPointRef,
    /// Which visit to that stop the departure is.
    #[serde(rename = "DistributorVisitNumber", default, skip_serializing_if = "Option::is_none")]
    pub distributor_visit_number: Option<u64>,
    /// Where that stop comes in the distributor's run.
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

/// A planned interchange that has been taken out of a timetable already published.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemovedServiceJourneyInterchange {
    /// The interchange being removed.
    #[serde(rename = "InterchangeRef")]
    pub interchange_ref: InterchangeRef,
    /// The link passengers would have walked along.
    #[serde(rename = "ConnectionLinkRef", default, skip_serializing_if = "Option::is_none")]
    pub connection_link_ref: Option<ConnectionLinkRef>,
    /// Why it was removed, in words.
    #[serde(rename = "ReasonForRemoval")]
    pub reason_for_removal: NaturalLanguageString,
    /// The run passengers would have arrived on.
    #[serde(rename = "FeederRef")]
    pub feeder_ref: ConnectingJourneyRef,
    /// Where they would have arrived.
    #[serde(rename = "FeederArrivalStopRef")]
    pub feeder_arrival_stop_ref: StopPointRef,
    /// Which visit to that stop the arrival would have been.
    #[serde(rename = "FeederVisitNumber", default, skip_serializing_if = "Option::is_none")]
    pub feeder_visit_number: Option<u64>,
    /// Where that stop comes in the feeder's run.
    #[serde(rename = "FeederStopOrder", default, skip_serializing_if = "Option::is_none")]
    pub feeder_stop_order: Option<u64>,
    /// When the feeder was planned to arrive.
    #[serde(rename = "AimedArrivalTimeOfFeeder", default, skip_serializing_if = "Option::is_none")]
    pub aimed_arrival_time_of_feeder: Option<DateTime<FixedOffset>>,
    /// The run passengers would have left on.
    #[serde(rename = "DistributorRef")]
    pub distributor_ref: ConnectingJourneyRef,
    /// Where they would have left from.
    #[serde(rename = "DistributorDepartureStopRef")]
    pub distributor_departure_stop_ref: StopPointRef,
    /// Which visit to that stop the departure would have been.
    #[serde(rename = "DistributorVisitNumber", default, skip_serializing_if = "Option::is_none")]
    pub distributor_visit_number: Option<u64>,
    /// Where that stop comes in the distributor's run.
    #[serde(rename = "DistributorStopOrder", default, skip_serializing_if = "Option::is_none")]
    pub distributor_stop_order: Option<u64>,
    /// When the distributor was planned to leave.
    #[serde(rename = "AimedDepartureTimeOfDistributor", default, skip_serializing_if = "Option::is_none")]
    pub aimed_departure_time_of_distributor: Option<DateTime<FixedOffset>>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}
