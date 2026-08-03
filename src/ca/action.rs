//! The control actions themselves, and the parts of a plan they name.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::{
    ChangeModel, ChangeOfJourneyTimingType, ControlActionReasonCategory, StopPlaceStatus,
    TypeOfActivatedJourney,
};
use crate::model::{
    AuthorityRef, ControlActionRef, DirectionRef, FramedVehicleJourneyRef, JourneyPatternRef,
    Location, OperatorRef, QuayRef, SituationRef, StopPlaceRef, StopPointRef, TypeOfValue,
    ValidityCondition, VehicleJourneyRef, VehicleRef, IFOPT_NAMESPACE,
};
use crate::sx::situation::{Images, InfoLinks};
use crate::types::{
    DefaultedText, Duration, Extensions, ItemIdentifier, ItemRef, NaturalLanguageString,
    ParticipantRef,
};

pub(super) fn ifopt_namespace() -> String {
    IFOPT_NAMESPACE.to_owned()
}

/// One decision a control room has taken about the plan.
///
/// Every action carries the same header — who took it, when, and why — and then
/// exactly one of the eleven things a control room can decide, which
/// [`ControlAction::kind`] reports.
///
/// The `ifopt` prefix is bound here rather than on each element that uses it: the
/// stop places and validity conditions an action may name are in that namespace,
/// and binding it once covers everything the action carries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlAction {
    /// Binding of the `ifopt` prefix the places and periods below may use.
    #[serde(rename = "@xmlns:ifopt", default = "ifopt_namespace")]
    pub ifopt_namespace: String,
    /// The producer's identifier for this record, so that it can be superseded.
    #[serde(rename = "ItemIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub item_identifier: Option<ItemIdentifier>,
    /// When the action was decided.
    #[serde(rename = "CreationTime")]
    pub creation_time: DateTime<FixedOffset>,
    /// The set of related actions this one belongs to.
    #[serde(rename = "GroupOfControlActionRef", default, skip_serializing_if = "Option::is_none")]
    pub group_of_control_action_ref: Option<ItemRef>,
    /// Who took the action, when it is not clear from context.
    #[serde(rename = "ParticipantRef", default, skip_serializing_if = "Option::is_none")]
    pub participant_ref: Option<ParticipantRef>,
    /// The participant's identifier for the action, excluding its version.
    #[serde(rename = "ControlActionCode")]
    pub control_action_code: ControlActionRef,
    /// Which update to the action this is, when it supersedes an earlier one.
    #[serde(rename = "Version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,
    /// When that version was fixed, after which it cannot change again.
    #[serde(rename = "VersionedAtTime", default, skip_serializing_if = "Option::is_none")]
    pub versioned_at_time: Option<DateTime<FixedOffset>>,
    /// Why the action was taken.
    #[serde(rename = "Reason", default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<ControlActionReason>,
    /// The official time the action counts as taken from, for penalty decisions.
    #[serde(rename = "RegistrationDateTime", default, skip_serializing_if = "Option::is_none")]
    pub registration_date_time: Option<DateTime<FixedOffset>>,
    /// Who reported what led to the action.
    #[serde(rename = "SourceNote", default, skip_serializing_if = "Option::is_none")]
    pub source_note: Option<NaturalLanguageString>,
    /// What the action is, in words, one text per language.
    #[serde(rename = "Description", default, skip_serializing_if = "Vec::is_empty")]
    pub description: Vec<NaturalLanguageString>,
    /// What to tell the driver beyond what the data says.
    #[serde(rename = "MessageToDriver", default, skip_serializing_if = "Option::is_none")]
    pub message_to_driver: Option<NaturalLanguageString>,
    /// A journey added to today's plan.
    #[serde(rename = "JourneyCreation", default, skip_serializing_if = "Option::is_none")]
    pub journey_creation: Option<JourneyCreation>,
    /// A journey taken out of it.
    #[serde(rename = "JourneyCancellation", default, skip_serializing_if = "Option::is_none")]
    pub journey_cancellation: Option<JourneyScope>,
    /// Part of a journey taken out of it.
    #[serde(rename = "PartialJourneyCancellation", default, skip_serializing_if = "Option::is_none")]
    pub partial_journey_cancellation: Option<CallCancellationAction>,
    /// A journey that had to be booked, now running.
    #[serde(rename = "FlexibleJourneyActivation", default, skip_serializing_if = "Option::is_none")]
    pub flexible_journey_activation: Option<FlexibleJourneyActivation>,
    /// A journey now calling somewhere else.
    #[serde(rename = "ChangeOfJourneyPattern", default, skip_serializing_if = "Option::is_none")]
    pub change_of_journey_pattern: Option<JourneyPatternModification>,
    /// A journey now running to different times.
    #[serde(rename = "ChangeOfJourneyTiming", default, skip_serializing_if = "Option::is_none")]
    pub change_of_journey_timing: Option<ChangeOfJourneyTiming>,
    /// A stop closed, or made harder to use, for every journey calling there.
    #[serde(rename = "ChangeOfStopPointStatus", default, skip_serializing_if = "Option::is_none")]
    pub change_of_stop_point_status: Option<ChangeOfStopPointStatus>,
    /// An interchange added between two journeys.
    #[serde(rename = "InterchangeCreation", default, skip_serializing_if = "Option::is_none")]
    pub interchange_creation: Option<ExtraConnection>,
    /// An interchange withdrawn, so that nothing waits for the feeder.
    #[serde(rename = "InterchangeCancellation", default, skip_serializing_if = "Option::is_none")]
    pub interchange_cancellation: Option<CancelledConnection>,
    /// An interchange now made by another journey, or for another length of time.
    #[serde(rename = "InterchangeModification", default, skip_serializing_if = "Option::is_none")]
    pub interchange_modification: Option<ModifiedConnection>,
    /// A vehicle put on a piece of work, or taken off one.
    #[serde(rename = "VehicleWorkAssignment", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_work_assignment: Option<VehicleWorkAssignment>,
    /// The situation this action belongs to.
    #[serde(rename = "SituationRef", default, skip_serializing_if = "Option::is_none")]
    pub situation_ref: Option<SituationRef>,
    /// What to tell passengers, when no situation says it already.
    #[serde(rename = "SituationDescription", default, skip_serializing_if = "Option::is_none")]
    pub situation_description: Option<SituationDescription>,
    /// Whether passenger information still has to be written for this action.
    #[serde(
        rename = "NeedsAssociatedPassengerInformation",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub needs_associated_passenger_information: Option<bool>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl ControlAction {
    /// An action with its header filled in and nothing decided yet.
    ///
    /// The schema requires exactly one of the eleven actions, so a message is only
    /// valid once one of them has been set.
    pub fn new(
        creation_time: DateTime<FixedOffset>,
        control_action_code: impl Into<ControlActionRef>,
    ) -> Self {
        Self {
            ifopt_namespace: ifopt_namespace(),
            item_identifier: None,
            creation_time,
            group_of_control_action_ref: None,
            participant_ref: None,
            control_action_code: control_action_code.into(),
            version: None,
            versioned_at_time: None,
            reason: None,
            registration_date_time: None,
            source_note: None,
            description: Vec::new(),
            message_to_driver: None,
            journey_creation: None,
            journey_cancellation: None,
            partial_journey_cancellation: None,
            flexible_journey_activation: None,
            change_of_journey_pattern: None,
            change_of_journey_timing: None,
            change_of_stop_point_status: None,
            interchange_creation: None,
            interchange_cancellation: None,
            interchange_modification: None,
            vehicle_work_assignment: None,
            situation_ref: None,
            situation_description: None,
            needs_associated_passenger_information: None,
            extensions: None,
        }
    }

    /// Which of the eleven actions this record carries, or `None` when it carries
    /// none — which the schema does not allow, but a document can still say.
    pub fn kind(&self) -> Option<ControlActionKind<'_>> {
        if let Some(action) = &self.journey_creation {
            return Some(ControlActionKind::JourneyCreation(action));
        }
        if let Some(action) = &self.journey_cancellation {
            return Some(ControlActionKind::JourneyCancellation(action));
        }
        if let Some(action) = &self.partial_journey_cancellation {
            return Some(ControlActionKind::PartialJourneyCancellation(action));
        }
        if let Some(action) = &self.flexible_journey_activation {
            return Some(ControlActionKind::FlexibleJourneyActivation(action));
        }
        if let Some(action) = &self.change_of_journey_pattern {
            return Some(ControlActionKind::ChangeOfJourneyPattern(action));
        }
        if let Some(action) = &self.change_of_journey_timing {
            return Some(ControlActionKind::ChangeOfJourneyTiming(action));
        }
        if let Some(action) = &self.change_of_stop_point_status {
            return Some(ControlActionKind::ChangeOfStopPointStatus(action));
        }
        if let Some(action) = &self.interchange_creation {
            return Some(ControlActionKind::InterchangeCreation(action));
        }
        if let Some(action) = &self.interchange_cancellation {
            return Some(ControlActionKind::InterchangeCancellation(action));
        }
        if let Some(action) = &self.interchange_modification {
            return Some(ControlActionKind::InterchangeModification(action));
        }
        if let Some(action) = &self.vehicle_work_assignment {
            return Some(ControlActionKind::VehicleWorkAssignment(action));
        }
        None
    }
}

/// Which alternative of the schema's choice of action a [`ControlAction`] carries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControlActionKind<'a> {
    /// A journey added to today's plan.
    JourneyCreation(&'a JourneyCreation),
    /// A journey taken out of it.
    JourneyCancellation(&'a JourneyScope),
    /// Part of a journey taken out of it.
    PartialJourneyCancellation(&'a CallCancellationAction),
    /// A journey that had to be booked, now running.
    FlexibleJourneyActivation(&'a FlexibleJourneyActivation),
    /// A journey now calling somewhere else.
    ChangeOfJourneyPattern(&'a JourneyPatternModification),
    /// A journey now running to different times.
    ChangeOfJourneyTiming(&'a ChangeOfJourneyTiming),
    /// A stop closed, or made harder to use.
    ChangeOfStopPointStatus(&'a ChangeOfStopPointStatus),
    /// An interchange added.
    InterchangeCreation(&'a ExtraConnection),
    /// An interchange withdrawn.
    InterchangeCancellation(&'a CancelledConnection),
    /// An interchange changed.
    InterchangeModification(&'a ModifiedConnection),
    /// A vehicle put on a piece of work, or taken off one.
    VehicleWorkAssignment(&'a VehicleWorkAssignment),
}

/// Why a control action was taken.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlActionReason {
    /// The reason from the standard's list.
    #[serde(rename = "StandardCategory")]
    pub standard_category: ControlActionReasonCategory,
    /// A finer reason from the producer's own vocabulary.
    #[serde(rename = "CustomCategory", default, skip_serializing_if = "Option::is_none")]
    pub custom_category: Option<TypeOfValue>,
}

impl ControlActionReason {
    /// A reason from the standard's list alone.
    pub fn new(standard_category: ControlActionReasonCategory) -> Self {
        Self {
            standard_category,
            custom_category: None,
        }
    }
}

/// Which journeys an action is about.
///
/// The schema offers two ways of saying it: a dated vehicle journey by name, or a
/// line, direction or authority together with the period the action holds over.
/// The fields of both alternatives are here; [`JourneyScope::for_journey`] and
/// [`JourneyScope::on_line`] build the two, and [`JourneyScope::names_one_journey`]
/// reports which was used.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct JourneyScope {
    /// The journey, by the timetable identifier alone.
    #[serde(rename = "VehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_journey_ref: Option<VehicleJourneyRef>,
    /// Every journey running in this direction of its line.
    #[serde(rename = "DirectionOfLineRef", default, skip_serializing_if = "Option::is_none")]
    pub direction_of_line_ref: Option<DirectionRef>,
    /// Every journey on this line.
    #[serde(rename = "LineRef", default, skip_serializing_if = "Option::is_none")]
    pub line_ref: Option<crate::model::LineRef>,
    /// Every journey belonging to this authority.
    #[serde(rename = "TransportAuthorityRef", default, skip_serializing_if = "Option::is_none")]
    pub transport_authority_ref: Option<AuthorityRef>,
    /// When the action holds, required alongside any of the four fields above.
    ///
    /// The children of this element are in the `ifopt` namespace, which the
    /// enclosing [`ControlAction`] binds.
    #[serde(rename = "TimeScope", default, skip_serializing_if = "Option::is_none")]
    pub time_scope: Option<ValidityCondition>,
    /// Whose journeys, when the scope above covers more than one operator's.
    #[serde(rename = "OperatorRef", default, skip_serializing_if = "Option::is_none")]
    pub operator_ref: Option<OperatorRef>,
    /// One journey on one operational day, named in full.
    #[serde(rename = "DatedVehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub dated_vehicle_journey_ref: Option<FramedVehicleJourneyRef>,
}

impl JourneyScope {
    /// One journey on one operational day.
    pub fn for_journey(dated_vehicle_journey_ref: FramedVehicleJourneyRef) -> Self {
        Self {
            dated_vehicle_journey_ref: Some(dated_vehicle_journey_ref),
            ..Self::default()
        }
    }

    /// Every journey on one line, over the given period.
    pub fn on_line(
        line_ref: impl Into<crate::model::LineRef>,
        time_scope: ValidityCondition,
    ) -> Self {
        Self {
            line_ref: Some(line_ref.into()),
            time_scope: Some(time_scope),
            ..Self::default()
        }
    }

    /// Whether the scope names a single dated journey rather than a set of them.
    pub fn names_one_journey(&self) -> bool {
        self.dated_vehicle_journey_ref.is_some()
    }
}

/// A stop in a journey pattern, and which visit to it is meant.
///
/// The stop is named as a stop point, a stop place or, where neither exists, a
/// position; [`PointInJourneyPatternRef::place`] reports which.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PointInJourneyPatternRef {
    /// The stop point.
    #[serde(rename = "StopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_point_ref: Option<StopPointRef>,
    /// The stop place, when the point is one.
    #[serde(rename = "ifopt:StopPlaceRef", alias = "StopPlaceRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_place_ref: Option<StopPlaceRef>,
    /// A position, when the point is neither of the above.
    #[serde(rename = "arbitraryPoint", default, skip_serializing_if = "Option::is_none")]
    pub arbitrary_point: Option<Location>,
    /// The quay within the stop.
    #[serde(rename = "QuayRef", default, skip_serializing_if = "Option::is_none")]
    pub quay_ref: Option<QuayRef>,
    /// Whether the vehicle's passing time there is a timing point.
    #[serde(rename = "TimingPoint", default, skip_serializing_if = "Option::is_none")]
    pub timing_point: Option<bool>,
    /// Which visit to the point this is, when the journey calls more than once.
    #[serde(rename = "VisitNumber", default, skip_serializing_if = "Option::is_none")]
    pub visit_number: Option<u64>,
    /// Where the point comes in the journey, counting from one.
    #[serde(rename = "Order", default, skip_serializing_if = "Option::is_none")]
    pub order: Option<u64>,
}

impl PointInJourneyPatternRef {
    /// The first visit to the given stop point.
    pub fn at_stop(stop_point_ref: impl Into<StopPointRef>) -> Self {
        Self {
            stop_point_ref: Some(stop_point_ref.into()),
            ..Self::default()
        }
    }

    /// How the point is named, or `None` when it is not named at all.
    pub fn place(&self) -> Option<Place<'_>> {
        if let Some(stop_point_ref) = &self.stop_point_ref {
            return Some(Place::StopPoint(stop_point_ref));
        }
        if let Some(stop_place_ref) = &self.stop_place_ref {
            return Some(Place::StopPlace(stop_place_ref));
        }
        self.arbitrary_point.as_ref().map(Place::Position)
    }
}

/// How a point in a journey pattern is named.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Place<'a> {
    /// A stop point.
    StopPoint(&'a StopPointRef),
    /// A stop place.
    StopPlace(&'a StopPlaceRef),
    /// A position, where no stop is defined.
    Position(&'a Location),
}

/// A stop, named the way a new point in a pattern is named.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TargetPoint {
    /// The stop point.
    #[serde(rename = "StopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_point_ref: Option<StopPointRef>,
    /// The stop place, when the point is one.
    #[serde(rename = "ifopt:StopPlaceRef", alias = "StopPlaceRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_place_ref: Option<StopPlaceRef>,
    /// A position, when the point is neither of the above.
    #[serde(rename = "arbitraryPoint", default, skip_serializing_if = "Option::is_none")]
    pub arbitrary_point: Option<Location>,
    /// The quay within the stop.
    #[serde(rename = "QuayRef", default, skip_serializing_if = "Option::is_none")]
    pub quay_ref: Option<QuayRef>,
    /// Whether the passing time there is a timing point.
    #[serde(rename = "TimingPoint", default, skip_serializing_if = "Option::is_none")]
    pub timing_point: Option<bool>,
}

impl TargetPoint {
    /// The given stop point.
    pub fn at_stop(stop_point_ref: impl Into<StopPointRef>) -> Self {
        Self {
            stop_point_ref: Some(stop_point_ref.into()),
            ..Self::default()
        }
    }
}

/// One call of one dated journey.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatedCallRef {
    /// The journey.
    #[serde(rename = "DatedVehicleJourneyRef")]
    pub dated_vehicle_journey_ref: FramedVehicleJourneyRef,
    /// The stop point it calls at.
    #[serde(rename = "StopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_point_ref: Option<StopPointRef>,
    /// The stop place, when the call is at one.
    #[serde(rename = "ifopt:StopPlaceRef", alias = "StopPlaceRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_place_ref: Option<StopPlaceRef>,
    /// A position, when the call is at neither of the above.
    #[serde(rename = "arbitraryPoint", default, skip_serializing_if = "Option::is_none")]
    pub arbitrary_point: Option<Location>,
    /// The quay within the stop.
    #[serde(rename = "QuayRef", default, skip_serializing_if = "Option::is_none")]
    pub quay_ref: Option<QuayRef>,
    /// Whether the passing time there is a timing point.
    #[serde(rename = "TimingPoint", default, skip_serializing_if = "Option::is_none")]
    pub timing_point: Option<bool>,
    /// Which visit to the stop this is, when the journey calls more than once.
    #[serde(rename = "VisitNumber", default, skip_serializing_if = "Option::is_none")]
    pub visit_number: Option<u64>,
    /// Where the call comes in the journey, counting from one.
    #[serde(rename = "Order", default, skip_serializing_if = "Option::is_none")]
    pub order: Option<u64>,
}

impl DatedCallRef {
    /// The first call of the given journey at the given stop point.
    pub fn at_stop(
        dated_vehicle_journey_ref: FramedVehicleJourneyRef,
        stop_point_ref: impl Into<StopPointRef>,
    ) -> Self {
        Self {
            dated_vehicle_journey_ref,
            stop_point_ref: Some(stop_point_ref.into()),
            stop_place_ref: None,
            arbitrary_point: None,
            quay_ref: None,
            timing_point: None,
            visit_number: None,
            order: None,
        }
    }
}

/// A journey added to today's plan.
///
/// The new journey either clones an existing one — same pattern, same run times,
/// a new departure time — or names the journey pattern it runs on and, optionally,
/// the calls in between. [`JourneyCreation::clone_of`] and
/// [`JourneyCreation::on_pattern`] build the two.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JourneyCreation {
    /// Where and when the new journey starts.
    #[serde(rename = "Start")]
    pub start: JourneyStart,
    /// Where and when it ends, when that is not where the model journey ends.
    #[serde(rename = "End", default, skip_serializing_if = "Option::is_none")]
    pub end: Option<JourneyEnd>,
    /// The journey used as the model for this one.
    #[serde(rename = "ClonedVehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub cloned_vehicle_journey_ref: Option<FramedVehicleJourneyRef>,
    /// The pattern the new journey runs on, when it does not clone a journey.
    #[serde(rename = "JourneyPatternRef", default, skip_serializing_if = "Option::is_none")]
    pub journey_pattern_ref: Option<JourneyPatternRef>,
    /// The calls between the start and the end.
    #[serde(rename = "MiddleCall", default, skip_serializing_if = "Vec::is_empty")]
    pub middle_call: Vec<MiddleCall>,
    /// The journey this one reinforces, when it runs to help another.
    #[serde(rename = "ReinforcedVehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub reinforced_vehicle_journey_ref: Option<FramedVehicleJourneyRef>,
    /// The identifier the new journey is to be known by.
    #[serde(rename = "NewJourneyCode")]
    pub new_journey_code: String,
    /// The operational day it runs on, as an `xsd:date`; today when absent.
    #[serde(rename = "OperatingDayDate", default, skip_serializing_if = "Option::is_none")]
    pub operating_day_date: Option<String>,
}

impl JourneyCreation {
    /// A journey copying an existing one, starting at the given time.
    pub fn clone_of(
        start: JourneyStart,
        cloned_vehicle_journey_ref: FramedVehicleJourneyRef,
        new_journey_code: impl Into<String>,
    ) -> Self {
        Self {
            start,
            end: None,
            cloned_vehicle_journey_ref: Some(cloned_vehicle_journey_ref),
            journey_pattern_ref: None,
            middle_call: Vec::new(),
            reinforced_vehicle_journey_ref: None,
            new_journey_code: new_journey_code.into(),
            operating_day_date: None,
        }
    }

    /// A journey running on the given pattern.
    pub fn on_pattern(
        start: JourneyStart,
        journey_pattern_ref: impl Into<JourneyPatternRef>,
        new_journey_code: impl Into<String>,
    ) -> Self {
        Self {
            start,
            end: None,
            cloned_vehicle_journey_ref: None,
            journey_pattern_ref: Some(journey_pattern_ref.into()),
            middle_call: Vec::new(),
            reinforced_vehicle_journey_ref: None,
            new_journey_code: new_journey_code.into(),
            operating_day_date: None,
        }
    }
}

/// Where and when a created journey starts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JourneyStart {
    /// The point it starts from.
    #[serde(rename = "PointInJourneyPattern")]
    pub point_in_journey_pattern: PointInJourneyPatternRef,
    /// When it is to leave there.
    #[serde(rename = "AimedDepartureDateTime")]
    pub aimed_departure_date_time: DateTime<FixedOffset>,
}

impl JourneyStart {
    /// A start from the given point at the given time.
    pub fn new(
        point_in_journey_pattern: PointInJourneyPatternRef,
        aimed_departure_date_time: DateTime<FixedOffset>,
    ) -> Self {
        Self {
            point_in_journey_pattern,
            aimed_departure_date_time,
        }
    }
}

/// Where and when a created journey ends.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JourneyEnd {
    /// The point it ends at.
    #[serde(rename = "PointInJourneyPattern")]
    pub point_in_journey_pattern: PointInJourneyPatternRef,
    /// The latest it is to arrive there.
    #[serde(rename = "AimedLatestDateTime")]
    pub aimed_latest_date_time: DateTime<FixedOffset>,
}

impl JourneyEnd {
    /// An end at the given point by the given time.
    pub fn new(
        point_in_journey_pattern: PointInJourneyPatternRef,
        aimed_latest_date_time: DateTime<FixedOffset>,
    ) -> Self {
        Self {
            point_in_journey_pattern,
            aimed_latest_date_time,
        }
    }
}

/// A call of a created journey between its start and its end.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MiddleCall {
    /// The point called at.
    #[serde(rename = "PointInJourneyPattern")]
    pub point_in_journey_pattern: PointInJourneyPatternRef,
    /// When the vehicle is to arrive.
    #[serde(rename = "AimedArrivalTime", default, skip_serializing_if = "Option::is_none")]
    pub aimed_arrival_time: Option<DateTime<FixedOffset>>,
    /// When it is to leave.
    #[serde(rename = "AimedDepartureTime", default, skip_serializing_if = "Option::is_none")]
    pub aimed_departure_time: Option<DateTime<FixedOffset>>,
}

impl MiddleCall {
    /// A call at the given point, with no times stated yet.
    pub fn at(point_in_journey_pattern: PointInJourneyPatternRef) -> Self {
        Self {
            point_in_journey_pattern,
            aimed_arrival_time: None,
            aimed_departure_time: None,
        }
    }
}

/// Part of a journey taken out of the plan.
///
/// The calls dropped are named in one of four ways — everything before a point,
/// everything after one, a list of points, or everything between two — which
/// [`CallCancellationAction::before`], [`CallCancellationAction::after`],
/// [`CallCancellationAction::at`] and [`CallCancellationAction::between`] build.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallCancellationAction {
    /// The journey, by the timetable identifier alone — the schema's journey scope,
    /// whose seven elements are written here as the schema writes them.
    #[serde(rename = "VehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_journey_ref: Option<VehicleJourneyRef>,
    /// Every journey running in this direction of its line.
    #[serde(rename = "DirectionOfLineRef", default, skip_serializing_if = "Option::is_none")]
    pub direction_of_line_ref: Option<DirectionRef>,
    /// Every journey on this line.
    #[serde(rename = "LineRef", default, skip_serializing_if = "Option::is_none")]
    pub line_ref: Option<crate::model::LineRef>,
    /// Every journey belonging to this authority.
    #[serde(rename = "TransportAuthorityRef", default, skip_serializing_if = "Option::is_none")]
    pub transport_authority_ref: Option<AuthorityRef>,
    /// When the action holds, required alongside any of the four fields above.
    #[serde(rename = "TimeScope", default, skip_serializing_if = "Option::is_none")]
    pub time_scope: Option<ValidityCondition>,
    /// Whose journeys, when the scope above covers more than one operator's.
    #[serde(rename = "OperatorRef", default, skip_serializing_if = "Option::is_none")]
    pub operator_ref: Option<OperatorRef>,
    /// One journey on one operational day, named in full.
    #[serde(rename = "DatedVehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub dated_vehicle_journey_ref: Option<FramedVehicleJourneyRef>,
    /// Every call before this point, not including it.
    #[serde(
        rename = "AllCallsBeforePointInJourneyPatterRef",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub all_calls_before_point: Option<PointInJourneyPatternRef>,
    /// Every call after this point, not including it.
    #[serde(
        rename = "AllCallsAfterPointInJourneyPatternRef",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub all_calls_after_point: Option<PointInJourneyPatternRef>,
    /// The calls listed, which need not be consecutive.
    #[serde(rename = "SelectedCalls", default, skip_serializing_if = "Option::is_none")]
    pub selected_calls: Option<SelectedCalls>,
    /// Every call between two points, not including either.
    #[serde(rename = "CallsBetweenPoints", default, skip_serializing_if = "Option::is_none")]
    pub calls_between_points: Option<CallsBetweenPoints>,
    /// Whether the arrivals at those calls are cancelled.
    #[serde(rename = "ConcernsArrivals", default, skip_serializing_if = "Option::is_none")]
    pub concerns_arrivals: Option<bool>,
    /// Whether the departures from them are.
    #[serde(rename = "ConcernsDepartures", default, skip_serializing_if = "Option::is_none")]
    pub concerns_departures: Option<bool>,
}

impl CallCancellationAction {
    /// Every call before the given point.
    pub fn before(journey_scope: JourneyScope, point: PointInJourneyPatternRef) -> Self {
        Self {
            all_calls_before_point: Some(point),
            ..Self::of(journey_scope)
        }
    }

    /// Every call after the given point.
    pub fn after(journey_scope: JourneyScope, point: PointInJourneyPatternRef) -> Self {
        Self {
            all_calls_after_point: Some(point),
            ..Self::of(journey_scope)
        }
    }

    /// The given calls, which need not be consecutive.
    pub fn at(journey_scope: JourneyScope, points: Vec<PointInJourneyPatternRef>) -> Self {
        Self {
            selected_calls: Some(SelectedCalls {
                point_in_journey_pattern_ref: points,
            }),
            ..Self::of(journey_scope)
        }
    }

    /// Every call between the two given points.
    pub fn between(
        journey_scope: JourneyScope,
        from: PointInJourneyPatternRef,
        to: PointInJourneyPatternRef,
    ) -> Self {
        Self {
            calls_between_points: Some(CallsBetweenPoints {
                from_point_in_journey_pattern_ref: from,
                to_point_in_journey_pattern_ref: to,
            }),
            ..Self::of(journey_scope)
        }
    }

    /// The action with its journey named and no calls chosen yet.
    fn of(journey_scope: JourneyScope) -> Self {
        Self {
            vehicle_journey_ref: journey_scope.vehicle_journey_ref,
            direction_of_line_ref: journey_scope.direction_of_line_ref,
            line_ref: journey_scope.line_ref,
            transport_authority_ref: journey_scope.transport_authority_ref,
            time_scope: journey_scope.time_scope,
            operator_ref: journey_scope.operator_ref,
            dated_vehicle_journey_ref: journey_scope.dated_vehicle_journey_ref,
            all_calls_before_point: None,
            all_calls_after_point: None,
            selected_calls: None,
            calls_between_points: None,
            concerns_arrivals: None,
            concerns_departures: None,
        }
    }
}

/// Calls named one by one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectedCalls {
    /// The points called at.
    #[serde(rename = "PointInJourneyPatternRef")]
    pub point_in_journey_pattern_ref: Vec<PointInJourneyPatternRef>,
}

/// Every call between two points, not including either.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallsBetweenPoints {
    /// The point the stretch starts after.
    #[serde(rename = "FromPointInJourneyPatternRef")]
    pub from_point_in_journey_pattern_ref: PointInJourneyPatternRef,
    /// The point it ends before.
    #[serde(rename = "ToPointInJourneyPatternRef")]
    pub to_point_in_journey_pattern_ref: PointInJourneyPatternRef,
}

/// A journey that had to be booked, now running.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlexibleJourneyActivation {
    /// The journey now running.
    #[serde(rename = "DatedVehicleJourneyRef")]
    pub dated_vehicle_journey_ref: FramedVehicleJourneyRef,
    /// What kind of flexible journey it is.
    #[serde(rename = "TypeOfActivatedJourney", default, skip_serializing_if = "Option::is_none")]
    pub type_of_activated_journey: Option<TypeOfActivatedJourney>,
}

impl FlexibleJourneyActivation {
    /// The activation of the given journey.
    pub fn new(dated_vehicle_journey_ref: FramedVehicleJourneyRef) -> Self {
        Self {
            dated_vehicle_journey_ref,
            type_of_activated_journey: None,
        }
    }
}

/// A journey now calling somewhere else.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JourneyPatternModification {
    /// The journey, by the timetable identifier alone — the schema's journey scope,
    /// whose seven elements are written here as the schema writes them.
    #[serde(rename = "VehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_journey_ref: Option<VehicleJourneyRef>,
    /// Every journey running in this direction of its line.
    #[serde(rename = "DirectionOfLineRef", default, skip_serializing_if = "Option::is_none")]
    pub direction_of_line_ref: Option<DirectionRef>,
    /// Every journey on this line.
    #[serde(rename = "LineRef", default, skip_serializing_if = "Option::is_none")]
    pub line_ref: Option<crate::model::LineRef>,
    /// Every journey belonging to this authority.
    #[serde(rename = "TransportAuthorityRef", default, skip_serializing_if = "Option::is_none")]
    pub transport_authority_ref: Option<AuthorityRef>,
    /// When the action holds, required alongside any of the four fields above.
    #[serde(rename = "TimeScope", default, skip_serializing_if = "Option::is_none")]
    pub time_scope: Option<ValidityCondition>,
    /// Whose journeys, when the scope above covers more than one operator's.
    #[serde(rename = "OperatorRef", default, skip_serializing_if = "Option::is_none")]
    pub operator_ref: Option<OperatorRef>,
    /// One journey on one operational day, named in full.
    #[serde(rename = "DatedVehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub dated_vehicle_journey_ref: Option<FramedVehicleJourneyRef>,
    /// The points that have changed.
    #[serde(rename = "ChangedPoints")]
    pub changed_points: ChangedPoints,
}

impl JourneyPatternModification {
    /// A change to the given points of the given journey.
    pub fn new(journey_scope: JourneyScope, changed_point: Vec<ChangedPoint>) -> Self {
        Self {
            vehicle_journey_ref: journey_scope.vehicle_journey_ref,
            direction_of_line_ref: journey_scope.direction_of_line_ref,
            line_ref: journey_scope.line_ref,
            transport_authority_ref: journey_scope.transport_authority_ref,
            time_scope: journey_scope.time_scope,
            operator_ref: journey_scope.operator_ref,
            dated_vehicle_journey_ref: journey_scope.dated_vehicle_journey_ref,
            changed_points: ChangedPoints { changed_point },
        }
    }
}

/// The points a [`JourneyPatternModification`] changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangedPoints {
    /// One entry per point.
    #[serde(rename = "ChangedPoint")]
    pub changed_point: Vec<ChangedPoint>,
}

/// One point of a pattern replaced by another.
///
/// At least one of the two target points is required; which of them says whether
/// the arrival, the departure or both have moved.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangedPoint {
    /// The point as the timetable has it.
    #[serde(rename = "TimetabledPointInJourneyPatternRef")]
    pub timetabled_point_in_journey_pattern_ref: PointInJourneyPatternRef,
    /// Where the vehicle now arrives instead.
    #[serde(rename = "TargetArrivalPoint", default, skip_serializing_if = "Option::is_none")]
    pub target_arrival_point: Option<TargetPoint>,
    /// Where it now leaves from instead.
    #[serde(rename = "TargetDeparturePoint", default, skip_serializing_if = "Option::is_none")]
    pub target_departure_point: Option<TargetPoint>,
}

impl ChangedPoint {
    /// The timetabled point, with nowhere else named yet.
    pub fn new(timetabled_point_in_journey_pattern_ref: PointInJourneyPatternRef) -> Self {
        Self {
            timetabled_point_in_journey_pattern_ref,
            target_arrival_point: None,
            target_departure_point: None,
        }
    }
}

/// A journey now running to different times.
///
/// The new times are given either as an offset applied from a point onwards, or as
/// the calls themselves.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeOfJourneyTiming {
    /// The journey affected.
    #[serde(rename = "DatedVehicleJourneyRef")]
    pub dated_vehicle_journey_ref: FramedVehicleJourneyRef,
    /// Why the times changed.
    #[serde(rename = "TypeOfChange")]
    pub type_of_change: ChangeOfJourneyTimingType,
    /// The offset to apply from a point onwards.
    #[serde(rename = "RelativeTime", default, skip_serializing_if = "Option::is_none")]
    pub relative_time: Option<RelativeTime>,
    /// The calls with their new times.
    #[serde(rename = "TimedCalls", default, skip_serializing_if = "Option::is_none")]
    pub timed_calls: Option<TimedCalls>,
}

impl ChangeOfJourneyTiming {
    /// A change stated as an offset.
    pub fn by(
        dated_vehicle_journey_ref: FramedVehicleJourneyRef,
        type_of_change: ChangeOfJourneyTimingType,
        relative_time: RelativeTime,
    ) -> Self {
        Self {
            dated_vehicle_journey_ref,
            type_of_change,
            relative_time: Some(relative_time),
            timed_calls: None,
        }
    }

    /// A change stated as the new calls.
    pub fn to_calls(
        dated_vehicle_journey_ref: FramedVehicleJourneyRef,
        type_of_change: ChangeOfJourneyTimingType,
        timed_call: Vec<MiddleCall>,
    ) -> Self {
        Self {
            dated_vehicle_journey_ref,
            type_of_change,
            relative_time: None,
            timed_calls: Some(TimedCalls { timed_call }),
        }
    }
}

/// How much later, or earlier, a journey now runs from a point onwards.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelativeTime {
    /// The point the offset applies from, including it.
    #[serde(
        rename = "FromPointInJourneyPatternRef",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub from_point_in_journey_pattern_ref: Option<PointInJourneyPatternRef>,
    /// The offset itself, which may be negative.
    #[serde(rename = "Offset")]
    pub offset: Duration,
    /// How the offset is carried over to the calls that follow.
    #[serde(rename = "ChangeModel", default, skip_serializing_if = "Option::is_none")]
    pub change_model: Option<ChangeModel>,
}

impl RelativeTime {
    /// The given offset, applied to the whole journey.
    pub fn of(offset: Duration) -> Self {
        Self {
            from_point_in_journey_pattern_ref: None,
            offset,
            change_model: None,
        }
    }
}

/// The calls of a journey with their new times.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimedCalls {
    /// One entry per call.
    #[serde(rename = "TimedCall")]
    pub timed_call: Vec<MiddleCall>,
}

/// A stop closed, or made harder to use, for every journey calling there.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeOfStopPointStatus {
    /// The stops affected.
    #[serde(rename = "StopRefs")]
    pub stop_refs: StopRefs,
    /// When the change holds.
    #[serde(rename = "TimeScope")]
    pub time_scope: StopPointStatusTimeScope,
    /// What the stops are now.
    #[serde(rename = "Status")]
    pub status: StopPlaceStatus,
}

impl ChangeOfStopPointStatus {
    /// The given stops, in the given state, over the given period.
    pub fn new(
        stop_refs: StopRefs,
        time_scope: StopPointStatusTimeScope,
        status: StopPlaceStatus,
    ) -> Self {
        Self {
            stop_refs,
            time_scope,
            status,
        }
    }
}

/// The stops a [`ChangeOfStopPointStatus`] is about.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StopRefs {
    /// Stop points.
    #[serde(rename = "StopPointRef", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_point_ref: Vec<StopPointRef>,
    /// Stop places.
    #[serde(rename = "ifopt:StopPlaceRef", alias = "StopPlaceRef", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_place_ref: Vec<StopPlaceRef>,
}

impl StopRefs {
    /// The given stop points.
    pub fn at_stops(stop_point_ref: Vec<StopPointRef>) -> Self {
        Self {
            stop_point_ref,
            stop_place_ref: Vec::new(),
        }
    }
}

/// When a change to a stop's status holds.
///
/// Beyond the period itself, a pair of times of day narrows the change to part of
/// each day within it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StopPointStatusTimeScope {
    /// When the change starts.
    #[serde(rename = "FromDateTime")]
    pub from_date_time: DateTime<FixedOffset>,
    /// When it ends.
    #[serde(rename = "UptoDateTime")]
    pub upto_date_time: DateTime<FixedOffset>,
    /// The time of day it starts at, as an `xsd:time`.
    #[serde(rename = "FromTimeOffset", default, skip_serializing_if = "Option::is_none")]
    pub from_time_offset: Option<String>,
    /// The time of day it ends at; an earlier one than the start means the next day.
    #[serde(rename = "UptoTimeOffset", default, skip_serializing_if = "Option::is_none")]
    pub upto_time_offset: Option<String>,
}

impl StopPointStatusTimeScope {
    /// The whole period between the two instants.
    pub fn between(
        from_date_time: DateTime<FixedOffset>,
        upto_date_time: DateTime<FixedOffset>,
    ) -> Self {
        Self {
            from_date_time,
            upto_date_time,
            from_time_offset: None,
            upto_time_offset: None,
        }
    }
}

/// An interchange added between two journeys.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtraConnection {
    /// The call passengers arrive on.
    #[serde(rename = "FeederCallRef")]
    pub feeder_call_ref: DatedCallRef,
    /// The call they leave on.
    #[serde(rename = "FetcherCallRef")]
    pub fetcher_call_ref: DatedCallRef,
    /// How long the departing service will wait for the feeder at most.
    #[serde(rename = "MaxWaitForFeederDuration")]
    pub max_wait_for_feeder_duration: Duration,
    /// How long passengers need to make the change.
    #[serde(rename = "MinChangeDuration", default, skip_serializing_if = "Option::is_none")]
    pub min_change_duration: Option<Duration>,
    /// Whether passengers can stay in the vehicle, both journeys being the same one.
    #[serde(rename = "StaySeated", default, skip_serializing_if = "Option::is_none")]
    pub stay_seated: Option<bool>,
    /// Whether staff are to be told about the interchange.
    #[serde(rename = "IsExposedToStaff", default, skip_serializing_if = "Option::is_none")]
    pub is_exposed_to_staff: Option<bool>,
    /// Whether passengers are.
    #[serde(rename = "IsExposedToPassengers", default, skip_serializing_if = "Option::is_none")]
    pub is_exposed_to_passengers: Option<bool>,
}

impl ExtraConnection {
    /// An interchange between the two calls, held for at most the given time.
    pub fn new(
        feeder_call_ref: DatedCallRef,
        fetcher_call_ref: DatedCallRef,
        max_wait_for_feeder_duration: Duration,
    ) -> Self {
        Self {
            feeder_call_ref,
            fetcher_call_ref,
            max_wait_for_feeder_duration,
            min_change_duration: None,
            stay_seated: None,
            is_exposed_to_staff: None,
            is_exposed_to_passengers: None,
        }
    }
}

/// An interchange withdrawn, so that nothing waits for the feeder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelledConnection {
    /// The call passengers were to arrive on.
    #[serde(rename = "FeederCallRef")]
    pub feeder_call_ref: DatedCallRef,
    /// The call they were to leave on.
    #[serde(rename = "FetcherCallRef")]
    pub fetcher_call_ref: DatedCallRef,
}

impl CancelledConnection {
    /// The withdrawal of the interchange between the two calls.
    pub fn new(feeder_call_ref: DatedCallRef, fetcher_call_ref: DatedCallRef) -> Self {
        Self {
            feeder_call_ref,
            fetcher_call_ref,
        }
    }
}

/// An interchange now made by another journey, or held for another length of time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModifiedConnection {
    /// The call passengers arrive on.
    #[serde(rename = "FeederCallRef")]
    pub feeder_call_ref: DatedCallRef,
    /// The call they were to leave on.
    #[serde(rename = "OriginalFetcherCallRef")]
    pub original_fetcher_call_ref: DatedCallRef,
    /// The call they are to leave on now, when the departure has changed.
    #[serde(rename = "NewFetcherCallRef", default, skip_serializing_if = "Option::is_none")]
    pub new_fetcher_call_ref: Option<DatedCallRef>,
    /// Until when the departing service now waits.
    #[serde(
        rename = "WaitForFeederUntilDateTime",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub wait_for_feeder_until_date_time: Option<DateTime<FixedOffset>>,
}

impl ModifiedConnection {
    /// A change to the interchange between the two calls.
    pub fn new(feeder_call_ref: DatedCallRef, original_fetcher_call_ref: DatedCallRef) -> Self {
        Self {
            feeder_call_ref,
            original_fetcher_call_ref,
            new_fetcher_call_ref: None,
            wait_for_feeder_until_date_time: None,
        }
    }
}

/// A vehicle put on a piece of work, or taken off one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleWorkAssignment {
    /// The vehicle.
    #[serde(rename = "VehicleRef")]
    pub vehicle_ref: VehicleRef,
    /// Whether the vehicle is being taken off the work rather than put on it.
    #[serde(rename = "DeAssignment", default, skip_serializing_if = "Option::is_none")]
    pub de_assignment: Option<bool>,
    /// The driver or crew logged in to it.
    #[serde(rename = "DriverRef", default, skip_serializing_if = "Option::is_none")]
    pub driver_ref: Option<String>,
    /// Their name.
    #[serde(rename = "DriverName", default, skip_serializing_if = "Option::is_none")]
    pub driver_name: Option<String>,
    /// The block of work assigned.
    #[serde(rename = "DatedBlockRef", default, skip_serializing_if = "Option::is_none")]
    pub dated_block_ref: Option<crate::model::BlockRef>,
    /// The single journey assigned, when the work is one journey.
    #[serde(rename = "DatedVehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub dated_vehicle_journey_ref: Option<FramedVehicleJourneyRef>,
}

impl VehicleWorkAssignment {
    /// The vehicle put on the given block of work.
    pub fn on_block(
        vehicle_ref: impl Into<VehicleRef>,
        dated_block_ref: impl Into<crate::model::BlockRef>,
    ) -> Self {
        Self {
            dated_block_ref: Some(dated_block_ref.into()),
            ..Self::of(vehicle_ref)
        }
    }

    /// The vehicle put on the given journey.
    pub fn on_journey(
        vehicle_ref: impl Into<VehicleRef>,
        dated_vehicle_journey_ref: FramedVehicleJourneyRef,
    ) -> Self {
        Self {
            dated_vehicle_journey_ref: Some(dated_vehicle_journey_ref),
            ..Self::of(vehicle_ref)
        }
    }

    /// The vehicle, with no work named yet.
    fn of(vehicle_ref: impl Into<VehicleRef>) -> Self {
        Self {
            vehicle_ref: vehicle_ref.into(),
            de_assignment: None,
            driver_ref: None,
            driver_name: None,
            dated_block_ref: None,
            dated_vehicle_journey_ref: None,
        }
    }
}

/// What to tell passengers about a control action, when no situation says it.
///
/// This is the schema's description group, the same wording a situation carries.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SituationDescription {
    /// The language the texts are in unless one says otherwise.
    #[serde(rename = "Language", default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// A headline, one per language offered.
    #[serde(rename = "Summary", default, skip_serializing_if = "Vec::is_empty")]
    pub summary: Vec<DefaultedText>,
    /// The body text, which should not repeat the summary.
    #[serde(rename = "Description", default, skip_serializing_if = "Vec::is_empty")]
    pub description: Vec<DefaultedText>,
    /// Additional detail beyond the description.
    #[serde(rename = "Detail", default, skip_serializing_if = "Vec::is_empty")]
    pub detail: Vec<DefaultedText>,
    /// What passengers are advised to do.
    #[serde(rename = "Advice", default, skip_serializing_if = "Vec::is_empty")]
    pub advice: Vec<DefaultedText>,
    /// Text for the operator's own staff rather than for passengers.
    #[serde(rename = "Internal", default, skip_serializing_if = "Option::is_none")]
    pub internal: Option<DefaultedText>,
    /// Pictures illustrating what has happened.
    #[serde(rename = "Images", default, skip_serializing_if = "Option::is_none")]
    pub images: Option<Images>,
    /// Links to further information.
    #[serde(rename = "InfoLinks", default, skip_serializing_if = "Option::is_none")]
    pub info_links: Option<InfoLinks>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timestamp() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2026-03-04T08:15:00+01:00").expect("valid timestamp")
    }

    fn journey() -> FramedVehicleJourneyRef {
        FramedVehicleJourneyRef {
            data_frame_ref: "2026-03-04".into(),
            dated_vehicle_journey_ref: "10-0815".into(),
        }
    }

    #[test]
    fn an_action_reports_which_of_the_eleven_decisions_it_carries() {
        let action = ControlAction {
            journey_cancellation: Some(JourneyScope::for_journey(journey())),
            ..ControlAction::new(timestamp(), "CA-4711")
        };
        assert!(matches!(
            action.kind(),
            Some(ControlActionKind::JourneyCancellation(_))
        ));

        let undecided = ControlAction::new(timestamp(), "CA-4712");
        assert_eq!(undecided.kind(), None, "nothing was decided yet");
    }

    #[test]
    fn an_action_writes_its_header_before_the_decision_and_reads_back() {
        let action = ControlAction {
            reason: Some(ControlActionReason::new(
                ControlActionReasonCategory::Weather,
            )),
            change_of_stop_point_status: Some(ChangeOfStopPointStatus::new(
                StopRefs::at_stops(vec!["de:03241:101".into()]),
                StopPointStatusTimeScope::between(timestamp(), timestamp()),
                StopPlaceStatus::Closed,
            )),
            needs_associated_passenger_information: Some(true),
            ..ControlAction::new(timestamp(), "CA-4711")
        };

        let xml = quick_xml::se::to_string_with_root("ControlAction", &action)
            .expect("the action serialises");
        let creation = xml.find("<CreationTime>").expect("the header is written");
        let code = xml
            .find("<ControlActionCode>")
            .expect("the code is written");
        let decision = xml
            .find("<ChangeOfStopPointStatus>")
            .expect("the decision is written");
        assert!(creation < code && code < decision, "{xml}");

        let read: ControlAction = quick_xml::de::from_str(&xml).expect("the action round-trips");
        assert_eq!(read, action);
    }

    #[test]
    fn a_journey_scope_names_either_one_journey_or_a_set_of_them() {
        assert!(JourneyScope::for_journey(journey()).names_one_journey());

        let by_line = JourneyScope::on_line("10", ValidityCondition::default());
        assert!(!by_line.names_one_journey());
        assert_eq!(by_line.line_ref.as_ref().map(|line| line.as_str()), Some("10"));
    }

    #[test]
    fn a_point_reports_how_the_place_it_names_is_named() {
        let stop = PointInJourneyPatternRef::at_stop("de:03241:101");
        assert!(matches!(stop.place(), Some(Place::StopPoint(_))));

        let elsewhere = PointInJourneyPatternRef {
            arbitrary_point: Some(Location::wgs84(9.7411, 52.3759)),
            ..PointInJourneyPatternRef::default()
        };
        assert!(matches!(elsewhere.place(), Some(Place::Position(_))));
        assert_eq!(PointInJourneyPatternRef::default().place(), None);
    }
}
