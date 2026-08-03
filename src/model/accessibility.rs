//! Accessibility: what a passenger needs, and whether a service provides it.
//!
//! These types come from the CEN accessibility annex that SIRI and IFOPT share. A
//! [`UserNeed`] states one requirement a passenger has; a [`Suitability`] judges a
//! service against one such need; an [`AccessibilityAssessment`] describes what a
//! place or vehicle offers.
//!
//! # Namespaces
//!
//! The annex has namespaces of its own — `acsb` for accessibility, `ifopt` for the
//! validity conditions it borrows — and SIRI reuses its *types* while naming the
//! elements itself. So an element such as `<AccessibilityAssessment>` is in the SIRI
//! namespace while everything under it is in `acsb`, and each type here names its
//! children accordingly. Every foreign element is also accepted without its prefix,
//! for documents that bind the annex namespaces as the default.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::{Accessibility, Encumbrance, MedicalNeed, Mobility, PyschosensoryNeed};
use crate::types::Extensions;

/// The CEN accessibility namespace, `http://www.ifopt.org.uk/acsb`.
pub const ACSB_NAMESPACE: &str = "http://www.ifopt.org.uk/acsb";

/// The IFOPT namespace, `http://www.ifopt.org.uk/ifopt`.
pub const IFOPT_NAMESPACE: &str = "http://www.ifopt.org.uk/ifopt";

pub(crate) fn acsb_namespace() -> String {
    ACSB_NAMESPACE.to_owned()
}

fn ifopt_namespace() -> String {
    IFOPT_NAMESPACE.to_owned()
}

/// The needs of a passenger a producer should take into account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PassengerAccessibilityNeeds {
    /// Binding of the `acsb` prefix used by this element's children.
    #[serde(rename = "@xmlns:acsb", default = "acsb_namespace")]
    pub acsb_namespace: String,
    /// The individual needs.
    #[serde(rename = "acsb:UserNeed", alias = "UserNeed", default, skip_serializing_if = "Vec::is_empty")]
    pub user_need: Vec<UserNeed>,
    /// Whether the passenger travels with someone assisting them.
    #[serde(rename = "acsb:AccompaniedByCarer", alias = "AccompaniedByCarer", default, skip_serializing_if = "Option::is_none")]
    pub accompanied_by_carer: Option<bool>,
}

impl Default for PassengerAccessibilityNeeds {
    fn default() -> Self {
        Self {
            acsb_namespace: acsb_namespace(),
            user_need: Vec::new(),
            accompanied_by_carer: None,
        }
    }
}

impl PassengerAccessibilityNeeds {
    /// The needs of one passenger, travelling unassisted.
    pub fn new(user_need: Vec<UserNeed>) -> Self {
        Self {
            user_need,
            ..Self::default()
        }
    }
}

/// A requirement a passenger has that may constrain which services they can use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserNeed {
    /// Binding of the `acsb` prefix used by this element's children.
    #[serde(rename = "@xmlns:acsb", default = "acsb_namespace")]
    pub acsb_namespace: String,
    /// Which kind of need this is.
    #[serde(rename = "$value")]
    pub need: UserNeedKind,
    /// Whether the need is being excluded rather than included; absent means the
    /// need is included.
    #[serde(rename = "acsb:Excluded", alias = "Excluded", default, skip_serializing_if = "Option::is_none")]
    pub excluded: Option<bool>,
    /// How important this need is relative to the passenger's other needs.
    #[serde(rename = "acsb:NeedRanking", alias = "NeedRanking", default, skip_serializing_if = "Option::is_none")]
    pub need_ranking: Option<i64>,
    /// Implementation-defined content.
    #[serde(rename = "acsb:Extensions", alias = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl UserNeed {
    /// A need with no ranking and no exclusion.
    pub fn new(need: UserNeedKind) -> Self {
        Self {
            acsb_namespace: acsb_namespace(),
            need,
            excluded: None,
            need_ranking: None,
            extensions: None,
        }
    }
}

/// The four families of passenger need, of which a [`UserNeed`] names exactly one.
///
/// The variant is written as the element name, in the `acsb` namespace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserNeedKind {
    /// A need arising from how the passenger moves, e.g. using a wheelchair.
    #[serde(rename = "acsb:MobilityNeed", alias = "MobilityNeed")]
    MobilityNeed(Mobility),
    /// A need arising from sight, hearing or cognition, or from an aversion to
    /// lifts, escalators, confined spaces or crowds.
    #[serde(rename = "acsb:PsychosensoryNeed", alias = "PsychosensoryNeed")]
    PsychosensoryNeed(PyschosensoryNeed),
    /// A medical condition that constrains the choice of service.
    #[serde(rename = "acsb:MedicalNeed", alias = "MedicalNeed")]
    MedicalNeed(MedicalNeed),
    /// Something the passenger is carrying or accompanied by, e.g. a pushchair or
    /// oversize baggage.
    #[serde(rename = "acsb:EncumbranceNeed", alias = "EncumbranceNeed")]
    EncumbranceNeed(Encumbrance),
}

/// Judgements about passenger needs, written with SIRI's own element names.
///
/// SIRI declares this wrapper itself wherever a message judges a service against
/// passenger needs, so the entries are `<Suitability>` rather than
/// `<acsb:Suitability>`; the entries themselves are annex content either way. Inside
/// an [`AccessibilityAssessment`] the wrapper belongs to the annex instead — that
/// one is [`AssessmentSuitabilities`].
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

/// The judgements an [`AccessibilityAssessment`] carries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssessmentSuitabilities {
    /// The judgement about one passenger need.
    #[serde(rename = "acsb:Suitability", alias = "Suitability")]
    pub suitability: Vec<Suitability>,
}

impl AssessmentSuitabilities {
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
    /// Binding of the `acsb` prefix used by this element's children.
    #[serde(rename = "@xmlns:acsb", default = "acsb_namespace")]
    pub acsb_namespace: String,
    /// Whether it suits the passenger need named below.
    #[serde(rename = "acsb:Suitable", alias = "Suitable")]
    pub suitable: crate::enumerations::Suitability,
    /// The passenger need this judgement is about.
    #[serde(rename = "acsb:UserNeed", alias = "UserNeed")]
    pub user_need: UserNeed,
}

impl Suitability {
    /// A judgement about one passenger need.
    pub fn new(suitable: crate::enumerations::Suitability, user_need: UserNeed) -> Self {
        Self {
            acsb_namespace: acsb_namespace(),
            suitable,
            user_need,
        }
    }
}

/// What a place or vehicle offers passengers with restricted mobility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessibilityAssessment {
    /// Binding of the `acsb` prefix used by this element and everything under it.
    #[serde(rename = "@xmlns:acsb", default = "acsb_namespace")]
    pub acsb_namespace: String,
    /// Binding of the `ifopt` prefix the validity conditions and extensions use.
    #[serde(rename = "@xmlns:ifopt", default = "ifopt_namespace")]
    pub ifopt_namespace: String,
    /// Whether passengers with restricted mobility can use it at all.
    #[serde(rename = "acsb:MobilityImpairedAccess", alias = "MobilityImpairedAccess")]
    pub mobility_impaired_access: bool,
    /// The specific limitations, where they are known in detail.
    #[serde(rename = "acsb:Limitations", alias = "Limitations", default, skip_serializing_if = "Option::is_none")]
    pub limitations: Option<AccessibilityLimitations>,
    /// Judgements against individual passenger needs.
    #[serde(rename = "acsb:Suitabilities", alias = "Suitabilities", default, skip_serializing_if = "Option::is_none")]
    pub suitabilities: Option<AssessmentSuitabilities>,
    /// Implementation-defined content.
    #[serde(rename = "ifopt:Extensions", alias = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl AccessibilityAssessment {
    /// An assessment stating only whether the place or vehicle is usable at all.
    pub fn new(mobility_impaired_access: bool) -> Self {
        Self {
            acsb_namespace: acsb_namespace(),
            ifopt_namespace: ifopt_namespace(),
            mobility_impaired_access,
            limitations: None,
            suitabilities: None,
            extensions: None,
        }
    }
}

/// The limitations an [`AccessibilityAssessment`] records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessibilityLimitations {
    /// One limitation, or one set of limitations valid over a given period.
    #[serde(rename = "acsb:AccessibilityLimitation", alias = "AccessibilityLimitation")]
    pub accessibility_limitation: Vec<AccessibilityLimitation>,
}

/// What is and is not possible for a passenger with restricted mobility.
///
/// Each value is three-valued: the producer may state that a facility is present,
/// that it is absent, or that it does not know.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessibilityLimitation {
    /// Identifier of this limitation within the document.
    #[serde(rename = "acsb:LimitationId", alias = "LimitationId", default, skip_serializing_if = "Option::is_none")]
    pub limitation_id: Option<String>,
    /// When this limitation applies, if it is not permanent.
    #[serde(rename = "acsb:ValidityCondition", alias = "ValidityCondition", default, skip_serializing_if = "Option::is_none")]
    pub validity_condition: Option<ValidityCondition>,
    /// Whether a wheelchair user can get through.
    #[serde(rename = "acsb:WheelchairAccess", alias = "WheelchairAccess")]
    pub wheelchair_access: Accessibility,
    /// Whether the route avoids steps.
    #[serde(rename = "acsb:StepFreeAccess", alias = "StepFreeAccess", default, skip_serializing_if = "Option::is_none")]
    pub step_free_access: Option<Accessibility>,
    /// Whether the route avoids escalators.
    #[serde(rename = "acsb:EscalatorFreeAccess", alias = "EscalatorFreeAccess", default, skip_serializing_if = "Option::is_none")]
    pub escalator_free_access: Option<Accessibility>,
    /// Whether the route avoids lifts.
    #[serde(rename = "acsb:LiftFreeAccess", alias = "LiftFreeAccess", default, skip_serializing_if = "Option::is_none")]
    pub lift_free_access: Option<Accessibility>,
    /// Whether announcements are made audibly.
    #[serde(rename = "acsb:AudibleSignalsAvailable", alias = "AudibleSignalsAvailable", default, skip_serializing_if = "Option::is_none")]
    pub audible_signals_available: Option<Accessibility>,
    /// Whether information is shown visually.
    #[serde(rename = "acsb:VisualSignsAvailable", alias = "VisualSignsAvailable", default, skip_serializing_if = "Option::is_none")]
    pub visual_signs_available: Option<Accessibility>,
    /// Implementation-defined content.
    #[serde(rename = "ifopt:Extensions", alias = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl AccessibilityLimitation {
    /// A limitation stating only whether a wheelchair user can get through.
    pub fn new(wheelchair_access: Accessibility) -> Self {
        Self {
            limitation_id: None,
            validity_condition: None,
            wheelchair_access,
            step_free_access: None,
            escalator_free_access: None,
            lift_free_access: None,
            audible_signals_available: None,
            visual_signs_available: None,
            extensions: None,
        }
    }
}

/// When a statement holds: between two instants, on certain days, at certain times.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidityCondition {
    /// The instant the condition starts to hold.
    #[serde(rename = "ifopt:FromDateTime", alias = "FromDateTime", default, skip_serializing_if = "Option::is_none")]
    pub from_date_time: Option<DateTime<FixedOffset>>,
    /// The instant the condition stops holding.
    #[serde(rename = "ifopt:ToDateTime", alias = "ToDateTime", default, skip_serializing_if = "Option::is_none")]
    pub to_date_time: Option<DateTime<FixedOffset>>,
    /// The kind of day the condition applies on, in the producer's own vocabulary.
    #[serde(rename = "ifopt:DayType", alias = "DayType", default, skip_serializing_if = "Option::is_none")]
    pub day_type: Option<String>,
    /// The times of day the condition applies at.
    #[serde(rename = "ifopt:Timebands", alias = "Timebands", default, skip_serializing_if = "Vec::is_empty")]
    pub timebands: Vec<Timebands>,
}

/// A group of times of day.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timebands {
    /// The individual bands.
    #[serde(rename = "ifopt:Timeband", alias = "Timeband")]
    pub timeband: Timeband,
}

/// A stretch of the day, open-ended when no end time is given.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timeband {
    /// When the band starts, as an `xsd:time`.
    #[serde(rename = "ifopt:StartTime", alias = "StartTime")]
    pub start_time: String,
    /// When the band ends, as an `xsd:time`.
    #[serde(rename = "ifopt:EndTime", alias = "EndTime", default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_need_writes_its_family_as_an_annex_element() {
        let need = UserNeed::new(UserNeedKind::MobilityNeed(Mobility::Wheelchair));
        assert_eq!(
            quick_xml::se::to_string_with_root("UserNeed", &need).unwrap(),
            concat!(
                r#"<UserNeed xmlns:acsb="http://www.ifopt.org.uk/acsb">"#,
                "<acsb:MobilityNeed>wheelchair</acsb:MobilityNeed></UserNeed>"
            )
        );
    }

    #[test]
    fn a_need_round_trips_with_its_ranking() {
        let mut need = UserNeed::new(UserNeedKind::EncumbranceNeed(Encumbrance::Pushchair));
        need.need_ranking = Some(2);
        let xml = quick_xml::se::to_string_with_root("UserNeed", &need).unwrap();
        assert_eq!(quick_xml::de::from_str::<UserNeed>(&xml).unwrap(), need);
    }

    /// Documents that bind the annex namespace as their default carry the same
    /// content without any prefix, and have to read as the same value.
    #[test]
    fn an_unprefixed_annex_element_reads_as_the_same_need() {
        let prefixed: UserNeed = quick_xml::de::from_str(
            r#"<UserNeed><acsb:MobilityNeed>wheelchair</acsb:MobilityNeed><acsb:Excluded>true</acsb:Excluded></UserNeed>"#,
        )
        .unwrap();
        let bare: UserNeed = quick_xml::de::from_str(
            r#"<UserNeed><MobilityNeed>wheelchair</MobilityNeed><Excluded>true</Excluded></UserNeed>"#,
        )
        .unwrap();
        assert_eq!(prefixed, bare);
        assert_eq!(bare.excluded, Some(true));
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
            concat!(
                r#"<Suitability xmlns:acsb="http://www.ifopt.org.uk/acsb">"#,
                "<acsb:Suitable>notSuitable</acsb:Suitable>",
                r#"<acsb:UserNeed xmlns:acsb="http://www.ifopt.org.uk/acsb">"#,
                "<acsb:MobilityNeed>wheelchair</acsb:MobilityNeed></acsb:UserNeed></Suitability>"
            )
        );
        assert_eq!(
            quick_xml::de::from_str::<Suitability>(&xml).unwrap(),
            suitability
        );
    }

    #[test]
    fn an_assessment_declares_both_annex_namespaces_it_uses() {
        let mut assessment = AccessibilityAssessment::new(true);
        assessment.limitations = Some(AccessibilityLimitations {
            accessibility_limitation: vec![AccessibilityLimitation {
                validity_condition: Some(ValidityCondition {
                    from_date_time: Some(
                        DateTime::parse_from_rfc3339("2004-12-17T09:30:47-05:00").unwrap(),
                    ),
                    ..ValidityCondition::default()
                }),
                ..AccessibilityLimitation::new(Accessibility::False)
            }],
        });

        let xml =
            quick_xml::se::to_string_with_root("AccessibilityAssessment", &assessment).unwrap();
        assert!(xml.contains(r#"xmlns:acsb="http://www.ifopt.org.uk/acsb""#), "{xml}");
        assert!(xml.contains(r#"xmlns:ifopt="http://www.ifopt.org.uk/ifopt""#), "{xml}");
        assert!(xml.contains("<acsb:WheelchairAccess>false</acsb:WheelchairAccess>"), "{xml}");
        assert!(xml.contains("<ifopt:FromDateTime>"), "{xml}");

        assert_eq!(
            quick_xml::de::from_str::<AccessibilityAssessment>(&xml).unwrap(),
            assessment
        );
    }
}
