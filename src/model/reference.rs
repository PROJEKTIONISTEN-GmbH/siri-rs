//! References to the entities of a transport network.

use serde::{Deserialize, Serialize};

siri_ref! {
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
