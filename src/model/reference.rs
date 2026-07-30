//! References to the entities of a transport network.

use serde::{Deserialize, Serialize};

use crate::types::{CountryRef, ParticipantRef};

siri_ref! {
    /// Identifies a block, i.e. a day's work for one vehicle.
    BlockRef;
    /// Identifies the sequence of journeys a single vehicle runs within a block.
    CourseOfJourneyRef;
    /// Identifies an edition of a timetable.
    VersionRef;
    /// Identifies a situation within the participant that raised it.
    ///
    /// The number stays the same across every update to the situation; the update's
    /// `Version` distinguishes the revisions.
    SituationNumber;
    /// Identifies an operator, i.e. a company running services.
    OperatorRef;
    /// Identifies a part of an operator's organisation.
    OperationalUnitRef;
    /// Identifies a line — a named group of routes presented to the public as one.
    LineRef;
    /// Identifies a direction of travel along a line.
    DirectionRef;
    /// Identifies a scheduled stopping place.
    StopPointRef;
    /// Identifies a group of stop points treated as one place.
    StopAreaRef;
    /// Identifies a place where passengers board and alight, in IFOPT terms.
    StopPlaceRef;
    /// Identifies a component of a stop place, e.g. a quay or an entrance.
    StopPlaceComponentRef;
    /// Identifies a physical link between two stop points used for interchange.
    ConnectionLinkRef;
    /// Identifies a passenger facility, e.g. a lift or a ticket machine.
    FacilityRef;
    /// Identifies a physical vehicle.
    VehicleRef;
    /// Identifies a planned journey of a vehicle along a route.
    VehicleJourneyRef;
    /// Identifies a planned interchange between two journeys.
    InterchangeRef;
    /// Identifies a control action taken to manage operations.
    ControlActionRef;
    /// Identifies the operational day a dated journey belongs to.
    DataFrameRef;
    /// Identifies a journey on a particular operational day.
    DatedVehicleJourneyRef;
    /// Identifies the destination shown to passengers.
    DestinationRef;
    /// Identifies a journey pattern, i.e. an ordered list of stop points.
    JourneyPatternRef;
    /// Identifies a route.
    RouteRef;
    /// Identifies a place in a topographic gazetteer.
    PlaceRef;
    /// Identifies a service feature, e.g. `lowFloor`.
    ServiceFeatureRef;
    /// Identifies a product category, e.g. `express`.
    ProductCategoryRef;
    /// Identifies a vehicle feature.
    VehicleFeatureRef;
    /// Identifies a quay — the boarding position within a stop place.
    QuayRef;
}

/// A reference to a situation, either by its number alone or in full.
///
/// The short form works within one participant's own data; the full form names the
/// participant, and optionally the revision, so that a situation raised elsewhere
/// can be referred to without ambiguity.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SituationRef {
    /// The situation, by number alone.
    #[serde(rename = "SituationSimpleRef", default, skip_serializing_if = "Option::is_none")]
    pub situation_simple_ref: Option<SituationNumber>,
    /// The situation, named in full.
    #[serde(rename = "SituationFullRef", default, skip_serializing_if = "Option::is_none")]
    pub situation_full_ref: Option<SituationFullRef>,
}

impl SituationRef {
    /// A reference by situation number alone.
    pub fn simple(situation_number: impl Into<SituationNumber>) -> Self {
        Self {
            situation_simple_ref: Some(situation_number.into()),
            situation_full_ref: None,
        }
    }
}

/// A situation named by the participant that raised it, and optionally by revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SituationFullRef {
    /// The country the raising participant belongs to.
    #[serde(rename = "VersionCountryRef", default, skip_serializing_if = "Option::is_none")]
    pub version_country_ref: Option<CountryRef>,
    /// The participant that raised the situation.
    #[serde(rename = "ParticipantRef")]
    pub participant_ref: ParticipantRef,
    /// The situation's number within that participant.
    #[serde(rename = "SituationNumber")]
    pub situation_number: SituationNumber,
    /// The country the updating participant belongs to.
    #[serde(rename = "UpdateCountryRef", default, skip_serializing_if = "Option::is_none")]
    pub update_country_ref: Option<CountryRef>,
    /// The participant that made the revision being referred to.
    #[serde(rename = "UpdateParticipantRef", default, skip_serializing_if = "Option::is_none")]
    pub update_participant_ref: Option<ParticipantRef>,
    /// The revision being referred to.
    #[serde(rename = "Version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,
}

impl SituationFullRef {
    /// A reference to the given participant's situation.
    pub fn new(
        participant_ref: impl Into<ParticipantRef>,
        situation_number: impl Into<SituationNumber>,
    ) -> Self {
        Self {
            version_country_ref: None,
            participant_ref: participant_ref.into(),
            situation_number: situation_number.into(),
            update_country_ref: None,
            update_participant_ref: None,
            version: None,
        }
    }
}

/// A line, optionally narrowed to one direction of travel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineDirection {
    /// The line.
    #[serde(rename = "LineRef")]
    pub line_ref: LineRef,
    /// The direction along the line, if the reference is direction-specific.
    #[serde(rename = "DirectionRef", default, skip_serializing_if = "Option::is_none")]
    pub direction_ref: Option<DirectionRef>,
}

impl LineDirection {
    /// A reference to a whole line.
    pub fn new(line_ref: impl Into<LineRef>) -> Self {
        Self {
            line_ref: line_ref.into(),
            direction_ref: None,
        }
    }

    /// A reference to one direction of a line.
    pub fn with_direction(line_ref: impl Into<LineRef>, direction_ref: impl Into<DirectionRef>) -> Self {
        Self {
            line_ref: line_ref.into(),
            direction_ref: Some(direction_ref.into()),
        }
    }
}

/// Lines, each optionally narrowed to one direction, that a request filters on.
///
/// The schema gives every service that filters by line the same `Lines` wrapper.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestedLines {
    /// The lines and directions.
    #[serde(rename = "LineDirection")]
    pub line_direction: Vec<LineDirection>,
}

/// A journey identified by the operational day it runs on plus its timetable id.
///
/// A `DatedVehicleJourneyRef` is only unique within one operational day, so both
/// halves are needed to name a specific run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FramedVehicleJourneyRef {
    /// The operational day.
    #[serde(rename = "DataFrameRef")]
    pub data_frame_ref: DataFrameRef,
    /// The journey within that day.
    #[serde(rename = "DatedVehicleJourneyRef")]
    pub dated_vehicle_journey_ref: DatedVehicleJourneyRef,
}
