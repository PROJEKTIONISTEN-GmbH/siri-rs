//! The situation record: an incident or a planned disruption and everything known
//! about it.
//!
//! A situation is the unit of exchange of SIRI-SX. It carries its own identity and
//! version, the source the information came from, when it applies, how it is
//! classified, the text a passenger reads, and — through the sibling modules — which
//! parts of the network it affects, what it does to the service, and how it may be
//! published.
//!
//! Two flavours exist: [`PtSituationElement`] for public transport and
//! [`RoadSituationElement`] for the road network, which adds a DATEX II record.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::{
    AlertCause, Audience, DayType, EndTimePrecision, EndTimeStatus, ImageContent, InformationStatus,
    LinkContent, ProbabilityOfOccurrence, PublicEventType, QualityIndex, RelatedTo, ReportType,
    ScopeType, Sensitivity, Severity, SituationSourceType, SourceType, VerificationStatus,
    WorkflowStatus,
};
use crate::model::ControlActionRef;
use crate::sx::action::Actions;
use crate::sx::affects::AffectsScope;
use crate::sx::consequence::PtConsequences;
use crate::types::{CountryRef, DefaultedText, Extensions, NaturalLanguageString, ParticipantRef};

siri_ref! {
    /// Identifies a situation within the participant that raised it.
    ///
    /// The number stays the same across every update to the situation; the update's
    /// `Version` distinguishes the revisions.
    SituationNumber;
}

/// An incident or planned disruption affecting public transport.
///
/// The record is self-contained: identity, provenance, validity, classification,
/// passenger text and the affected network all travel together, so a consumer can
/// act on one element without holding earlier messages. An update repeats the
/// situation number with a higher version and replaces the values it restates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PtSituationElement {
    /// When the situation record was created.
    #[serde(rename = "CreationTime")]
    pub creation_time: DateTime<FixedOffset>,
    /// Country of the participant that raised the situation, giving the namespace
    /// its participant reference is unique in.
    #[serde(rename = "CountryRef", default, skip_serializing_if = "Option::is_none")]
    pub country_ref: Option<CountryRef>,
    /// Participant that raised the situation, giving the namespace the situation
    /// number is unique in. May be supplied from context.
    #[serde(rename = "ParticipantRef", default, skip_serializing_if = "Option::is_none")]
    pub participant_ref: Option<ParticipantRef>,
    /// Identifier of the situation within its participant, excluding the version.
    #[serde(rename = "SituationNumber")]
    pub situation_number: SituationNumber,
    /// Country of the participant that raised this particular update.
    #[serde(rename = "UpdateCountryRef", default, skip_serializing_if = "Option::is_none")]
    pub update_country_ref: Option<CountryRef>,
    /// Participant that raised this particular update, if not the originator.
    #[serde(rename = "UpdateParticipantRef", default, skip_serializing_if = "Option::is_none")]
    pub update_participant_ref: Option<ParticipantRef>,
    /// Revision of the situation, increasing monotonically within the situation
    /// number. Absent on the original record.
    #[serde(rename = "Version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,
    /// Links to other situations, e.g. the one that caused this one.
    #[serde(rename = "References", default, skip_serializing_if = "Option::is_none")]
    pub references: Option<References>,
    /// Where the information came from, and how to go back to it.
    #[serde(rename = "Source")]
    pub source: SituationSource,
    /// When the record was frozen. A versioned record accepts no further change.
    #[serde(rename = "VersionedAtTime", default, skip_serializing_if = "Option::is_none")]
    pub versioned_at_time: Option<DateTime<FixedOffset>>,
    /// Whether the report has been confirmed by a second party.
    #[serde(rename = "Verification", default, skip_serializing_if = "Option::is_none")]
    pub verification: Option<VerificationStatus>,
    /// How far the situation has moved through the editorial workflow, from draft
    /// to closed.
    #[serde(rename = "Progress", default, skip_serializing_if = "Option::is_none")]
    pub progress: Option<WorkflowStatus>,
    /// How much the producer trusts the data.
    #[serde(rename = "QualityIndex", default, skip_serializing_if = "Option::is_none")]
    pub quality_index: Option<QualityIndex>,
    /// Whether the situation is real or part of an exercise or test.
    #[serde(rename = "Reality", default, skip_serializing_if = "Option::is_none")]
    pub reality: Option<InformationStatus>,
    /// How likely a situation that has not yet happened is to happen.
    #[serde(rename = "Likelihood", default, skip_serializing_if = "Option::is_none")]
    pub likelihood: Option<ProbabilityOfOccurrence>,
    /// Producer-defined publishing sub-states the situation has been assigned to.
    #[serde(rename = "Publication", default, skip_serializing_if = "Vec::is_empty")]
    pub publication: Vec<String>,
    /// The periods the situation applies over, at least one.
    #[serde(rename = "ValidityPeriod")]
    pub validity_period: Vec<HalfOpenTimestampOutputRange>,
    /// Day types the situation recurs on within its validity periods, e.g. Sundays
    /// only. Absent means it applies throughout.
    #[serde(rename = "Repetitions", default, skip_serializing_if = "Option::is_none")]
    pub repetitions: Option<SituationRepetitions>,
    /// When the situation may be shown to its audience, if that differs from when
    /// it applies.
    #[serde(rename = "PublicationWindow", default, skip_serializing_if = "Vec::is_empty")]
    pub publication_window: Vec<HalfOpenTimestampOutputRange>,
    /// What caused the situation, as a TPEG alert cause. The current way of stating
    /// a reason; see [`PtSituationElement::reason`].
    #[serde(rename = "AlertCause", default, skip_serializing_if = "Option::is_none")]
    pub alert_cause: Option<AlertCause>,
    /// Superseded alternative to `alert_cause`: the cause is not known.
    #[serde(rename = "UnknownReason", default, skip_serializing_if = "Option::is_none")]
    pub unknown_reason: Option<String>,
    /// Superseded alternative to `alert_cause`: a cause outside the other families.
    #[serde(rename = "MiscellaneousReason", default, skip_serializing_if = "Option::is_none")]
    pub miscellaneous_reason: Option<String>,
    /// Superseded alternative to `alert_cause`: a cause to do with staff.
    #[serde(rename = "PersonnelReason", default, skip_serializing_if = "Option::is_none")]
    pub personnel_reason: Option<String>,
    /// Superseded alternative to `alert_cause`: a cause to do with equipment.
    #[serde(rename = "EquipmentReason", default, skip_serializing_if = "Option::is_none")]
    pub equipment_reason: Option<String>,
    /// Superseded alternative to `alert_cause`: a cause to do with the weather or
    /// the terrain.
    #[serde(rename = "EnvironmentReason", default, skip_serializing_if = "Option::is_none")]
    pub environment_reason: Option<String>,
    /// Superseded alternative to `alert_cause`: a cause that does not fit the code
    /// list.
    #[serde(rename = "UndefinedReason", default, skip_serializing_if = "Option::is_none")]
    pub undefined_reason: Option<String>,
    /// Superseded refinement of `miscellaneous_reason`.
    #[serde(rename = "MiscellaneousSubReason", default, skip_serializing_if = "Option::is_none")]
    pub miscellaneous_sub_reason: Option<String>,
    /// Superseded refinement of `personnel_reason`.
    #[serde(rename = "PersonnelSubReason", default, skip_serializing_if = "Option::is_none")]
    pub personnel_sub_reason: Option<String>,
    /// Superseded refinement of `equipment_reason`.
    #[serde(rename = "EquipmentSubReason", default, skip_serializing_if = "Option::is_none")]
    pub equipment_sub_reason: Option<String>,
    /// Superseded refinement of `environment_reason`.
    #[serde(rename = "EnvironmentSubReason", default, skip_serializing_if = "Option::is_none")]
    pub environment_sub_reason: Option<String>,
    /// The kind of public event behind the situation, e.g. a football match, when
    /// an event rather than a fault is the cause.
    #[serde(rename = "PublicEventReason", default, skip_serializing_if = "Option::is_none")]
    pub public_event_reason: Option<PublicEventType>,
    /// A few words naming the reason, for use when no coded reason fits. Not a
    /// complete message: it is meant to be dropped into a generated one.
    #[serde(rename = "ReasonName", default, skip_serializing_if = "Vec::is_empty")]
    pub reason_name: Vec<NaturalLanguageString>,
    /// A control action that is itself the reason for the situation, rather than a
    /// response to it.
    #[serde(rename = "ControlActionRef", default, skip_serializing_if = "Option::is_none")]
    pub control_action_ref: Option<ControlActionRef>,
    /// How badly the situation disrupts travel.
    #[serde(rename = "Severity", default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<Severity>,
    /// Producer-defined ranking, 1 being the most important.
    #[serde(rename = "Priority", default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<u64>,
    /// How confidential the situation is.
    #[serde(rename = "Sensitivity", default, skip_serializing_if = "Option::is_none")]
    pub sensitivity: Option<Sensitivity>,
    /// Who the situation is written for.
    #[serde(rename = "Audience", default, skip_serializing_if = "Option::is_none")]
    pub audience: Option<Audience>,
    /// The kind of thing the situation is about, e.g. a stop place or a whole
    /// network.
    #[serde(rename = "ScopeType", default, skip_serializing_if = "Option::is_none")]
    pub scope_type: Option<ScopeType>,
    /// The kind of report, e.g. an incident or general information.
    #[serde(rename = "ReportType", default, skip_serializing_if = "Option::is_none")]
    pub report_type: Option<ReportType>,
    /// Whether the situation was planned, such as engineering works, rather than
    /// unexpected.
    #[serde(rename = "Planned", default, skip_serializing_if = "Option::is_none")]
    pub planned: Option<bool>,
    /// Application-specific classifiers, as whitespace-separated tokens.
    #[serde(rename = "Keywords", default, skip_serializing_if = "Option::is_none")]
    pub keywords: Option<String>,
    /// Further reasons beyond the main one.
    #[serde(rename = "SecondaryReasons", default, skip_serializing_if = "Option::is_none")]
    pub secondary_reasons: Option<SecondaryReasons>,
    /// Language the texts below are written in unless they say otherwise.
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
    /// A note for staff only, never shown to passengers.
    #[serde(rename = "Internal", default, skip_serializing_if = "Option::is_none")]
    pub internal: Option<DefaultedText>,
    /// Pictures illustrating the situation, e.g. a diversion map.
    #[serde(rename = "Images", default, skip_serializing_if = "Option::is_none")]
    pub images: Option<Images>,
    /// Links to pages carrying more about the situation.
    #[serde(rename = "InfoLinks", default, skip_serializing_if = "Option::is_none")]
    pub info_links: Option<InfoLinks>,
    /// The parts of the network the situation applies to. Absent means the scope is
    /// taken from the delivery's context.
    #[serde(rename = "Affects", default, skip_serializing_if = "Option::is_none")]
    pub affects: Option<AffectsScope>,
    /// What the situation does to the service, one entry per distinct effect.
    #[serde(rename = "Consequences", default, skip_serializing_if = "Option::is_none")]
    pub consequences: Option<PtConsequences>,
    /// Where and how the situation may be published. Any actions given replace
    /// those from context entirely.
    #[serde(rename = "PublishingActions", default, skip_serializing_if = "Option::is_none")]
    pub publishing_actions: Option<Actions>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl PtSituationElement {
    /// A situation with the elements the schema requires: when it was raised, what
    /// identifies it, where it came from, when it applies and what caused it.
    ///
    /// Everything else is optional and set on the returned value.
    pub fn new(
        creation_time: DateTime<FixedOffset>,
        situation_number: impl Into<SituationNumber>,
        source: SituationSource,
        validity_period: HalfOpenTimestampOutputRange,
        alert_cause: AlertCause,
    ) -> Self {
        Self {
            creation_time,
            country_ref: None,
            participant_ref: None,
            situation_number: situation_number.into(),
            update_country_ref: None,
            update_participant_ref: None,
            version: None,
            references: None,
            source,
            versioned_at_time: None,
            verification: None,
            progress: None,
            quality_index: None,
            reality: None,
            likelihood: None,
            publication: Vec::new(),
            validity_period: vec![validity_period],
            repetitions: None,
            publication_window: Vec::new(),
            alert_cause: Some(alert_cause),
            unknown_reason: None,
            miscellaneous_reason: None,
            personnel_reason: None,
            equipment_reason: None,
            environment_reason: None,
            undefined_reason: None,
            miscellaneous_sub_reason: None,
            personnel_sub_reason: None,
            equipment_sub_reason: None,
            environment_sub_reason: None,
            public_event_reason: None,
            reason_name: Vec::new(),
            control_action_ref: None,
            severity: None,
            priority: None,
            sensitivity: None,
            audience: None,
            scope_type: None,
            report_type: None,
            planned: None,
            keywords: None,
            secondary_reasons: None,
            language: None,
            summary: Vec::new(),
            description: Vec::new(),
            detail: Vec::new(),
            advice: Vec::new(),
            internal: None,
            images: None,
            info_links: None,
            affects: None,
            consequences: None,
            publishing_actions: None,
            extensions: None,
        }
    }
}

/// An incident or planned disruption affecting the road network.
///
/// The record repeats the public transport situation's structure and adds a DATEX
/// II situation record, so a road authority's feed can be carried through SIRI
/// without translating its native description.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoadSituationElement {
    /// When the situation record was created.
    #[serde(rename = "CreationTime")]
    pub creation_time: DateTime<FixedOffset>,
    /// Country of the participant that raised the situation.
    #[serde(rename = "CountryRef", default, skip_serializing_if = "Option::is_none")]
    pub country_ref: Option<CountryRef>,
    /// Participant that raised the situation. May be supplied from context.
    #[serde(rename = "ParticipantRef", default, skip_serializing_if = "Option::is_none")]
    pub participant_ref: Option<ParticipantRef>,
    /// Identifier of the situation within its participant, excluding the version.
    #[serde(rename = "SituationNumber")]
    pub situation_number: SituationNumber,
    /// Country of the participant that raised this particular update.
    #[serde(rename = "UpdateCountryRef", default, skip_serializing_if = "Option::is_none")]
    pub update_country_ref: Option<CountryRef>,
    /// Participant that raised this particular update, if not the originator.
    #[serde(rename = "UpdateParticipantRef", default, skip_serializing_if = "Option::is_none")]
    pub update_participant_ref: Option<ParticipantRef>,
    /// Revision of the situation, increasing monotonically within the situation
    /// number.
    #[serde(rename = "Version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,
    /// Links to other situations, e.g. the one that caused this one.
    #[serde(rename = "References", default, skip_serializing_if = "Option::is_none")]
    pub references: Option<References>,
    /// Where the information came from, and how to go back to it.
    #[serde(rename = "Source")]
    pub source: SituationSource,
    /// When the record was frozen. A versioned record accepts no further change.
    #[serde(rename = "VersionedAtTime", default, skip_serializing_if = "Option::is_none")]
    pub versioned_at_time: Option<DateTime<FixedOffset>>,
    /// Whether the report has been confirmed by a second party.
    #[serde(rename = "Verification", default, skip_serializing_if = "Option::is_none")]
    pub verification: Option<VerificationStatus>,
    /// How far the situation has moved through the editorial workflow.
    #[serde(rename = "Progress", default, skip_serializing_if = "Option::is_none")]
    pub progress: Option<WorkflowStatus>,
    /// How much the producer trusts the data.
    #[serde(rename = "QualityIndex", default, skip_serializing_if = "Option::is_none")]
    pub quality_index: Option<QualityIndex>,
    /// Whether the situation is real or part of an exercise or test.
    #[serde(rename = "Reality", default, skip_serializing_if = "Option::is_none")]
    pub reality: Option<InformationStatus>,
    /// How likely a situation that has not yet happened is to happen.
    #[serde(rename = "Likelihood", default, skip_serializing_if = "Option::is_none")]
    pub likelihood: Option<ProbabilityOfOccurrence>,
    /// Producer-defined publishing sub-states the situation has been assigned to.
    #[serde(rename = "Publication", default, skip_serializing_if = "Vec::is_empty")]
    pub publication: Vec<String>,
    /// The periods the situation applies over, at least one.
    #[serde(rename = "ValidityPeriod")]
    pub validity_period: Vec<HalfOpenTimestampOutputRange>,
    /// Day types the situation recurs on within its validity periods.
    #[serde(rename = "Repetitions", default, skip_serializing_if = "Option::is_none")]
    pub repetitions: Option<SituationRepetitions>,
    /// When the situation may be shown, if that differs from when it applies.
    #[serde(rename = "PublicationWindow", default, skip_serializing_if = "Vec::is_empty")]
    pub publication_window: Vec<HalfOpenTimestampOutputRange>,
    /// What caused the situation, as a TPEG alert cause; see
    /// [`RoadSituationElement::reason`].
    #[serde(rename = "AlertCause", default, skip_serializing_if = "Option::is_none")]
    pub alert_cause: Option<AlertCause>,
    /// Superseded alternative to `alert_cause`: the cause is not known.
    #[serde(rename = "UnknownReason", default, skip_serializing_if = "Option::is_none")]
    pub unknown_reason: Option<String>,
    /// Superseded alternative to `alert_cause`: a cause outside the other families.
    #[serde(rename = "MiscellaneousReason", default, skip_serializing_if = "Option::is_none")]
    pub miscellaneous_reason: Option<String>,
    /// Superseded alternative to `alert_cause`: a cause to do with staff.
    #[serde(rename = "PersonnelReason", default, skip_serializing_if = "Option::is_none")]
    pub personnel_reason: Option<String>,
    /// Superseded alternative to `alert_cause`: a cause to do with equipment.
    #[serde(rename = "EquipmentReason", default, skip_serializing_if = "Option::is_none")]
    pub equipment_reason: Option<String>,
    /// Superseded alternative to `alert_cause`: a cause to do with the weather or
    /// the terrain.
    #[serde(rename = "EnvironmentReason", default, skip_serializing_if = "Option::is_none")]
    pub environment_reason: Option<String>,
    /// Superseded alternative to `alert_cause`: a cause that does not fit the code
    /// list.
    #[serde(rename = "UndefinedReason", default, skip_serializing_if = "Option::is_none")]
    pub undefined_reason: Option<String>,
    /// Superseded refinement of `miscellaneous_reason`.
    #[serde(rename = "MiscellaneousSubReason", default, skip_serializing_if = "Option::is_none")]
    pub miscellaneous_sub_reason: Option<String>,
    /// Superseded refinement of `personnel_reason`.
    #[serde(rename = "PersonnelSubReason", default, skip_serializing_if = "Option::is_none")]
    pub personnel_sub_reason: Option<String>,
    /// Superseded refinement of `equipment_reason`.
    #[serde(rename = "EquipmentSubReason", default, skip_serializing_if = "Option::is_none")]
    pub equipment_sub_reason: Option<String>,
    /// Superseded refinement of `environment_reason`.
    #[serde(rename = "EnvironmentSubReason", default, skip_serializing_if = "Option::is_none")]
    pub environment_sub_reason: Option<String>,
    /// The kind of public event behind the situation, e.g. a marathon.
    #[serde(rename = "PublicEventReason", default, skip_serializing_if = "Option::is_none")]
    pub public_event_reason: Option<PublicEventType>,
    /// A few words naming the reason, for use when no coded reason fits.
    #[serde(rename = "ReasonName", default, skip_serializing_if = "Vec::is_empty")]
    pub reason_name: Vec<NaturalLanguageString>,
    /// A control action that is itself the reason for the situation.
    #[serde(rename = "ControlActionRef", default, skip_serializing_if = "Option::is_none")]
    pub control_action_ref: Option<ControlActionRef>,
    /// How badly the situation disrupts travel.
    #[serde(rename = "Severity", default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<Severity>,
    /// Producer-defined ranking, 1 being the most important.
    #[serde(rename = "Priority", default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<u64>,
    /// How confidential the situation is.
    #[serde(rename = "Sensitivity", default, skip_serializing_if = "Option::is_none")]
    pub sensitivity: Option<Sensitivity>,
    /// Who the situation is written for.
    #[serde(rename = "Audience", default, skip_serializing_if = "Option::is_none")]
    pub audience: Option<Audience>,
    /// The kind of thing the situation is about, e.g. a road or an area.
    #[serde(rename = "ScopeType", default, skip_serializing_if = "Option::is_none")]
    pub scope_type: Option<ScopeType>,
    /// The kind of report, e.g. an incident or general information.
    #[serde(rename = "ReportType", default, skip_serializing_if = "Option::is_none")]
    pub report_type: Option<ReportType>,
    /// Whether the situation was planned, such as roadworks, rather than
    /// unexpected.
    #[serde(rename = "Planned", default, skip_serializing_if = "Option::is_none")]
    pub planned: Option<bool>,
    /// Application-specific classifiers, as whitespace-separated tokens.
    #[serde(rename = "Keywords", default, skip_serializing_if = "Option::is_none")]
    pub keywords: Option<String>,
    /// Further reasons beyond the main one.
    #[serde(rename = "SecondaryReasons", default, skip_serializing_if = "Option::is_none")]
    pub secondary_reasons: Option<SecondaryReasons>,
    /// Language the texts below are written in unless they say otherwise.
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
    /// What travellers are advised to do.
    #[serde(rename = "Advice", default, skip_serializing_if = "Vec::is_empty")]
    pub advice: Vec<DefaultedText>,
    /// A note for staff only, never shown to the public.
    #[serde(rename = "Internal", default, skip_serializing_if = "Option::is_none")]
    pub internal: Option<DefaultedText>,
    /// Pictures illustrating the situation, e.g. a diversion map.
    #[serde(rename = "Images", default, skip_serializing_if = "Option::is_none")]
    pub images: Option<Images>,
    /// Links to pages carrying more about the situation.
    #[serde(rename = "InfoLinks", default, skip_serializing_if = "Option::is_none")]
    pub info_links: Option<InfoLinks>,
    /// The parts of the network the situation applies to.
    #[serde(rename = "Affects", default, skip_serializing_if = "Option::is_none")]
    pub affects: Option<AffectsScope>,
    /// What the situation does to the service. The schema reuses the public
    /// transport consequences here, so a road works entry can state its knock-on
    /// effect on buses.
    #[serde(rename = "Consequences", default, skip_serializing_if = "Option::is_none")]
    pub consequences: Option<PtConsequences>,
    /// Where and how the situation may be published.
    #[serde(rename = "PublishingActions", default, skip_serializing_if = "Option::is_none")]
    pub publishing_actions: Option<Actions>,
    /// The situation as the road authority described it, in DATEX II terms.
    #[serde(rename = "SituationRecord", default, skip_serializing_if = "Option::is_none")]
    pub situation_record: Option<Datex2SituationRecord>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl RoadSituationElement {
    /// A road situation with the elements the schema requires: when it was raised,
    /// what identifies it, where it came from, when it applies and what caused it.
    ///
    /// Everything else is optional and set on the returned value.
    pub fn new(
        creation_time: DateTime<FixedOffset>,
        situation_number: impl Into<SituationNumber>,
        source: SituationSource,
        validity_period: HalfOpenTimestampOutputRange,
        alert_cause: AlertCause,
    ) -> Self {
        Self {
            creation_time,
            country_ref: None,
            participant_ref: None,
            situation_number: situation_number.into(),
            update_country_ref: None,
            update_participant_ref: None,
            version: None,
            references: None,
            source,
            versioned_at_time: None,
            verification: None,
            progress: None,
            quality_index: None,
            reality: None,
            likelihood: None,
            publication: Vec::new(),
            validity_period: vec![validity_period],
            repetitions: None,
            publication_window: Vec::new(),
            alert_cause: Some(alert_cause),
            unknown_reason: None,
            miscellaneous_reason: None,
            personnel_reason: None,
            equipment_reason: None,
            environment_reason: None,
            undefined_reason: None,
            miscellaneous_sub_reason: None,
            personnel_sub_reason: None,
            equipment_sub_reason: None,
            environment_sub_reason: None,
            public_event_reason: None,
            reason_name: Vec::new(),
            control_action_ref: None,
            severity: None,
            priority: None,
            sensitivity: None,
            audience: None,
            scope_type: None,
            report_type: None,
            planned: None,
            keywords: None,
            secondary_reasons: None,
            language: None,
            summary: Vec::new(),
            description: Vec::new(),
            detail: Vec::new(),
            advice: Vec::new(),
            internal: None,
            images: None,
            info_links: None,
            affects: None,
            consequences: None,
            publishing_actions: None,
            situation_record: None,
            extensions: None,
        }
    }
}

/// A road situation as the DATEX II standard describes it.
///
/// The element's content model belongs to DATEX II rather than SIRI, so this
/// release round-trips the element and any character data it holds without
/// interpreting it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Datex2SituationRecord {
    /// Character data directly inside the element, if any.
    #[serde(rename = "$text", default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// The situations this one is related to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct References {
    /// Each related situation, with the nature of the relationship.
    #[serde(rename = "RelatedToRef")]
    pub related_to_ref: Vec<RelatedSituation>,
}

impl References {
    /// References to the given situations.
    pub fn new(related_to_ref: Vec<RelatedSituation>) -> Self {
        Self { related_to_ref }
    }
}

/// A pointer to another situation and what it has to do with this one.
///
/// A plain update — same situation number, higher version — is implied and need not
/// be stated here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelatedSituation {
    /// When the association was made.
    #[serde(rename = "CreationTime")]
    pub creation_time: DateTime<FixedOffset>,
    /// Country of the participant that raised the referenced situation.
    #[serde(rename = "CountryRef", default, skip_serializing_if = "Option::is_none")]
    pub country_ref: Option<CountryRef>,
    /// Participant that raised the referenced situation. May be supplied from
    /// context.
    #[serde(rename = "ParticipantRef", default, skip_serializing_if = "Option::is_none")]
    pub participant_ref: Option<ParticipantRef>,
    /// Identifier of the referenced situation within its participant.
    #[serde(rename = "SituationNumber")]
    pub situation_number: SituationNumber,
    /// Country of the participant that raised the referenced update.
    #[serde(rename = "UpdateCountryRef", default, skip_serializing_if = "Option::is_none")]
    pub update_country_ref: Option<CountryRef>,
    /// Participant that raised the referenced update.
    #[serde(rename = "UpdateParticipantRef", default, skip_serializing_if = "Option::is_none")]
    pub update_participant_ref: Option<ParticipantRef>,
    /// Revision of the referenced situation. Absent points at the base situation.
    #[serde(rename = "Version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,
    /// The referenced situation as a single identifier from another system.
    #[serde(rename = "ExternalReference", default, skip_serializing_if = "Option::is_none")]
    pub external_reference: Option<String>,
    /// How the referenced situation relates to this one, e.g. as its cause.
    #[serde(rename = "RelatedAs", default, skip_serializing_if = "Option::is_none")]
    pub related_as: Option<RelatedTo>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl RelatedSituation {
    /// A relationship to the situation identified by `situation_number`.
    pub fn new(
        creation_time: DateTime<FixedOffset>,
        situation_number: impl Into<SituationNumber>,
        related_as: RelatedTo,
    ) -> Self {
        Self {
            creation_time,
            country_ref: None,
            participant_ref: None,
            situation_number: situation_number.into(),
            update_country_ref: None,
            update_participant_ref: None,
            version: None,
            external_reference: None,
            related_as: Some(related_as),
            extensions: None,
        }
    }
}

/// Where the information in a situation came from.
///
/// Enough is recorded to go back to the informant: an agent taking a phone call
/// keeps the number, an automated feed identifies itself, and both note when the
/// message arrived if that differs from when the record was written.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SituationSource {
    /// Country the source is in.
    #[serde(rename = "Country", default, skip_serializing_if = "Option::is_none")]
    pub country: Option<CountryRef>,
    /// The channel the information arrived by, e.g. a phone call or a data feed.
    #[serde(rename = "SourceType")]
    pub source_type: SituationSourceType,
    /// Email address to reach the informant at.
    #[serde(rename = "Email", default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Telephone number to reach the informant at.
    #[serde(rename = "Phone", default, skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    /// Fax number of the informant.
    #[serde(rename = "Fax", default, skip_serializing_if = "Option::is_none")]
    pub fax: Option<String>,
    /// Page the information was taken from.
    #[serde(rename = "Web", default, skip_serializing_if = "Option::is_none")]
    pub web: Option<String>,
    /// Any other way of reaching the informant.
    #[serde(rename = "Other", default, skip_serializing_if = "Option::is_none")]
    pub other: Option<String>,
    /// How the information was gathered, e.g. by a camera or a patrol.
    #[serde(rename = "SourceMethod", default, skip_serializing_if = "Option::is_none")]
    pub source_method: Option<SourceType>,
    /// The capture client user who entered the situation, for internal exchange.
    #[serde(rename = "AgentReference", default, skip_serializing_if = "Option::is_none")]
    pub agent_reference: Option<String>,
    /// Name of the source.
    #[serde(rename = "Name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<NaturalLanguageString>,
    /// Job title of the informant.
    #[serde(rename = "SourceRole", default, skip_serializing_if = "Option::is_none")]
    pub source_role: Option<String>,
    /// When the message was communicated, if not when the record was created.
    #[serde(rename = "TimeOfCommunication", default, skip_serializing_if = "Option::is_none")]
    pub time_of_communication: Option<DateTime<FixedOffset>>,
    /// The situation's identifier in the external system it came from.
    #[serde(rename = "ExternalCode", default, skip_serializing_if = "Option::is_none")]
    pub external_code: Option<String>,
    /// An attachment holding further information about the situation.
    #[serde(rename = "SourceFile", default, skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl SituationSource {
    /// A source that states only the channel the information arrived by.
    pub fn new(source_type: SituationSourceType) -> Self {
        Self {
            country: None,
            source_type,
            email: None,
            phone: None,
            fax: None,
            web: None,
            other: None,
            source_method: None,
            agent_reference: None,
            name: None,
            source_role: None,
            time_of_communication: None,
            external_code: None,
            source_file: None,
            extensions: None,
        }
    }
}

/// A period that starts at a known instant and may not have a stated end.
///
/// This is the form a producer publishes: when the end is unknown, the status says
/// whether the disruption is expected to be a short or a long one, which is what a
/// passenger information system needs in order to word the message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HalfOpenTimestampOutputRange {
    /// The inclusive start of the period.
    #[serde(rename = "StartTime")]
    pub start_time: DateTime<FixedOffset>,
    /// The inclusive end of the period, if it is known.
    #[serde(rename = "EndTime", default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime<FixedOffset>>,
    /// How to read an absent end: as short term, long term, or simply unknown.
    #[serde(rename = "EndTimeStatus", default, skip_serializing_if = "Option::is_none")]
    pub end_time_status: Option<EndTimeStatus>,
}

impl HalfOpenTimestampOutputRange {
    /// A period whose end is not yet known.
    pub fn starting_at(start_time: DateTime<FixedOffset>) -> Self {
        Self {
            start_time,
            end_time: None,
            end_time_status: None,
        }
    }

    /// A period between two instants, both inclusive.
    pub fn between(start_time: DateTime<FixedOffset>, end_time: DateTime<FixedOffset>) -> Self {
        Self {
            start_time,
            end_time: Some(end_time),
            end_time_status: None,
        }
    }
}

/// A period that starts at a known instant and may not have a stated end.
///
/// This is the form a consumer sends when asking for data: the precision says how
/// exactly the end is meant, so that a request ending "today" is not read as ending
/// at midnight exactly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HalfOpenTimestampInputRange {
    /// The inclusive start of the period.
    #[serde(rename = "StartTime")]
    pub start_time: DateTime<FixedOffset>,
    /// The inclusive end of the period, if it is bounded.
    #[serde(rename = "EndTime", default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime<FixedOffset>>,
    /// How exactly the end is to be taken; the default is to the second.
    #[serde(rename = "EndTimePrecision", default, skip_serializing_if = "Option::is_none")]
    pub end_time_precision: Option<EndTimePrecision>,
}

impl HalfOpenTimestampInputRange {
    /// A period whose end is left open.
    pub fn starting_at(start_time: DateTime<FixedOffset>) -> Self {
        Self {
            start_time,
            end_time: None,
            end_time_precision: None,
        }
    }

    /// A period between two instants, both inclusive.
    pub fn between(start_time: DateTime<FixedOffset>, end_time: DateTime<FixedOffset>) -> Self {
        Self {
            start_time,
            end_time: Some(end_time),
            end_time_precision: None,
        }
    }
}

/// The day types a situation recurs on within its validity periods.
///
/// A closure that applies only at weekends is one validity period spanning the
/// months plus the weekend day types, rather than one period per weekend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SituationRepetitions {
    /// The day types the situation applies on, at least one.
    #[serde(rename = "DayType", deserialize_with = "crate::xml::token_list::deserialize")]
    pub day_type: Vec<DayType>,
}

impl SituationRepetitions {
    /// A repetition on the given day types.
    pub fn new(day_type: Vec<DayType>) -> Self {
        Self { day_type }
    }
}

/// Reasons beyond the situation's main one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecondaryReasons {
    /// Each further reason, at least one.
    #[serde(rename = "Reason")]
    pub reason: Vec<SituationReason>,
}

impl SecondaryReasons {
    /// Secondary reasons holding the given entries.
    pub fn new(reason: Vec<SituationReason>) -> Self {
        Self { reason }
    }
}

/// One statement of why a situation arose.
///
/// The schema offers the cause as a choice: the current TPEG alert cause, or one of
/// six superseded alternatives kept for older feeds. Only one may be present, so
/// each is an optional field; build one with the constructor for the alternative
/// you mean and read back which is present with [`SituationReason::reason`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SituationReason {
    /// What caused the situation, as a TPEG alert cause.
    #[serde(rename = "AlertCause", default, skip_serializing_if = "Option::is_none")]
    pub alert_cause: Option<AlertCause>,
    /// Superseded alternative to `alert_cause`: the cause is not known.
    #[serde(rename = "UnknownReason", default, skip_serializing_if = "Option::is_none")]
    pub unknown_reason: Option<String>,
    /// Superseded alternative to `alert_cause`: a cause outside the other families.
    #[serde(rename = "MiscellaneousReason", default, skip_serializing_if = "Option::is_none")]
    pub miscellaneous_reason: Option<String>,
    /// Superseded alternative to `alert_cause`: a cause to do with staff.
    #[serde(rename = "PersonnelReason", default, skip_serializing_if = "Option::is_none")]
    pub personnel_reason: Option<String>,
    /// Superseded alternative to `alert_cause`: a cause to do with equipment.
    #[serde(rename = "EquipmentReason", default, skip_serializing_if = "Option::is_none")]
    pub equipment_reason: Option<String>,
    /// Superseded alternative to `alert_cause`: a cause to do with the weather or
    /// the terrain.
    #[serde(rename = "EnvironmentReason", default, skip_serializing_if = "Option::is_none")]
    pub environment_reason: Option<String>,
    /// Superseded alternative to `alert_cause`: a cause that does not fit the code
    /// list.
    #[serde(rename = "UndefinedReason", default, skip_serializing_if = "Option::is_none")]
    pub undefined_reason: Option<String>,
    /// Superseded refinement of `miscellaneous_reason`.
    #[serde(rename = "MiscellaneousSubReason", default, skip_serializing_if = "Option::is_none")]
    pub miscellaneous_sub_reason: Option<String>,
    /// Superseded refinement of `personnel_reason`.
    #[serde(rename = "PersonnelSubReason", default, skip_serializing_if = "Option::is_none")]
    pub personnel_sub_reason: Option<String>,
    /// Superseded refinement of `equipment_reason`.
    #[serde(rename = "EquipmentSubReason", default, skip_serializing_if = "Option::is_none")]
    pub equipment_sub_reason: Option<String>,
    /// Superseded refinement of `environment_reason`.
    #[serde(rename = "EnvironmentSubReason", default, skip_serializing_if = "Option::is_none")]
    pub environment_sub_reason: Option<String>,
    /// The kind of public event behind the situation.
    #[serde(rename = "PublicEventReason", default, skip_serializing_if = "Option::is_none")]
    pub public_event_reason: Option<PublicEventType>,
    /// A few words naming the reason, for use when no coded reason fits.
    #[serde(rename = "ReasonName", default, skip_serializing_if = "Vec::is_empty")]
    pub reason_name: Vec<NaturalLanguageString>,
    /// A control action that is itself the reason for the situation.
    #[serde(rename = "ControlActionRef", default, skip_serializing_if = "Option::is_none")]
    pub control_action_ref: Option<ControlActionRef>,
}

impl SituationReason {
    /// A reason given as a TPEG alert cause.
    pub fn alert_cause(alert_cause: AlertCause) -> Self {
        Self {
            alert_cause: Some(alert_cause),
            ..Self::default()
        }
    }

    /// A reason stating that the cause is not known.
    pub fn unknown_reason(unknown_reason: impl Into<String>) -> Self {
        Self {
            unknown_reason: Some(unknown_reason.into()),
            ..Self::default()
        }
    }

    /// A reason from the superseded miscellaneous code list.
    pub fn miscellaneous_reason(miscellaneous_reason: impl Into<String>) -> Self {
        Self {
            miscellaneous_reason: Some(miscellaneous_reason.into()),
            ..Self::default()
        }
    }

    /// A reason from the superseded personnel code list.
    pub fn personnel_reason(personnel_reason: impl Into<String>) -> Self {
        Self {
            personnel_reason: Some(personnel_reason.into()),
            ..Self::default()
        }
    }

    /// A reason from the superseded equipment code list.
    pub fn equipment_reason(equipment_reason: impl Into<String>) -> Self {
        Self {
            equipment_reason: Some(equipment_reason.into()),
            ..Self::default()
        }
    }

    /// A reason from the superseded environment code list.
    pub fn environment_reason(environment_reason: impl Into<String>) -> Self {
        Self {
            environment_reason: Some(environment_reason.into()),
            ..Self::default()
        }
    }

    /// A reason that does not fit any of the code lists.
    pub fn undefined_reason(undefined_reason: impl Into<String>) -> Self {
        Self {
            undefined_reason: Some(undefined_reason.into()),
            ..Self::default()
        }
    }
}

/// Which alternative of the schema's reason choice a record carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason<'a> {
    /// A TPEG alert cause, the way a current producer states a reason.
    AlertCause(AlertCause),
    /// A superseded statement that the cause is not known.
    UnknownReason(&'a str),
    /// A superseded code for a cause outside the other families.
    MiscellaneousReason(&'a str),
    /// A superseded code for a cause to do with staff.
    PersonnelReason(&'a str),
    /// A superseded code for a cause to do with equipment.
    EquipmentReason(&'a str),
    /// A superseded code for a cause to do with the weather or the terrain.
    EnvironmentReason(&'a str),
    /// A superseded code for a cause that does not fit the code lists.
    UndefinedReason(&'a str),
}

/// Which alternative of the schema's superseded sub-reason choice a record carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubReason<'a> {
    /// A refinement of a miscellaneous reason.
    MiscellaneousSubReason(&'a str),
    /// A refinement of a personnel reason.
    PersonnelSubReason(&'a str),
    /// A refinement of an equipment reason.
    EquipmentSubReason(&'a str),
    /// A refinement of an environment reason.
    EnvironmentSubReason(&'a str),
}

/// Implements the reason accessors on every type that carries the schema's reason
/// group inline.
macro_rules! reason_accessors {
    ($($name:ident),* $(,)?) => {
        $(
            impl $name {
                /// Which alternative of the schema's reason choice this record
                /// carries, or `None` when none is present.
                pub fn reason(&self) -> Option<Reason<'_>> {
                    if let Some(alert_cause) = self.alert_cause {
                        return Some(Reason::AlertCause(alert_cause));
                    }
                    if let Some(value) = self.unknown_reason.as_deref() {
                        return Some(Reason::UnknownReason(value));
                    }
                    if let Some(value) = self.miscellaneous_reason.as_deref() {
                        return Some(Reason::MiscellaneousReason(value));
                    }
                    if let Some(value) = self.personnel_reason.as_deref() {
                        return Some(Reason::PersonnelReason(value));
                    }
                    if let Some(value) = self.equipment_reason.as_deref() {
                        return Some(Reason::EquipmentReason(value));
                    }
                    if let Some(value) = self.environment_reason.as_deref() {
                        return Some(Reason::EnvironmentReason(value));
                    }
                    self.undefined_reason.as_deref().map(Reason::UndefinedReason)
                }

                /// Which alternative of the schema's superseded sub-reason choice
                /// this record carries, or `None` when none is present.
                pub fn sub_reason(&self) -> Option<SubReason<'_>> {
                    if let Some(value) = self.miscellaneous_sub_reason.as_deref() {
                        return Some(SubReason::MiscellaneousSubReason(value));
                    }
                    if let Some(value) = self.personnel_sub_reason.as_deref() {
                        return Some(SubReason::PersonnelSubReason(value));
                    }
                    if let Some(value) = self.equipment_sub_reason.as_deref() {
                        return Some(SubReason::EquipmentSubReason(value));
                    }
                    self.environment_sub_reason
                        .as_deref()
                        .map(SubReason::EnvironmentSubReason)
                }
            }
        )*
    };
}

reason_accessors!(PtSituationElement, RoadSituationElement, SituationReason);

/// Pictures illustrating a situation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Images {
    /// Each picture, at least one.
    #[serde(rename = "Image")]
    pub image: Vec<Image>,
}

impl Images {
    /// A set of pictures.
    pub fn new(image: Vec<Image>) -> Self {
        Self { image }
    }
}

/// A picture, either linked to or carried in the message.
///
/// The schema offers the two as a choice, so both fields are optional. Build one
/// with [`Image::referenced`] or [`Image::embedded`] and read back which form is
/// present with [`Image::source`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Image {
    /// Where the picture can be fetched from.
    #[serde(rename = "ImageRef", default, skip_serializing_if = "Option::is_none")]
    pub image_ref: Option<String>,
    /// The picture itself, base64-encoded.
    #[serde(rename = "ImageBinary", default, skip_serializing_if = "Option::is_none")]
    pub image_binary: Option<String>,
    /// What the picture shows, e.g. a map or a logo.
    #[serde(rename = "ImageContent", default, skip_serializing_if = "Option::is_none")]
    pub image_content: Option<ImageContent>,
}

/// Which of the two ways of carrying a picture an [`Image`] uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageSource<'a> {
    /// A location the picture is fetched from.
    ImageRef(&'a str),
    /// The picture inline, base64-encoded.
    ImageBinary(&'a str),
}

impl Image {
    /// A picture fetched from the given location.
    pub fn referenced(image_ref: impl Into<String>) -> Self {
        Self {
            image_ref: Some(image_ref.into()),
            image_binary: None,
            image_content: None,
        }
    }

    /// A picture carried in the message, base64-encoded.
    pub fn embedded(image_binary: impl Into<String>) -> Self {
        Self {
            image_ref: None,
            image_binary: Some(image_binary.into()),
            image_content: None,
        }
    }

    /// Which alternative of the schema's choice this picture carries, or `None`
    /// when neither is present.
    pub fn source(&self) -> Option<ImageSource<'_>> {
        if let Some(image_ref) = self.image_ref.as_deref() {
            return Some(ImageSource::ImageRef(image_ref));
        }
        self.image_binary.as_deref().map(ImageSource::ImageBinary)
    }
}

/// Links to resources carrying more about a situation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InfoLinks {
    /// Each link, at least one.
    #[serde(rename = "InfoLink")]
    pub info_link: Vec<InfoLink>,
}

impl InfoLinks {
    /// A set of links.
    pub fn new(info_link: Vec<InfoLink>) -> Self {
        Self { info_link }
    }
}

/// A hyperlink offered alongside a situation, with what to show for it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InfoLink {
    /// Where the link points.
    #[serde(rename = "Uri")]
    pub uri: String,
    /// The text to show for the link, one per language offered.
    #[serde(rename = "Label", default, skip_serializing_if = "Vec::is_empty")]
    pub label: Vec<NaturalLanguageString>,
    /// A picture to show for the link.
    #[serde(rename = "Image", default, skip_serializing_if = "Option::is_none")]
    pub image: Option<Image>,
    /// What is behind the link, e.g. a timetable or advice.
    #[serde(rename = "LinkContent", default, skip_serializing_if = "Option::is_none")]
    pub link_content: Option<LinkContent>,
}

impl InfoLink {
    /// A bare link to the given address.
    pub fn new(uri: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            label: Vec::new(),
            image: None,
            link_content: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timestamp(text: &str) -> DateTime<FixedOffset> {
        text.parse().expect("a valid timestamp")
    }

    fn situation() -> PtSituationElement {
        PtSituationElement::new(
            timestamp("2004-12-17T09:30:47-05:00"),
            "TW-01",
            SituationSource::new(SituationSourceType::DirectReport),
            HalfOpenTimestampOutputRange::between(
                timestamp("2004-12-17T10:00:00-05:00"),
                timestamp("2004-12-17T18:00:00-05:00"),
            ),
            AlertCause::ConstructionWork,
        )
    }

    #[test]
    fn a_situation_reports_the_alternative_of_the_reason_choice_it_carries() {
        let situation = situation();
        assert_eq!(
            situation.reason(),
            Some(Reason::AlertCause(AlertCause::ConstructionWork))
        );
        assert_eq!(situation.sub_reason(), None);
        assert_eq!(SituationReason::default().reason(), None);
    }

    #[test]
    fn a_superseded_reason_is_reported_under_its_own_element_name() {
        let mut reason = SituationReason::equipment_reason("liftFailure");
        assert_eq!(reason.reason(), Some(Reason::EquipmentReason("liftFailure")));

        reason.equipment_sub_reason = Some("escalatorFailure".to_owned());
        assert_eq!(
            reason.sub_reason(),
            Some(SubReason::EquipmentSubReason("escalatorFailure"))
        );

        // The alert cause outranks the superseded alternatives when both are given.
        reason.alert_cause = Some(AlertCause::LiftFailure);
        assert_eq!(
            reason.reason(),
            Some(Reason::AlertCause(AlertCause::LiftFailure))
        );
    }

    #[test]
    fn a_picture_reports_whether_it_is_linked_or_carried() {
        let linked = Image::referenced("http://example.org/diversion.png");
        assert_eq!(
            linked.source(),
            Some(ImageSource::ImageRef("http://example.org/diversion.png"))
        );
        assert_eq!(
            Image::embedded("aGVsbG8=").source(),
            Some(ImageSource::ImageBinary("aGVsbG8="))
        );
        assert_eq!(Image::default().source(), None);
    }

    #[test]
    fn a_situation_writes_its_elements_in_schema_order() {
        let mut situation = situation();
        situation.progress = Some(WorkflowStatus::Published);
        situation.summary = vec![DefaultedText::with_lang("EN", "Lift out of service")];

        let xml = quick_xml::se::to_string_with_root("PtSituationElement", &situation)
            .expect("the situation serialises");

        let at = |tag: &str| xml.find(tag).unwrap_or_else(|| panic!("{tag} is written"));
        assert!(at("<CreationTime>") < at("<SituationNumber>"));
        assert!(at("<SituationNumber>") < at("<Source>"));
        assert!(at("<Source>") < at("<Progress>"));
        assert!(at("<Progress>") < at("<ValidityPeriod>"));
        assert!(at("<ValidityPeriod>") < at("<AlertCause>"));
        assert!(at("<AlertCause>") < at("<Summary"));
    }

    #[test]
    fn a_situation_round_trips_through_xml() {
        let mut situation = situation();
        situation.participant_ref = Some(ParticipantRef::new("KUBRICK"));
        situation.version = Some(2);
        situation.severity = Some(Severity::Normal);
        situation.publication = vec!["publicTransport".to_owned()];
        situation.reason_name = vec![NaturalLanguageString::with_lang("EN", "Bridge works")];
        situation.repetitions = Some(SituationRepetitions::new(vec![DayType::Sunday]));
        situation.info_links = Some(InfoLinks::new(vec![InfoLink::new(
            "http://example.org/works",
        )]));
        situation.secondary_reasons = Some(SecondaryReasons::new(vec![
            SituationReason::miscellaneous_reason("congestion"),
        ]));

        let xml = quick_xml::se::to_string_with_root("PtSituationElement", &situation)
            .expect("the situation serialises");
        let parsed: PtSituationElement =
            quick_xml::de::from_str(&xml).expect("the situation parses back");

        assert_eq!(parsed, situation);
        assert_eq!(
            parsed
                .secondary_reasons
                .as_ref()
                .expect("the secondary reasons survive")
                .reason[0]
                .reason(),
            Some(Reason::MiscellaneousReason("congestion"))
        );
    }
}
