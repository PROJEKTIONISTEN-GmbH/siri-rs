//! Accessibility: what a passenger needs, and whether a service provides it.
//!
//! These types come from the CEN accessibility annex that SIRI and IFOPT share. A
//! [`UserNeed`] states one requirement a passenger has; a [`Suitability`] judges a
//! service against one such need; an [`AccessibilityAssessment`] describes what a
//! place or vehicle offers.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::{Accessibility, Encumbrance, MedicalNeed, Mobility, PyschosensoryNeed};
use crate::types::Extensions;

/// The needs of a passenger a producer should take into account.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PassengerAccessibilityNeeds {
    /// The individual needs.
    #[serde(rename = "UserNeed", default, skip_serializing_if = "Vec::is_empty")]
    pub user_need: Vec<UserNeed>,
    /// Whether the passenger travels with someone assisting them.
    #[serde(rename = "AccompaniedByCarer", default, skip_serializing_if = "Option::is_none")]
    pub accompanied_by_carer: Option<bool>,
}

/// A requirement a passenger has that may constrain which services they can use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserNeed {
    /// Which kind of need this is.
    #[serde(rename = "$value")]
    pub need: UserNeedKind,
    /// Whether the need is being excluded rather than included; absent means the
    /// need is included.
    #[serde(rename = "Excluded", default, skip_serializing_if = "Option::is_none")]
    pub excluded: Option<bool>,
    /// How important this need is relative to the passenger's other needs.
    #[serde(rename = "NeedRanking", default, skip_serializing_if = "Option::is_none")]
    pub need_ranking: Option<i64>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl UserNeed {
    /// A need with no ranking and no exclusion.
    pub fn new(need: UserNeedKind) -> Self {
        Self {
            need,
            excluded: None,
            need_ranking: None,
            extensions: None,
        }
    }
}

/// The four families of passenger need, of which a [`UserNeed`] names exactly one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserNeedKind {
    /// A need arising from how the passenger moves, e.g. using a wheelchair.
    MobilityNeed(Mobility),
    /// A need arising from sight, hearing or cognition, or from an aversion to
    /// lifts, escalators, confined spaces or crowds.
    PsychosensoryNeed(PyschosensoryNeed),
    /// A medical condition that constrains the choice of service.
    MedicalNeed(MedicalNeed),
    /// Something the passenger is carrying or accompanied by, e.g. a pushchair or
    /// oversize baggage.
    EncumbranceNeed(Encumbrance),
}

/// How something bears on passengers with particular accessibility needs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Suitabilities {
    /// The judgement about one passenger need.
    #[serde(rename = "Suitability")]
    pub suitability: Vec<Suitability>,
}

impl Suitabilities {
    /// A set of judgements.
    pub fn new(suitability: Vec<Suitability>) -> Self {
        Self { suitability }
    }
}

/// Whether something remains usable by a passenger with a given need.
///
/// A lift out of service, for example, is `notSuitable` for a wheelchair user while
/// leaving the service usable by everyone else.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Suitability {
    /// Whether it suits the passenger need named below.
    #[serde(rename = "Suitable")]
    pub suitable: crate::enumerations::Suitability,
    /// The passenger need this judgement is about.
    #[serde(rename = "UserNeed")]
    pub user_need: UserNeed,
}

impl Suitability {
    /// A judgement about one passenger need.
    pub fn new(suitable: crate::enumerations::Suitability, user_need: UserNeed) -> Self {
        Self {
            suitable,
            user_need,
        }
    }
}

/// What a place or vehicle offers passengers with restricted mobility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessibilityAssessment {
    /// Whether passengers with restricted mobility can use it at all.
    #[serde(rename = "MobilityImpairedAccess")]
    pub mobility_impaired_access: bool,
    /// The specific limitations, where they are known in detail.
    #[serde(rename = "Limitations", default, skip_serializing_if = "Option::is_none")]
    pub limitations: Option<AccessibilityLimitations>,
    /// Judgements against individual passenger needs.
    #[serde(rename = "Suitabilities", default, skip_serializing_if = "Option::is_none")]
    pub suitabilities: Option<Suitabilities>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// The limitations an [`AccessibilityAssessment`] records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessibilityLimitations {
    /// One limitation, or one set of limitations valid over a given period.
    #[serde(rename = "AccessibilityLimitation")]
    pub accessibility_limitation: Vec<AccessibilityLimitation>,
}

/// What is and is not possible for a passenger with restricted mobility.
///
/// Each value is three-valued: the producer may state that a facility is present,
/// that it is absent, or that it does not know.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessibilityLimitation {
    /// Identifier of this limitation within the document.
    #[serde(rename = "LimitationId", default, skip_serializing_if = "Option::is_none")]
    pub limitation_id: Option<String>,
    /// When this limitation applies, if it is not permanent.
    #[serde(rename = "ValidityCondition", default, skip_serializing_if = "Option::is_none")]
    pub validity_condition: Option<ValidityCondition>,
    /// Whether a wheelchair user can get through.
    #[serde(rename = "WheelchairAccess")]
    pub wheelchair_access: Accessibility,
    /// Whether the route avoids steps.
    #[serde(rename = "StepFreeAccess", default, skip_serializing_if = "Option::is_none")]
    pub step_free_access: Option<Accessibility>,
    /// Whether the route avoids escalators.
    #[serde(rename = "EscalatorFreeAccess", default, skip_serializing_if = "Option::is_none")]
    pub escalator_free_access: Option<Accessibility>,
    /// Whether the route avoids lifts.
    #[serde(rename = "LiftFreeAccess", default, skip_serializing_if = "Option::is_none")]
    pub lift_free_access: Option<Accessibility>,
    /// Whether announcements are made audibly.
    #[serde(rename = "AudibleSignalsAvailable", default, skip_serializing_if = "Option::is_none")]
    pub audible_signals_available: Option<Accessibility>,
    /// Whether information is shown visually.
    #[serde(rename = "VisualSignsAvailable", default, skip_serializing_if = "Option::is_none")]
    pub visual_signs_available: Option<Accessibility>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// When a statement holds: between two instants, on certain days, at certain times.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidityCondition {
    /// The instant the condition starts to hold.
    #[serde(rename = "FromDateTime", default, skip_serializing_if = "Option::is_none")]
    pub from_date_time: Option<DateTime<FixedOffset>>,
    /// The instant the condition stops holding.
    #[serde(rename = "ToDateTime", default, skip_serializing_if = "Option::is_none")]
    pub to_date_time: Option<DateTime<FixedOffset>>,
    /// The kind of day the condition applies on, in the producer's own vocabulary.
    #[serde(rename = "DayType", default, skip_serializing_if = "Option::is_none")]
    pub day_type: Option<String>,
    /// The times of day the condition applies at.
    #[serde(rename = "Timebands", default, skip_serializing_if = "Vec::is_empty")]
    pub timebands: Vec<Timebands>,
}

/// A group of times of day.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timebands {
    /// The individual bands.
    #[serde(rename = "Timeband")]
    pub timeband: Timeband,
}

/// A stretch of the day, open-ended when no end time is given.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timeband {
    /// When the band starts, as an `xsd:time`.
    #[serde(rename = "StartTime")]
    pub start_time: String,
    /// When the band ends, as an `xsd:time`.
    #[serde(rename = "EndTime", default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_need_writes_its_family_as_the_element_name() {
        let need = UserNeed::new(UserNeedKind::MobilityNeed(Mobility::Wheelchair));
        assert_eq!(
            quick_xml::se::to_string_with_root("UserNeed", &need).unwrap(),
            "<UserNeed><MobilityNeed>wheelchair</MobilityNeed></UserNeed>"
        );
    }

    #[test]
    fn a_need_round_trips_with_its_ranking() {
        let mut need = UserNeed::new(UserNeedKind::EncumbranceNeed(Encumbrance::Pushchair));
        need.need_ranking = Some(2);
        let xml = quick_xml::se::to_string_with_root("UserNeed", &need).unwrap();
        assert_eq!(quick_xml::de::from_str::<UserNeed>(&xml).unwrap(), need);
    }

    #[test]
    fn a_suitability_judges_one_need() {
        let suitability = Suitability::new(
            crate::enumerations::Suitability::NotSuitable,
            UserNeed::new(UserNeedKind::MobilityNeed(Mobility::Wheelchair)),
        );
        let xml = quick_xml::se::to_string_with_root("Suitability", &suitability).unwrap();
        assert_eq!(
            xml,
            "<Suitability><Suitable>notSuitable</Suitable>\
             <UserNeed><MobilityNeed>wheelchair</MobilityNeed></UserNeed></Suitability>"
        );
        assert_eq!(
            quick_xml::de::from_str::<Suitability>(&xml).unwrap(),
            suitability
        );
    }
}
