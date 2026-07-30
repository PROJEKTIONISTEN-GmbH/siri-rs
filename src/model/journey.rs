//! Elements of a vehicle journey that several services describe alike.
//!
//! Production Timetable, Estimated Timetable and Vehicle Monitoring each publish a
//! different view of the same thing — a vehicle running a route on a day — so the
//! schema factors the common parts into groups the three share: who runs the
//! journey, where it starts and ends, how it is progressing, which vehicle is on it.
//! Those groups are modelled here once and inlined into each service's own journey
//! type, because a schema group contributes its elements to the enclosing sequence
//! rather than nesting them.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::{JourneyRelationType, QualityIndex};
use crate::model::reference::{
    DirectionRef, FramedVehicleJourneyRef, LineRef, OperatorRef, StopPointRef,
};
use crate::types::{NaturalLanguagePlaceName, NaturalLanguageString, ParticipantRef};

siri_ref! {
    /// Identifies a train number allocated to a journey.
    TrainNumberRef;
    /// Identifies one physical part of a train that can be joined or split.
    TrainPartRef;
    /// Identifies a train formed by joining several trains.
    CompoundTrainRef;
    /// Identifies one part of a journey run by a distinct part of a train.
    JourneyPartRef;
    /// Identifies a place a journey starts at, passes or ends at.
    JourneyPlaceRef;
    /// Identifies a group of lines marketed together.
    GroupOfLinesRef;
    /// Identifies a brand a service is presented under.
    BrandingRef;
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

/// How to reach the people running or selling a service.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimpleContact {
    /// A telephone number.
    #[serde(rename = "PhoneNumber", default, skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    /// A web address.
    #[serde(rename = "Url", default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// A place a journey passes through, named to tell it apart from similar journeys.
///
/// "Luton to Luton via Sutton" is why these exist: the endpoints alone do not
/// identify the service.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViaName {
    /// The place, in a gazetteer.
    #[serde(rename = "PlaceRef", default, skip_serializing_if = "Option::is_none")]
    pub place_ref: Option<JourneyPlaceRef>,
    /// Names of the place, one per language.
    #[serde(rename = "PlaceName", default, skip_serializing_if = "Vec::is_empty")]
    pub place_name: Vec<NaturalLanguagePlaceName>,
    /// Shorter names of the place, one per language.
    #[serde(rename = "PlaceShortName", default, skip_serializing_if = "Vec::is_empty")]
    pub place_short_name: Vec<NaturalLanguagePlaceName>,
    /// How prominently to show this place; lower numbers come first.
    #[serde(rename = "ViaPriority", default, skip_serializing_if = "Option::is_none")]
    pub via_priority: Option<u64>,
}

impl ViaName {
    /// A via point given only by name.
    pub fn named(place_name: NaturalLanguagePlaceName) -> Self {
        Self {
            place_name: vec![place_name],
            ..Self::default()
        }
    }
}

/// A brand a service is presented under.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Branding {
    /// The producer's code for the brand.
    #[serde(rename = "BrandingCode")]
    pub branding_code: String,
    /// The brand's name.
    #[serde(rename = "Name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<NaturalLanguageString>,
    /// A shorter name for it.
    #[serde(rename = "ShortName", default, skip_serializing_if = "Option::is_none")]
    pub short_name: Option<NaturalLanguageString>,
    /// The brand described.
    #[serde(rename = "Description", default, skip_serializing_if = "Option::is_none")]
    pub description: Option<NaturalLanguageString>,
    /// A logo to present it with.
    #[serde(rename = "Image", default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// A web address for it.
    #[serde(rename = "Url", default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// The producer's own code for it.
    #[serde(rename = "PrivateCode", default, skip_serializing_if = "Option::is_none")]
    pub private_code: Option<String>,
}

/// How far a vehicle has come between the stop behind it and the stop ahead.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProgressBetweenStops {
    /// The distance between the two stops, in metres.
    #[serde(rename = "LinkDistance", default, skip_serializing_if = "Option::is_none")]
    pub link_distance: Option<f64>,
    /// How much of that distance has been covered, as a percentage.
    #[serde(rename = "Percentage", default, skip_serializing_if = "Option::is_none")]
    pub percentage: Option<f64>,
}

/// How much confidence to put in a predicted time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictionQuality {
    /// How good the prediction is thought to be.
    #[serde(rename = "PredictionLevel")]
    pub prediction_level: QualityIndex,
    /// The share of vehicles expected to fall within the limits below.
    #[serde(rename = "Percentile", default, skip_serializing_if = "Option::is_none")]
    pub percentile: Option<f64>,
    /// The earliest time within that share.
    #[serde(rename = "LowerTimeLimit", default, skip_serializing_if = "Option::is_none")]
    pub lower_time_limit: Option<DateTime<FixedOffset>>,
    /// The latest time within that share.
    #[serde(rename = "HigherTimeLimit", default, skip_serializing_if = "Option::is_none")]
    pub higher_time_limit: Option<DateTime<FixedOffset>>,
}

impl PredictionQuality {
    /// A prediction of the stated quality, with no confidence interval attached.
    pub fn new(prediction_level: QualityIndex) -> Self {
        Self {
            prediction_level,
            percentile: None,
            lower_time_limit: None,
            higher_time_limit: None,
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

/// The parts a journey is split into.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct JourneyParts {
    /// The parts, at least one.
    #[serde(rename = "JourneyPartInfo")]
    pub journey_part_info: Vec<JourneyPartInfo>,
}

/// The train numbers a journey runs under.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainNumbers {
    /// The numbers, at least one.
    #[serde(rename = "TrainNumberRef")]
    pub train_number_ref: Vec<TrainNumberRef>,
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

/// A journey named by where and when it runs rather than by an identifier.
///
/// Two operators that do not share journey identifiers can still mean the same run:
/// it is the one leaving that origin at that time and reaching that destination at
/// that time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatedVehicleJourneyIndirectRef {
    /// Where the journey starts.
    #[serde(rename = "OriginRef")]
    pub origin_ref: StopPointRef,
    /// When it is planned to leave there.
    #[serde(rename = "AimedDepartureTime")]
    pub aimed_departure_time: DateTime<FixedOffset>,
    /// Where it ends.
    #[serde(rename = "DestinationRef")]
    pub destination_ref: StopPointRef,
    /// When it is planned to arrive there.
    #[serde(rename = "AimedArrivalTime")]
    pub aimed_arrival_time: DateTime<FixedOffset>,
}

/// A journey referred to from another one, as a feeder, distributor or relation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectingJourneyRef {
    /// The journey on its operational day.
    #[serde(rename = "FramedVehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub framed_vehicle_journey_ref: Option<FramedVehicleJourneyRef>,
    /// The journey named by where and when it runs.
    #[serde(rename = "DatedVehicleJourneyIndirectRef", default, skip_serializing_if = "Option::is_none")]
    pub dated_vehicle_journey_indirect_ref: Option<DatedVehicleJourneyIndirectRef>,
    /// The line it runs on.
    #[serde(rename = "LineRef", default, skip_serializing_if = "Option::is_none")]
    pub line_ref: Option<LineRef>,
    /// The train number it runs under.
    #[serde(rename = "TrainNumberRef", default, skip_serializing_if = "Option::is_none")]
    pub train_number_ref: Option<TrainNumberRef>,
    /// The operator running it.
    #[serde(rename = "OperatorRef", default, skip_serializing_if = "Option::is_none")]
    pub operator_ref: Option<OperatorRef>,
    /// The participant whose identifiers the reference is drawn from.
    #[serde(rename = "ParticipantRef", default, skip_serializing_if = "Option::is_none")]
    pub participant_ref: Option<ParticipantRef>,
}

/// The other journeys a journey stands in a relation to.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct JourneyRelations {
    /// The relations, at least one.
    #[serde(rename = "JourneyRelation")]
    pub journey_relation: Vec<JourneyRelation>,
}

/// One relation between a journey and one or more others.
///
/// A journey may be joined to another, split from it, replaced by it or continue as
/// it. `journey_relation_type` says which; the call or the journey parts say where
/// the relation takes effect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JourneyRelation {
    /// What kind of relation it is.
    #[serde(rename = "JourneyRelationType")]
    pub journey_relation_type: JourneyRelationType,
    /// The call the relation takes effect at.
    #[serde(rename = "CallInfo", default, skip_serializing_if = "Option::is_none")]
    pub call_info: Option<RelatedCall>,
    /// The journey parts the relation is about, instead of `call_info`.
    #[serde(rename = "JourneyParts", default, skip_serializing_if = "Option::is_none")]
    pub journey_parts: Option<JourneyParts>,
    /// The journeys on the other side of the relation, at least one.
    #[serde(rename = "RelatedJourney")]
    pub related_journey: Vec<RelatedJourney>,
}

/// Which alternative of a journey relation's choice is present.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JourneyRelationScope<'a> {
    /// The relation takes effect at one call.
    Call(&'a RelatedCall),
    /// The relation is about whole journey parts.
    Parts(&'a JourneyParts),
}

impl JourneyRelation {
    /// A relation of the given kind to the given journeys.
    pub fn new(
        journey_relation_type: JourneyRelationType,
        related_journey: Vec<RelatedJourney>,
    ) -> Self {
        Self {
            journey_relation_type,
            call_info: None,
            journey_parts: None,
            related_journey,
        }
    }

    /// Which alternative of the schema's choice this relation carries, or `None`
    /// when neither is present.
    pub fn scope(&self) -> Option<JourneyRelationScope<'_>> {
        self.call_info
            .as_ref()
            .map(JourneyRelationScope::Call)
            .or_else(|| self.journey_parts.as_ref().map(JourneyRelationScope::Parts))
    }
}

/// The call a journey relation takes effect at.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelatedCall {
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
    /// When the vehicle is planned to leave.
    #[serde(rename = "AimedDepartureTime", default, skip_serializing_if = "Option::is_none")]
    pub aimed_departure_time: Option<DateTime<FixedOffset>>,
    /// When it is planned to arrive.
    #[serde(rename = "AimedArrivalTime", default, skip_serializing_if = "Option::is_none")]
    pub aimed_arrival_time: Option<DateTime<FixedOffset>>,
}

impl RelatedCall {
    /// A call at the given stop with no times attached.
    pub fn at(stop_point_ref: impl Into<StopPointRef>) -> Self {
        Self {
            stop_point_ref: stop_point_ref.into(),
            visit_number: None,
            order: None,
            stop_point_name: Vec::new(),
            aimed_departure_time: None,
            aimed_arrival_time: None,
        }
    }
}

/// A journey on the other side of a relation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelatedJourney {
    /// The journey on its operational day.
    #[serde(rename = "FramedVehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub framed_vehicle_journey_ref: Option<FramedVehicleJourneyRef>,
    /// The journey named by where and when it runs.
    #[serde(rename = "DatedVehicleJourneyIndirectRef", default, skip_serializing_if = "Option::is_none")]
    pub dated_vehicle_journey_indirect_ref: Option<DatedVehicleJourneyIndirectRef>,
    /// The line it runs on.
    #[serde(rename = "LineRef", default, skip_serializing_if = "Option::is_none")]
    pub line_ref: Option<LineRef>,
    /// The train number it runs under.
    #[serde(rename = "TrainNumberRef", default, skip_serializing_if = "Option::is_none")]
    pub train_number_ref: Option<TrainNumberRef>,
    /// The operator running it.
    #[serde(rename = "OperatorRef", default, skip_serializing_if = "Option::is_none")]
    pub operator_ref: Option<OperatorRef>,
    /// The participant whose identifiers the reference is drawn from.
    #[serde(rename = "ParticipantRef", default, skip_serializing_if = "Option::is_none")]
    pub participant_ref: Option<ParticipantRef>,
    /// The call the relation takes effect at, seen from this journey.
    #[serde(rename = "CallInfo", default, skip_serializing_if = "Option::is_none")]
    pub call_info: Option<RelatedCall>,
    /// The journey parts the relation is about, seen from this journey.
    #[serde(rename = "JourneyParts", default, skip_serializing_if = "Option::is_none")]
    pub journey_parts: Option<JourneyParts>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_journey_relation_reports_which_alternative_it_carries() {
        let mut relation = JourneyRelation::new(
            JourneyRelationType::JoiningOfJourneys,
            vec![RelatedJourney {
                line_ref: Some(LineRef::new("S1")),
                ..RelatedJourney::default()
            }],
        );
        assert_eq!(relation.scope(), None);

        relation.call_info = Some(RelatedCall::at("HLTS001"));
        assert!(matches!(relation.scope(), Some(JourneyRelationScope::Call(call))
            if call.stop_point_ref.as_str() == "HLTS001"));

        relation.call_info = None;
        relation.journey_parts = Some(JourneyParts::default());
        assert!(matches!(relation.scope(), Some(JourneyRelationScope::Parts(_))));
    }

    #[test]
    fn a_via_point_writes_its_names_before_its_priority() {
        let via = ViaName {
            via_priority: Some(1),
            ..ViaName::named(NaturalLanguagePlaceName::with_lang("EN", "Sutton"))
        };
        let xml = quick_xml::se::to_string_with_root("Via", &via).unwrap();
        assert_eq!(
            xml,
            r#"<Via><PlaceName xml:lang="EN">Sutton</PlaceName><ViaPriority>1</ViaPriority></Via>"#
        );

        let read: ViaName = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(read, via);
    }
}
