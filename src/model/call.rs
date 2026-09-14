//! Where a vehicle stands at a stop.
//!
//! A call says *when* a vehicle serves a stop; a stop assignment says *where* — at
//! which quay, at which boarding position along it, or, for a service that is not
//! bound to a fixed stop at all, within which area. Each of those is given up to
//! three times: as planned (`Aimed`), as currently predicted (`Expected`) and as it
//! turned out (`Actual`).

use serde::{Deserialize, Serialize};

use crate::enumerations::TypeOfNestedQuay;
use crate::model::location::FlexibleArea;
use crate::model::reference::QuayRef;
use crate::types::NaturalLanguageString;

siri_ref! {
    /// Identifies one position along a quay a vehicle door comes to rest at.
    BoardingPositionRef;
    /// Identifies an area a flexible service may be boarded within.
    FlexibleAreaRef;
}

/// Where a vehicle stands at a stop, as planned, predicted and observed.
///
/// The schema offers a choice between naming a quay and naming a boarding position
/// along one; both sets of fields are optional here, and
/// [`StopAssignment::place`] reports which one is filled in.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StopAssignment {
    /// The quay the vehicle is planned to use.
    #[serde(rename = "AimedQuayRef", default, skip_serializing_if = "Option::is_none")]
    pub aimed_quay_ref: Option<QuayRef>,
    /// Names of the planned quay, one per language.
    #[serde(rename = "AimedQuayName", default, skip_serializing_if = "Vec::is_empty")]
    pub aimed_quay_name: Vec<NaturalLanguageString>,
    /// The quay the vehicle is now expected to use.
    #[serde(rename = "ExpectedQuayRef", default, skip_serializing_if = "Option::is_none")]
    pub expected_quay_ref: Option<QuayRef>,
    /// Names of the expected quay, one per language.
    #[serde(rename = "ExpectedQuayName", default, skip_serializing_if = "Vec::is_empty")]
    pub expected_quay_name: Vec<NaturalLanguageString>,
    /// The quay the vehicle used.
    #[serde(rename = "ActualQuayRef", default, skip_serializing_if = "Option::is_none")]
    pub actual_quay_ref: Option<QuayRef>,
    /// Names of the quay used, one per language.
    #[serde(rename = "ActualQuayName", default, skip_serializing_if = "Vec::is_empty")]
    pub actual_quay_name: Vec<NaturalLanguageString>,
    /// What kind of quay it is.
    #[serde(rename = "QuayType", default, skip_serializing_if = "Option::is_none")]
    pub quay_type: Option<TypeOfNestedQuay>,
    /// The boarding position the vehicle is planned to come to rest at.
    #[serde(rename = "AimedBoardingPositionRef", default, skip_serializing_if = "Option::is_none")]
    pub aimed_boarding_position_ref: Option<BoardingPositionRef>,
    /// Names of the planned boarding position, one per language.
    #[serde(rename = "AimedBoardingPositionName", default, skip_serializing_if = "Vec::is_empty")]
    pub aimed_boarding_position_name: Vec<NaturalLanguageString>,
    /// The boarding position the vehicle is now expected at.
    #[serde(rename = "ExpectedBoardingPositionRef", default, skip_serializing_if = "Option::is_none")]
    pub expected_boarding_position_ref: Option<BoardingPositionRef>,
    /// Names of the expected boarding position, one per language.
    #[serde(rename = "ExpectedBoardingPositionName", default, skip_serializing_if = "Vec::is_empty")]
    pub expected_boarding_position_name: Vec<NaturalLanguageString>,
    /// The boarding position the vehicle came to rest at.
    #[serde(rename = "ActualBoardingPositionRef", default, skip_serializing_if = "Option::is_none")]
    pub actual_boarding_position_ref: Option<BoardingPositionRef>,
    /// Names of the boarding position used, one per language.
    #[serde(rename = "ActualBoardingPositionName", default, skip_serializing_if = "Vec::is_empty")]
    pub actual_boarding_position_name: Vec<NaturalLanguageString>,
    /// The area boarding is planned to happen within.
    #[serde(rename = "AimedFlexibleArea", default, skip_serializing_if = "Option::is_none")]
    pub aimed_flexible_area: Option<FlexibleArea>,
    /// The planned area, stated elsewhere.
    #[serde(rename = "AimedFlexibleAreaRef", default, skip_serializing_if = "Option::is_none")]
    pub aimed_flexible_area_ref: Option<FlexibleAreaRef>,
    /// Names of the planned area, one per language.
    #[serde(rename = "AimedLocationName", default, skip_serializing_if = "Vec::is_empty")]
    pub aimed_location_name: Vec<NaturalLanguageString>,
    /// The area boarding is now expected within.
    #[serde(rename = "ExpectedFlexibleArea", default, skip_serializing_if = "Option::is_none")]
    pub expected_flexible_area: Option<FlexibleArea>,
    /// The expected area, stated elsewhere.
    #[serde(rename = "ExpectedFlexibleAreaRef", default, skip_serializing_if = "Option::is_none")]
    pub expected_flexible_area_ref: Option<FlexibleAreaRef>,
    /// Names of the expected area, one per language.
    #[serde(rename = "ExpectedLocationName", default, skip_serializing_if = "Vec::is_empty")]
    pub expected_location_name: Vec<NaturalLanguageString>,
    /// The area boarding happened within.
    #[serde(rename = "ActualFlexibleArea", default, skip_serializing_if = "Option::is_none")]
    pub actual_flexible_area: Option<FlexibleArea>,
    /// The area used, stated elsewhere.
    #[serde(rename = "ActualFlexibleAreaRef", default, skip_serializing_if = "Option::is_none")]
    pub actual_flexible_area_ref: Option<FlexibleAreaRef>,
    /// Names of the area used, one per language.
    #[serde(rename = "ActualLocationName", default, skip_serializing_if = "Vec::is_empty")]
    pub actual_location_name: Vec<NaturalLanguageString>,
}

/// Which of the two ways of naming a standing place an assignment uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StandingPlace {
    /// The assignment names quays.
    Quay,
    /// The assignment names boarding positions along a quay.
    BoardingPosition,
}

impl StopAssignment {
    /// A vehicle standing at the given planned quay.
    pub fn at_quay(aimed_quay_ref: impl Into<QuayRef>) -> Self {
        Self {
            aimed_quay_ref: Some(aimed_quay_ref.into()),
            ..Self::default()
        }
    }

    /// A vehicle stopping at the given planned boarding position.
    pub fn at_boarding_position(aimed_boarding_position_ref: impl Into<BoardingPositionRef>) -> Self {
        Self {
            aimed_boarding_position_ref: Some(aimed_boarding_position_ref.into()),
            ..Self::default()
        }
    }

    /// Which alternative of the schema's choice this assignment carries, or `None`
    /// when it names neither.
    pub fn place(&self) -> Option<StandingPlace> {
        let quay = self.aimed_quay_ref.is_some()
            || self.expected_quay_ref.is_some()
            || self.actual_quay_ref.is_some()
            || !self.aimed_quay_name.is_empty()
            || !self.expected_quay_name.is_empty()
            || !self.actual_quay_name.is_empty()
            || self.quay_type.is_some();
        let position = self.aimed_boarding_position_ref.is_some()
            || self.expected_boarding_position_ref.is_some()
            || self.actual_boarding_position_ref.is_some()
            || !self.aimed_boarding_position_name.is_empty()
            || !self.expected_boarding_position_name.is_empty()
            || !self.actual_boarding_position_name.is_empty();
        match (quay, position) {
            (true, _) => Some(StandingPlace::Quay),
            (false, true) => Some(StandingPlace::BoardingPosition),
            (false, false) => None,
        }
    }
}

/// Where a vehicle is *planned* to stand at a stop.
///
/// The planned form of [`StopAssignment`]: a timetable states only the aimed values,
/// so the expected and actual ones are absent from the schema rather than merely
/// left empty.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PlannedStopAssignment {
    /// The quay the vehicle is planned to use.
    #[serde(rename = "AimedQuayRef", default, skip_serializing_if = "Option::is_none")]
    pub aimed_quay_ref: Option<QuayRef>,
    /// Names of the planned quay, one per language.
    #[serde(rename = "AimedQuayName", default, skip_serializing_if = "Vec::is_empty")]
    pub aimed_quay_name: Vec<NaturalLanguageString>,
    /// What kind of quay it is.
    #[serde(rename = "QuayType", default, skip_serializing_if = "Option::is_none")]
    pub quay_type: Option<TypeOfNestedQuay>,
    /// The boarding position the vehicle is planned to come to rest at.
    #[serde(rename = "AimedBoardingPositionRef", default, skip_serializing_if = "Option::is_none")]
    pub aimed_boarding_position_ref: Option<BoardingPositionRef>,
    /// Names of the planned boarding position, one per language.
    #[serde(rename = "AimedBoardingPositionName", default, skip_serializing_if = "Vec::is_empty")]
    pub aimed_boarding_position_name: Vec<NaturalLanguageString>,
    /// The area boarding is planned to happen within.
    #[serde(rename = "AimedFlexibleArea", default, skip_serializing_if = "Option::is_none")]
    pub aimed_flexible_area: Option<FlexibleArea>,
    /// The planned area, stated elsewhere.
    #[serde(rename = "AimedFlexibleAreaRef", default, skip_serializing_if = "Option::is_none")]
    pub aimed_flexible_area_ref: Option<FlexibleAreaRef>,
    /// Names of the planned area, one per language.
    #[serde(rename = "AimedLocationName", default, skip_serializing_if = "Vec::is_empty")]
    pub aimed_location_name: Vec<NaturalLanguageString>,
}

impl PlannedStopAssignment {
    /// A vehicle planned to stand at the given quay.
    pub fn at_quay(aimed_quay_ref: impl Into<QuayRef>) -> Self {
        Self {
            aimed_quay_ref: Some(aimed_quay_ref.into()),
            ..Self::default()
        }
    }

    /// Which alternative of the schema's choice this assignment carries, or `None`
    /// when it names neither.
    pub fn place(&self) -> Option<StandingPlace> {
        if self.aimed_quay_ref.is_some() || !self.aimed_quay_name.is_empty() || self.quay_type.is_some()
        {
            Some(StandingPlace::Quay)
        } else if self.aimed_boarding_position_ref.is_some()
            || !self.aimed_boarding_position_name.is_empty()
        {
            Some(StandingPlace::BoardingPosition)
        } else {
            None
        }
    }
}

/// How many calls before and after the current one a request asks for.
///
/// The stop and vehicle monitoring services both cap the calling pattern this way.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaximumNumberOfCalls {
    /// At most this many stops already served.
    #[serde(rename = "Previous", default, skip_serializing_if = "Option::is_none")]
    pub previous: Option<u64>,
    /// At most this many stops still to come.
    #[serde(rename = "Onwards", default, skip_serializing_if = "Option::is_none")]
    pub onwards: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_assignment_writes_the_three_stages_of_a_quay_in_schema_order() {
        let assignment = StopAssignment {
            expected_quay_ref: Some(QuayRef::new("Q2")),
            quay_type: Some(TypeOfNestedQuay::Platform),
            ..StopAssignment::at_quay("Q1")
        };

        let xml = quick_xml::se::to_string_with_root("ArrivalStopAssignment", &assignment).unwrap();
        assert_eq!(
            xml,
            "<ArrivalStopAssignment><AimedQuayRef>Q1</AimedQuayRef>\
             <ExpectedQuayRef>Q2</ExpectedQuayRef><QuayType>platform</QuayType>\
             </ArrivalStopAssignment>"
        );

        let read: StopAssignment = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(read, assignment);
        assert_eq!(read.place(), Some(StandingPlace::Quay));
    }

    #[test]
    fn an_assignment_reports_which_kind_of_place_it_names() {
        assert_eq!(
            StopAssignment::at_boarding_position("B7").place(),
            Some(StandingPlace::BoardingPosition)
        );
        assert_eq!(StopAssignment::default().place(), None);
        assert_eq!(
            PlannedStopAssignment::at_quay("Q1").place(),
            Some(StandingPlace::Quay)
        );
        assert_eq!(PlannedStopAssignment::default().place(), None);
    }
}
