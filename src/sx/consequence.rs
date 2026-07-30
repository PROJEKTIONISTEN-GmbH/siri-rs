//! What a situation does to the transport service: the consequences.
//!
//! A situation describes an event — a fire, a strike, a flooded tunnel. Its
//! *consequences* describe what that event does to the operation of the network:
//! which services are diverted or cancelled, how severe the effect is, how long
//! passengers are delayed, what they should do instead, and whether the affected
//! journeys should still be offered by a journey planner at all.
//!
//! One situation carries a list of consequences, because a single event usually
//! affects different parts of the network in different ways. Every field of a
//! consequence that also exists on the situation — the period, the severity, the
//! affected scope — overrides the situation-level value for the part of the network
//! that consequence talks about; when omitted, the situation-level value applies.

use serde::{Deserialize, Serialize};

use crate::enumerations::{
    AdviceType, ArrivalBoardingActivity, DelayBand, DelaysType, DepartureBoardingActivity,
    Encumbrance, MedicalNeed, Mobility, PyschosensoryNeed, ServiceCondition, Severity,
    TicketRestriction,
};
use crate::sx::affects::AffectsScope;
use crate::sx::situation::HalfOpenTimestampOutputRange;
use crate::types::{Duration, Extensions, NaturalLanguageString};

siri_ref! {
    /// Identifies a pre-agreed advisory notice held by both sender and receiver.
    ///
    /// Referencing a notice instead of spelling one out lets a producer reuse a
    /// wording that a consumer has already translated and approved.
    AdviceRef;
}

/// Everything a situation does to the transport service.
///
/// The list is ordered by the producer; no significance attaches to the position of
/// an entry beyond the producer's own presentation preference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PtConsequences {
    /// One effect of the situation on the service.
    #[serde(rename = "Consequence")]
    pub consequence: Vec<Consequence>,
}

impl PtConsequences {
    /// The consequences of one situation.
    pub fn new(consequence: Vec<Consequence>) -> Self {
        Self { consequence }
    }
}

/// One effect of a situation on the transport service.
///
/// Each field that restates something the situation already says — `period`,
/// `severity`, `affects` — narrows or overrides it for this consequence alone. A
/// consequence with none of them set applies to the situation's own period,
/// severity and affected scope.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Consequence {
    /// When this effect is in force, if that differs from the situation's own
    /// validity period. Several periods express a recurring effect.
    #[serde(rename = "Period", default, skip_serializing_if = "Vec::is_empty")]
    pub period: Vec<HalfOpenTimestampOutputRange>,
    /// How the service is affected, as a code a receiving system can turn into a
    /// standard message in the passenger's own language.
    #[serde(
        rename = "Condition",
        default,
        deserialize_with = "crate::xml::token_list::deserialize",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub condition: Vec<ServiceCondition>,
    /// How the service is affected, in words.
    ///
    /// This is a few words to drop into a generated message, not a whole message,
    /// and is normally only sent when no `condition` code fits.
    #[serde(rename = "ConditionName", default, skip_serializing_if = "Vec::is_empty")]
    pub condition_name: Vec<NaturalLanguageString>,
    /// How badly passengers are affected, if that differs from the situation's own
    /// severity.
    #[serde(rename = "Severity", default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<Severity>,
    /// The part of the network this effect applies to, if narrower than the part the
    /// situation as a whole names.
    #[serde(rename = "Affects", default, skip_serializing_if = "Option::is_none")]
    pub affects: Option<AffectsScope>,
    /// How the effect bears on passengers with particular accessibility needs.
    #[serde(rename = "Suitabilities", default, skip_serializing_if = "Option::is_none")]
    pub suitabilities: Option<Suitabilities>,
    /// What passengers should do about it.
    #[serde(rename = "Advice", default, skip_serializing_if = "Option::is_none")]
    pub advice: Option<PtAdvice>,
    /// Whether affected services should be hidden from information systems rather
    /// than shown as disrupted.
    #[serde(rename = "Blocking", default, skip_serializing_if = "Option::is_none")]
    pub blocking: Option<Blocking>,
    /// Changes to whether passengers may board or alight at the affected stops.
    #[serde(rename = "Boarding", default, skip_serializing_if = "Option::is_none")]
    pub boarding: Option<Boarding>,
    /// How much extra time the effect costs passengers.
    #[serde(rename = "Delays", default, skip_serializing_if = "Option::is_none")]
    pub delays: Option<Delays>,
    /// Deaths and injuries, where the situation is an accident that caused them.
    #[serde(rename = "Casualties", default, skip_serializing_if = "Option::is_none")]
    pub casualties: Option<Casualties>,
    /// Fare rules relaxed while the effect lasts, so that passengers forced onto a
    /// different service are not charged again.
    #[serde(rename = "Easements", default, skip_serializing_if = "Vec::is_empty")]
    pub easements: Vec<Easements>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// How an effect bears on passengers with particular accessibility needs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Suitabilities {
    /// The effect on one passenger need.
    #[serde(rename = "Suitability")]
    pub suitability: Vec<Suitability>,
}

impl Suitabilities {
    /// The effects on a set of passenger needs.
    pub fn new(suitability: Vec<Suitability>) -> Self {
        Self { suitability }
    }
}

/// Whether the affected service remains usable by a passenger with a given need.
///
/// A lift out of service, for example, is `notSuitable` for a wheelchair user while
/// leaving the service usable by everyone else.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Suitability {
    /// Whether the service suits the passenger need named below.
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
    /// How important this need is relative to the passenger's other needs, on a
    /// scale of one to five.
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

/// What passengers should do about a disruption.
///
/// The advice can be given three ways, and a producer may combine them: as a code a
/// receiver turns into its own standard wording, as a few words to drop into a
/// generated message, or as a reference to a notice both sides already hold.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PtAdvice {
    /// A pre-agreed advisory notice to give to passengers.
    #[serde(rename = "AdviceRef", default, skip_serializing_if = "Option::is_none")]
    pub advice_ref: Option<AdviceRef>,
    /// The advice as a code, from which a standard message can be generated.
    #[serde(rename = "AdviceType", default, skip_serializing_if = "Option::is_none")]
    pub advice_type: Option<AdviceType>,
    /// The advice in words — a few words for a generated message, not a whole
    /// message. Normally only sent when no `advice_type` code fits.
    #[serde(rename = "AdviceName", default, skip_serializing_if = "Vec::is_empty")]
    pub advice_name: Vec<NaturalLanguageString>,
    /// Further advice to passengers, one entry per language.
    #[serde(rename = "Details", default, skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<NaturalLanguageString>,
}

/// Whether the affected services should be withheld from information systems.
///
/// Blocking is how an operator says "do not offer this at all" rather than "offer
/// this, disrupted": a journey planner that respects the flag will route around the
/// affected part of the network instead of proposing it with a warning.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blocking {
    /// Whether journey planners should suppress the affected part of the network.
    /// Absent means do not suppress.
    #[serde(rename = "JourneyPlanner", default, skip_serializing_if = "Option::is_none")]
    pub journey_planner: Option<bool>,
    /// Whether real-time departure displays should suppress it. Absent means do not
    /// suppress.
    #[serde(rename = "RealTime", default, skip_serializing_if = "Option::is_none")]
    pub real_time: Option<bool>,
}

/// Changes to whether passengers may board or alight at the affected stops.
///
/// This is how a situation expresses "the stop is served but closed to boarding",
/// as opposed to cancelling the call altogether.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Boarding {
    /// What alighting is allowed on arrival. Absent means alighting as normal.
    #[serde(rename = "ArrivalBoardingActivity", default, skip_serializing_if = "Option::is_none")]
    pub arrival_boarding_activity: Option<ArrivalBoardingActivity>,
    /// What boarding is allowed on departure. Absent means boarding as normal.
    #[serde(rename = "DepartureBoardingActivity", default, skip_serializing_if = "Option::is_none")]
    pub departure_boarding_activity: Option<DepartureBoardingActivity>,
}

/// How much extra time the disruption costs passengers.
///
/// A band is often all an operator can honestly commit to early in an incident; the
/// exact `delay` is added once it is known. Both may be sent together.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Delays {
    /// The band the delay falls into, e.g. up to ten minutes.
    #[serde(rename = "DelayBand", default, skip_serializing_if = "Option::is_none")]
    pub delay_band: Option<DelayBand>,
    /// The kind of delay, e.g. long delays or delays of uncertain duration.
    #[serde(rename = "DelayType", default, skip_serializing_if = "Option::is_none")]
    pub delay_type: Option<DelaysType>,
    /// The extra journey time the disruption adds.
    #[serde(rename = "Delay", default, skip_serializing_if = "Option::is_none")]
    pub delay: Option<Duration>,
}

/// Deaths and injuries caused by the incident behind the situation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Casualties {
    /// How many people were killed.
    #[serde(rename = "NumberOfDeaths", default, skip_serializing_if = "Option::is_none")]
    pub number_of_deaths: Option<u64>,
    /// How many people were injured.
    #[serde(rename = "NumberOfInjured", default, skip_serializing_if = "Option::is_none")]
    pub number_of_injured: Option<u64>,
}

/// Fare rules relaxed for the duration of the disruption.
///
/// An easement lets a ticket be used where it normally could not — a metro ticket
/// accepted on a replacement bus, say — so that passengers displaced by the
/// disruption are not asked to pay twice.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Easements {
    /// Which ticket classes remain valid while the easement is in force.
    #[serde(rename = "TicketRestrictions", default, skip_serializing_if = "Option::is_none")]
    pub ticket_restrictions: Option<TicketRestriction>,
    /// The easement in words, one entry per language.
    #[serde(rename = "Easement", default, skip_serializing_if = "Vec::is_empty")]
    pub easement: Vec<NaturalLanguageString>,
    /// A pre-agreed fare-exception code, where both sides already know the terms.
    #[serde(rename = "EasementRef", default, skip_serializing_if = "Option::is_none")]
    pub easement_ref: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::de::from_str;
    use quick_xml::se::to_string_with_root;

    /// A diversion, shaped like the consequences in the published SIRI-SX examples.
    const DIVERSION: &str = concat!(
        "<Consequence>",
        "<Period><StartTime>2004-12-17T09:30:47-05:00</StartTime>",
        "<EndTime>2004-12-17T18:00:00-05:00</EndTime></Period>",
        "<Condition>diverted</Condition>",
        "<Severity>severe</Severity>",
        "<Affects/>",
        "<Blocking><JourneyPlanner>false</JourneyPlanner><RealTime>true</RealTime></Blocking>",
        "<Boarding><ArrivalBoardingActivity>noAlighting</ArrivalBoardingActivity>",
        "<DepartureBoardingActivity>noBoarding</DepartureBoardingActivity></Boarding>",
        "</Consequence>",
    );

    #[test]
    fn a_consequence_round_trips_keeping_schema_element_order() {
        let consequence: Consequence = from_str(DIVERSION).unwrap();
        assert_eq!(consequence.period.len(), 1);
        assert_eq!(consequence.condition.len(), 1);
        assert!(consequence.severity.is_some());
        assert!(consequence.affects.is_some());
        assert_eq!(
            consequence.blocking.as_ref().unwrap().journey_planner,
            Some(false)
        );

        let written = to_string_with_root("Consequence", &consequence).unwrap();
        let at = |tag: &str| written.find(tag).unwrap_or_else(|| panic!("no {tag}"));
        assert!(at("<Period>") < at("<Condition>"));
        assert!(at("<Condition>") < at("<Severity>"));
        assert!(at("<Severity>") < at("<Affects"));
        assert!(at("<Affects") < at("<Blocking>"));
        assert!(at("<Blocking>") < at("<Boarding>"));

        assert_eq!(from_str::<Consequence>(&written).unwrap(), consequence);
    }

    #[test]
    fn a_consequence_may_state_several_conditions_and_name_them_per_language() {
        let xml = concat!(
            "<Consequence>",
            "<Condition>delayed</Condition>",
            "<Condition>diverted</Condition>",
            "<ConditionName xml:lang=\"EN\">Diverted and running late</ConditionName>",
            "<ConditionName xml:lang=\"DE\">Umleitung mit Verspätung</ConditionName>",
            "</Consequence>",
        );

        let mut consequence: Consequence = from_str(xml).unwrap();
        assert_eq!(consequence.condition.len(), 2);
        assert_eq!(consequence.condition_name.len(), 2);
        assert_eq!(consequence.condition_name[0].value, "Diverted and running late");
        assert_eq!(consequence.condition_name[1].value, "Umleitung mit Verspätung");

        let written = to_string_with_root("Consequence", &consequence).unwrap();
        assert_eq!(written.matches("<Condition>").count(), 2);
        assert!(written.find("<Condition>").unwrap() < written.find("<ConditionName").unwrap());
        assert_eq!(from_str::<Consequence>(&written).unwrap(), consequence);

        consequence.condition_name[1] = NaturalLanguageString::with_lang("DE", "Umleitung");
        let tagged = to_string_with_root("Consequence", &consequence).unwrap();
        assert!(tagged.contains("<ConditionName xml:lang=\"DE\">Umleitung</ConditionName>"));
    }

    #[test]
    fn delay_casualty_and_fare_detail_survive_a_round_trip() {
        let xml = concat!(
            "<Consequence>",
            "<Advice><AdviceType>useReplacementBus</AdviceType>",
            "<Details xml:lang=\"EN\">Replacement buses depart from the forecourt</Details></Advice>",
            "<Delays><DelayBand>upToTenMinutes</DelayBand><DelayType>longDelays</DelayType>",
            "<Delay>PT8M</Delay></Delays>",
            "<Casualties><NumberOfInjured>3</NumberOfInjured></Casualties>",
            "<Easements><TicketRestrictions>allTicketClassesValid</TicketRestrictions>",
            "<Easement xml:lang=\"EN\">Metro tickets are accepted on the buses</Easement>",
            "<EasementRef>METRO_ON_BUS</EasementRef></Easements>",
            "</Consequence>",
        );

        let consequence: Consequence = from_str(xml).unwrap();
        let delays = consequence.delays.as_ref().unwrap();
        assert_eq!(delays.delay.as_ref().unwrap().as_str(), "PT8M");
        assert_eq!(
            consequence.casualties.as_ref().unwrap().number_of_injured,
            Some(3)
        );
        assert_eq!(consequence.casualties.as_ref().unwrap().number_of_deaths, None);
        assert_eq!(consequence.easements.len(), 1);
        assert_eq!(
            consequence.easements[0].easement_ref.as_deref(),
            Some("METRO_ON_BUS")
        );
        assert_eq!(consequence.advice.as_ref().unwrap().details.len(), 1);

        let written = to_string_with_root("Consequence", &consequence).unwrap();
        assert!(!written.contains("NumberOfDeaths"), "absent fields stay absent");
        assert_eq!(from_str::<Consequence>(&written).unwrap(), consequence);
    }

    #[test]
    fn a_suitability_names_exactly_one_kind_of_passenger_need() {
        let xml = concat!(
            "<Suitabilities><Suitability>",
            "<Suitable>notSuitable</Suitable>",
            "<UserNeed><MobilityNeed>wheelchair</MobilityNeed><NeedRanking>1</NeedRanking></UserNeed>",
            "</Suitability></Suitabilities>",
        );

        let suitabilities: Suitabilities = from_str(xml).unwrap();
        assert_eq!(suitabilities.suitability.len(), 1);
        let need = &suitabilities.suitability[0].user_need;
        assert!(matches!(need.need, UserNeedKind::MobilityNeed(_)));
        assert_eq!(need.need_ranking, Some(1));

        let written = to_string_with_root("Suitabilities", &suitabilities).unwrap();
        assert!(written.contains("<MobilityNeed>wheelchair</MobilityNeed>"));
        assert!(!written.contains("PsychosensoryNeed"));
        assert_eq!(from_str::<Suitabilities>(&written).unwrap(), suitabilities);
    }

    #[test]
    fn one_situation_carries_several_consequences() {
        let xml = concat!(
            "<Consequences>",
            "<Consequence><Condition>cancelled</Condition></Consequence>",
            "<Consequence><Condition>diverted</Condition>",
            "<Blocking><JourneyPlanner>true</JourneyPlanner></Blocking></Consequence>",
            "</Consequences>",
        );

        let consequences: PtConsequences = from_str(xml).unwrap();
        assert_eq!(consequences.consequence.len(), 2);
        assert!(consequences.consequence[0].blocking.is_none());
        assert!(consequences.consequence[1].blocking.is_some());

        let written = to_string_with_root("Consequences", &consequences).unwrap();
        assert_eq!(written.matches("<Consequence>").count(), 2);
        assert_eq!(from_str::<PtConsequences>(&written).unwrap(), consequences);
    }
}
