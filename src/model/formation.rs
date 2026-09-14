//! Train formation: how a train is put together, and what that means for a stop.
//!
//! A rail journey is not always run by one indivisible vehicle. SIRI 2.1 borrows
//! NeTEx's formation model to say what a train is made of — [`TrainElement`] for a
//! single carriage or engine, [`Train`] for the set of them that run coupled, and
//! [`CompoundTrain`] for several trains joined for part of a journey — and lets a
//! call point at those parts: which sector of a platform a carriage will stop at
//! ([`FormationAssignment`]), and whether a part is missing, added or out of order
//! ([`FormationCondition`]).
//!
//! The German VDV 454 profile is the reason the model exists: its `FoFahrzeuge`,
//! `FoFahrzeugPosition` and `FoFahrzeugGruppe` are [`TrainElement`],
//! [`TrainComponent`] and [`Train`] respectively.
//!
//! Occupancy and capacity ([`VehicleOccupancy`], [`PassengerCapacity`]) are here
//! too, because both are stated *per part of a train*: a train may be full in
//! second class and empty in first.

use serde::{Deserialize, Serialize};

use crate::enumerations::{
    FareClass, FormationChange, Occupancy, TrainElementType, TrainSize, TypeOfFuel,
    VehicleInFormationStatus,
};
use crate::model::accessibility::AccessibilityAssessment;
use crate::model::call::StopAssignment;
use crate::model::journey::{CompoundTrainRef, JourneyPlaceRef, ViaName};
use crate::model::reference::{DestinationRef, SituationRef};
use crate::types::{Extensions, NaturalLanguagePlaceName, NaturalLanguageString};
use crate::xml::token_list;

siri_ref! {
    /// Identifies one train element — a carriage, an engine, a van.
    TrainElementRef;
    /// Identifies the position one train element occupies within a train.
    TrainComponentRef;
    /// Identifies a train, i.e. a set of coupled train elements.
    TrainRef;
    /// Identifies one train within a compound train.
    TrainInCompoundTrainRef;
    /// Identifies a door or other entrance of a vehicle.
    EntranceToVehicleRef;
    /// Identifies a type of value in a producer's own vocabulary.
    TypeOfActionRef;
}

/// Which part of a train something is said about.
///
/// The schema states this as a group repeated inside several structures, so its
/// fields are inlined wherever it appears rather than nested; this type is the
/// documentation of what those fields mean together. The references narrow from
/// the outside in: a compound train, one train within it, one element or position
/// within that train, and finally one entrance of that element.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainFormationReference {
    /// The compound train concerned.
    #[serde(rename = "CompoundTrainRef", default, skip_serializing_if = "Option::is_none")]
    pub compound_train_ref: Option<CompoundTrainRef>,
    /// The train concerned.
    #[serde(rename = "TrainRef", default, skip_serializing_if = "Option::is_none")]
    pub train_ref: Option<TrainRef>,
    /// The train within the compound train concerned, instead of `train_ref`.
    #[serde(rename = "TrainInCompoundTrainRef", default, skip_serializing_if = "Option::is_none")]
    pub train_in_compound_train_ref: Option<TrainInCompoundTrainRef>,
    /// The train element concerned.
    #[serde(rename = "TrainElementRef", default, skip_serializing_if = "Option::is_none")]
    pub train_element_ref: Option<TrainElementRef>,
    /// The position within the train concerned, instead of `train_element_ref`.
    #[serde(rename = "TrainComponentRef", default, skip_serializing_if = "Option::is_none")]
    pub train_component_ref: Option<TrainComponentRef>,
    /// The entrance of the vehicle concerned.
    #[serde(rename = "EntranceToVehicleRef", default, skip_serializing_if = "Option::is_none")]
    pub entrance_to_vehicle_ref: Option<EntranceToVehicleRef>,
}

/// Where a part of a train will stand at a stop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormationAssignment {
    /// The compound train concerned.
    #[serde(rename = "CompoundTrainRef", default, skip_serializing_if = "Option::is_none")]
    pub compound_train_ref: Option<CompoundTrainRef>,
    /// The train concerned.
    #[serde(rename = "TrainRef", default, skip_serializing_if = "Option::is_none")]
    pub train_ref: Option<TrainRef>,
    /// The train within the compound train concerned.
    #[serde(rename = "TrainInCompoundTrainRef", default, skip_serializing_if = "Option::is_none")]
    pub train_in_compound_train_ref: Option<TrainInCompoundTrainRef>,
    /// The train element concerned.
    #[serde(rename = "TrainElementRef", default, skip_serializing_if = "Option::is_none")]
    pub train_element_ref: Option<TrainElementRef>,
    /// The position within the train concerned.
    #[serde(rename = "TrainComponentRef", default, skip_serializing_if = "Option::is_none")]
    pub train_component_ref: Option<TrainComponentRef>,
    /// The entrance of the vehicle concerned.
    #[serde(rename = "EntranceToVehicleRef", default, skip_serializing_if = "Option::is_none")]
    pub entrance_to_vehicle_ref: Option<EntranceToVehicleRef>,
    /// Whether that part is available, missing, defective and so on.
    #[serde(rename = "VehicleInFormationStatus", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_in_formation_status: Option<VehicleInFormationStatusRecord>,
    /// Where the part stands, at least one assignment.
    #[serde(rename = "TrainStopAssignment")]
    pub train_stop_assignment: Vec<StopAssignment>,
}

/// Something out of the ordinary about a train's formation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormationCondition {
    /// The compound train concerned.
    #[serde(rename = "CompoundTrainRef", default, skip_serializing_if = "Option::is_none")]
    pub compound_train_ref: Option<CompoundTrainRef>,
    /// The train concerned.
    #[serde(rename = "TrainRef", default, skip_serializing_if = "Option::is_none")]
    pub train_ref: Option<TrainRef>,
    /// The train within the compound train concerned.
    #[serde(rename = "TrainInCompoundTrainRef", default, skip_serializing_if = "Option::is_none")]
    pub train_in_compound_train_ref: Option<TrainInCompoundTrainRef>,
    /// The train element concerned.
    #[serde(rename = "TrainElementRef", default, skip_serializing_if = "Option::is_none")]
    pub train_element_ref: Option<TrainElementRef>,
    /// The position within the train concerned.
    #[serde(rename = "TrainComponentRef", default, skip_serializing_if = "Option::is_none")]
    pub train_component_ref: Option<TrainComponentRef>,
    /// The entrance of the vehicle concerned.
    #[serde(rename = "EntranceToVehicleRef", default, skip_serializing_if = "Option::is_none")]
    pub entrance_to_vehicle_ref: Option<EntranceToVehicleRef>,
    /// What changed about the formation as a whole.
    #[serde(rename = "FormationStatus", default, skip_serializing_if = "Option::is_none")]
    pub formation_status: Option<FormationStatus>,
    /// What changed about one vehicle in it, instead of `formation_status`.
    #[serde(rename = "VehicleInFormationStatus", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_in_formation_status: Option<VehicleInFormationStatusRecord>,
    /// The situation that explains the change.
    #[serde(rename = "SituationRef", default, skip_serializing_if = "Option::is_none")]
    pub situation_ref: Option<SituationRef>,
    /// What passengers should do about it.
    #[serde(rename = "RecommendedAction", default, skip_serializing_if = "Option::is_none")]
    pub recommended_action: Option<RecommendedAction>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// Which alternative of a [`FormationCondition`]'s choice is present.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormationConditionStatus<'a> {
    /// The formation as a whole changed.
    Formation(&'a FormationStatus),
    /// One vehicle within the formation changed.
    Vehicle(&'a VehicleInFormationStatusRecord),
}

impl FormationCondition {
    /// A condition describing a change to the formation as a whole.
    pub fn formation(status: FormationStatus) -> Self {
        Self {
            compound_train_ref: None,
            train_ref: None,
            train_in_compound_train_ref: None,
            train_element_ref: None,
            train_component_ref: None,
            entrance_to_vehicle_ref: None,
            formation_status: Some(status),
            vehicle_in_formation_status: None,
            situation_ref: None,
            recommended_action: None,
            extensions: None,
        }
    }

    /// A condition describing a change to one vehicle in the formation.
    pub fn vehicle(status: VehicleInFormationStatusRecord) -> Self {
        Self {
            formation_status: None,
            vehicle_in_formation_status: Some(status),
            ..Self::formation(FormationStatus::new(FormationChange::ChangedFormation))
        }
    }

    /// Which alternative of the schema's choice this condition carries, or `None`
    /// when neither is present.
    pub fn status(&self) -> Option<FormationConditionStatus<'_>> {
        self.formation_status
            .as_ref()
            .map(FormationConditionStatus::Formation)
            .or_else(|| {
                self.vehicle_in_formation_status
                    .as_ref()
                    .map(FormationConditionStatus::Vehicle)
            })
    }
}

/// A change to a train's formation as a whole.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormationStatus {
    /// What changed.
    #[serde(rename = "Status")]
    pub status: FormationChange,
    /// The change described for passengers, one text per language.
    #[serde(rename = "Description", default, skip_serializing_if = "Vec::is_empty")]
    pub description: Vec<NaturalLanguageString>,
    /// How the change affects passengers with accessibility needs.
    #[serde(rename = "AccessibilityAssessment", default, skip_serializing_if = "Option::is_none")]
    pub accessibility_assessment: Option<AccessibilityAssessment>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl FormationStatus {
    /// A change of the given kind, with nothing said about it yet.
    pub fn new(status: FormationChange) -> Self {
        Self {
            status,
            description: Vec::new(),
            accessibility_assessment: None,
            extensions: None,
        }
    }
}

/// A change to one vehicle within a train's formation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleInFormationStatusRecord {
    /// What changed.
    #[serde(rename = "Status")]
    pub status: VehicleInFormationStatus,
    /// The change described for passengers, one text per language.
    #[serde(rename = "Description", default, skip_serializing_if = "Vec::is_empty")]
    pub description: Vec<NaturalLanguageString>,
    /// How the change affects passengers with accessibility needs.
    #[serde(rename = "AccessibilityAssessment", default, skip_serializing_if = "Option::is_none")]
    pub accessibility_assessment: Option<AccessibilityAssessment>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl VehicleInFormationStatusRecord {
    /// A change of the given kind, with nothing said about it yet.
    pub fn new(status: VehicleInFormationStatus) -> Self {
        Self {
            status,
            description: Vec::new(),
            accessibility_assessment: None,
            extensions: None,
        }
    }
}

/// What passengers should do about a formation change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecommendedAction {
    /// The kind of action, in the producer's own vocabulary.
    #[serde(rename = "TypeOfActionRef")]
    pub type_of_action_ref: TypeOfActionRef,
    /// The action described for passengers, one text per language.
    #[serde(rename = "Description", default, skip_serializing_if = "Vec::is_empty")]
    pub description: Vec<NaturalLanguageString>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// How full a vehicle, or one part of one, is.
///
/// The leading references and the fare class say *what* the counts are about; a
/// record without them is about the whole vehicle.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VehicleOccupancy {
    /// The compound train concerned.
    #[serde(rename = "CompoundTrainRef", default, skip_serializing_if = "Option::is_none")]
    pub compound_train_ref: Option<CompoundTrainRef>,
    /// The train concerned.
    #[serde(rename = "TrainRef", default, skip_serializing_if = "Option::is_none")]
    pub train_ref: Option<TrainRef>,
    /// The train within the compound train concerned.
    #[serde(rename = "TrainInCompoundTrainRef", default, skip_serializing_if = "Option::is_none")]
    pub train_in_compound_train_ref: Option<TrainInCompoundTrainRef>,
    /// The train element concerned.
    #[serde(rename = "TrainElementRef", default, skip_serializing_if = "Option::is_none")]
    pub train_element_ref: Option<TrainElementRef>,
    /// The position within the train concerned.
    #[serde(rename = "TrainComponentRef", default, skip_serializing_if = "Option::is_none")]
    pub train_component_ref: Option<TrainComponentRef>,
    /// The entrance of the vehicle concerned.
    #[serde(rename = "EntranceToVehicleRef", default, skip_serializing_if = "Option::is_none")]
    pub entrance_to_vehicle_ref: Option<EntranceToVehicleRef>,
    /// The fare class the counts are about.
    #[serde(rename = "FareClass", default, skip_serializing_if = "Option::is_none")]
    pub fare_class: Option<FareClass>,
    /// The kind of passenger the counts are about, in the producer's own words.
    #[serde(rename = "PassengerCategory", default, skip_serializing_if = "Option::is_none")]
    pub passenger_category: Option<NaturalLanguageString>,
    /// How full it is, as a band.
    #[serde(rename = "OccupancyLevel", default, skip_serializing_if = "Option::is_none")]
    pub occupancy_level: Option<Occupancy>,
    /// How full it is, as a percentage of capacity.
    #[serde(rename = "OccupancyPercentage", default, skip_serializing_if = "Option::is_none")]
    pub occupancy_percentage: Option<f64>,
    /// How many passengers alighted.
    #[serde(rename = "AlightingCount", default, skip_serializing_if = "Option::is_none")]
    pub alighting_count: Option<u64>,
    /// How many passengers boarded.
    #[serde(rename = "BoardingCount", default, skip_serializing_if = "Option::is_none")]
    pub boarding_count: Option<u64>,
    /// How many passengers are on board.
    #[serde(rename = "OnboardCount", default, skip_serializing_if = "Option::is_none")]
    pub onboard_count: Option<u64>,
    /// How many places reserved for particular needs are taken.
    #[serde(rename = "SpecialPlacesOccupied", default, skip_serializing_if = "Option::is_none")]
    pub special_places_occupied: Option<u64>,
    /// How many pushchairs are on board.
    #[serde(rename = "PushchairsOnboardCount", default, skip_serializing_if = "Option::is_none")]
    pub pushchairs_onboard_count: Option<u64>,
    /// How many wheelchairs are on board.
    #[serde(rename = "WheelchairsOnboardCount", default, skip_serializing_if = "Option::is_none")]
    pub wheelchairs_onboard_count: Option<u64>,
    /// How many prams are on board.
    #[serde(rename = "PramsOnboardCount", default, skip_serializing_if = "Option::is_none")]
    pub prams_onboard_count: Option<u64>,
    /// How many bicycles are on board.
    #[serde(rename = "BicycleOnboardCount", default, skip_serializing_if = "Option::is_none")]
    pub bicycle_onboard_count: Option<u64>,
    /// How many seats are reserved in total.
    #[serde(rename = "TotalNumberOfReservedSeats", default, skip_serializing_if = "Option::is_none")]
    pub total_number_of_reserved_seats: Option<u64>,
    /// Group bookings held on this vehicle.
    #[serde(rename = "GroupReservation", default, skip_serializing_if = "Vec::is_empty")]
    pub group_reservation: Vec<GroupReservation>,
}

/// A block of seats reserved for a named group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupReservation {
    /// Who the group is.
    #[serde(rename = "NameOfGroup")]
    pub name_of_group: NaturalLanguageString,
    /// How many seats they hold.
    #[serde(rename = "NumberOfReservedSeats")]
    pub number_of_reserved_seats: u64,
}

/// How many passengers a vehicle, or one part of one, can take.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PassengerCapacity {
    /// The compound train concerned.
    #[serde(rename = "CompoundTrainRef", default, skip_serializing_if = "Option::is_none")]
    pub compound_train_ref: Option<CompoundTrainRef>,
    /// The train concerned.
    #[serde(rename = "TrainRef", default, skip_serializing_if = "Option::is_none")]
    pub train_ref: Option<TrainRef>,
    /// The train within the compound train concerned.
    #[serde(rename = "TrainInCompoundTrainRef", default, skip_serializing_if = "Option::is_none")]
    pub train_in_compound_train_ref: Option<TrainInCompoundTrainRef>,
    /// The train element concerned.
    #[serde(rename = "TrainElementRef", default, skip_serializing_if = "Option::is_none")]
    pub train_element_ref: Option<TrainElementRef>,
    /// The position within the train concerned.
    #[serde(rename = "TrainComponentRef", default, skip_serializing_if = "Option::is_none")]
    pub train_component_ref: Option<TrainComponentRef>,
    /// The entrance of the vehicle concerned.
    #[serde(rename = "EntranceToVehicleRef", default, skip_serializing_if = "Option::is_none")]
    pub entrance_to_vehicle_ref: Option<EntranceToVehicleRef>,
    /// The fare class the capacities are about.
    #[serde(rename = "FareClass", default, skip_serializing_if = "Option::is_none")]
    pub fare_class: Option<FareClass>,
    /// The kind of passenger the capacities are about.
    #[serde(rename = "PassengerCategory", default, skip_serializing_if = "Option::is_none")]
    pub passenger_category: Option<NaturalLanguageString>,
    /// Seated and standing places together.
    #[serde(rename = "TotalCapacity", default, skip_serializing_if = "Option::is_none")]
    pub total_capacity: Option<u64>,
    /// Seated places.
    #[serde(rename = "SeatingCapacity", default, skip_serializing_if = "Option::is_none")]
    pub seating_capacity: Option<u64>,
    /// Standing places.
    #[serde(rename = "StandingCapacity", default, skip_serializing_if = "Option::is_none")]
    pub standing_capacity: Option<u64>,
    /// Places reserved for particular needs.
    #[serde(rename = "SpecialPlaceCapacity", default, skip_serializing_if = "Option::is_none")]
    pub special_place_capacity: Option<u64>,
    /// Places for pushchairs.
    #[serde(rename = "PushchairCapacity", default, skip_serializing_if = "Option::is_none")]
    pub pushchair_capacity: Option<u64>,
    /// Places for wheelchairs.
    #[serde(rename = "WheelchairPlaceCapacity", default, skip_serializing_if = "Option::is_none")]
    pub wheelchair_place_capacity: Option<u64>,
    /// Places for prams.
    #[serde(rename = "PramPlaceCapacity", default, skip_serializing_if = "Option::is_none")]
    pub pram_place_capacity: Option<u64>,
    /// Places on the bicycle rack.
    #[serde(rename = "BicycleRackCapacity", default, skip_serializing_if = "Option::is_none")]
    pub bicycle_rack_capacity: Option<u64>,
}

/// One carriage, engine or van — NeTEx's TRAIN ELEMENT, VDV 454's `FoFahrzeuge`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrainElement {
    /// The producer's identifier for this element.
    #[serde(rename = "TrainElementCode")]
    pub train_element_code: TrainElementRef,
    /// What kind of element it is.
    #[serde(rename = "TrainElementType", default, skip_serializing_if = "Option::is_none")]
    pub train_element_type: Option<TrainElementType>,
    /// The number painted on the vehicle.
    #[serde(rename = "VehicleNumber", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_number: Option<String>,
    /// The fare classes this element carries, written as a space-separated list.
    #[serde(
        rename = "FareClasses",
        default,
        with = "token_list::space_separated",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub fare_classes: Vec<FareClass>,
    /// The vehicle type's name.
    #[serde(rename = "Name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<NaturalLanguageString>,
    /// A shorter name for it.
    #[serde(rename = "ShortName", default, skip_serializing_if = "Option::is_none")]
    pub short_name: Option<NaturalLanguageString>,
    /// The vehicle type described.
    #[serde(rename = "Description", default, skip_serializing_if = "Option::is_none")]
    pub description: Option<NaturalLanguageString>,
    /// The producer's own code for it.
    #[serde(rename = "PrivateCode", default, skip_serializing_if = "Option::is_none")]
    pub private_code: Option<String>,
    /// Whether it can be driven from either end.
    #[serde(rename = "ReversingDirection", default, skip_serializing_if = "Option::is_none")]
    pub reversing_direction: Option<bool>,
    /// Whether it can move under its own power.
    #[serde(rename = "SelfPropelled", default, skip_serializing_if = "Option::is_none")]
    pub self_propelled: Option<bool>,
    /// What it runs on.
    #[serde(rename = "TypeOfFuel", default, skip_serializing_if = "Option::is_none")]
    pub type_of_fuel: Option<TypeOfFuel>,
    /// Its emission class.
    #[serde(rename = "EuroClass", default, skip_serializing_if = "Option::is_none")]
    pub euro_class: Option<String>,
    /// How many passengers it can take.
    #[serde(rename = "MaximumPassengerCapacities", default, skip_serializing_if = "Option::is_none")]
    pub maximum_passenger_capacities: Option<MaximumPassengerCapacities>,
    /// Whether it has a low floor.
    #[serde(rename = "LowFloor", default, skip_serializing_if = "Option::is_none")]
    pub low_floor: Option<bool>,
    /// Whether it has a lift or a ramp.
    #[serde(rename = "HasLiftOrRamp", default, skip_serializing_if = "Option::is_none")]
    pub has_lift_or_ramp: Option<bool>,
    /// Whether it has a hoist.
    #[serde(rename = "HasHoist", default, skip_serializing_if = "Option::is_none")]
    pub has_hoist: Option<bool>,
    /// Its length in metres.
    #[serde(rename = "Length", default, skip_serializing_if = "Option::is_none")]
    pub length: Option<f64>,
    /// Its width in metres.
    #[serde(rename = "Width", default, skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    /// Its height in metres.
    #[serde(rename = "Height", default, skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
    /// Its weight in tonnes.
    #[serde(rename = "Weight", default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
}

impl TrainElement {
    /// An element with the given code and nothing else said about it.
    pub fn new(train_element_code: impl Into<TrainElementRef>) -> Self {
        Self {
            train_element_code: train_element_code.into(),
            train_element_type: None,
            vehicle_number: None,
            fare_classes: Vec::new(),
            name: None,
            short_name: None,
            description: None,
            private_code: None,
            reversing_direction: None,
            self_propelled: None,
            type_of_fuel: None,
            euro_class: None,
            maximum_passenger_capacities: None,
            low_floor: None,
            has_lift_or_ramp: None,
            has_hoist: None,
            length: None,
            width: None,
            height: None,
            weight: None,
        }
    }
}

/// One position within a train — NeTEx's TRAIN COMPONENT, VDV 454's
/// `FoFahrzeugPosition`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrainComponent {
    /// The producer's identifier for this position.
    #[serde(rename = "TrainComponentCode", default, skip_serializing_if = "Option::is_none")]
    pub train_component_code: Option<TrainComponentRef>,
    /// Where in the train this position is, counting from one.
    #[serde(rename = "Order")]
    pub order: u64,
    /// A short label for the position, as shown to passengers.
    #[serde(rename = "Label", default, skip_serializing_if = "Option::is_none")]
    pub label: Option<NaturalLanguageString>,
    /// The position described for passengers.
    #[serde(rename = "Description", default, skip_serializing_if = "Option::is_none")]
    pub description: Option<NaturalLanguageString>,
    /// The element standing in this position, by reference.
    #[serde(rename = "TrainElementRef", default, skip_serializing_if = "Option::is_none")]
    pub train_element_ref: Option<TrainElementRef>,
    /// The element standing in this position, in full.
    #[serde(rename = "TrainElement", default, skip_serializing_if = "Option::is_none")]
    pub train_element: Option<TrainElement>,
    /// Whether the element is coupled the other way round.
    #[serde(rename = "ReversedOrientation", default, skip_serializing_if = "Option::is_none")]
    pub reversed_orientation: Option<bool>,
}

/// A set of coupled train elements — NeTEx's TRAIN, VDV 454's `FoFahrzeugGruppe`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Train {
    /// The producer's identifier for this train.
    #[serde(rename = "TrainCode", default, skip_serializing_if = "Option::is_none")]
    pub train_code: Option<TrainRef>,
    /// The vehicle type's name.
    #[serde(rename = "Name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<NaturalLanguageString>,
    /// A shorter name for it.
    #[serde(rename = "ShortName", default, skip_serializing_if = "Option::is_none")]
    pub short_name: Option<NaturalLanguageString>,
    /// The vehicle type described.
    #[serde(rename = "Description", default, skip_serializing_if = "Option::is_none")]
    pub description: Option<NaturalLanguageString>,
    /// The producer's own code for it.
    #[serde(rename = "PrivateCode", default, skip_serializing_if = "Option::is_none")]
    pub private_code: Option<String>,
    /// Whether it can be driven from either end.
    #[serde(rename = "ReversingDirection", default, skip_serializing_if = "Option::is_none")]
    pub reversing_direction: Option<bool>,
    /// Whether it can move under its own power.
    #[serde(rename = "SelfPropelled", default, skip_serializing_if = "Option::is_none")]
    pub self_propelled: Option<bool>,
    /// What it runs on.
    #[serde(rename = "TypeOfFuel", default, skip_serializing_if = "Option::is_none")]
    pub type_of_fuel: Option<TypeOfFuel>,
    /// Its emission class.
    #[serde(rename = "EuroClass", default, skip_serializing_if = "Option::is_none")]
    pub euro_class: Option<String>,
    /// How many passengers it can take.
    #[serde(rename = "MaximumPassengerCapacities", default, skip_serializing_if = "Option::is_none")]
    pub maximum_passenger_capacities: Option<MaximumPassengerCapacities>,
    /// Whether it has a low floor.
    #[serde(rename = "LowFloor", default, skip_serializing_if = "Option::is_none")]
    pub low_floor: Option<bool>,
    /// Whether it has a lift or a ramp.
    #[serde(rename = "HasLiftOrRamp", default, skip_serializing_if = "Option::is_none")]
    pub has_lift_or_ramp: Option<bool>,
    /// Whether it has a hoist.
    #[serde(rename = "HasHoist", default, skip_serializing_if = "Option::is_none")]
    pub has_hoist: Option<bool>,
    /// Its length in metres.
    #[serde(rename = "Length", default, skip_serializing_if = "Option::is_none")]
    pub length: Option<f64>,
    /// Its width in metres.
    #[serde(rename = "Width", default, skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    /// Its height in metres.
    #[serde(rename = "Height", default, skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
    /// Its weight in tonnes.
    #[serde(rename = "Weight", default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    /// How many cars it has.
    #[serde(rename = "NumberOfCars", default, skip_serializing_if = "Option::is_none")]
    pub number_of_cars: Option<u64>,
    /// Whether it is running at normal, short or long length today.
    #[serde(rename = "TrainSizeType", default, skip_serializing_if = "Option::is_none")]
    pub train_size_type: Option<TrainSize>,
    /// The positions it is made up of.
    #[serde(rename = "TrainComponents", default, skip_serializing_if = "Option::is_none")]
    pub train_components: Option<TrainComponents>,
}

/// The positions a [`Train`] is made up of, each given by reference or in full.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TrainComponents {
    /// The positions, in the order the schema wrote them.
    #[serde(rename = "$value", default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<TrainComponentItem>,
}

/// One entry of a [`TrainComponents`] list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrainComponentItem {
    /// A position stated elsewhere.
    TrainComponentRef(TrainComponentRef),
    /// A position stated here.
    TrainComponent(Box<TrainComponent>),
}

/// Several trains joined for part of a journey — NeTEx's COMPOUND TRAIN, VDV 454's
/// `FoFahrzeugGruppenFahrtAbschnitt` when read together with the compound train
/// reference in a journey part.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CompoundTrain {
    /// The producer's identifier for this compound train.
    #[serde(rename = "CompoundTrainCode", default, skip_serializing_if = "Option::is_none")]
    pub compound_train_code: Option<CompoundTrainRef>,
    /// The vehicle type's name.
    #[serde(rename = "Name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<NaturalLanguageString>,
    /// A shorter name for it.
    #[serde(rename = "ShortName", default, skip_serializing_if = "Option::is_none")]
    pub short_name: Option<NaturalLanguageString>,
    /// The vehicle type described.
    #[serde(rename = "Description", default, skip_serializing_if = "Option::is_none")]
    pub description: Option<NaturalLanguageString>,
    /// The producer's own code for it.
    #[serde(rename = "PrivateCode", default, skip_serializing_if = "Option::is_none")]
    pub private_code: Option<String>,
    /// Whether it can be driven from either end.
    #[serde(rename = "ReversingDirection", default, skip_serializing_if = "Option::is_none")]
    pub reversing_direction: Option<bool>,
    /// Whether it can move under its own power.
    #[serde(rename = "SelfPropelled", default, skip_serializing_if = "Option::is_none")]
    pub self_propelled: Option<bool>,
    /// What it runs on.
    #[serde(rename = "TypeOfFuel", default, skip_serializing_if = "Option::is_none")]
    pub type_of_fuel: Option<TypeOfFuel>,
    /// Its emission class.
    #[serde(rename = "EuroClass", default, skip_serializing_if = "Option::is_none")]
    pub euro_class: Option<String>,
    /// How many passengers it can take.
    #[serde(rename = "MaximumPassengerCapacities", default, skip_serializing_if = "Option::is_none")]
    pub maximum_passenger_capacities: Option<MaximumPassengerCapacities>,
    /// Whether it has a low floor.
    #[serde(rename = "LowFloor", default, skip_serializing_if = "Option::is_none")]
    pub low_floor: Option<bool>,
    /// Whether it has a lift or a ramp.
    #[serde(rename = "HasLiftOrRamp", default, skip_serializing_if = "Option::is_none")]
    pub has_lift_or_ramp: Option<bool>,
    /// Whether it has a hoist.
    #[serde(rename = "HasHoist", default, skip_serializing_if = "Option::is_none")]
    pub has_hoist: Option<bool>,
    /// Its length in metres.
    #[serde(rename = "Length", default, skip_serializing_if = "Option::is_none")]
    pub length: Option<f64>,
    /// Its width in metres.
    #[serde(rename = "Width", default, skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    /// Its height in metres.
    #[serde(rename = "Height", default, skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
    /// Its weight in tonnes.
    #[serde(rename = "Weight", default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    /// The trains it is made up of.
    #[serde(rename = "TrainsInCompoundTrain", default, skip_serializing_if = "Option::is_none")]
    pub trains_in_compound_train: Option<TrainsInCompoundTrain>,
}

/// The trains a [`CompoundTrain`] is made up of, each by reference or in full.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TrainsInCompoundTrain {
    /// The trains, in the order the schema wrote them.
    #[serde(rename = "$value", default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<TrainInCompoundTrainItem>,
}

/// One entry of a [`TrainsInCompoundTrain`] list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrainInCompoundTrainItem {
    /// A train stated elsewhere.
    TrainInCompoundTrainRef(TrainInCompoundTrainRef),
    /// A train stated here.
    TrainInCompoundTrain(Box<TrainInCompoundTrain>),
}

/// One train's place within a compound train.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrainInCompoundTrain {
    /// The producer's identifier for this place.
    #[serde(rename = "TrainInCompoundTrainCode", default, skip_serializing_if = "Option::is_none")]
    pub train_in_compound_train_code: Option<TrainInCompoundTrainRef>,
    /// Where in the compound train it is, counting from one.
    #[serde(rename = "Order")]
    pub order: u64,
    /// A short label for the place, as shown to passengers.
    #[serde(rename = "Label", default, skip_serializing_if = "Option::is_none")]
    pub label: Option<NaturalLanguageString>,
    /// The place described for passengers.
    #[serde(rename = "Description", default, skip_serializing_if = "Option::is_none")]
    pub description: Option<NaturalLanguageString>,
    /// The train in this place, by reference.
    #[serde(rename = "TrainRef", default, skip_serializing_if = "Option::is_none")]
    pub train_ref: Option<TrainRef>,
    /// The train in this place, in full.
    #[serde(rename = "Train", default, skip_serializing_if = "Option::is_none")]
    pub train: Option<Train>,
    /// Where the train starts, when it differs from the whole journey.
    #[serde(rename = "OriginRef", default, skip_serializing_if = "Option::is_none")]
    pub origin_ref: Option<JourneyPlaceRef>,
    /// Names of that origin, one per language.
    #[serde(rename = "OriginName", default, skip_serializing_if = "Vec::is_empty")]
    pub origin_name: Vec<NaturalLanguagePlaceName>,
    /// Shorter names of that origin, one per language.
    #[serde(rename = "OriginShortName", default, skip_serializing_if = "Vec::is_empty")]
    pub origin_short_name: Vec<NaturalLanguagePlaceName>,
    /// What is shown at the origin as the destination, one per language.
    #[serde(rename = "DestinationDisplayAtOrigin", default, skip_serializing_if = "Vec::is_empty")]
    pub destination_display_at_origin: Vec<NaturalLanguagePlaceName>,
    /// Places passed through that tell this train apart from similar ones.
    #[serde(rename = "Via", default, skip_serializing_if = "Vec::is_empty")]
    pub via: Vec<ViaName>,
    /// Where the train ends, when it differs from the whole journey.
    #[serde(rename = "DestinationRef", default, skip_serializing_if = "Option::is_none")]
    pub destination_ref: Option<DestinationRef>,
    /// Names of that destination, one per language.
    #[serde(rename = "DestinationName", default, skip_serializing_if = "Vec::is_empty")]
    pub destination_name: Vec<NaturalLanguageString>,
    /// Shorter names of that destination, one per language.
    #[serde(rename = "DestinationShortName", default, skip_serializing_if = "Vec::is_empty")]
    pub destination_short_name: Vec<NaturalLanguagePlaceName>,
    /// What is shown at the destination as the origin, one per language.
    #[serde(rename = "OriginDisplayAtDestination", default, skip_serializing_if = "Vec::is_empty")]
    pub origin_display_at_destination: Vec<NaturalLanguagePlaceName>,
    /// Whether the train is coupled the other way round.
    #[serde(rename = "ReversedOrientation", default, skip_serializing_if = "Option::is_none")]
    pub reversed_orientation: Option<bool>,
    /// Whether passengers can walk between this train and its neighbours.
    #[serde(rename = "Passages", default, skip_serializing_if = "Option::is_none")]
    pub passages: Option<Passages>,
}

/// Whether passengers can walk between two coupled trains.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Passages {
    /// One entry per end of the train, so at most two.
    #[serde(rename = "PassageBetweenTrains")]
    pub passage_between_trains: Vec<PassageBetweenTrains>,
}

/// Whether passengers can walk into the train on one particular side.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PassageBetweenTrains {
    /// The train on the other side.
    #[serde(rename = "TrainRef")]
    pub train_ref: TrainRef,
    /// The position within that train that adjoins this one.
    #[serde(rename = "TrainComponentRef", default, skip_serializing_if = "Option::is_none")]
    pub train_component_ref: Option<TrainComponentRef>,
    /// Whether the passage is open.
    #[serde(rename = "PassageIsPossible")]
    pub passage_is_possible: bool,
}

/// The capacities a vehicle type offers, one entry per fare class or passenger kind.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MaximumPassengerCapacities {
    /// The capacities, at least one.
    #[serde(rename = "MaximumPassengerCapacity")]
    pub maximum_passenger_capacity: Vec<PassengerCapacity>,
}

/// The train elements a journey runs with, each given by reference or in full.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TrainElements {
    /// The elements, in the order the producer wrote them.
    #[serde(rename = "$value", default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<TrainElementItem>,
}

/// One entry of a [`TrainElements`] list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrainElementItem {
    /// An element stated elsewhere.
    TrainElementRef(TrainElementRef),
    /// An element stated here.
    TrainElement(Box<TrainElement>),
}

/// The trains a journey runs with, each given by reference or in full.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Trains {
    /// The trains, in the order the producer wrote them.
    #[serde(rename = "$value", default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<TrainItem>,
}

/// One entry of a [`Trains`] list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrainItem {
    /// A train stated elsewhere.
    TrainRef(TrainRef),
    /// A train stated here.
    Train(Box<Train>),
}

/// The compound trains a journey runs with, each by reference or in full.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CompoundTrains {
    /// The compound trains, in the order the producer wrote them.
    #[serde(rename = "$value", default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<CompoundTrainItem>,
}

/// One entry of a [`CompoundTrains`] list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompoundTrainItem {
    /// A compound train stated elsewhere.
    CompoundTrainRef(CompoundTrainRef),
    /// A compound train stated here.
    CompoundTrain(Box<CompoundTrain>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_train_element_writes_its_fare_classes_as_one_space_separated_list() {
        let element = TrainElement {
            train_element_type: Some(TrainElementType::Carriage),
            fare_classes: vec![FareClass::FirstClass, FareClass::SecondClass],
            low_floor: Some(true),
            ..TrainElement::new("TE1")
        };

        let xml = quick_xml::se::to_string_with_root("TrainElement", &element).unwrap();
        assert!(
            xml.contains("<FareClasses>firstClass secondClass</FareClasses>"),
            "{xml}"
        );
        assert!(xml.contains("<LowFloor>true</LowFloor>"), "{xml}");

        let read: TrainElement = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(read, element);
    }

    #[test]
    fn a_fare_class_the_schema_does_not_define_is_kept_as_it_was_written() {
        let document = "<TrainElement><TrainElementCode>TE1</TrainElementCode>\
                        <FareClasses>firstClass sleeperClass</FareClasses></TrainElement>";
        let element: TrainElement = quick_xml::de::from_str(document).unwrap();
        assert_eq!(
            element.fare_classes,
            vec![
                FareClass::FirstClass,
                FareClass::Unrecognised("sleeperClass".to_owned())
            ]
        );
        assert_eq!(
            quick_xml::se::to_string_with_root("TrainElement", &element).unwrap(),
            document
        );
    }

    #[test]
    fn a_formation_condition_reports_which_alternative_it_carries() {
        let formation = FormationCondition::formation(FormationStatus::new(
            FormationChange::MissingRestaurantCoach,
        ));
        assert!(matches!(
            formation.status(),
            Some(FormationConditionStatus::Formation(status)) if status.status == FormationChange::MissingRestaurantCoach
        ));

        let vehicle = FormationCondition::vehicle(VehicleInFormationStatusRecord::new(
            VehicleInFormationStatus::Defective,
        ));
        assert!(matches!(
            vehicle.status(),
            Some(FormationConditionStatus::Vehicle(_))
        ));
        assert_eq!(vehicle.formation_status, None);
    }

    #[test]
    fn a_train_keeps_the_order_of_the_positions_it_is_made_of() {
        let train = Train {
            train_code: Some(TrainRef::new("T1")),
            number_of_cars: Some(2),
            train_size_type: Some(TrainSize::Short),
            train_components: Some(TrainComponents {
                items: vec![
                    TrainComponentItem::TrainComponentRef(TrainComponentRef::new("TC1")),
                    TrainComponentItem::TrainComponent(Box::new(TrainComponent {
                        train_component_code: Some(TrainComponentRef::new("TC2")),
                        order: 2,
                        label: None,
                        description: None,
                        train_element_ref: Some(TrainElementRef::new("TE2")),
                        train_element: None,
                        reversed_orientation: Some(true),
                    })),
                ],
            }),
            ..Train::default()
        };

        let xml = quick_xml::se::to_string_with_root("Train", &train).unwrap();
        let read: Train = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(read, train);
    }
}
