//! Elements of a vehicle journey that several services describe alike.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::model::reference::{DirectionRef, OperatorRef, StopPointRef};
use crate::types::NaturalLanguageString;

siri_ref! {
    /// Identifies a train number allocated to a journey.
    TrainNumberRef;
    /// Identifies one physical part of a train that can be joined or split.
    TrainPartRef;
    /// Identifies a train formed by joining several trains.
    CompoundTrainRef;
    /// Identifies one part of a journey run by a distinct part of a train.
    JourneyPartRef;
}

/// A direction of travel along a line, with the names shown to passengers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Direction {
    /// The direction.
    #[serde(rename = "DirectionRef")]
    pub direction_ref: DirectionRef,
    /// Names of the direction, one per language.
    #[serde(rename = "DirectionName", default, skip_serializing_if = "Vec::is_empty")]
    pub direction_name: Vec<NaturalLanguageString>,
}

impl Direction {
    /// A direction with no names attached.
    pub fn new(direction_ref: impl Into<DirectionRef>) -> Self {
        Self {
            direction_ref: direction_ref.into(),
            direction_name: Vec::new(),
        }
    }
}

/// One part of a journey run by a distinct part of a train.
///
/// A train that splits en route runs as several journey parts over the same
/// timetabled journey, each with its own train number and its own stretch of route.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct JourneyPartInfo {
    /// The journey part.
    #[serde(rename = "JourneyPartRef", default, skip_serializing_if = "Option::is_none")]
    pub journey_part_ref: Option<JourneyPartRef>,
    /// The train number this part runs under.
    #[serde(rename = "TrainNumberRef", default, skip_serializing_if = "Option::is_none")]
    pub train_number_ref: Option<TrainNumberRef>,
    /// The operator running this part.
    #[serde(rename = "OperatorRef", default, skip_serializing_if = "Option::is_none")]
    pub operator_ref: Option<OperatorRef>,
    /// The joined train this part belongs to, while it is joined.
    #[serde(rename = "CompoundTrainRef", default, skip_serializing_if = "Option::is_none")]
    pub compound_train_ref: Option<CompoundTrainRef>,
    /// Where this part starts.
    #[serde(rename = "FromStopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub from_stop_point_ref: Option<StopPointRef>,
    /// Where this part ends.
    #[serde(rename = "ToStopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub to_stop_point_ref: Option<StopPointRef>,
    /// When this part starts.
    #[serde(rename = "StartTime", default, skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime<FixedOffset>>,
    /// When this part ends.
    #[serde(rename = "EndTime", default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime<FixedOffset>>,
}

/// One part of a train, and where it sits in the formation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainBlockPart {
    /// How many parts the train is formed of.
    #[serde(rename = "NumberOfBlockParts")]
    pub number_of_block_parts: u64,
    /// The part this entry describes.
    #[serde(rename = "TrainPartRef")]
    pub train_part_ref: TrainPartRef,
    /// Where the part sits, described for passengers, one per language.
    #[serde(rename = "PositionOfTrainBlockPart", default, skip_serializing_if = "Vec::is_empty")]
    pub position_of_train_block_part: Vec<NaturalLanguageString>,
}
