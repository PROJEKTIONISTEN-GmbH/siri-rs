//! What a situation affects: the operators, networks, stops, places, journeys,
//! vehicles and roads a disruption reaches.
//!
//! A situation says *what happened*; this module says *where it lands*. The scope
//! is deliberately layered, and a producer states it at whichever layer it knows:
//! a whole operator, a network, one line, one direction of one route, a single
//! section between two stops, an individual call of a single journey, or a lift
//! inside a station. Consumers read the layers they can act on and ignore the rest,
//! so a feed that only knows "line 6 is disrupted" and one that knows "the third
//! call of the 08:14 run is cancelled" both remain valid.
//!
//! Values omitted at a lower layer are inherited from the layer above — an affected
//! line without a mode takes the mode of its affected network, which in turn takes
//! it from the situation's general context.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::{
    AccessModes, AccessibilityFeature, AirSubmodesOfTransport, AreaOfInterest,
    ArrivalBoardingActivity, BusSubmodesOfTransport, CallStatus, CoachSubmodesOfTransport,
    ConnectionDirection, DepartureBoardingActivity, FacilityStatus, InterchangeStatus,
    MetroSubmodesOfTransport, RailSubmodesOfTransport, RoutePointType, ServiceCondition,
    StopPlaceComponentType, StopPlaceType, StopPointType, TelecabinSubmodesOfTransport,
    TramSubmodesOfTransport, VehicleModesOfTransport, WaterSubmodesOfTransport,
};
use crate::model::{
    AccessibilityAssessment, BlockRef, ConnectionLinkRef, CourseOfJourneyRef, DatedVehicleJourneyRef,
    Direction, DirectionRef, FacilityRef, FramedVehicleJourneyRef, InterchangeRef, JourneyPartInfo,
    LineRef, LinkProjection, Location, OperationalUnitRef, OperatorRef, PlaceRef, PointProjection,
    ProductCategoryRef, QuayRef, RouteRef, ServiceFeatureRef, StopPlaceComponentRef, StopPlaceRef,
    StopPointRef, TrainBlockPart, TrainNumberRef, VehicleFeatureRef, VehicleJourneyRef, VehicleRef,
    ZoneProjection,
};
use crate::types::{
    AnyContent, Duration, Empty, Extensions, NaturalLanguagePlaceName, NaturalLanguageString,
};

siri_ref! {
    /// Identifies a fare or administrative zone, or the locality a stop lies in.
    ZoneRef;
    /// Identifies a network, i.e. the set of lines an operator presents as a whole.
    NetworkRef;
    /// Identifies a common section of route shared by several lines.
    SectionRef;
    /// Identifies a link between two consecutive points of a route.
    RouteLinkRef;
    /// Identifies a signposted walking route through a stop place.
    NavigationPathRef;
}

/// Everything a situation affects.
///
/// Each field narrows the scope in a different dimension, and a producer fills in
/// only the dimensions it can describe. An empty scope means the situation is not
/// tied to any part of the network — it applies wherever its context applies.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AffectsScope {
    /// How widely the situation reaches geographically, e.g. regional or national.
    #[serde(rename = "AreaOfInterest", default, skip_serializing_if = "Option::is_none")]
    pub area_of_interest: Option<AreaOfInterest>,
    /// Operators whose services are affected. If given, these replace any operators
    /// the situation would otherwise inherit from its context.
    #[serde(rename = "Operators", default, skip_serializing_if = "Option::is_none")]
    pub operators: Option<AffectsOperators>,
    /// Networks, and the lines, routes or sections within them, that are affected.
    #[serde(rename = "Networks", default, skip_serializing_if = "Option::is_none")]
    pub networks: Option<AffectsNetworks>,
    /// Scheduled stop points that are affected.
    #[serde(rename = "StopPoints", default, skip_serializing_if = "Option::is_none")]
    pub stop_points: Option<AffectedStopPoints>,
    /// Stop places — stations and interchanges — that are affected.
    #[serde(rename = "StopPlaces", default, skip_serializing_if = "Option::is_none")]
    pub stop_places: Option<AffectedStopPlaces>,
    /// Topographic places and sites, other than stops, that are affected.
    #[serde(rename = "Places", default, skip_serializing_if = "Option::is_none")]
    pub places: Option<AffectsPlaces>,
    /// Individual journeys that are affected.
    #[serde(rename = "VehicleJourneys", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_journeys: Option<AffectsVehicleJourneys>,
    /// Individual vehicles that are affected.
    #[serde(rename = "Vehicles", default, skip_serializing_if = "Option::is_none")]
    pub vehicles: Option<AffectsVehicles>,
    /// Roads that are affected, for situations that also concern road traffic.
    #[serde(rename = "Roads", default, skip_serializing_if = "Option::is_none")]
    pub roads: Option<AffectedRoads>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// The operators a situation affects.
///
/// The schema offers two alternatives — every operator, or a named list — so the
/// fields for both are optional. Build one with [`AffectsOperators::all`] or
/// [`AffectsOperators::these`] and read back which form is present with
/// [`AffectsOperators::scope`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AffectsOperators {
    /// Present when every operator is affected.
    #[serde(rename = "AllOperators", default, skip_serializing_if = "Option::is_none")]
    pub all_operators: Option<Empty>,
    /// The individual operators affected.
    #[serde(rename = "AffectedOperator", default, skip_serializing_if = "Vec::is_empty")]
    pub affected_operator: Vec<AffectedOperator>,
}

/// Which of the two ways of naming affected operators an [`AffectsOperators`] uses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperatorScope<'a> {
    /// Every operator is affected.
    AllOperators,
    /// Only the listed operators are affected.
    Operators(&'a [AffectedOperator]),
}

impl AffectsOperators {
    /// Every operator is affected.
    pub fn all() -> Self {
        Self {
            all_operators: Some(Empty::new()),
            affected_operator: Vec::new(),
        }
    }

    /// Only the given operators are affected.
    pub fn these(affected_operator: Vec<AffectedOperator>) -> Self {
        Self {
            all_operators: None,
            affected_operator,
        }
    }

    /// Which alternative of the schema's choice this value carries, or `None` when
    /// neither is present.
    pub fn scope(&self) -> Option<OperatorScope<'_>> {
        if self.all_operators.is_some() {
            Some(OperatorScope::AllOperators)
        } else if self.affected_operator.is_empty() {
            None
        } else {
            Some(OperatorScope::Operators(&self.affected_operator))
        }
    }
}

/// The networks a situation affects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AffectsNetworks {
    /// Each affected network, with the lines, routes or sections within it.
    #[serde(rename = "AffectedNetwork")]
    pub affected_network: Vec<AffectedNetwork>,
}

impl AffectsNetworks {
    /// The given networks are affected.
    pub fn new(affected_network: Vec<AffectedNetwork>) -> Self {
        Self { affected_network }
    }
}

/// A list of affected scheduled stop points.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AffectedStopPoints {
    /// Each affected stop point.
    #[serde(rename = "AffectedStopPoint")]
    pub affected_stop_point: Vec<AffectedStopPoint>,
}

impl AffectedStopPoints {
    /// The given stop points are affected.
    pub fn new(affected_stop_point: Vec<AffectedStopPoint>) -> Self {
        Self { affected_stop_point }
    }
}

/// A list of affected stop places.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AffectedStopPlaces {
    /// Each affected stop place.
    #[serde(rename = "AffectedStopPlace")]
    pub affected_stop_place: Vec<AffectedStopPlace>,
}

impl AffectedStopPlaces {
    /// The given stop places are affected.
    pub fn new(affected_stop_place: Vec<AffectedStopPlace>) -> Self {
        Self { affected_stop_place }
    }
}

/// The topographic places and sites a situation affects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AffectsPlaces {
    /// Each affected place.
    #[serde(rename = "AffectedPlace")]
    pub affected_place: Vec<AffectedPlace>,
}

impl AffectsPlaces {
    /// The given places are affected.
    pub fn new(affected_place: Vec<AffectedPlace>) -> Self {
        Self { affected_place }
    }
}

/// The individual journeys a situation affects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AffectsVehicleJourneys {
    /// Each affected journey.
    #[serde(rename = "AffectedVehicleJourney")]
    pub affected_vehicle_journey: Vec<AffectedVehicleJourney>,
}

impl AffectsVehicleJourneys {
    /// The given journeys are affected.
    pub fn new(affected_vehicle_journey: Vec<AffectedVehicleJourney>) -> Self {
        Self {
            affected_vehicle_journey,
        }
    }
}

/// The individual vehicles a situation affects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AffectsVehicles {
    /// Each affected vehicle.
    #[serde(rename = "AffectedVehicle")]
    pub affected_vehicle: Vec<AffectedVehicle>,
}

impl AffectsVehicles {
    /// The given vehicles are affected.
    pub fn new(affected_vehicle: Vec<AffectedVehicle>) -> Self {
        Self { affected_vehicle }
    }
}

/// The roads a situation affects.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AffectedRoads {
    /// Road network locations expressed in the DATEX II location referencing model.
    #[serde(rename = "Datex2Locations", default, skip_serializing_if = "Option::is_none")]
    pub datex2_locations: Option<Datex2Locations>,
    /// Each affected road, described individually.
    #[serde(rename = "AffectedRoad", default, skip_serializing_if = "Vec::is_empty")]
    pub affected_road: Vec<AffectedRoad>,
}

/// A road affected by a situation.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AffectedRoad {
    /// The road, as a DATEX II linear reference between two roadside reference
    /// points.
    #[serde(rename = "Road", default, skip_serializing_if = "Option::is_none")]
    pub road: Option<Datex2Road>,
    /// The path of the road, so that a consumer that does not already hold the road
    /// network can draw it.
    #[serde(rename = "LinkProjection", default, skip_serializing_if = "Option::is_none")]
    pub link_projection: Option<LinkProjection>,
    /// Which part of the projected path the situation applies to.
    #[serde(rename = "Offset", default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<Offset>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// A group of road network locations described by the DATEX II standard.
///
/// The element's content model belongs to DATEX II — the SIRI schema imports it and
/// types the element as `D2LogicalModel:GroupOfLocations` — so it is carried through
/// as the subtree it is, DATEX namespace included, rather than interpreted. A
/// consumer that holds the DATEX types can read it into them with
/// [`AnyContent::parse`].
pub type Datex2Locations = AnyContent;

/// A stretch of road between two reference points, as DATEX II describes it.
///
/// As with [`Datex2Locations`], the content model belongs to DATEX II — here
/// `D2LogicalModel:RoadsideReferencePointLinear` — and is carried through rather
/// than interpreted.
pub type Datex2Road = AnyContent;

/// How far along a projected link a situation begins and ends.
///
/// Both distances are measured in metres; an absent value means the situation runs
/// to that end of the link.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offset {
    /// Distance from the start of the link at which the situation begins.
    #[serde(rename = "DistanceFromStart", default, skip_serializing_if = "Option::is_none")]
    pub distance_from_start: Option<u64>,
    /// Distance from the end of the link at which the situation ends.
    #[serde(rename = "DistanceFromEnd", default, skip_serializing_if = "Option::is_none")]
    pub distance_from_end: Option<u64>,
}

/// An operator affected by a situation, named rather than merely referenced.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AffectedOperator {
    /// The operator.
    #[serde(rename = "OperatorRef", default, skip_serializing_if = "Option::is_none")]
    pub operator_ref: Option<OperatorRef>,
    /// The operator's public name, one entry per language.
    #[serde(rename = "OperatorName", default, skip_serializing_if = "Vec::is_empty")]
    pub operator_name: Vec<NaturalLanguageString>,
    /// A shortened form of the name, for displays with little room.
    #[serde(rename = "OperatorShortName", default, skip_serializing_if = "Vec::is_empty")]
    pub operator_short_name: Vec<NaturalLanguageString>,
    /// The parts of the operator's organisation that run the affected services.
    #[serde(rename = "OperationalUnitRef", default, skip_serializing_if = "Vec::is_empty")]
    pub operational_unit_ref: Vec<OperationalUnitRef>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// Which submode of the schema's transport mode choice a value carries.
///
/// The schema lets a producer refine a vehicle mode with exactly one submode drawn
/// from the family that matches it — a bus submode for a bus, a rail submode for a
/// train — so the alternatives are mutually exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtSubmode {
    /// A refinement of air transport, e.g. a domestic scheduled flight.
    Air(AirSubmodesOfTransport),
    /// A refinement of bus transport, e.g. a night bus or a rail replacement bus.
    Bus(BusSubmodesOfTransport),
    /// A refinement of coach transport, e.g. an international coach.
    Coach(CoachSubmodesOfTransport),
    /// A refinement of metro transport, e.g. an urban railway.
    Metro(MetroSubmodesOfTransport),
    /// A refinement of rail transport, e.g. a suburban or high-speed service.
    Rail(RailSubmodesOfTransport),
    /// A refinement of tram transport, e.g. a city or regional tram.
    Tram(TramSubmodesOfTransport),
    /// A refinement of water transport, e.g. a car ferry or a river bus.
    Water(WaterSubmodesOfTransport),
    /// A refinement of cable-hauled transport, e.g. a cable car or a chair lift.
    Telecabin(TelecabinSubmodesOfTransport),
}

/// Selects the single submode present among the eight alternatives.
macro_rules! pt_submode {
    ($self:expr) => {
        if let Some(value) = $self.air_submode {
            Some(PtSubmode::Air(value))
        } else if let Some(value) = $self.bus_submode {
            Some(PtSubmode::Bus(value))
        } else if let Some(value) = $self.coach_submode {
            Some(PtSubmode::Coach(value))
        } else if let Some(value) = $self.metro_submode {
            Some(PtSubmode::Metro(value))
        } else if let Some(value) = $self.rail_submode {
            Some(PtSubmode::Rail(value))
        } else if let Some(value) = $self.tram_submode {
            Some(PtSubmode::Tram(value))
        } else if let Some(value) = $self.water_submode {
            Some(PtSubmode::Water(value))
        } else {
            $self.telecabin_submode.map(PtSubmode::Telecabin)
        }
    };
}

/// A transport mode affected within a stop or a network.
///
/// The mode stated here overrides whatever mode the affected network or the
/// situation's general context would otherwise supply. The eight submode fields are
/// the alternatives of one schema choice: at most one is present, and
/// [`AffectedMode::pt_submode`] reports which.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AffectedMode {
    /// The broad mode of transport, e.g. bus, rail or ferry.
    #[serde(rename = "VehicleMode", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_mode: Option<VehicleModesOfTransport>,
    /// Refinement of the mode when it is air transport.
    #[serde(rename = "AirSubmode", default, skip_serializing_if = "Option::is_none")]
    pub air_submode: Option<AirSubmodesOfTransport>,
    /// Refinement of the mode when it is bus transport.
    #[serde(rename = "BusSubmode", default, skip_serializing_if = "Option::is_none")]
    pub bus_submode: Option<BusSubmodesOfTransport>,
    /// Refinement of the mode when it is coach transport.
    #[serde(rename = "CoachSubmode", default, skip_serializing_if = "Option::is_none")]
    pub coach_submode: Option<CoachSubmodesOfTransport>,
    /// Refinement of the mode when it is metro transport.
    #[serde(rename = "MetroSubmode", default, skip_serializing_if = "Option::is_none")]
    pub metro_submode: Option<MetroSubmodesOfTransport>,
    /// Refinement of the mode when it is rail transport.
    #[serde(rename = "RailSubmode", default, skip_serializing_if = "Option::is_none")]
    pub rail_submode: Option<RailSubmodesOfTransport>,
    /// Refinement of the mode when it is tram transport.
    #[serde(rename = "TramSubmode", default, skip_serializing_if = "Option::is_none")]
    pub tram_submode: Option<TramSubmodesOfTransport>,
    /// Refinement of the mode when it is water transport.
    #[serde(rename = "WaterSubmode", default, skip_serializing_if = "Option::is_none")]
    pub water_submode: Option<WaterSubmodesOfTransport>,
    /// Refinement of the mode when it is cable-hauled transport.
    #[serde(rename = "TelecabinSubmode", default, skip_serializing_if = "Option::is_none")]
    pub telecabin_submode: Option<TelecabinSubmodesOfTransport>,
    /// How passengers reach the affected place on foot or by other private means.
    #[serde(rename = "AccessMode", default, skip_serializing_if = "Option::is_none")]
    pub access_mode: Option<AccessModes>,
}

impl AffectedMode {
    /// The submode this value refines its vehicle mode with, if any.
    pub fn pt_submode(&self) -> Option<PtSubmode> {
        pt_submode!(self)
    }
}

/// The transport modes affected within a stop.
///
/// Either every mode served there is affected, or a list of individual modes is
/// given; build one with [`AffectedModes::all`] or [`AffectedModes::these`] and read
/// back which form is present with [`AffectedModes::scope`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AffectedModes {
    /// Present when every mode known at the stop is affected.
    #[serde(rename = "AllModes", default, skip_serializing_if = "Option::is_none")]
    pub all_modes: Option<Empty>,
    /// The individual modes affected.
    #[serde(rename = "Mode", default, skip_serializing_if = "Vec::is_empty")]
    pub mode: Vec<AffectedMode>,
}

/// Which of the two ways of naming affected modes an [`AffectedModes`] uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeScope<'a> {
    /// Every mode served at the stop is affected.
    AllModes,
    /// Only the listed modes are affected.
    Modes(&'a [AffectedMode]),
}

impl AffectedModes {
    /// Every mode served at the stop is affected.
    pub fn all() -> Self {
        Self {
            all_modes: Some(Empty::new()),
            mode: Vec::new(),
        }
    }

    /// Only the given modes are affected.
    pub fn these(mode: Vec<AffectedMode>) -> Self {
        Self {
            all_modes: None,
            mode,
        }
    }

    /// Which alternative of the schema's choice this value carries, or `None` when
    /// neither is present.
    pub fn scope(&self) -> Option<ModeScope<'_>> {
        if self.all_modes.is_some() {
            Some(ModeScope::AllModes)
        } else if self.mode.is_empty() {
            None
        } else {
            Some(ModeScope::Modes(&self.mode))
        }
    }
}

/// A network, named rather than merely referenced, together with its mode.
///
/// This is the shape a producer uses to state a default network for a batch of
/// situations, so that each situation need not repeat it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Network {
    /// The network. The schema types this reference as an operator reference,
    /// because a network is identified by the operator that presents it.
    #[serde(rename = "NetworkRef", default, skip_serializing_if = "Option::is_none")]
    pub network_ref: Option<OperatorRef>,
    /// The network's name, one entry per language.
    #[serde(rename = "NetworkName", default, skip_serializing_if = "Vec::is_empty")]
    pub network_name: Vec<NaturalLanguageString>,
    /// The broad mode of transport the network runs.
    #[serde(rename = "VehicleMode", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_mode: Option<VehicleModesOfTransport>,
    /// Refinement of the mode when it is air transport.
    #[serde(rename = "AirSubmode", default, skip_serializing_if = "Option::is_none")]
    pub air_submode: Option<AirSubmodesOfTransport>,
    /// Refinement of the mode when it is bus transport.
    #[serde(rename = "BusSubmode", default, skip_serializing_if = "Option::is_none")]
    pub bus_submode: Option<BusSubmodesOfTransport>,
    /// Refinement of the mode when it is coach transport.
    #[serde(rename = "CoachSubmode", default, skip_serializing_if = "Option::is_none")]
    pub coach_submode: Option<CoachSubmodesOfTransport>,
    /// Refinement of the mode when it is metro transport.
    #[serde(rename = "MetroSubmode", default, skip_serializing_if = "Option::is_none")]
    pub metro_submode: Option<MetroSubmodesOfTransport>,
    /// Refinement of the mode when it is rail transport.
    #[serde(rename = "RailSubmode", default, skip_serializing_if = "Option::is_none")]
    pub rail_submode: Option<RailSubmodesOfTransport>,
    /// Refinement of the mode when it is tram transport.
    #[serde(rename = "TramSubmode", default, skip_serializing_if = "Option::is_none")]
    pub tram_submode: Option<TramSubmodesOfTransport>,
    /// Refinement of the mode when it is water transport.
    #[serde(rename = "WaterSubmode", default, skip_serializing_if = "Option::is_none")]
    pub water_submode: Option<WaterSubmodesOfTransport>,
    /// Refinement of the mode when it is cable-hauled transport.
    #[serde(rename = "TelecabinSubmode", default, skip_serializing_if = "Option::is_none")]
    pub telecabin_submode: Option<TelecabinSubmodesOfTransport>,
    /// How passengers reach the network on foot or by other private means.
    #[serde(rename = "AccessMode", default, skip_serializing_if = "Option::is_none")]
    pub access_mode: Option<AccessModes>,
}

impl Network {
    /// The submode this network refines its vehicle mode with, if any.
    pub fn pt_submode(&self) -> Option<PtSubmode> {
        pt_submode!(self)
    }
}

/// The part of a network a situation affects.
///
/// The last four fields are the alternatives of one schema choice — every line, a
/// list of routes, a list of common sections, or a list of lines — so exactly one
/// of them is filled. Build one with [`AffectedNetwork::all_lines`],
/// [`AffectedNetwork::routes`], [`AffectedNetwork::sections`] or
/// [`AffectedNetwork::lines`], and read back which form is present with
/// [`AffectedNetwork::scope`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AffectedNetwork {
    /// Operators of the affected lines. These override any operator the situation's
    /// general context supplies.
    #[serde(rename = "AffectedOperator", default, skip_serializing_if = "Vec::is_empty")]
    pub affected_operator: Vec<AffectedOperator>,
    /// The network the affected lines belong to. If absent, taken from context.
    #[serde(rename = "NetworkRef", default, skip_serializing_if = "Option::is_none")]
    pub network_ref: Option<NetworkRef>,
    /// The network's name, one entry per language.
    #[serde(rename = "NetworkName", default, skip_serializing_if = "Vec::is_empty")]
    pub network_name: Vec<NaturalLanguageString>,
    /// Prose describing the routes affected, matching whatever the structured
    /// alternatives below state.
    #[serde(rename = "RoutesAffected", default, skip_serializing_if = "Vec::is_empty")]
    pub routes_affected: Vec<NaturalLanguageString>,
    /// The broad mode of transport affected within the network.
    #[serde(rename = "VehicleMode", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_mode: Option<VehicleModesOfTransport>,
    /// Refinement of the mode when it is air transport.
    #[serde(rename = "AirSubmode", default, skip_serializing_if = "Option::is_none")]
    pub air_submode: Option<AirSubmodesOfTransport>,
    /// Refinement of the mode when it is bus transport.
    #[serde(rename = "BusSubmode", default, skip_serializing_if = "Option::is_none")]
    pub bus_submode: Option<BusSubmodesOfTransport>,
    /// Refinement of the mode when it is coach transport.
    #[serde(rename = "CoachSubmode", default, skip_serializing_if = "Option::is_none")]
    pub coach_submode: Option<CoachSubmodesOfTransport>,
    /// Refinement of the mode when it is metro transport.
    #[serde(rename = "MetroSubmode", default, skip_serializing_if = "Option::is_none")]
    pub metro_submode: Option<MetroSubmodesOfTransport>,
    /// Refinement of the mode when it is rail transport.
    #[serde(rename = "RailSubmode", default, skip_serializing_if = "Option::is_none")]
    pub rail_submode: Option<RailSubmodesOfTransport>,
    /// Refinement of the mode when it is tram transport.
    #[serde(rename = "TramSubmode", default, skip_serializing_if = "Option::is_none")]
    pub tram_submode: Option<TramSubmodesOfTransport>,
    /// Refinement of the mode when it is water transport.
    #[serde(rename = "WaterSubmode", default, skip_serializing_if = "Option::is_none")]
    pub water_submode: Option<WaterSubmodesOfTransport>,
    /// Refinement of the mode when it is cable-hauled transport.
    #[serde(rename = "TelecabinSubmode", default, skip_serializing_if = "Option::is_none")]
    pub telecabin_submode: Option<TelecabinSubmodesOfTransport>,
    /// How passengers reach the network on foot or by other private means.
    #[serde(rename = "AccessMode", default, skip_serializing_if = "Option::is_none")]
    pub access_mode: Option<AccessModes>,
    /// Present when every line in the network is affected.
    #[serde(rename = "AllLines", default, skip_serializing_if = "Option::is_none")]
    pub all_lines: Option<Empty>,
    /// Affected routes, for a producer that knows routes but not the lines that run
    /// over them.
    #[serde(rename = "SelectedRoutes", default, skip_serializing_if = "Vec::is_empty")]
    pub selected_routes: Vec<AffectedRoute>,
    /// Affected common sections, for a producer that knows the shared stretch of
    /// network but not the lines that use it.
    #[serde(rename = "AffectedSection", default, skip_serializing_if = "Vec::is_empty")]
    pub affected_section: Vec<AffectedSection>,
    /// The individual affected lines.
    #[serde(rename = "AffectedLine", default, skip_serializing_if = "Vec::is_empty")]
    pub affected_line: Vec<AffectedLine>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// Which of the four ways of naming an affected part of a network an
/// [`AffectedNetwork`] uses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NetworkScope<'a> {
    /// Every line in the network is affected.
    AllLines,
    /// Only the listed routes are affected; line-level detail is not available.
    SelectedRoutes(&'a [AffectedRoute]),
    /// Only the listed common sections are affected.
    Sections(&'a [AffectedSection]),
    /// The listed lines are affected.
    Lines(&'a [AffectedLine]),
}

impl AffectedNetwork {
    /// Every line in the network is affected.
    pub fn all_lines() -> Self {
        Self {
            all_lines: Some(Empty::new()),
            ..Self::default()
        }
    }

    /// Only the given routes are affected.
    pub fn routes(selected_routes: Vec<AffectedRoute>) -> Self {
        Self {
            selected_routes,
            ..Self::default()
        }
    }

    /// Only the given common sections are affected.
    pub fn sections(affected_section: Vec<AffectedSection>) -> Self {
        Self {
            affected_section,
            ..Self::default()
        }
    }

    /// The given lines are affected.
    pub fn lines(affected_line: Vec<AffectedLine>) -> Self {
        Self {
            affected_line,
            ..Self::default()
        }
    }

    /// Which alternative of the schema's choice this value carries, or `None` when
    /// none is present.
    pub fn scope(&self) -> Option<NetworkScope<'_>> {
        if self.all_lines.is_some() {
            Some(NetworkScope::AllLines)
        } else if !self.selected_routes.is_empty() {
            Some(NetworkScope::SelectedRoutes(&self.selected_routes))
        } else if !self.affected_section.is_empty() {
            Some(NetworkScope::Sections(&self.affected_section))
        } else if !self.affected_line.is_empty() {
            Some(NetworkScope::Lines(&self.affected_line))
        } else {
            None
        }
    }

    /// The submode this network refines its vehicle mode with, if any.
    pub fn pt_submode(&self) -> Option<PtSubmode> {
        pt_submode!(self)
    }
}

/// A line affected by a situation, optionally narrowed to parts of that line.
///
/// The line itself is always named. Everything after it restricts the scope: only
/// journeys from certain origins, towards certain destinations, in certain
/// directions, over certain routes or sections, or calling at certain stops.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AffectedLine {
    /// Operators of the line. These override any operator the affected network or
    /// the general context supplies.
    #[serde(rename = "AffectedOperator", default, skip_serializing_if = "Vec::is_empty")]
    pub affected_operator: Vec<AffectedOperator>,
    /// The line.
    #[serde(rename = "LineRef")]
    pub line_ref: LineRef,
    /// The line number as shown to passengers, one entry per language.
    #[serde(rename = "PublishedLineName", default, skip_serializing_if = "Vec::is_empty")]
    pub published_line_name: Vec<NaturalLanguageString>,
    /// Restricts the scope to journeys starting from these stops.
    #[serde(rename = "Origins", default, skip_serializing_if = "Vec::is_empty")]
    pub origins: Vec<AffectedStopPoint>,
    /// Restricts the scope to journeys running towards these stops.
    #[serde(rename = "Destinations", default, skip_serializing_if = "Vec::is_empty")]
    pub destinations: Vec<AffectedStopPoint>,
    /// Restricts the scope to these directions of travel along the line.
    #[serde(rename = "Direction", default, skip_serializing_if = "Vec::is_empty")]
    pub direction: Vec<Direction>,
    /// Restricts the scope to these routes of the line.
    #[serde(rename = "Routes", default, skip_serializing_if = "Option::is_none")]
    pub routes: Option<AffectedRoutes>,
    /// Restricts the scope to these sections of the line.
    #[serde(rename = "Sections", default, skip_serializing_if = "Option::is_none")]
    pub sections: Option<AffectedSections>,
    /// Restricts the scope to these stop points on the line.
    #[serde(rename = "StopPoints", default, skip_serializing_if = "Option::is_none")]
    pub stop_points: Option<AffectedStopPoints>,
    /// Restricts the scope to these stop places on the line.
    #[serde(rename = "StopPlaces", default, skip_serializing_if = "Option::is_none")]
    pub stop_places: Option<AffectedStopPlaces>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl AffectedLine {
    /// A whole line is affected.
    pub fn new(line_ref: impl Into<LineRef>) -> Self {
        Self {
            affected_operator: Vec::new(),
            line_ref: line_ref.into(),
            published_line_name: Vec::new(),
            origins: Vec::new(),
            destinations: Vec::new(),
            direction: Vec::new(),
            routes: None,
            sections: None,
            stop_points: None,
            stop_places: None,
            extensions: None,
        }
    }
}

/// A list of affected lines, used to restrict a stop-oriented scope to some lines.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AffectedLines {
    /// Each affected line.
    #[serde(rename = "AffectedLine")]
    pub affected_line: Vec<AffectedLine>,
}

impl AffectedLines {
    /// The given lines are affected.
    pub fn new(affected_line: Vec<AffectedLine>) -> Self {
        Self { affected_line }
    }
}

/// A list of affected routes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AffectedRoutes {
    /// Each affected route.
    #[serde(rename = "AffectedRoute")]
    pub affected_route: Vec<AffectedRoute>,
}

impl AffectedRoutes {
    /// The given routes are affected.
    pub fn new(affected_route: Vec<AffectedRoute>) -> Self {
        Self { affected_route }
    }
}

/// A list of affected sections.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AffectedSections {
    /// Each affected section.
    #[serde(rename = "AffectedSection")]
    pub affected_section: Vec<AffectedSection>,
}

impl AffectedSections {
    /// The given sections are affected.
    pub fn new(affected_section: Vec<AffectedSection>) -> Self {
        Self { affected_section }
    }
}

/// A stretch of route affected by a situation.
///
/// The section is named either directly, by its identifier, or indirectly, by the
/// stops at its ends — the latter lets a producer describe a closure between two
/// stations without both sides agreeing on section identifiers first.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AffectedSection {
    /// How the section is identified.
    #[serde(rename = "$value", default, skip_serializing_if = "Option::is_none")]
    pub section: Option<SectionReference>,
    /// The path of the section, so that it can be drawn on a map.
    #[serde(rename = "LinkProjection", default, skip_serializing_if = "Option::is_none")]
    pub link_projection: Option<LinkProjection>,
    /// Which part of the projected path the situation applies to.
    #[serde(rename = "Offset", default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<Offset>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

// The indirect form carries seven fields against the direct form's single
// identifier, which is the schema's design rather than an oversight.
#[allow(clippy::large_enum_variant)]
/// The two ways of naming an affected section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SectionReference {
    /// The section's own identifier.
    SectionRef(SectionRef),
    /// The stops at the ends of the section, and any stops it must pass through.
    IndirectSectionRef(IndirectSectionRef),
}

/// A section named by the stops at its ends rather than by its identifier.
///
/// Intermediate points narrow the match further: where several sections share the
/// same first and last stop, listing a stop in between picks out the one that
/// passes through it and excludes the others.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IndirectSectionRef {
    /// The scheduled stop point at the start of the section.
    #[serde(rename = "FirstStopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub first_stop_point_ref: Option<StopPointRef>,
    /// Any stop point assigned to this stop place starts the section.
    #[serde(rename = "FirstStopPlaceRef", default, skip_serializing_if = "Option::is_none")]
    pub first_stop_place_ref: Option<StopPlaceRef>,
    /// Any stop point assigned to this quay starts the section.
    #[serde(rename = "FirstQuayRef", default, skip_serializing_if = "Option::is_none")]
    pub first_quay_ref: Option<QuayRef>,
    /// Points the section must pass through, in the order they were stated.
    #[serde(rename = "$value", default, skip_serializing_if = "Vec::is_empty")]
    pub intermediate: Vec<IntermediateSectionPoint>,
    /// The scheduled stop point at the end of the section.
    #[serde(rename = "LastStopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub last_stop_point_ref: Option<StopPointRef>,
    /// Any stop point assigned to this stop place ends the section.
    #[serde(rename = "LastStopPlaceRef", default, skip_serializing_if = "Option::is_none")]
    pub last_stop_place_ref: Option<StopPlaceRef>,
    /// Any stop point assigned to this quay ends the section.
    #[serde(rename = "LastQuayRef", default, skip_serializing_if = "Option::is_none")]
    pub last_quay_ref: Option<QuayRef>,
}

/// A point the section must pass through, at whichever level of detail is known.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntermediateSectionPoint {
    /// A scheduled stop point the section must include.
    IntermediateStopPointRef(StopPointRef),
    /// At least one stop point assigned to this stop place must be included.
    IntermediateStopPlaceRef(StopPlaceRef),
    /// At least one stop point assigned to this quay must be included.
    IntermediateQuayRef(QuayRef),
}

/// How one end of an indirectly named section is identified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionBoundary<'a> {
    /// A named scheduled stop point.
    StopPoint(&'a StopPointRef),
    /// Any stop point assigned to a named stop place.
    StopPlace(&'a StopPlaceRef),
    /// Any stop point assigned to a named quay.
    Quay(&'a QuayRef),
}

impl IndirectSectionRef {
    /// How the start of the section is identified, or `None` when it is not stated.
    pub fn first(&self) -> Option<SectionBoundary<'_>> {
        if let Some(value) = &self.first_stop_point_ref {
            Some(SectionBoundary::StopPoint(value))
        } else if let Some(value) = &self.first_stop_place_ref {
            Some(SectionBoundary::StopPlace(value))
        } else {
            self.first_quay_ref.as_ref().map(SectionBoundary::Quay)
        }
    }

    /// How the end of the section is identified, or `None` when it is not stated.
    pub fn last(&self) -> Option<SectionBoundary<'_>> {
        if let Some(value) = &self.last_stop_point_ref {
            Some(SectionBoundary::StopPoint(value))
        } else if let Some(value) = &self.last_stop_place_ref {
            Some(SectionBoundary::StopPlace(value))
        } else {
            self.last_quay_ref.as_ref().map(SectionBoundary::Quay)
        }
    }
}

/// A route affected by a situation, i.e. one path through the network.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AffectedRoute {
    /// The route.
    #[serde(rename = "RouteRef", default, skip_serializing_if = "Option::is_none")]
    pub route_ref: Option<RouteRef>,
    /// The directions of travel along the route that are affected.
    #[serde(rename = "Direction", default, skip_serializing_if = "Vec::is_empty")]
    pub direction: Vec<Direction>,
    /// The sections of the route that are affected.
    #[serde(rename = "Sections", default, skip_serializing_if = "Option::is_none")]
    pub sections: Option<AffectedSections>,
    /// The stop points of the route, either all of them or only the affected ones.
    #[serde(rename = "StopPoints", default, skip_serializing_if = "Option::is_none")]
    pub stop_points: Option<AffectedRouteStopPoints>,
    /// The links between route points that are affected.
    #[serde(rename = "RouteLinks", default, skip_serializing_if = "Option::is_none")]
    pub route_links: Option<AffectedRouteLinks>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// The stop points of a route, with the shape of the path between them.
///
/// The stop points and the projections between them share one ordered list, because
/// a projection describes the leg from the stop point before it to the one after.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AffectedRouteStopPoints {
    /// Whether the list holds only the affected stop points rather than the whole
    /// route. Absent means the whole route is listed.
    #[serde(rename = "AffectedOnly", default, skip_serializing_if = "Option::is_none")]
    pub affected_only: Option<bool>,
    /// The stop points and the projections between them, in order of travel.
    #[serde(rename = "$value", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_points: Vec<AffectedRouteStopPoint>,
}

// A stop point carries a full description where a projection carries only geometry,
// so the two alternatives differ in size by the schema's design.
#[allow(clippy::large_enum_variant)]
/// An entry in a route's ordered list of stop points and connecting geometry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AffectedRouteStopPoint {
    /// A stop point of the route.
    AffectedStopPoint(AffectedStopPoint),
    /// The shape of the path from the preceding stop point to the next one.
    LinkProjectionToNextStopPoint(LinkProjection),
}

/// The route links a situation affects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AffectedRouteLinks {
    /// Each affected link between two consecutive route points.
    #[serde(rename = "RouteLinkRef")]
    pub route_link_ref: Vec<RouteLinkRef>,
}

impl AffectedRouteLinks {
    /// The given route links are affected.
    pub fn new(route_link_ref: Vec<RouteLinkRef>) -> Self {
        Self { route_link_ref }
    }
}

/// A scheduled stop point affected by a situation.
///
/// Most of the descriptive fields — name, type, position, locality — can be derived
/// from the stop point reference, and a producer sends them so that a consumer
/// without the reference data can still render something useful.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AffectedStopPoint {
    /// The stop point.
    #[serde(rename = "StopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_point_ref: Option<StopPointRef>,
    /// An alternative code for the stop, private to the producer.
    #[serde(rename = "PrivateRef", default, skip_serializing_if = "Option::is_none")]
    pub private_ref: Option<String>,
    /// The stop's name, one entry per language.
    #[serde(rename = "StopPointName", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_point_name: Vec<NaturalLanguageString>,
    /// What kind of stop it is, e.g. a bus stop or a platform. Usually implied by
    /// the mode.
    #[serde(rename = "StopPointType", default, skip_serializing_if = "Option::is_none")]
    pub stop_point_type: Option<StopPointType>,
    /// Where the stop is.
    #[serde(rename = "Location", default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    /// The stop place this stop point belongs to.
    #[serde(rename = "StopPlaceRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_place_ref: Option<StopPlaceRef>,
    /// The stop place's name, one entry per language.
    #[serde(rename = "StopPlaceName", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_place_name: Vec<NaturalLanguageString>,
    /// The modes affected within the stop. If absent, every mode served there.
    #[serde(rename = "AffectedModes", default, skip_serializing_if = "Option::is_none")]
    pub affected_modes: Option<AffectedModes>,
    /// The locality or zone the stop lies in.
    #[serde(rename = "PlaceRef", default, skip_serializing_if = "Option::is_none")]
    pub place_ref: Option<ZoneRef>,
    /// The locality's name, one entry per language.
    #[serde(rename = "PlaceName", default, skip_serializing_if = "Vec::is_empty")]
    pub place_name: Vec<NaturalLanguageString>,
    /// How accessible the stop is while the situation lasts.
    #[serde(rename = "AccessibilityAssessment", default, skip_serializing_if = "Option::is_none")]
    pub accessibility_assessment: Option<AccessibilityAssessment>,
    /// What is happening at the stop, e.g. that services no longer call there or
    /// call at a temporary replacement.
    #[serde(
        rename = "StopCondition",
        default,
        deserialize_with = "crate::xml::token_list::deserialize",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub stop_condition: Vec<RoutePointType>,
    /// Interchange links from this stop that are affected.
    #[serde(rename = "ConnectionLinks", default, skip_serializing_if = "Option::is_none")]
    pub connection_links: Option<AffectedConnectionLinks>,
    /// Restricts the scope to these lines calling at the stop.
    #[serde(rename = "Lines", default, skip_serializing_if = "Option::is_none")]
    pub lines: Option<AffectedLines>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl AffectedStopPoint {
    /// A stop point affected by a situation.
    pub fn new(stop_point_ref: impl Into<StopPointRef>) -> Self {
        Self {
            stop_point_ref: Some(stop_point_ref.into()),
            ..Self::default()
        }
    }
}

/// The connection links from a stop that a situation affects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AffectedConnectionLinks {
    /// Each affected connection link.
    #[serde(rename = "AffectedConnectionLink")]
    pub affected_connection_link: Vec<AffectedConnectionLink>,
}

impl AffectedConnectionLinks {
    /// The given connection links are affected.
    pub fn new(affected_connection_link: Vec<AffectedConnectionLink>) -> Self {
        Self {
            affected_connection_link,
        }
    }
}

/// A connection link from a stop that a situation affects.
///
/// A connection link is the physical way a passenger gets from one service to
/// another. The scope is either every line reachable over the link, or a named line
/// and the stop the link leads to; build the first with
/// [`AffectedConnectionLink::all_lines`] and read back which form is present with
/// [`AffectedConnectionLink::scope`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AffectedConnectionLink {
    /// The connection links affected.
    #[serde(rename = "ConnectionLinkRef", default, skip_serializing_if = "Vec::is_empty")]
    pub connection_link_ref: Vec<ConnectionLinkRef>,
    /// The link's name.
    #[serde(rename = "ConnectionName", default, skip_serializing_if = "Option::is_none")]
    pub connection_name: Option<NaturalLanguageString>,
    /// Present when every line reachable over the link is affected.
    #[serde(rename = "AllLines", default, skip_serializing_if = "Option::is_none")]
    pub all_lines: Option<Empty>,
    /// The line reached over the link.
    #[serde(rename = "LineRef", default, skip_serializing_if = "Option::is_none")]
    pub line_ref: Option<LineRef>,
    /// That line's number as shown to passengers, one entry per language.
    #[serde(rename = "PublishedLineName", default, skip_serializing_if = "Vec::is_empty")]
    pub published_line_name: Vec<NaturalLanguageString>,
    /// The stop at the far end of the link. If absent, the feeder and the
    /// distributor use the same stop.
    #[serde(rename = "ConnectingStopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub connecting_stop_point_ref: Option<StopPointRef>,
    /// That stop's name, one entry per language.
    #[serde(rename = "ConnectingStopPointName", default, skip_serializing_if = "Vec::is_empty")]
    pub connecting_stop_point_name: Vec<NaturalLanguageString>,
    /// The zone the connecting stop lies in.
    #[serde(rename = "ConnectingZoneRef", default, skip_serializing_if = "Option::is_none")]
    pub connecting_zone_ref: Option<ZoneRef>,
    /// Which way along the link the situation applies. Absent means both ways.
    #[serde(rename = "ConnectionDirection", default, skip_serializing_if = "Option::is_none")]
    pub connection_direction: Option<ConnectionDirection>,
    /// The walking links making up the connection that are affected.
    #[serde(rename = "AffectedPathLink", default, skip_serializing_if = "Vec::is_empty")]
    pub affected_path_link: Vec<AffectedPathLink>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// Which of the two ways of scoping a connection link an
/// [`AffectedConnectionLink`] uses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionLinkScope<'a> {
    /// Every line reachable over the link is affected.
    AllLines,
    /// Only the connection to the named line or stop is affected.
    Connecting {
        /// The line reached over the link, if stated.
        line_ref: Option<&'a LineRef>,
        /// The stop at the far end of the link, if stated.
        connecting_stop_point_ref: Option<&'a StopPointRef>,
        /// The zone that stop lies in, if stated.
        connecting_zone_ref: Option<&'a ZoneRef>,
    },
}

impl AffectedConnectionLink {
    /// Every line reachable over the link is affected.
    pub fn all_lines() -> Self {
        Self {
            all_lines: Some(Empty::new()),
            ..Self::default()
        }
    }

    /// Which alternative of the schema's choice this value carries, or `None` when
    /// neither is present.
    pub fn scope(&self) -> Option<ConnectionLinkScope<'_>> {
        if self.all_lines.is_some() {
            return Some(ConnectionLinkScope::AllLines);
        }
        if self.line_ref.is_none()
            && self.connecting_stop_point_ref.is_none()
            && self.connecting_zone_ref.is_none()
            && self.published_line_name.is_empty()
            && self.connecting_stop_point_name.is_empty()
        {
            return None;
        }
        Some(ConnectionLinkScope::Connecting {
            line_ref: self.line_ref.as_ref(),
            connecting_stop_point_ref: self.connecting_stop_point_ref.as_ref(),
            connecting_zone_ref: self.connecting_zone_ref.as_ref(),
        })
    }
}

/// A walking link affected by a situation, e.g. a passage or a footbridge.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AffectedPathLink {
    /// Identifiers of the affected links.
    #[serde(rename = "LinkRef", default, skip_serializing_if = "Vec::is_empty")]
    pub link_ref: Vec<String>,
    /// The links' names, one entry per language.
    #[serde(rename = "LinkName", default, skip_serializing_if = "Vec::is_empty")]
    pub link_name: Vec<NaturalLanguageString>,
    /// What the passenger meets along the link, e.g. stairs, a lift or a ramp.
    #[serde(rename = "AccessibilityFeature", default, skip_serializing_if = "Option::is_none")]
    pub accessibility_feature: Option<AccessibilityFeature>,
    /// Prose describing which way along the link the situation applies.
    #[serde(rename = "LinkDirection", default, skip_serializing_if = "Vec::is_empty")]
    pub link_direction: Vec<String>,
    /// The shape of the link, so that it can be drawn on a map.
    #[serde(rename = "LinkProjection", default, skip_serializing_if = "Option::is_none")]
    pub link_projection: Option<LinkProjection>,
    /// Which part of the projected path the situation applies to.
    #[serde(rename = "Offset", default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<Offset>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// A planned interchange between two journeys that a situation affects.
///
/// This is what tells a consumer that a guaranteed connection will or will not be
/// held — the difference between a delay that is absorbed and one that strands a
/// passenger.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AffectedInterchange {
    /// The interchange.
    #[serde(rename = "InterchangeRef", default, skip_serializing_if = "Option::is_none")]
    pub interchange_ref: Option<InterchangeRef>,
    /// The stop the two journeys meet at. If absent, the same stop as the
    /// destination.
    #[serde(rename = "InterchangeStopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub interchange_stop_point_ref: Option<StopPointRef>,
    /// That stop's name, one entry per language.
    #[serde(rename = "InterchangeStopPointName", default, skip_serializing_if = "Vec::is_empty")]
    pub interchange_stop_point_name: Vec<NaturalLanguageString>,
    /// The journey on the other side of the interchange.
    #[serde(rename = "ConnectingVehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub connecting_vehicle_journey_ref: Option<DatedVehicleJourneyRef>,
    /// Whether the connection will be held, has been broken, or is uncertain.
    #[serde(rename = "InterchangeStatusType", default, skip_serializing_if = "Option::is_none")]
    pub interchange_status_type: Option<InterchangeStatus>,
    /// The connection links used to make the interchange.
    #[serde(rename = "ConnectionLink", default, skip_serializing_if = "Vec::is_empty")]
    pub connection_link: Vec<AffectedConnectionLink>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// The interchanges at a call that a situation affects.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AffectedInterchanges {
    /// Each affected interchange.
    #[serde(rename = "AffectedInterchange", default, skip_serializing_if = "Vec::is_empty")]
    pub affected_interchange: Vec<AffectedInterchange>,
}

/// A single journey affected by a situation.
///
/// The journey is identified either by a framed reference — the operating day plus
/// the journey's timetable identifier, which is the only form unique across days —
/// or by a bare journey reference, which the schema keeps for compatibility with
/// producers that predate the framed form.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AffectedVehicleJourney {
    /// The journey, framed by the operating day it runs on.
    #[serde(rename = "FramedVehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub framed_vehicle_journey_ref: Option<FramedVehicleJourneyRef>,
    /// Superseded alternative to `framed_vehicle_journey_ref`: bare journey
    /// references, which are only unique within one operating day.
    #[serde(rename = "VehicleJourneyRef", default, skip_serializing_if = "Vec::is_empty")]
    pub vehicle_journey_ref: Vec<VehicleJourneyRef>,
    /// Specific dated runs of the journey that are affected.
    #[serde(rename = "DatedVehicleJourneyRef", default, skip_serializing_if = "Vec::is_empty")]
    pub dated_vehicle_journey_ref: Vec<DatedVehicleJourneyRef>,
    /// The journey's name, one entry per language.
    #[serde(rename = "JourneyName", default, skip_serializing_if = "Vec::is_empty")]
    pub journey_name: Vec<NaturalLanguageString>,
    /// The operator running the journey.
    #[serde(rename = "Operator", default, skip_serializing_if = "Option::is_none")]
    pub operator: Option<AffectedOperator>,
    /// The line the journey runs on.
    #[serde(rename = "LineRef", default, skip_serializing_if = "Option::is_none")]
    pub line_ref: Option<LineRef>,
    /// The line number as shown to passengers, one entry per language.
    #[serde(rename = "PublishedLineName", default, skip_serializing_if = "Vec::is_empty")]
    pub published_line_name: Vec<NaturalLanguageString>,
    /// The direction of the line the journey runs in.
    #[serde(rename = "DirectionRef", default, skip_serializing_if = "Option::is_none")]
    pub direction_ref: Option<DirectionRef>,
    /// The block — the day's work for one vehicle — the journey belongs to.
    #[serde(rename = "BlockRef", default, skip_serializing_if = "Option::is_none")]
    pub block_ref: Option<BlockRef>,
    /// The train numbers assigned to the journey.
    #[serde(rename = "TrainNumbers", default, skip_serializing_if = "Option::is_none")]
    pub train_numbers: Option<TrainNumbers>,
    /// The parts the journey splits into, for coupled or divided trains.
    #[serde(rename = "JourneyParts", default, skip_serializing_if = "Option::is_none")]
    pub journey_parts: Option<JourneyParts>,
    /// Restricts the scope to journeys starting from these stops.
    #[serde(rename = "Origins", default, skip_serializing_if = "Vec::is_empty")]
    pub origins: Vec<AffectedStopPoint>,
    /// Restricts the scope to journeys running towards these stops.
    #[serde(rename = "Destinations", default, skip_serializing_if = "Vec::is_empty")]
    pub destinations: Vec<AffectedStopPoint>,
    /// The routes the journey follows that are affected.
    #[serde(rename = "Route", default, skip_serializing_if = "Vec::is_empty")]
    pub route: Vec<AffectedRoute>,
    /// The journey's timetabled departure from its origin.
    #[serde(rename = "OriginAimedDepartureTime", default, skip_serializing_if = "Option::is_none")]
    pub origin_aimed_departure_time: Option<DateTime<FixedOffset>>,
    /// The journey's timetabled arrival at its destination.
    #[serde(rename = "DestinationAimedArrivalTime", default, skip_serializing_if = "Option::is_none")]
    pub destination_aimed_arrival_time: Option<DateTime<FixedOffset>>,
    /// The origin as shown on displays at the destination, useful for identifying
    /// the journey to a passenger.
    #[serde(rename = "OriginDisplayAtDestination", default, skip_serializing_if = "Vec::is_empty")]
    pub origin_display_at_destination: Vec<NaturalLanguagePlaceName>,
    /// The destination as shown on displays at the origin.
    #[serde(rename = "DestinationDisplayAtOrigin", default, skip_serializing_if = "Vec::is_empty")]
    pub destination_display_at_origin: Vec<NaturalLanguagePlaceName>,
    /// How accessible the journey is while the situation lasts.
    #[serde(rename = "AccessibilityAssessment", default, skip_serializing_if = "Option::is_none")]
    pub accessibility_assessment: Option<AccessibilityAssessment>,
    /// What is happening to the journey, e.g. that it is cancelled, diverted or
    /// running as an extra. Several conditions may hold at once.
    #[serde(
        rename = "JourneyCondition",
        default,
        deserialize_with = "crate::xml::token_list::deserialize",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub journey_condition: Vec<ServiceCondition>,
    /// The calls making up the journey that are affected.
    #[serde(rename = "Calls", default, skip_serializing_if = "Option::is_none")]
    pub calls: Option<AffectedCalls>,
    /// Facilities on board the journey that are affected.
    #[serde(rename = "Facilities", default, skip_serializing_if = "Option::is_none")]
    pub facilities: Option<AffectedFacilities>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// Which of the two ways of identifying a journey an [`AffectedVehicleJourney`] uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AffectedJourney<'a> {
    /// The journey is framed by the operating day it runs on.
    Framed(&'a FramedVehicleJourneyRef),
    /// The journey is named by bare references, valid within one operating day.
    VehicleJourneys(&'a [VehicleJourneyRef]),
}

impl AffectedVehicleJourney {
    /// A journey identified by the operating day it runs on.
    pub fn framed(framed_vehicle_journey_ref: FramedVehicleJourneyRef) -> Self {
        Self {
            framed_vehicle_journey_ref: Some(framed_vehicle_journey_ref),
            ..Self::default()
        }
    }

    /// Journeys identified by bare references, valid within one operating day.
    pub fn vehicle_journeys(vehicle_journey_ref: Vec<VehicleJourneyRef>) -> Self {
        Self {
            vehicle_journey_ref,
            ..Self::default()
        }
    }

    /// Which alternative of the schema's choice this value carries, or `None` when
    /// neither is present.
    pub fn journey(&self) -> Option<AffectedJourney<'_>> {
        if let Some(framed) = &self.framed_vehicle_journey_ref {
            Some(AffectedJourney::Framed(framed))
        } else if self.vehicle_journey_ref.is_empty() {
            None
        } else {
            Some(AffectedJourney::VehicleJourneys(&self.vehicle_journey_ref))
        }
    }
}

/// The train numbers assigned to an affected journey.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainNumbers {
    /// Each train number.
    #[serde(rename = "TrainNumberRef")]
    pub train_number_ref: Vec<TrainNumberRef>,
}

impl TrainNumbers {
    /// The journey carries the given train numbers.
    pub fn new(train_number_ref: Vec<TrainNumberRef>) -> Self {
        Self { train_number_ref }
    }
}

/// The parts an affected journey splits into.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JourneyParts {
    /// Each part of the journey.
    #[serde(rename = "JourneyPartInfo")]
    pub journey_part_info: Vec<JourneyPartInfo>,
}

impl JourneyParts {
    /// The journey is made up of the given parts.
    pub fn new(journey_part_info: Vec<JourneyPartInfo>) -> Self {
        Self { journey_part_info }
    }
}

/// The calls of an affected journey.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AffectedCalls {
    /// Each affected call, in order of visit.
    #[serde(rename = "Call")]
    pub call: Vec<AffectedCall>,
}

impl AffectedCalls {
    /// The given calls are affected.
    pub fn new(call: Vec<AffectedCall>) -> Self {
        Self { call }
    }
}

/// The facilities a situation affects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AffectedFacilities {
    /// Each affected facility.
    #[serde(rename = "AffectedFacility")]
    pub affected_facility: Vec<AffectedFacility>,
}

impl AffectedFacilities {
    /// The given facilities are affected.
    pub fn new(affected_facility: Vec<AffectedFacility>) -> Self {
        Self { affected_facility }
    }
}

/// A passenger facility affected by a situation, e.g. a lift or an on-board toilet.
///
/// When the facility belongs to a journey rather than to a place, the two stop point
/// references bound the part of the journey over which the stated status holds.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AffectedFacility {
    /// The facility.
    #[serde(rename = "FacilityRef", default, skip_serializing_if = "Option::is_none")]
    pub facility_ref: Option<FacilityRef>,
    /// The stop from which the stated status applies.
    #[serde(rename = "StartStopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub start_stop_point_ref: Option<StopPointRef>,
    /// The stop up to which the stated status applies.
    #[serde(rename = "EndStopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub end_stop_point_ref: Option<StopPointRef>,
    /// The facility's name, one entry per language.
    #[serde(rename = "FacilityName", default, skip_serializing_if = "Vec::is_empty")]
    pub facility_name: Vec<NaturalLanguageString>,
    /// Whether the facility is available, not available or only partly available.
    #[serde(
        rename = "FacilityStatus",
        default,
        deserialize_with = "crate::xml::token_list::deserialize",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub facility_status: Vec<FacilityStatus>,
    /// Implementation-defined content. The schema allows more than one element here.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Vec::is_empty")]
    pub extensions: Vec<Extensions>,
}

/// A single call of a journey at a stop, affected by a situation.
///
/// The schema derives a call from an affected stop point, so every field describing
/// the stop appears first, in that type's order, followed by the fields that only
/// make sense for one visit: where it comes in the journey, when the vehicle is due,
/// and which connections depend on it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AffectedCall {
    /// The stop point called at.
    #[serde(rename = "StopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_point_ref: Option<StopPointRef>,
    /// An alternative code for the stop, private to the producer.
    #[serde(rename = "PrivateRef", default, skip_serializing_if = "Option::is_none")]
    pub private_ref: Option<String>,
    /// The stop's name, one entry per language.
    #[serde(rename = "StopPointName", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_point_name: Vec<NaturalLanguageString>,
    /// What kind of stop it is. Usually implied by the mode.
    #[serde(rename = "StopPointType", default, skip_serializing_if = "Option::is_none")]
    pub stop_point_type: Option<StopPointType>,
    /// Where the stop is.
    #[serde(rename = "Location", default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    /// The stop place this stop point belongs to.
    #[serde(rename = "StopPlaceRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_place_ref: Option<StopPlaceRef>,
    /// The stop place's name, one entry per language.
    #[serde(rename = "StopPlaceName", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_place_name: Vec<NaturalLanguageString>,
    /// The modes affected within the stop.
    #[serde(rename = "AffectedModes", default, skip_serializing_if = "Option::is_none")]
    pub affected_modes: Option<AffectedModes>,
    /// The locality or zone the stop lies in.
    #[serde(rename = "PlaceRef", default, skip_serializing_if = "Option::is_none")]
    pub place_ref: Option<ZoneRef>,
    /// The locality's name, one entry per language.
    #[serde(rename = "PlaceName", default, skip_serializing_if = "Vec::is_empty")]
    pub place_name: Vec<NaturalLanguageString>,
    /// How accessible the stop is while the situation lasts.
    #[serde(rename = "AccessibilityAssessment", default, skip_serializing_if = "Option::is_none")]
    pub accessibility_assessment: Option<AccessibilityAssessment>,
    /// What is happening at the stop itself.
    #[serde(
        rename = "StopCondition",
        default,
        deserialize_with = "crate::xml::token_list::deserialize",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub stop_condition: Vec<RoutePointType>,
    /// Interchange links from this stop that are affected.
    #[serde(rename = "ConnectionLinks", default, skip_serializing_if = "Option::is_none")]
    pub connection_links: Option<AffectedConnectionLinks>,
    /// Restricts the scope to these lines calling at the stop.
    #[serde(rename = "Lines", default, skip_serializing_if = "Option::is_none")]
    pub lines: Option<AffectedLines>,
    /// Implementation-defined content describing the stop.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
    /// Where this call comes in the journey's pattern, counting from one.
    #[serde(rename = "Order", default, skip_serializing_if = "Option::is_none")]
    pub order: Option<u64>,
    /// What is happening to this call, e.g. that it is cancelled or has moved to a
    /// temporary stop. Several conditions may hold at once.
    #[serde(
        rename = "CallCondition",
        default,
        deserialize_with = "crate::xml::token_list::deserialize",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub call_condition: Vec<RoutePointType>,
    /// Whether the vehicle is currently standing at the stop.
    #[serde(rename = "VehicleAtStop", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_at_stop: Option<bool>,
    /// Where the vehicle is standing, when that is more precise than the stop's own
    /// position.
    #[serde(rename = "VehicleLocationAtStop", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_location_at_stop: Option<Location>,
    /// Whether the call is a timing point, i.e. one the timetable is measured
    /// against rather than an intermediate stop.
    #[serde(rename = "TimingPoint", default, skip_serializing_if = "Option::is_none")]
    pub timing_point: Option<bool>,
    /// Whether the call lies in a stretch where passengers may board anywhere.
    #[serde(rename = "BoardingStretch", default, skip_serializing_if = "Option::is_none")]
    pub boarding_stretch: Option<bool>,
    /// Whether the vehicle only calls here when asked to.
    #[serde(rename = "RequestStop", default, skip_serializing_if = "Option::is_none")]
    pub request_stop: Option<bool>,
    /// The origin shown on the vehicle at this call, one entry per language.
    #[serde(rename = "OriginDisplay", default, skip_serializing_if = "Vec::is_empty")]
    pub origin_display: Vec<NaturalLanguageString>,
    /// The destination shown on the vehicle at this call, one entry per language.
    #[serde(rename = "DestinationDisplay", default, skip_serializing_if = "Vec::is_empty")]
    pub destination_display: Vec<NaturalLanguageString>,
    /// When the vehicle is timetabled to arrive.
    #[serde(rename = "AimedArrivalTime", default, skip_serializing_if = "Option::is_none")]
    pub aimed_arrival_time: Option<DateTime<FixedOffset>>,
    /// When the vehicle actually arrived.
    #[serde(rename = "ActualArrivalTime", default, skip_serializing_if = "Option::is_none")]
    pub actual_arrival_time: Option<DateTime<FixedOffset>>,
    /// When the vehicle is now expected to arrive.
    #[serde(rename = "ExpectedArrivalTime", default, skip_serializing_if = "Option::is_none")]
    pub expected_arrival_time: Option<DateTime<FixedOffset>>,
    /// How the arrival compares with the timetable, e.g. early, delayed or
    /// cancelled.
    #[serde(rename = "ArrivalStatus", default, skip_serializing_if = "Option::is_none")]
    pub arrival_status: Option<CallStatus>,
    /// The platform the vehicle arrives at, one entry per language.
    #[serde(rename = "ArrivalPlatformName", default, skip_serializing_if = "Vec::is_empty")]
    pub arrival_platform_name: Vec<NaturalLanguageString>,
    /// Whether passengers may alight here.
    #[serde(rename = "ArrivalBoardingActivity", default, skip_serializing_if = "Option::is_none")]
    pub arrival_boarding_activity: Option<ArrivalBoardingActivity>,
    /// When the vehicle is timetabled to depart.
    #[serde(rename = "AimedDepartureTime", default, skip_serializing_if = "Option::is_none")]
    pub aimed_departure_time: Option<DateTime<FixedOffset>>,
    /// When the vehicle actually departed.
    #[serde(rename = "ActualDepartureTime", default, skip_serializing_if = "Option::is_none")]
    pub actual_departure_time: Option<DateTime<FixedOffset>>,
    /// When the vehicle is now expected to depart.
    #[serde(rename = "ExpectedDepartureTime", default, skip_serializing_if = "Option::is_none")]
    pub expected_departure_time: Option<DateTime<FixedOffset>>,
    /// How the departure compares with the timetable.
    #[serde(rename = "DepartureStatus", default, skip_serializing_if = "Option::is_none")]
    pub departure_status: Option<CallStatus>,
    /// The platform the vehicle departs from, one entry per language.
    #[serde(rename = "DeparturePlatformName", default, skip_serializing_if = "Vec::is_empty")]
    pub departure_platform_name: Vec<NaturalLanguageString>,
    /// Whether passengers may board here.
    #[serde(rename = "DepartureBoardingActivity", default, skip_serializing_if = "Option::is_none")]
    pub departure_boarding_activity: Option<DepartureBoardingActivity>,
    /// The timetabled interval between vehicles, for services shown as a frequency
    /// rather than at fixed times.
    #[serde(rename = "AimedHeadwayInterval", default, skip_serializing_if = "Option::is_none")]
    pub aimed_headway_interval: Option<Duration>,
    /// The interval now expected between vehicles.
    #[serde(rename = "ExpectedHeadwayInterval", default, skip_serializing_if = "Option::is_none")]
    pub expected_headway_interval: Option<Duration>,
    /// Interchanges that depend on this call.
    #[serde(rename = "AffectedInterchanges", default, skip_serializing_if = "Option::is_none")]
    pub affected_interchanges: Option<AffectedInterchanges>,
}

/// A real-time time at a call, and whether it has happened yet.
///
/// The schema lets a producer give an actual time or an expected one, never both:
/// once the vehicle has been, the prediction is replaced by the observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservedTime {
    /// The event has happened, at this time.
    Actual(DateTime<FixedOffset>),
    /// The event has not happened yet and is predicted for this time.
    Expected(DateTime<FixedOffset>),
}

impl AffectedCall {
    /// A call at the given stop point.
    pub fn new(stop_point_ref: impl Into<StopPointRef>) -> Self {
        Self {
            stop_point_ref: Some(stop_point_ref.into()),
            ..Self::default()
        }
    }

    /// The real-time arrival, actual if the vehicle has been, expected otherwise.
    pub fn arrival(&self) -> Option<ObservedTime> {
        match (self.actual_arrival_time, self.expected_arrival_time) {
            (Some(actual), _) => Some(ObservedTime::Actual(actual)),
            (_, Some(expected)) => Some(ObservedTime::Expected(expected)),
            _ => None,
        }
    }

    /// The real-time departure, actual if the vehicle has gone, expected otherwise.
    pub fn departure(&self) -> Option<ObservedTime> {
        match (self.actual_departure_time, self.expected_departure_time) {
            (Some(actual), _) => Some(ObservedTime::Actual(actual)),
            (_, Some(expected)) => Some(ObservedTime::Expected(expected)),
            _ => None,
        }
    }
}

/// A physical vehicle affected by a situation.
///
/// This is the level at which a control room describes an incident that has
/// happened to a particular bus or train — a breakdown, an alarm, a vehicle stuck in
/// traffic — as opposed to a change to the service it was running.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AffectedVehicle {
    /// The vehicle.
    #[serde(rename = "VehicleRef")]
    pub vehicle_ref: VehicleRef,
    /// The vehicle's registration plates.
    #[serde(rename = "VehicleRegistrationNumberPlate", default, skip_serializing_if = "Vec::is_empty")]
    pub vehicle_registration_number_plate: Vec<String>,
    /// A telephone number the vehicle can be reached on.
    #[serde(rename = "PhoneNumber", default, skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    /// The vehicle's internet protocol address.
    #[serde(rename = "IPAddress", default, skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    /// The vehicle's radio address.
    #[serde(rename = "RadioAddress", default, skip_serializing_if = "Option::is_none")]
    pub radio_address: Option<String>,
    /// The journey the vehicle is running, framed by the operating day.
    #[serde(rename = "FramedVehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub framed_vehicle_journey_ref: Option<FramedVehicleJourneyRef>,
    /// Where the vehicle was when the situation arose.
    #[serde(rename = "Location", default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    /// Where the vehicle is now.
    #[serde(rename = "CurrentLocation", default, skip_serializing_if = "Option::is_none")]
    pub current_location: Option<Location>,
    /// How accessible the vehicle is while the situation lasts.
    #[serde(rename = "AccessibilityAssessment", default, skip_serializing_if = "Option::is_none")]
    pub accessibility_assessment: Option<AccessibilityAssessment>,
    /// The operator running the vehicle.
    #[serde(rename = "OperatorRef", default, skip_serializing_if = "Option::is_none")]
    pub operator_ref: Option<OperatorRef>,
    /// The commercial category of the service the vehicle runs.
    #[serde(rename = "ProductCategoryRef", default, skip_serializing_if = "Option::is_none")]
    pub product_category_ref: Option<ProductCategoryRef>,
    /// Properties of the service the vehicle runs, e.g. that it is an express.
    #[serde(rename = "ServiceFeatureRef", default, skip_serializing_if = "Vec::is_empty")]
    pub service_feature_ref: Vec<ServiceFeatureRef>,
    /// Properties of the vehicle itself, e.g. that it is low-floor.
    #[serde(rename = "VehicleFeatureRef", default, skip_serializing_if = "Vec::is_empty")]
    pub vehicle_feature_ref: Vec<VehicleFeatureRef>,
    /// The parts of a coupled train and the vehicles making them up.
    #[serde(rename = "TrainBlockPart", default, skip_serializing_if = "Vec::is_empty")]
    pub train_block_part: Vec<TrainBlockPart>,
    /// The block — the day's work for the vehicle — it is working.
    #[serde(rename = "BlockRef", default, skip_serializing_if = "Option::is_none")]
    pub block_ref: Option<BlockRef>,
    /// The sequence of journeys the vehicle runs within its block.
    #[serde(rename = "CourseOfJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub course_of_journey_ref: Option<CourseOfJourneyRef>,
    /// Whether the vehicle is held up in traffic. Absent means not known.
    #[serde(rename = "InCongestion", default, skip_serializing_if = "Option::is_none")]
    pub in_congestion: Option<bool>,
    /// Whether the vehicle's panic alarm has been raised.
    #[serde(rename = "InPanic", default, skip_serializing_if = "Option::is_none")]
    pub in_panic: Option<bool>,
    /// Whether the vehicle runs to a frequency rather than to fixed times.
    #[serde(rename = "HeadwayService", default, skip_serializing_if = "Option::is_none")]
    pub headway_service: Option<bool>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl AffectedVehicle {
    /// A vehicle affected by a situation.
    pub fn new(vehicle_ref: impl Into<VehicleRef>) -> Self {
        Self {
            vehicle_ref: vehicle_ref.into(),
            vehicle_registration_number_plate: Vec::new(),
            phone_number: None,
            ip_address: None,
            radio_address: None,
            framed_vehicle_journey_ref: None,
            location: None,
            current_location: None,
            accessibility_assessment: None,
            operator_ref: None,
            product_category_ref: None,
            service_feature_ref: Vec::new(),
            vehicle_feature_ref: Vec::new(),
            train_block_part: Vec::new(),
            block_ref: None,
            course_of_journey_ref: None,
            in_congestion: None,
            in_panic: None,
            headway_service: None,
            extensions: None,
        }
    }
}

/// A topographic place or site affected by a situation.
///
/// This covers everything that is not a stop: a town whose streets are closed, a
/// stadium, a car park, a pedestrian area.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AffectedPlace {
    /// The place.
    #[serde(rename = "PlaceRef", default, skip_serializing_if = "Option::is_none")]
    pub place_ref: Option<PlaceRef>,
    /// An alternative code for the place, private to the producer.
    #[serde(rename = "PrivateCode", default, skip_serializing_if = "Option::is_none")]
    pub private_code: Option<String>,
    /// The place's name, one entry per language.
    #[serde(rename = "PlaceName", default, skip_serializing_if = "Vec::is_empty")]
    pub place_name: Vec<NaturalLanguageString>,
    /// Where the place is.
    #[serde(rename = "Location", default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    /// What kind of place it is.
    #[serde(rename = "PlaceCategory", default, skip_serializing_if = "Option::is_none")]
    pub place_category: Option<String>,
    /// Equipment found at the place that the situation concerns.
    #[serde(rename = "EquipmentRef", default, skip_serializing_if = "Vec::is_empty")]
    pub equipment_ref: Vec<String>,
    /// How accessible the place is while the situation lasts.
    #[serde(rename = "AccessibilityAssessment", default, skip_serializing_if = "Option::is_none")]
    pub accessibility_assessment: Option<AccessibilityAssessment>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// A stop place — a station, interchange or bus station — affected by a situation.
///
/// The accessibility assessment comes first because the schema derives both a stop
/// place and its components from a shared base that carries it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AffectedStopPlace {
    /// How accessible the stop place is while the situation lasts.
    #[serde(rename = "AccessibilityAssessment", default, skip_serializing_if = "Option::is_none")]
    pub accessibility_assessment: Option<AccessibilityAssessment>,
    /// The stop place.
    #[serde(rename = "StopPlaceRef")]
    pub stop_place_ref: StopPlaceRef,
    /// The stop place's name, one entry per language.
    #[serde(rename = "PlaceName", default, skip_serializing_if = "Vec::is_empty")]
    pub place_name: Vec<NaturalLanguageString>,
    /// What kind of stop place it is, e.g. a railway station or a ferry port.
    #[serde(rename = "StopPlaceType", default, skip_serializing_if = "Option::is_none")]
    pub stop_place_type: Option<StopPlaceType>,
    /// Facilities in the stop place that are affected.
    #[serde(rename = "AffectedFacilities", default, skip_serializing_if = "Option::is_none")]
    pub affected_facilities: Option<AffectedFacilities>,
    /// Parts of the stop place that are affected, e.g. a quay or an entrance.
    #[serde(rename = "AffectedComponents", default, skip_serializing_if = "Option::is_none")]
    pub affected_components: Option<AffectedStopPlaceComponents>,
    /// Walking routes through the stop place that are affected.
    #[serde(rename = "AffectedNavigationPaths", default, skip_serializing_if = "Option::is_none")]
    pub affected_navigation_paths: Option<AffectedNavigationPaths>,
    /// Restricts the scope to these lines calling at the stop place.
    #[serde(rename = "Lines", default, skip_serializing_if = "Option::is_none")]
    pub lines: Option<AffectedLines>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl AffectedStopPlace {
    /// A stop place affected by a situation.
    pub fn new(stop_place_ref: impl Into<StopPlaceRef>) -> Self {
        Self {
            accessibility_assessment: None,
            stop_place_ref: stop_place_ref.into(),
            place_name: Vec::new(),
            stop_place_type: None,
            affected_facilities: None,
            affected_components: None,
            affected_navigation_paths: None,
            lines: None,
            extensions: None,
        }
    }
}

/// The parts of a stop place that a situation affects.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AffectedStopPlaceComponents {
    /// Each affected component.
    #[serde(rename = "AffectedComponent", default, skip_serializing_if = "Vec::is_empty")]
    pub affected_component: Vec<AffectedStopPlaceComponent>,
}

/// The walking routes through a stop place that a situation affects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AffectedNavigationPaths {
    /// Each affected walking route.
    #[serde(rename = "NavigationPathRef")]
    pub navigation_path_ref: Vec<NavigationPathRef>,
}

impl AffectedNavigationPaths {
    /// The given walking routes are affected.
    pub fn new(navigation_path_ref: Vec<NavigationPathRef>) -> Self {
        Self {
            navigation_path_ref,
        }
    }
}

/// A part of a stop place affected by a situation, e.g. a quay, an entrance or a
/// lift.
///
/// This is the level at which a station operator reports that one platform is closed
/// or one entrance is blocked, without implying anything about the rest of the
/// station.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AffectedStopPlaceComponent {
    /// How accessible the component is while the situation lasts.
    #[serde(rename = "AccessibilityAssessment", default, skip_serializing_if = "Option::is_none")]
    pub accessibility_assessment: Option<AccessibilityAssessment>,
    /// The component.
    #[serde(rename = "ComponentRef")]
    pub component_ref: StopPlaceComponentRef,
    /// The component's name, one entry per language.
    #[serde(rename = "ComponentName", default, skip_serializing_if = "Vec::is_empty")]
    pub component_name: Vec<NaturalLanguageString>,
    /// What kind of component it is, e.g. a quay, an entrance or an access space.
    #[serde(rename = "ComponentType", default, skip_serializing_if = "Option::is_none")]
    pub component_type: Option<StopPlaceComponentType>,
    /// The component's position, as a single point.
    #[serde(rename = "PointProjection", default, skip_serializing_if = "Option::is_none")]
    pub point_projection: Option<PointProjection>,
    /// The component's shape, as a line.
    #[serde(rename = "LinkProjection", default, skip_serializing_if = "Option::is_none")]
    pub link_projection: Option<LinkProjection>,
    /// The component's extent, as an area.
    #[serde(rename = "ZoneProjection", default, skip_serializing_if = "Option::is_none")]
    pub zone_projection: Option<ZoneProjection>,
    /// Which part of the projected shape the situation applies to.
    #[serde(rename = "Offset", default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<Offset>,
    /// What the passenger meets at the component, e.g. stairs, a lift or a ramp.
    #[serde(rename = "AccessFeatureType", default, skip_serializing_if = "Option::is_none")]
    pub access_feature_type: Option<AccessibilityFeature>,
    /// Facilities at the component that are affected.
    #[serde(rename = "AffectedFacilities", default, skip_serializing_if = "Option::is_none")]
    pub affected_facilities: Option<AffectedFacilities>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl AffectedStopPlaceComponent {
    /// A part of a stop place affected by a situation.
    pub fn new(component_ref: impl Into<StopPlaceComponentRef>) -> Self {
        Self {
            accessibility_assessment: None,
            component_ref: component_ref.into(),
            component_name: Vec::new(),
            component_type: None,
            point_projection: None,
            link_projection: None,
            zone_projection: None,
            offset: None,
            access_feature_type: None,
            affected_facilities: None,
            extensions: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Position of a substring, or a failure naming what was missing.
    fn at(xml: &str, needle: &str) -> usize {
        xml.find(needle)
            .unwrap_or_else(|| panic!("{needle} is missing from {xml}"))
    }

    #[test]
    fn an_affected_network_writes_its_lines_in_order_and_reads_them_back() {
        let network = AffectedNetwork::lines(vec![
            AffectedLine::new("ch:vbl:VBL006"),
            AffectedLine::new("ch:vbl:VBL008"),
        ]);

        let xml = quick_xml::se::to_string_with_root("AffectedNetwork", &network)
            .expect("the network is written");
        assert!(
            at(&xml, "ch:vbl:VBL006") < at(&xml, "ch:vbl:VBL008"),
            "the lines keep the order they were given: {xml}"
        );

        let read_back: AffectedNetwork =
            quick_xml::de::from_str(&xml).expect("the network parses back");
        assert_eq!(read_back, network);
    }

    #[test]
    fn an_affected_network_reports_which_alternative_it_carries() {
        assert_eq!(AffectedNetwork::all_lines().scope(), Some(NetworkScope::AllLines));
        assert_eq!(AffectedNetwork::default().scope(), None);

        let sections = AffectedNetwork::sections(vec![AffectedSection::default()]);
        assert!(matches!(
            sections.scope(),
            Some(NetworkScope::Sections(sections)) if sections.len() == 1
        ));

        let lines = AffectedNetwork::lines(vec![AffectedLine::new("6")]);
        assert!(matches!(
            lines.scope(),
            Some(NetworkScope::Lines(lines)) if lines[0].line_ref.as_str() == "6"
        ));
    }

    #[test]
    fn an_affected_stop_point_keeps_the_order_the_schema_prescribes() {
        let xml = "<AffectedStopPoint>\
                     <StopPointRef>ch:vbl:621::01</StopPointRef>\
                     <StopPointName xml:lang=\"DE\">Casino-Palace</StopPointName>\
                     <StopPlaceName xml:lang=\"DE\">Casino-Palace</StopPlaceName>\
                   </AffectedStopPoint>";

        let stop_point: AffectedStopPoint =
            quick_xml::de::from_str(xml).expect("the stop point parses");
        assert_eq!(
            stop_point.stop_point_ref.as_ref().map(StopPointRef::as_str),
            Some("ch:vbl:621::01")
        );
        assert_eq!(stop_point.stop_point_name[0].lang.as_deref(), Some("DE"));

        let written = quick_xml::se::to_string_with_root("AffectedStopPoint", &stop_point)
            .expect("the stop point is written");
        assert!(
            at(&written, "<StopPointRef>") < at(&written, "<StopPointName")
                && at(&written, "<StopPointName") < at(&written, "<StopPlaceName"),
            "the elements keep schema order: {written}"
        );
    }

    #[test]
    fn a_call_prefers_the_observed_time_over_the_predicted_one() {
        let actual = DateTime::parse_from_rfc3339("2024-03-01T08:16:30+01:00").unwrap();
        let expected = DateTime::parse_from_rfc3339("2024-03-01T08:15:00+01:00").unwrap();

        let mut call = AffectedCall::new("ch:vbl:621::01");
        call.expected_arrival_time = Some(expected);
        assert_eq!(call.arrival(), Some(ObservedTime::Expected(expected)));
        assert_eq!(call.departure(), None);

        call.actual_arrival_time = Some(actual);
        assert_eq!(call.arrival(), Some(ObservedTime::Actual(actual)));
    }

    #[test]
    fn an_indirect_section_reference_names_both_of_its_ends() {
        let section = IndirectSectionRef {
            first_stop_point_ref: Some(StopPointRef::new("BAAR0003")),
            intermediate: vec![IntermediateSectionPoint::IntermediateQuayRef(QuayRef::new(
                "BAR00021",
            ))],
            last_quay_ref: Some(QuayRef::new("BAR00099")),
            ..IndirectSectionRef::default()
        };

        assert_eq!(
            section.first(),
            Some(SectionBoundary::StopPoint(&StopPointRef::new("BAAR0003")))
        );
        assert_eq!(
            section.last(),
            Some(SectionBoundary::Quay(&QuayRef::new("BAR00099")))
        );
        assert_eq!(IndirectSectionRef::default().first(), None);

        let xml = quick_xml::se::to_string_with_root("IndirectSectionRef", &section)
            .expect("the section is written");
        assert!(
            at(&xml, "<FirstStopPointRef>") < at(&xml, "<IntermediateQuayRef>")
                && at(&xml, "<IntermediateQuayRef>") < at(&xml, "<LastQuayRef>"),
            "the ends bracket the intermediate points: {xml}"
        );
    }
}
