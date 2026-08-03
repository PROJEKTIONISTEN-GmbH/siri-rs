//! Passenger facilities: what a place or vehicle offers, and whether it works.
//!
//! A [`Facility`] is something passengers use — a lift, a ticket machine, a waiting
//! room, a cycle rack. A [`FacilityCondition`] says what state one is in, which is
//! what the Facility Monitoring service delivers and what any journey message may
//! embed to explain a disruption.
//!
//! The older SIRI 1.0 spelling of the same idea, [`FacilityChange`], is kept because
//! the schema still allows it inside a journey.

use serde::{Deserialize, Serialize};

use crate::enumerations::{
    AccessFacility, Accessibility, AccommodationFacility, AssistanceFacility, CountedFeatureUnit,
    CountingTrend, CountingType, DaysOfWeek, EquipmentStatus, FacilityCategory, FareClassFacility,
    HireFacility,
    HolidayType, LuggageFacility, MobilityFacility, MonitoringType, NuisanceFacility,
    ParkingFacility, PassengerCommsFacility, PassengerInformationFacility, RefreshmentFacility,
    RemedyType, ReservedSpaceFacility, RetailFacility, SanitaryFacility, TicketingFacility,
};
use crate::model::accessibility::{acsb_namespace, AccessibilityAssessment, Suitabilities};
use crate::model::formation::{
    EntranceToVehicleRef, TrainComponentRef, TrainElementRef, TrainInCompoundTrainRef, TrainRef,
};
use crate::model::journey::CompoundTrainRef;
use crate::model::location::Location;
use crate::model::reference::{
    ConnectionLinkRef, DatedVehicleJourneyRef, EquipmentRef, EquipmentTypeRef, FacilityRef,
    FeatureRef, InterchangeRef, LineRef, OperatorRef, OrganisationRef, ProductCategoryRef,
    ServiceFeatureRef, SituationRef, StopPlaceComponentRef, StopPlaceRef, StopPointRef,
    VehicleFeatureRef, VehicleRef,
};
use crate::types::{
    Duration, Extensions, HalfOpenTimeRange, HalfOpenTimestampInputRange,
    HalfOpenTimestampOutputRange, NaturalLanguageString,
};

/// Something passengers use, described in enough detail to be recognised.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Facility {
    /// The producer's code for this facility.
    #[serde(rename = "FacilityCode", default, skip_serializing_if = "Option::is_none")]
    pub facility_code: Option<String>,
    /// What the facility is, in words, one text per language.
    #[serde(rename = "Description", default, skip_serializing_if = "Vec::is_empty")]
    pub description: Vec<NaturalLanguageString>,
    /// What kind of thing it is: fixed equipment, a service, a reserved area.
    #[serde(
        rename = "FacilityClass",
        default,
        deserialize_with = "crate::xml::token_list::deserialize",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub facility_class: Vec<FacilityCategory>,
    /// What the facility offers.
    #[serde(rename = "Features", default, skip_serializing_if = "Option::is_none")]
    pub features: Option<FacilityFeatures>,
    /// The organisation answerable for the facility.
    #[serde(rename = "OwnerRef", default, skip_serializing_if = "Option::is_none")]
    pub owner_ref: Option<OrganisationRef>,
    /// That organisation's name.
    #[serde(rename = "OwnerName", default, skip_serializing_if = "Option::is_none")]
    pub owner_name: Option<NaturalLanguageString>,
    /// When the facility is there to be used.
    #[serde(rename = "ValidityCondition", default, skip_serializing_if = "Option::is_none")]
    pub validity_condition: Option<MonitoringValidityCondition>,
    /// Where it is, and which services it serves.
    #[serde(rename = "FacilityLocation", default, skip_serializing_if = "Option::is_none")]
    pub facility_location: Option<FacilityLocation>,
    /// What it does and does not allow a passenger with restricted mobility to do.
    #[serde(rename = "Limitations", default, skip_serializing_if = "Option::is_none")]
    pub limitations: Option<FacilityLimitations>,
    /// Judgements of the facility against individual passenger needs.
    #[serde(rename = "Suitabilities", default, skip_serializing_if = "Option::is_none")]
    pub suitabilities: Option<Suitabilities>,
    /// A fuller statement of what the facility offers such passengers.
    #[serde(rename = "AccessibilityAssessment", default, skip_serializing_if = "Option::is_none")]
    pub accessibility_assessment: Option<AccessibilityAssessment>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl Facility {
    /// A facility identified by the producer's own code.
    pub fn with_code(facility_code: impl Into<String>) -> Self {
        Self {
            facility_code: Some(facility_code.into()),
            ..Self::default()
        }
    }
}

/// What a facility offers, one entry per feature.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FacilityFeatures {
    /// The features.
    #[serde(rename = "Feature")]
    pub feature: Vec<FacilityFeature>,
}

/// One thing a facility offers, drawn from one of the sixteen TPEG feature lists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FacilityFeature {
    /// Which list the feature comes from, and which value within it.
    #[serde(rename = "$value")]
    pub feature: FacilityFeatureKind,
}

impl FacilityFeature {
    /// A feature drawn from one of the lists.
    pub fn new(feature: FacilityFeatureKind) -> Self {
        Self { feature }
    }
}

/// The sixteen families of facility feature, of which a [`FacilityFeature`] names one.
///
/// The variant is written as the element name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FacilityFeatureKind {
    /// How passengers get to or through the place: lifts, ramps, stairs.
    AccessFacility(AccessFacility),
    /// What the place or vehicle offers passengers to sit or sleep in.
    AccommodationFacility(AccommodationFacility),
    /// Help available to passengers, from staff or from a device.
    AssistanceFacility(AssistanceFacility),
    /// Which class of travel the facility belongs to.
    FareClassFacility(FareClassFacility),
    /// Vehicles and equipment passengers can hire.
    HireFacility(HireFacility),
    /// What passengers can do with their luggage.
    LuggageFacility(LuggageFacility),
    /// Equipment for passengers with restricted mobility.
    MobilityFacility(MobilityFacility),
    /// Restrictions on what other passengers may do, e.g. a quiet coach.
    NuisanceFacility(NuisanceFacility),
    /// Parking for cars and cycles.
    ParkingFacility(ParkingFacility),
    /// How passengers can communicate from the place or vehicle.
    PassengerCommsFacility(PassengerCommsFacility),
    /// How passengers are told what is happening.
    PassengerInformationFacility(PassengerInformationFacility),
    /// Food and drink.
    RefreshmentFacility(RefreshmentFacility),
    /// Space set aside for a particular use or group.
    ReservedSpaceFacility(ReservedSpaceFacility),
    /// Shops.
    RetailFacility(RetailFacility),
    /// Lavatories, showers and baby-changing.
    SanitaryFacility(SanitaryFacility),
    /// Where and how passengers can buy or validate a ticket.
    TicketingFacility(TicketingFacility),
}

/// What a facility does and does not allow a passenger with restricted mobility to do.
///
/// This is the accessibility annex's limitation group as SIRI embeds it directly in a
/// facility, without the surrounding `AccessibilityLimitation` the annex's own
/// assessment uses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FacilityLimitations {
    /// Binding of the `acsb` prefix used by this element's children.
    #[serde(rename = "@xmlns:acsb", default = "acsb_namespace")]
    pub acsb_namespace: String,
    /// Whether a wheelchair user can use the facility.
    #[serde(rename = "acsb:WheelchairAccess", alias = "WheelchairAccess")]
    pub wheelchair_access: Accessibility,
    /// Whether using it avoids steps.
    #[serde(rename = "acsb:StepFreeAccess", alias = "StepFreeAccess", default, skip_serializing_if = "Option::is_none")]
    pub step_free_access: Option<Accessibility>,
    /// Whether using it avoids escalators.
    #[serde(rename = "acsb:EscalatorFreeAccess", alias = "EscalatorFreeAccess", default, skip_serializing_if = "Option::is_none")]
    pub escalator_free_access: Option<Accessibility>,
    /// Whether using it avoids lifts.
    #[serde(rename = "acsb:LiftFreeAccess", alias = "LiftFreeAccess", default, skip_serializing_if = "Option::is_none")]
    pub lift_free_access: Option<Accessibility>,
    /// Whether it signals audibly.
    #[serde(rename = "acsb:AudibleSignalsAvailable", alias = "AudibleSignalsAvailable", default, skip_serializing_if = "Option::is_none")]
    pub audible_signals_available: Option<Accessibility>,
    /// Whether it signals visually.
    #[serde(rename = "acsb:VisualSignsAvailable", alias = "VisualSignsAvailable", default, skip_serializing_if = "Option::is_none")]
    pub visual_signs_available: Option<Accessibility>,
}

impl FacilityLimitations {
    /// Limitations stating only whether a wheelchair user can use the facility.
    pub fn new(wheelchair_access: Accessibility) -> Self {
        Self {
            acsb_namespace: acsb_namespace(),
            wheelchair_access,
            step_free_access: None,
            escalator_free_access: None,
            lift_free_access: None,
            audible_signals_available: None,
            visual_signs_available: None,
        }
    }
}

/// Where a facility is, and which services it serves.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FacilityLocation {
    /// The line the facility serves.
    #[serde(rename = "LineRef", default, skip_serializing_if = "Option::is_none")]
    pub line_ref: Option<LineRef>,
    /// The stop it is at.
    #[serde(rename = "StopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_point_ref: Option<StopPointRef>,
    /// The vehicle it is aboard.
    #[serde(rename = "VehicleRef", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_ref: Option<VehicleRef>,
    /// The compound train it is aboard.
    #[serde(rename = "CompoundTrainRef", default, skip_serializing_if = "Option::is_none")]
    pub compound_train_ref: Option<CompoundTrainRef>,
    /// The train it is aboard.
    #[serde(rename = "TrainRef", default, skip_serializing_if = "Option::is_none")]
    pub train_ref: Option<TrainRef>,
    /// The train within the compound train it is aboard, instead of `train_ref`.
    #[serde(rename = "TrainInCompoundTrainRef", default, skip_serializing_if = "Option::is_none")]
    pub train_in_compound_train_ref: Option<TrainInCompoundTrainRef>,
    /// The carriage it is in.
    #[serde(rename = "TrainElementRef", default, skip_serializing_if = "Option::is_none")]
    pub train_element_ref: Option<TrainElementRef>,
    /// The position within the train it is at, instead of `train_element_ref`.
    #[serde(rename = "TrainComponentRef", default, skip_serializing_if = "Option::is_none")]
    pub train_component_ref: Option<TrainComponentRef>,
    /// The vehicle entrance it is at.
    #[serde(rename = "EntranceToVehicleRef", default, skip_serializing_if = "Option::is_none")]
    pub entrance_to_vehicle_ref: Option<EntranceToVehicleRef>,
    /// The journey it serves.
    #[serde(rename = "DatedVehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub dated_vehicle_journey_ref: Option<DatedVehicleJourneyRef>,
    /// The connection link it is on.
    #[serde(rename = "ConnectionLinkRef", default, skip_serializing_if = "Option::is_none")]
    pub connection_link_ref: Option<ConnectionLinkRef>,
    /// The interchange it serves.
    #[serde(rename = "InterchangeRef", default, skip_serializing_if = "Option::is_none")]
    pub interchange_ref: Option<InterchangeRef>,
    /// The stop place it is in.
    #[serde(rename = "StopPlaceRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_place_ref: Option<StopPlaceRef>,
    /// The part of that stop place it is in.
    #[serde(rename = "StopPlaceComponentId", default, skip_serializing_if = "Option::is_none")]
    pub stop_place_component_id: Option<StopPlaceComponentRef>,
    /// The operator whose services it serves.
    #[serde(rename = "OperatorRef", default, skip_serializing_if = "Option::is_none")]
    pub operator_ref: Option<OperatorRef>,
    /// The commercial category of those services.
    #[serde(rename = "ProductCategoryRef", default, skip_serializing_if = "Option::is_none")]
    pub product_category_ref: Option<ProductCategoryRef>,
    /// Properties of those services.
    #[serde(rename = "ServiceFeatureRef", default, skip_serializing_if = "Vec::is_empty")]
    pub service_feature_ref: Vec<ServiceFeatureRef>,
    /// Properties of the vehicles running them.
    #[serde(rename = "VehicleFeatureRef", default, skip_serializing_if = "Vec::is_empty")]
    pub vehicle_feature_ref: Vec<VehicleFeatureRef>,
}

/// When a facility is available, or when a producer is watching it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitoringValidityCondition {
    /// The stretches of calendar time it holds over.
    #[serde(rename = "Period", default, skip_serializing_if = "Vec::is_empty")]
    pub period: Vec<HalfOpenTimestampOutputRange>,
    /// The times of day within those stretches.
    #[serde(rename = "Timeband", default, skip_serializing_if = "Vec::is_empty")]
    pub timeband: Vec<HalfOpenTimeRange>,
    /// The kinds of day it holds on.
    #[serde(
        rename = "DayType",
        default,
        deserialize_with = "crate::xml::token_list::deserialize",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub day_type: Vec<DaysOfWeek>,
    /// The kinds of holiday it holds on.
    #[serde(
        rename = "HolidayType",
        default,
        deserialize_with = "crate::xml::token_list::deserialize",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub holiday_type: Vec<HolidayType>,
}

/// What state a facility is in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FacilityStatus {
    /// Whether it is available, and how far.
    #[serde(rename = "Status")]
    pub status: crate::enumerations::FacilityStatus,
    /// What is the matter with it, one text per language.
    #[serde(rename = "Description", default, skip_serializing_if = "Vec::is_empty")]
    pub description: Vec<NaturalLanguageString>,
    /// What the facility now offers passengers with restricted mobility.
    #[serde(rename = "AccessibilityAssessment", default, skip_serializing_if = "Option::is_none")]
    pub accessibility_assessment: Option<AccessibilityAssessment>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl FacilityStatus {
    /// A bare statement of availability.
    pub fn new(status: crate::enumerations::FacilityStatus) -> Self {
        Self {
            status,
            description: Vec::new(),
            accessibility_assessment: None,
            extensions: None,
        }
    }
}

/// A count taken at a facility: free parking bays, charge remaining, people waiting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonitoredCounting {
    /// What the count means: how many are available, in use, out of order.
    #[serde(rename = "CountingType")]
    pub counting_type: CountingType,
    /// The unit counted in.
    #[serde(rename = "CountedFeatureUnit", default, skip_serializing_if = "Option::is_none")]
    pub counted_feature_unit: Option<CountedFeatureUnit>,
    /// What is counted, where the unit alone does not say.
    #[serde(rename = "TypeOfCountedFeature", default, skip_serializing_if = "Option::is_none")]
    pub type_of_counted_feature: Option<TypeOfValue>,
    /// The figure itself, either absolute or as a percentage.
    #[serde(rename = "$value")]
    pub amount: CountedAmount,
    /// Which way the figure is moving.
    #[serde(rename = "Trend", default, skip_serializing_if = "Option::is_none")]
    pub trend: Option<CountingTrend>,
    /// How exact the figure is, as a percentage.
    #[serde(rename = "Accuracy", default, skip_serializing_if = "Option::is_none")]
    pub accuracy: Option<f64>,
    /// What the count is about, in words, one text per language.
    #[serde(rename = "Description", default, skip_serializing_if = "Vec::is_empty")]
    pub description: Vec<NaturalLanguageString>,
    /// The individual things counted, where the producer can name them.
    #[serde(rename = "CountedItemsIdList", default, skip_serializing_if = "Option::is_none")]
    pub counted_items_id_list: Option<CountedItemsIdList>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl MonitoredCounting {
    /// A count of the given kind and size.
    pub fn new(counting_type: CountingType, amount: CountedAmount) -> Self {
        Self {
            counting_type,
            counted_feature_unit: None,
            type_of_counted_feature: None,
            amount,
            trend: None,
            accuracy: None,
            description: Vec::new(),
            counted_items_id_list: None,
            extensions: None,
        }
    }
}

/// How large a [`MonitoredCounting`] is, stated one of the two ways the schema allows.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CountedAmount {
    /// A number of whatever is being counted.
    Count(i64),
    /// A share of the whole, in per cent.
    Percentage(f64),
}

/// The identifiers of the things a count covers.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CountedItemsIdList {
    /// One identifier per thing counted.
    #[serde(rename = "ItemId")]
    pub item_id: Vec<String>,
}

/// An open-ended classification value, named by a code within a class.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeOfValue {
    /// The code identifying the value.
    #[serde(rename = "TypeOfValueCode")]
    pub type_of_value_code: String,
    /// The class of values the code belongs to.
    #[serde(rename = "NameOfClass")]
    pub name_of_class: String,
    /// The value's name.
    #[serde(rename = "Name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<NaturalLanguageString>,
    /// A shorter name for it.
    #[serde(rename = "ShortName", default, skip_serializing_if = "Option::is_none")]
    pub short_name: Option<NaturalLanguageString>,
    /// What it means, in words.
    #[serde(rename = "Description", default, skip_serializing_if = "Option::is_none")]
    pub description: Option<NaturalLanguageString>,
    /// An image standing for it.
    #[serde(rename = "Image", default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// A page describing it.
    #[serde(rename = "Url", default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// The sender's own code for it.
    #[serde(rename = "PrivateCode", default, skip_serializing_if = "Option::is_none")]
    pub private_code: Option<String>,
}

impl TypeOfValue {
    /// A value named by its code within a class.
    pub fn new(type_of_value_code: impl Into<String>, name_of_class: impl Into<String>) -> Self {
        Self {
            type_of_value_code: type_of_value_code.into(),
            name_of_class: name_of_class.into(),
            name: None,
            short_name: None,
            description: None,
            image: None,
            url: None,
            private_code: None,
        }
    }
}

/// What is being done about a facility that is not working.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Remedy {
    /// What kind of remedy: repair it, replace it, send passengers another way.
    #[serde(rename = "RemedyType", default, skip_serializing_if = "Option::is_none")]
    pub remedy_type: Option<RemedyType>,
    /// What is being done, in words, one text per language.
    #[serde(rename = "Description", default, skip_serializing_if = "Vec::is_empty")]
    pub description: Vec<NaturalLanguageString>,
    /// When the remedy is expected to be in place.
    #[serde(rename = "RemedyPeriod", default, skip_serializing_if = "Option::is_none")]
    pub remedy_period: Option<HalfOpenTimestampInputRange>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// How a producer watches a facility, so that a consumer knows what silence means.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitoringInformation {
    /// How often the facility is looked at.
    #[serde(rename = "MonitoringInterval", default, skip_serializing_if = "Option::is_none")]
    pub monitoring_interval: Option<Duration>,
    /// Whether it is looked at by a person or by a machine.
    #[serde(rename = "MonitoringType", default, skip_serializing_if = "Option::is_none")]
    pub monitoring_type: Option<MonitoringType>,
    /// When it is looked at.
    #[serde(rename = "MonitoringPeriod", default, skip_serializing_if = "Option::is_none")]
    pub monitoring_period: Option<MonitoringValidityCondition>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// The state of one facility, and what is being done about it.
///
/// The facility is either described in full or referred to by identifier; use
/// [`FacilityCondition::subject`] to see which.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FacilityCondition {
    /// The facility, described here.
    #[serde(rename = "Facility", default, skip_serializing_if = "Option::is_none")]
    pub facility: Option<Facility>,
    /// The facility, described elsewhere, instead of `facility`.
    #[serde(rename = "FacilityRef", default, skip_serializing_if = "Option::is_none")]
    pub facility_ref: Option<FacilityRef>,
    /// What state it is in.
    #[serde(rename = "FacilityStatus")]
    pub facility_status: FacilityStatus,
    /// Counts taken at the facility.
    #[serde(rename = "MonitoredCounting", default, skip_serializing_if = "Vec::is_empty")]
    pub monitored_counting: Vec<MonitoredCounting>,
    /// Where the facility has moved to, for one that moves.
    #[serde(rename = "FacilityUpdatedPosition", default, skip_serializing_if = "Option::is_none")]
    pub facility_updated_position: Option<Location>,
    /// The situation that explains the state.
    #[serde(rename = "SituationRef", default, skip_serializing_if = "Option::is_none")]
    pub situation_ref: Option<SituationRef>,
    /// What is being done about it.
    #[serde(rename = "Remedy", default, skip_serializing_if = "Option::is_none")]
    pub remedy: Option<Remedy>,
    /// How the producer is watching it.
    #[serde(rename = "MonitoringInfo", default, skip_serializing_if = "Option::is_none")]
    pub monitoring_info: Option<MonitoringInformation>,
    /// How long this statement holds.
    #[serde(rename = "ValidityPeriod", default, skip_serializing_if = "Option::is_none")]
    pub validity_period: Option<HalfOpenTimestampOutputRange>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// Which alternative of the schema's choice a [`FacilityCondition`] names its
/// facility with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FacilitySubject<'a> {
    /// The facility, described in the condition itself.
    Facility(&'a Facility),
    /// The facility, named by a reference the consumer already knows.
    Reference(&'a FacilityRef),
}

impl FacilityCondition {
    /// The state of a facility the consumer already knows by reference.
    pub fn for_reference(
        facility_ref: impl Into<FacilityRef>,
        facility_status: FacilityStatus,
    ) -> Self {
        Self::new(None, Some(facility_ref.into()), facility_status)
    }

    /// The state of a facility described here.
    pub fn for_facility(facility: Facility, facility_status: FacilityStatus) -> Self {
        Self::new(Some(facility), None, facility_status)
    }

    fn new(
        facility: Option<Facility>,
        facility_ref: Option<FacilityRef>,
        facility_status: FacilityStatus,
    ) -> Self {
        Self {
            facility,
            facility_ref,
            facility_status,
            monitored_counting: Vec::new(),
            facility_updated_position: None,
            situation_ref: None,
            remedy: None,
            monitoring_info: None,
            validity_period: None,
            extensions: None,
        }
    }

    /// Which alternative of the schema's choice this condition carries, or `None`
    /// when it carries neither.
    pub fn subject(&self) -> Option<FacilitySubject<'_>> {
        self.facility
            .as_ref()
            .map(FacilitySubject::Facility)
            .or_else(|| self.facility_ref.as_ref().map(FacilitySubject::Reference))
    }
}

/// A facility a producer will report on, paired with whether it is being watched.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnnotatedFacility {
    /// The facility.
    #[serde(rename = "FacilityRef")]
    pub facility_ref: FacilityRef,
    /// Whether the producer is watching it in real time.
    #[serde(rename = "Monitored", default, skip_serializing_if = "Option::is_none")]
    pub monitored: Option<bool>,
    /// What the facility is.
    #[serde(rename = "Facility", default, skip_serializing_if = "Option::is_none")]
    pub facility: Option<Facility>,
}

/// A change to a facility, in the SIRI 1.0 spelling the schema still allows.
///
/// [`FacilityCondition`] says the same thing in more detail and is what a new
/// implementation should send; this stays so that documents written the older way
/// keep their meaning.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FacilityChange {
    /// Which piece of equipment has changed, and how.
    #[serde(rename = "EquipmentAvailability", default, skip_serializing_if = "Option::is_none")]
    pub equipment_availability: Option<EquipmentAvailability>,
    /// The situation that explains the change.
    #[serde(rename = "SituationRef", default, skip_serializing_if = "Option::is_none")]
    pub situation_ref: Option<SituationRef>,
    /// What the change means for passengers with restricted mobility.
    #[serde(rename = "MobilityDisruption", default, skip_serializing_if = "Option::is_none")]
    pub mobility_disruption: Option<MobilityDisruption>,
}

/// Whether one piece of equipment can be used.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentAvailability {
    /// The equipment concerned.
    #[serde(rename = "EquipmentRef", default, skip_serializing_if = "Option::is_none")]
    pub equipment_ref: Option<EquipmentRef>,
    /// What it is, in words, one text per language.
    #[serde(rename = "Description", default, skip_serializing_if = "Vec::is_empty")]
    pub description: Vec<NaturalLanguageString>,
    /// What kind of equipment it is.
    #[serde(rename = "EquipmentTypeRef", default, skip_serializing_if = "Option::is_none")]
    pub equipment_type_ref: Option<EquipmentTypeRef>,
    /// How long this statement holds.
    #[serde(rename = "ValidityPeriod", default, skip_serializing_if = "Option::is_none")]
    pub validity_period: Option<HalfOpenTimestampOutputRange>,
    /// Whether the equipment can be used.
    #[serde(rename = "EquipmentStatus")]
    pub equipment_status: EquipmentStatus,
    /// What the equipment offers.
    #[serde(rename = "EquipmentFeatures", default, skip_serializing_if = "Option::is_none")]
    pub equipment_features: Option<EquipmentFeatures>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl EquipmentAvailability {
    /// A bare statement that a piece of equipment is in the given state.
    pub fn new(equipment_status: EquipmentStatus) -> Self {
        Self {
            equipment_ref: None,
            description: Vec::new(),
            equipment_type_ref: None,
            validity_period: None,
            equipment_status,
            equipment_features: None,
            extensions: None,
        }
    }
}

/// What a piece of equipment offers.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentFeatures {
    /// The features.
    #[serde(rename = "FeatureRef")]
    pub feature_ref: Vec<FeatureRef>,
}

/// What a change means for passengers with restricted mobility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MobilityDisruption {
    /// Whether such passengers can still get through.
    #[serde(rename = "MobilityImpairedAccess")]
    pub mobility_impaired_access: bool,
    /// The ways in and out that are affected.
    #[serde(
        rename = "AccessFacility",
        default,
        deserialize_with = "crate::xml::token_list::deserialize",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub access_facility: Vec<AccessFacility>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enumerations::Suitability as SuitabilityValue;
    use crate::enumerations::{FacilityStatus as FacilityStatusValue, Mobility};
    use crate::model::accessibility::{Suitability, UserNeed, UserNeedKind};

    #[test]
    fn a_condition_reports_whether_it_describes_its_facility_or_refers_to_one() {
        let referred = FacilityCondition::for_reference(
            "LIFT-4",
            FacilityStatus::new(FacilityStatusValue::NotAvailable),
        );
        assert!(matches!(referred.subject(), Some(FacilitySubject::Reference(r)) if r.as_str() == "LIFT-4"));

        let described = FacilityCondition::for_facility(
            Facility::with_code("134567-L4"),
            FacilityStatus::new(FacilityStatusValue::Available),
        );
        assert!(matches!(described.subject(), Some(FacilitySubject::Facility(_))));
        assert!(described.facility_ref.is_none());
    }

    #[test]
    fn a_facility_writes_its_annex_content_under_the_annex_namespace() {
        let facility = Facility {
            features: Some(FacilityFeatures {
                feature: vec![FacilityFeature::new(FacilityFeatureKind::AccessFacility(
                    AccessFacility::Lift,
                ))],
            }),
            limitations: Some(FacilityLimitations::new(Accessibility::True)),
            suitabilities: Some(Suitabilities::new(vec![Suitability::new(
                SuitabilityValue::Suitable,
                UserNeed::new(UserNeedKind::MobilityNeed(Mobility::Wheelchair)),
            )])),
            ..Facility::with_code("134567-L4")
        };

        let xml = quick_xml::se::to_string_with_root("Facility", &facility).expect("serialises");
        assert!(xml.contains("<Features><Feature><AccessFacility>lift</AccessFacility></Feature></Features>"), "{xml}");
        assert!(xml.contains("<acsb:WheelchairAccess>true</acsb:WheelchairAccess>"), "{xml}");
        assert!(xml.contains("<Suitabilities><Suitability"), "{xml}");

        let read: Facility = quick_xml::de::from_str(&xml).expect("round-trips");
        assert_eq!(read, facility);
    }

    #[test]
    fn a_count_is_written_as_either_a_number_or_a_percentage() {
        let absolute = MonitoredCounting::new(CountingType::AvailabilityCount, CountedAmount::Count(12));
        let xml = quick_xml::se::to_string_with_root("MonitoredCounting", &absolute).expect("serialises");
        assert!(xml.contains("<Count>12</Count>"), "{xml}");
        assert_eq!(
            quick_xml::de::from_str::<MonitoredCounting>(&xml).expect("round-trips"),
            absolute
        );

        let share = MonitoredCounting::new(CountingType::ChargingLevel, CountedAmount::Percentage(62.5));
        let xml = quick_xml::se::to_string_with_root("MonitoredCounting", &share).expect("serialises");
        assert!(xml.contains("<Percentage>62.5</Percentage>"), "{xml}");
        assert_eq!(
            quick_xml::de::from_str::<MonitoredCounting>(&xml).expect("round-trips"),
            share
        );
    }
}
