//! What lenient reading reports, over every document it can be handed.
//!
//! Three kinds of document: the official examples, which read leniently exactly
//! as they read strictly, with nothing to report; a catalogue of the faults a
//! stream from a producer one does not control really carries, each in a place
//! the schema leaves optional and in one it does not, across the services; and
//! randomly damaged examples, for which the reader never panics and never lets a
//! unit go quietly — what was in the document is what was read plus what was
//! reported.
#![cfg(feature = "lenient")]

mod support;

use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};

use quick_xml::events::Event;
use siri_rs::lenient::{self, Findings};
use siri_rs::Siri;
use support::mutation::{copy, element, emptied, insert_after, insert_before, remove, replace, with_text};

/// The official example at `name` under `tests/fixtures`.
fn fixture(name: &str) -> String {
    let path = support::fixtures_dir().join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// The strict reading of a document that is expected to read.
fn strict(xml: &str) -> Siri {
    siri_rs::from_str(xml).expect("the repaired document reads strictly")
}

/// The lenient reading of a document that is expected to read, with its findings.
fn lenient(xml: &str) -> (Siri, Findings) {
    lenient::from_str(xml).expect("the document reads leniently")
}

const SX: &str = "xml/sx/exx_situationExchange_response.xml";
const ET: &str = "xml/et/ext_estimatedTimetable_response.xml";
const SM: &str = "xml/sm/exs_stopMonitoring_response.xml";
const VM: &str = "xml/vm/exv_vehicleMonitoring_response.xml";
const FM: &str = "xml/fm/exf_facilityMonitoring_response.xml";

#[test]
fn every_document_the_strict_reader_accepts_reads_the_same_with_no_findings() {
    let mut read = 0;
    for fixture in support::fixtures().into_iter().chain(support::derived_fixtures()) {
        if support::root_element(&fixture.xml) != "Siri" {
            continue;
        }
        let expected = siri_rs::from_str::<Siri>(&fixture.xml)
            .unwrap_or_else(|e| panic!("{}: reads strictly: {e}", fixture.name));
        let (actual, findings) =
            lenient::from_str(&fixture.xml).unwrap_or_else(|e| panic!("{}: reads leniently: {e}", fixture.name));
        assert_eq!(actual, expected, "{}", fixture.name);
        assert!(findings.is_empty(), "{}: {findings:?}", fixture.name);
        read += 1;
    }
    assert!(read > 60, "the conformance suite was read: {read} documents");
}

/// One way of breaking, or repairing, a document.
#[derive(Clone, Copy)]
enum Mutation {
    /// The nth element of the name is written empty.
    Empty(&'static str, usize),
    /// The nth element of the name is taken out.
    Remove(&'static str, usize),
    /// The nth element of the name holds the text and nothing else.
    Text(&'static str, usize, &'static str),
    /// The nth element of the name is replaced by other markup.
    Markup(&'static str, usize, &'static str),
}

impl Mutation {
    fn apply(self, xml: &str) -> String {
        match self {
            Mutation::Empty(name, nth) => emptied(xml, name, nth),
            Mutation::Remove(name, nth) => remove(xml, name, nth),
            Mutation::Text(name, nth, text) => with_text(xml, name, nth, text),
            Mutation::Markup(name, nth, markup) => replace(xml, name, nth, markup),
        }
    }
}

/// Where the fault sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Position {
    /// In an element the schema leaves optional: that element alone goes.
    Optional,
    /// In an element the schema demands: an element around it goes — the
    /// smallest the document reads without, up to the unit.
    Mandatory,
}

/// One fault class in one position in one service.
struct Case {
    class: &'static str,
    position: Position,
    fixture: &'static str,
    fault: Mutation,
    /// What to take out of the example to get the document the lenient reading
    /// has to equal.
    repair: Mutation,
    /// The element the finding names.
    path: &'static str,
    /// How the finding's reason begins.
    reason: &'static str,
    /// What the finding says was left out.
    discarded: &'static str,
    /// The unit's name and identifier when a unit was left out.
    unit: Option<(&'static str, Option<&'static str>)>,
}

const SX_SITUATION: &str = "ServiceDelivery.SituationExchangeDelivery[0].Situations.PtSituationElement[0]";
const ET_JOURNEY: &str =
    "ServiceDelivery.EstimatedTimetableDelivery[0].EstimatedJourneyVersionFrame[0].EstimatedVehicleJourney";

const CATALOGUE: &[Case] = &[
    // An empty timestamp.
    Case {
        class: "empty timestamp",
        position: Position::Optional,
        fixture: SM,
        fault: Mutation::Empty("AimedArrivalTime", 0),
        repair: Mutation::Remove("AimedArrivalTime", 0),
        path: "ServiceDelivery.StopMonitoringDelivery[0].MonitoredStopVisit[0].MonitoredVehicleJourney.MonitoredCall.AimedArrivalTime",
        reason: "premature end of input",
        discarded: "ServiceDelivery.StopMonitoringDelivery[0].MonitoredStopVisit[0].MonitoredVehicleJourney.MonitoredCall.AimedArrivalTime",
        unit: None,
    },
    Case {
        class: "empty timestamp",
        position: Position::Mandatory,
        fixture: SM,
        fault: Mutation::Empty("RecordedAtTime", 1),
        repair: Mutation::Remove("MonitoredStopVisit", 1),
        path: "ServiceDelivery.StopMonitoringDelivery[0].MonitoredStopVisit[1].RecordedAtTime",
        reason: "premature end of input",
        discarded: "ServiceDelivery.StopMonitoringDelivery[0].MonitoredStopVisit[1]",
        unit: Some(("MonitoredStopVisit", Some("SED9843214675434"))),
    },
    Case {
        class: "empty timestamp",
        position: Position::Optional,
        fixture: ET,
        fault: Mutation::Empty("ExpectedArrivalTime", 0),
        repair: Mutation::Remove("ExpectedArrivalTime", 0),
        path: "ServiceDelivery.EstimatedTimetableDelivery[0].EstimatedJourneyVersionFrame[0].EstimatedVehicleJourney[0].EstimatedCalls.EstimatedCall[1].ExpectedArrivalTime",
        reason: "premature end of input",
        discarded: "ServiceDelivery.EstimatedTimetableDelivery[0].EstimatedJourneyVersionFrame[0].EstimatedVehicleJourney[0].EstimatedCalls.EstimatedCall[1].ExpectedArrivalTime",
        unit: None,
    },
    Case {
        class: "empty timestamp",
        position: Position::Optional,
        fixture: VM,
        fault: Mutation::Empty("ActualDepartureTime", 0),
        repair: Mutation::Remove("ActualDepartureTime", 0),
        path: "ServiceDelivery.VehicleMonitoringDelivery[0].VehicleActivity[0].MonitoredVehicleJourney.PreviousCalls.PreviousCall[0].ActualDepartureTime",
        reason: "premature end of input",
        discarded: "ServiceDelivery.VehicleMonitoringDelivery[0].VehicleActivity[0].MonitoredVehicleJourney.PreviousCalls.PreviousCall[0].ActualDepartureTime",
        unit: None,
    },
    Case {
        class: "empty timestamp",
        position: Position::Mandatory,
        fixture: VM,
        fault: Mutation::Empty("ValidUntilTime", 0),
        repair: Mutation::Remove("VehicleActivity", 0),
        path: "ServiceDelivery.VehicleMonitoringDelivery[0].VehicleActivity[0].ValidUntilTime",
        reason: "premature end of input",
        discarded: "ServiceDelivery.VehicleMonitoringDelivery[0].VehicleActivity[0]",
        unit: Some(("VehicleActivity", Some("EV000123"))),
    },
    Case {
        class: "empty timestamp",
        position: Position::Optional,
        fixture: SX,
        fault: Mutation::Empty("TimeOfCommunication", 0),
        repair: Mutation::Remove("TimeOfCommunication", 0),
        path: "ServiceDelivery.SituationExchangeDelivery[0].Situations.PtSituationElement[0].Source.TimeOfCommunication",
        reason: "premature end of input",
        discarded: "ServiceDelivery.SituationExchangeDelivery[0].Situations.PtSituationElement[0].Source.TimeOfCommunication",
        unit: None,
    },
    Case {
        class: "empty timestamp",
        position: Position::Mandatory,
        fixture: SX,
        fault: Mutation::Empty("CreationTime", 0),
        repair: Mutation::Remove("PtSituationElement", 0),
        path: "ServiceDelivery.SituationExchangeDelivery[0].Situations.PtSituationElement[0].CreationTime",
        reason: "premature end of input",
        discarded: SX_SITUATION,
        unit: Some(("PtSituationElement", Some("000354"))),
    },
    // A mandatory element missing.
    Case {
        class: "missing mandatory element",
        position: Position::Optional,
        fixture: SX,
        fault: Mutation::Remove("StartTime", 1),
        repair: Mutation::Remove("Period", 0),
        path: "ServiceDelivery.SituationExchangeDelivery[0].Situations.PtSituationElement[0].Consequences.Consequence[0].Period[0]",
        reason: "missing field `StartTime`",
        discarded: "ServiceDelivery.SituationExchangeDelivery[0].Situations.PtSituationElement[0].Consequences.Consequence[0].Period[0]",
        unit: None,
    },
    Case {
        class: "missing mandatory element",
        position: Position::Mandatory,
        fixture: SX,
        fault: Mutation::Remove("SourceType", 0),
        repair: Mutation::Remove("PtSituationElement", 0),
        path: "ServiceDelivery.SituationExchangeDelivery[0].Situations.PtSituationElement[0].Source",
        reason: "missing field `SourceType`",
        discarded: SX_SITUATION,
        unit: Some(("PtSituationElement", Some("000354"))),
    },
    Case {
        class: "missing mandatory element",
        position: Position::Optional,
        fixture: ET,
        fault: Mutation::Remove("StopPointRef", 1),
        repair: Mutation::Remove("EstimatedCall", 1),
        path: "ServiceDelivery.EstimatedTimetableDelivery[0].EstimatedJourneyVersionFrame[0].EstimatedVehicleJourney[0].EstimatedCalls.EstimatedCall[1]",
        reason: "missing field `StopPointRef`",
        discarded: "ServiceDelivery.EstimatedTimetableDelivery[0].EstimatedJourneyVersionFrame[0].EstimatedVehicleJourney[0].EstimatedCalls.EstimatedCall[1]",
        unit: None,
    },
    Case {
        class: "missing mandatory element",
        position: Position::Mandatory,
        fixture: ET,
        fault: Mutation::Remove("LineRef", 1),
        repair: Mutation::Remove("EstimatedVehicleJourney", 1),
        path: "ServiceDelivery.EstimatedTimetableDelivery[0].EstimatedJourneyVersionFrame[0].EstimatedVehicleJourney[1]",
        reason: "missing field `LineRef`",
        discarded: "ServiceDelivery.EstimatedTimetableDelivery[0].EstimatedJourneyVersionFrame[0].EstimatedVehicleJourney[1]",
        unit: Some(("EstimatedVehicleJourney", Some("00009"))),
    },
    Case {
        class: "missing mandatory element",
        position: Position::Optional,
        fixture: SM,
        fault: Mutation::Remove("DataFrameRef", 0),
        repair: Mutation::Remove("FramedVehicleJourneyRef", 0),
        path: "ServiceDelivery.StopMonitoringDelivery[0].MonitoredStopVisit[0].MonitoredVehicleJourney.FramedVehicleJourneyRef",
        reason: "missing field `DataFrameRef`",
        discarded: "ServiceDelivery.StopMonitoringDelivery[0].MonitoredStopVisit[0].MonitoredVehicleJourney.FramedVehicleJourneyRef",
        unit: None,
    },
    Case {
        class: "missing mandatory element",
        position: Position::Mandatory,
        fixture: SM,
        fault: Mutation::Remove("MonitoredVehicleJourney", 0),
        repair: Mutation::Remove("MonitoredStopVisit", 0),
        path: "ServiceDelivery.StopMonitoringDelivery[0].MonitoredStopVisit[0]",
        reason: "missing field `MonitoredVehicleJourney`",
        discarded: "ServiceDelivery.StopMonitoringDelivery[0].MonitoredStopVisit[0]",
        unit: Some(("MonitoredStopVisit", Some("SED9843214675432"))),
    },
    Case {
        class: "missing mandatory element",
        position: Position::Optional,
        fixture: VM,
        fault: Mutation::Remove("DataFrameRef", 0),
        repair: Mutation::Remove("FramedVehicleJourneyRef", 0),
        path: "ServiceDelivery.VehicleMonitoringDelivery[0].VehicleActivity[0].MonitoredVehicleJourney.FramedVehicleJourneyRef",
        reason: "missing field `DataFrameRef`",
        discarded: "ServiceDelivery.VehicleMonitoringDelivery[0].VehicleActivity[0].MonitoredVehicleJourney.FramedVehicleJourneyRef",
        unit: None,
    },
    Case {
        class: "missing mandatory element",
        position: Position::Mandatory,
        fixture: VM,
        fault: Mutation::Remove("MonitoredVehicleJourney", 0),
        repair: Mutation::Remove("VehicleActivity", 0),
        path: "ServiceDelivery.VehicleMonitoringDelivery[0].VehicleActivity[0]",
        reason: "missing field `MonitoredVehicleJourney`",
        discarded: "ServiceDelivery.VehicleMonitoringDelivery[0].VehicleActivity[0]",
        unit: Some(("VehicleActivity", Some("EV000123"))),
    },
    // A value outside what the element takes: a choice the crate does not know, a
    // word where a boolean goes. The feature list demands at least one feature,
    // so the only feature's unknown choice costs the list.
    Case {
        class: "unknown choice",
        position: Position::Mandatory,
        fixture: FM,
        fault: Mutation::Markup("AccessFacility", 0, "<NoSuchFacility>lift</NoSuchFacility>"),
        repair: Mutation::Remove("Features", 0),
        path: "ServiceDelivery.FacilityMonitoringDelivery[0].FacilityCondition[0].Facility.Features.Feature[0]",
        reason: "unknown variant `NoSuchFacility`",
        discarded: "ServiceDelivery.FacilityMonitoringDelivery[0].FacilityCondition[0].Facility.Features",
        unit: None,
    },
    Case {
        class: "unknown token",
        position: Position::Optional,
        fixture: VM,
        fault: Mutation::Text("Monitored", 0, "maybe"),
        repair: Mutation::Remove("Monitored", 0),
        path: "ServiceDelivery.VehicleMonitoringDelivery[0].VehicleActivity[0].MonitoredVehicleJourney.Monitored",
        reason: "invalid type: string \"maybe\", expected a boolean",
        discarded: "ServiceDelivery.VehicleMonitoringDelivery[0].VehicleActivity[0].MonitoredVehicleJourney.Monitored",
        unit: None,
    },
    // A number that is not one.
    Case {
        class: "broken number",
        position: Position::Optional,
        fixture: SX,
        fault: Mutation::Text("Version", 0, "abc"),
        repair: Mutation::Remove("Version", 0),
        path: "ServiceDelivery.SituationExchangeDelivery[0].Situations.PtSituationElement[0].Version",
        reason: "invalid type: string \"abc\", expected i64",
        discarded: "ServiceDelivery.SituationExchangeDelivery[0].Situations.PtSituationElement[0].Version",
        unit: None,
    },
    Case {
        class: "broken number",
        position: Position::Mandatory,
        fixture: VM,
        fault: Mutation::Text("NumberOfBlockParts", 0, "many"),
        repair: Mutation::Remove("TrainBlockPart", 0),
        path: "ServiceDelivery.VehicleMonitoringDelivery[0].VehicleActivity[0].MonitoredVehicleJourney.TrainBlockPart[0].NumberOfBlockParts",
        reason: "invalid type: string \"many\"",
        discarded: "ServiceDelivery.VehicleMonitoringDelivery[0].VehicleActivity[0].MonitoredVehicleJourney.TrainBlockPart[0]",
        unit: None,
    },
];

#[test]
fn the_catalogue_of_faults_costs_exactly_what_each_fault_touches() {
    let mut covered: BTreeMap<(&str, Position), usize> = BTreeMap::new();
    for case in CATALOGUE {
        let label = format!("{} / {:?} / {}", case.class, case.position, case.fixture);
        let example = fixture(case.fixture);
        let broken = case.fault.apply(&example);
        assert!(siri_rs::from_str::<Siri>(&broken).is_err(), "{label}: the strict reader refuses it");

        let (read, findings) =
            lenient::from_str(&broken).unwrap_or_else(|e| panic!("{label}: reads leniently: {e}"));

        assert_eq!(read, strict(&case.repair.apply(&example)), "{label}");
        assert_eq!(findings.len(), 1, "{label}: {findings:?}");
        let finding = findings.iter().next().expect("one finding");
        assert_eq!(finding.path, case.path, "{label}");
        assert!(
            finding.reason.starts_with(case.reason),
            "{label}: reason {:?} does not begin with {:?}",
            finding.reason,
            case.reason
        );
        assert_eq!(finding.discarded.path(), case.discarded, "{label}");
        assert_eq!(
            finding.discarded.unit_name().zip(Some(finding.discarded.identifier())),
            case.unit,
            "{label}"
        );
        match case.position {
            Position::Optional => {
                assert_eq!(finding.discarded.path(), finding.path, "{label}: only the element at fault goes");
            }
            Position::Mandatory => assert!(
                finding.path.starts_with(finding.discarded.path()),
                "{label}: what goes holds the fault"
            ),
        }
        assert_eq!(finding.discarded.is_unit(), case.unit.is_some(), "{label}");
        *covered.entry((case.class, case.position)).or_default() += 1;
    }
    for class in ["empty timestamp", "missing mandatory element", "broken number"] {
        assert!(covered.contains_key(&(class, Position::Optional)), "{class} in an optional position");
        assert!(covered.contains_key(&(class, Position::Mandatory)), "{class} in a mandatory position");
    }
}

#[test]
fn an_unknown_enumeration_token_is_kept_not_a_fault() {
    // The schema enumerations keep a token they do not list, so a token from a
    // later release, or a producer's own, is no reason to leave anything out —
    // the crate reads it as unrecognised, strictly and leniently alike.
    let example = fixture(SX);
    let foreign = with_text(&example, "SourceType", 0, "carrierPigeon");
    let expected = strict(&foreign);
    let (read, findings) = lenient(&foreign);
    assert_eq!(read, expected);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn a_duration_the_type_keeps_verbatim_is_not_a_fault() {
    // `xsd:duration` is kept as written and judged when it is used, so a broken
    // duration reads — the same way, strictly and leniently.
    let example = fixture(VM);
    let broken = with_text(&example, "Delay", 0, "P1Y1Y");
    let expected = strict(&broken);
    let (read, findings) = lenient(&broken);
    assert_eq!(read, expected);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn an_empty_element_with_a_schema_default_is_not_a_fault() {
    let example = fixture(VM);
    let defaulted = emptied(&example, "Monitored", 0);
    let expected = strict(&defaulted);
    let (read, findings) = lenient(&defaulted);
    assert_eq!(read, expected);
    assert!(findings.is_empty(), "{findings:?}");
}

/// A copy of the `nth` element named `name` with its first `identifier` element
/// renamed to `id`.
fn copy_named(xml: &str, name: &str, nth: usize, identifier: &str, id: &str) -> String {
    let copied = copy(xml, name, nth);
    let old = element(&copied, identifier, 0);
    let mut renamed = copied.clone();
    renamed.replace_range(old, &format!("<{identifier}>{id}</{identifier}>"));
    renamed
}

#[test]
fn records_after_an_interruption_of_their_list_are_left_out_one_by_one() {
    struct Interrupted {
        fixture: &'static str,
        record: &'static str,
        identifier: &'static str,
        after: &'static str,
        list: &'static str,
        left_out: &'static str,
    }
    for case in [
        Interrupted {
            fixture: SX,
            record: "PtSituationElement",
            identifier: "SituationNumber",
            after: "RoadSituationElement",
            list: "ServiceDelivery.SituationExchangeDelivery[0].Situations",
            left_out: "ServiceDelivery.SituationExchangeDelivery[0].Situations.PtSituationElement[1]",
        },
        Interrupted {
            fixture: SM,
            record: "MonitoredStopVisit",
            identifier: "ItemIdentifier",
            after: "MonitoredStopVisitCancellation",
            list: "ServiceDelivery.StopMonitoringDelivery[0]",
            left_out: "ServiceDelivery.StopMonitoringDelivery[0].MonitoredStopVisit[2]",
        },
        Interrupted {
            fixture: VM,
            record: "VehicleActivity",
            identifier: "ItemIdentifier",
            after: "VehicleActivityCancellation",
            list: "ServiceDelivery.VehicleMonitoringDelivery[0]",
            left_out: "ServiceDelivery.VehicleMonitoringDelivery[0].VehicleActivity[2]",
        },
    ] {
        let example = fixture(case.fixture);
        let straggler = copy_named(&example, case.record, 0, case.identifier, "STRAGGLER");
        let broken = insert_after(&example, case.after, 0, &straggler);
        assert!(siri_rs::from_str::<Siri>(&broken).is_err(), "{}: the reader is order-sensitive", case.fixture);

        let (read, findings) = lenient(&broken);

        assert_eq!(read, strict(&example), "{}", case.fixture);
        assert_eq!(findings.len(), 1, "{}: {findings:?}", case.fixture);
        let finding = findings.iter().next().expect("one finding");
        assert_eq!(finding.path, case.list);
        assert_eq!(finding.reason, format!("duplicate field `{}`", case.record));
        assert_eq!(finding.discarded.path(), case.left_out);
        assert_eq!(finding.discarded.unit_name(), Some(case.record));
        assert_eq!(finding.discarded.identifier(), Some("STRAGGLER"));
    }
}

/// The situation exchange example with `copies` further situations numbered
/// `S1`, `S2`, … after the first.
fn situations(copies: usize) -> String {
    let example = fixture(SX);
    let mut document = example.clone();
    for i in (1..=copies).rev() {
        let situation = copy_named(&example, "PtSituationElement", 0, "SituationNumber", &format!("S{i}"));
        document = insert_after(&document, "PtSituationElement", 0, &situation);
    }
    document
}

#[test]
fn several_faults_in_one_document_are_each_reported_in_document_order() {
    let document = situations(4);
    // Situations S1 and S3 lose their creation time, S4 the time its source was
    // heard from; the original and S2 are untouched. A situation carries a second
    // CreationTime in the situation it refers to, so its own is every other one.
    let broken = emptied(&emptied(&emptied(&document, "CreationTime", 2), "CreationTime", 6), "TimeOfCommunication", 4);

    let (read, findings) = lenient(&broken);

    let expected = remove(&remove(&remove(&document, "TimeOfCommunication", 4), "PtSituationElement", 3), "PtSituationElement", 1);
    assert_eq!(read, strict(&expected));
    let reported: Vec<(&str, Option<&str>)> = findings
        .iter()
        .map(|finding| (finding.discarded.path(), finding.discarded.identifier()))
        .collect();
    let situations = "ServiceDelivery.SituationExchangeDelivery[0].Situations.PtSituationElement";
    assert_eq!(
        reported,
        vec![
            (&*format!("{situations}[1]"), Some("S1")),
            (&*format!("{situations}[3]"), Some("S3")),
            (&*format!("{situations}[4].Source.TimeOfCommunication"), None),
        ]
    );
    assert_eq!(findings.units().count(), 2);
    assert_eq!(findings.elements().count(), 1);
    assert_eq!(findings.by_reason(), BTreeMap::from([("premature end of input", 3)]));
    let shown = findings.iter().next().expect("a finding").to_string();
    assert_eq!(
        shown,
        format!("cannot read {situations}[1].CreationTime: premature end of input; left out the PtSituationElement S1 at {situations}[1]")
    );
    assert_eq!(findings.as_slice().len(), 3);
    assert_eq!(findings.clone().into_iter().count(), 3);
    assert_eq!((&findings).into_iter().count(), 3);
}

#[test]
fn a_delivery_left_out_reports_every_record_in_it() {
    let stop_monitoring = fixture(SM);
    let both = insert_after(&stop_monitoring, "StopMonitoringDelivery", 0, &copy(&situations(1), "SituationExchangeDelivery", 0));
    // The third ResponseTimestamp is the situation exchange delivery's own.
    let broken = emptied(&both, "ResponseTimestamp", 2);

    let (read, findings) = lenient(&broken);

    assert_eq!(read, strict(&stop_monitoring));
    let delivery = "ServiceDelivery.SituationExchangeDelivery[1]";
    let reported: Vec<(&str, &str, Option<&str>)> = findings
        .iter()
        .map(|finding| {
            assert_eq!(finding.path, format!("{delivery}.ResponseTimestamp"));
            assert_eq!(finding.reason, "premature end of input");
            (finding.discarded.unit_name().expect("a unit"), finding.discarded.path(), finding.discarded.identifier())
        })
        .collect();
    assert_eq!(
        reported,
        vec![
            ("SituationExchangeDelivery", delivery, None),
            ("PtSituationElement", &*format!("{delivery}.Situations.PtSituationElement[0]"), Some("000354")),
            ("PtSituationElement", &*format!("{delivery}.Situations.PtSituationElement[1]"), Some("S1")),
            ("RoadSituationElement", &*format!("{delivery}.Situations.RoadSituationElement[0]"), Some("000354")),
        ]
    );
}

#[test]
fn paths_count_the_document_as_it_arrived() {
    // Two of three situations go: the third is still the third.
    let document = situations(2);
    let broken = emptied(&emptied(&document, "CreationTime", 0), "CreationTime", 4);
    let (read, findings) = lenient(&broken);
    assert_eq!(read, strict(&remove(&remove(&document, "PtSituationElement", 2), "PtSituationElement", 0)));
    let paths: Vec<&str> = findings.iter().map(|finding| finding.discarded.path()).collect();
    let situations = "ServiceDelivery.SituationExchangeDelivery[0].Situations.PtSituationElement";
    assert_eq!(paths, vec![&*format!("{situations}[0]"), &*format!("{situations}[2]")]);

    // Two of three calls inside one journey go: the third is still the third.
    let example = fixture(ET);
    let broken = remove(&remove(&example, "StopPointRef", 2), "StopPointRef", 0);
    let (read, findings) = lenient(&broken);
    assert_eq!(read, strict(&remove(&remove(&example, "EstimatedCall", 2), "EstimatedCall", 0)));
    let paths: Vec<&str> = findings.iter().map(|finding| finding.discarded.path()).collect();
    let calls = format!("{ET_JOURNEY}[0].EstimatedCalls.EstimatedCall");
    assert_eq!(paths, vec![&*format!("{calls}[0]"), &*format!("{calls}[2]")]);
}

#[test]
fn what_is_wrong_with_the_document_rather_than_in_it_is_the_ordinary_error() {
    let example = fixture(SM);
    let unclosed = example.trim_end().trim_end_matches("</Siri>").to_string();
    let wrong_root = example.replacen("<Siri ", "<Sirius ", 1).replacen("</Siri>", "</Sirius>", 1);
    let no_payload = r#"<Siri xmlns="http://www.siri.org.uk/siri" version="2.0"></Siri>"#.to_string();
    for (label, document) in [
        ("not well-formed", unclosed),
        ("wrong root", wrong_root),
        ("no payload", no_payload),
    ] {
        let strict_error = siri_rs::from_str::<Siri>(&document).expect_err(label);
        let lenient_error = lenient::from_str(&document).expect_err(label);
        assert_eq!(lenient_error.to_string(), strict_error.to_string(), "{label}");
    }

    // The strict reader does not check the namespace a document binds by default,
    // and reads one bound to a foreign namespace as SIRI; the lenient reader is
    // no stricter than it. What the two do with such a document is the same.
    let wrong_namespace = example.replacen("http://www.siri.org.uk/siri", "http://example.org/not-siri", 1);
    let strictly = strict(&wrong_namespace);
    let (leniently, findings) = lenient(&wrong_namespace);
    assert_eq!(leniently, strictly);
    assert!(findings.is_empty());
}

/// The record elements a document holds, by the element they sit in — the units
/// this test counts, spelled independently of the crate's own table.
const UNITS: &[(&str, &str)] = &[
    ("DatedTimetableVersionFrame", "DatedVehicleJourney"),
    ("DatedTimetableVersionFrame", "RemovedDatedVehicleJourney"),
    ("DatedTimetableVersionFrame", "ServiceJourneyInterchange"),
    ("DatedTimetableVersionFrame", "RemovedServiceJourneyInterchange"),
    ("EstimatedJourneyVersionFrame", "EstimatedVehicleJourney"),
    ("EstimatedJourneyVersionFrame", "EstimatedServiceJourneyInterchange"),
    ("StopTimetableDelivery", "TimetabledStopVisit"),
    ("StopTimetableDelivery", "TimetabledStopVisitCancellation"),
    ("StopMonitoringDelivery", "MonitoredStopVisit"),
    ("StopMonitoringDelivery", "MonitoredStopVisitCancellation"),
    ("StopMonitoringDelivery", "StopLineNotice"),
    ("StopMonitoringDelivery", "StopLineNoticeCancellation"),
    ("StopMonitoringDelivery", "StopNotice"),
    ("StopMonitoringDelivery", "StopNoticeCancellation"),
    ("StopMonitoringDelivery", "ServiceException"),
    ("VehicleMonitoringDelivery", "VehicleActivity"),
    ("VehicleMonitoringDelivery", "VehicleActivityCancellation"),
    ("ConnectionTimetableDelivery", "TimetabledFeederArrival"),
    ("ConnectionTimetableDelivery", "TimetabledFeederArrivalCancellation"),
    ("ConnectionMonitoringFeederDelivery", "MonitoredFeederArrival"),
    ("ConnectionMonitoringFeederDelivery", "MonitoredFeederArrivalCancellation"),
    ("ConnectionMonitoringDistributorDelivery", "WaitProlongedDeparture"),
    ("ConnectionMonitoringDistributorDelivery", "StoppingPositionChangedDeparture"),
    ("ConnectionMonitoringDistributorDelivery", "DistributorDepartureCancellation"),
    ("GeneralMessageDelivery", "GeneralMessage"),
    ("GeneralMessageDelivery", "GeneralMessageCancellation"),
    ("FacilityMonitoringDelivery", "FacilityCondition"),
    ("Situations", "PtSituationElement"),
    ("Situations", "RoadSituationElement"),
    ("controlActions", "ControlAction"),
    ("groupsOfControlActions", "GroupOfControlActions"),
    ("revokedControlActions", "RevokedControlAction"),
    ("driverMessages", "DriverMessage"),
    ("vehicleDetectings", "VehicleDetecting"),
];

/// The elements of a service delivery that are not functional deliveries.
const SERVICE_DELIVERY_OWN: &[&str] = &[
    "ResponseTimestamp",
    "ProducerRef",
    "Address",
    "ResponseMessageIdentifier",
    "RequestMessageRef",
    "DelegatorAddress",
    "DelegatorRef",
    "Status",
    "ErrorCondition",
    "MoreData",
];

/// One element of a document, as the fuzzer sees it.
struct Node {
    /// The name as written, prefix included: what the end tag is spelled with.
    name: String,
    start: usize,
    end: usize,
    depth: usize,
    leaf: bool,
}

/// Every element of a document, in document order; `None` for a document the
/// tokeniser refuses.
fn nodes(xml: &str) -> Option<Vec<Node>> {
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut nodes = Vec::new();
    let mut open: Vec<(usize, bool)> = Vec::new();
    let mut depth = 0;
    loop {
        let at = usize::try_from(reader.buffer_position()).expect("fits");
        match reader.read_event().ok()? {
            Event::Eof => break,
            Event::Start(element) => {
                let name = String::from_utf8_lossy(element.name().as_ref()).into_owned();
                nodes.push(Node {
                    name,
                    start: at,
                    end: 0,
                    depth,
                    leaf: true,
                });
                open.push((nodes.len() - 1, true));
                depth += 1;
            }
            Event::Empty(element) => {
                let name = String::from_utf8_lossy(element.name().as_ref()).into_owned();
                if let Some(parent) = open.last_mut() {
                    parent.1 = false;
                }
                nodes.push(Node {
                    name,
                    start: at,
                    end: usize::try_from(reader.buffer_position()).expect("fits"),
                    depth,
                    leaf: true,
                });
            }
            Event::End(_) => {
                depth -= 1;
                let (index, leaf) = open.pop()?;
                nodes[index].end = usize::try_from(reader.buffer_position()).expect("fits");
                nodes[index].leaf = leaf;
                if let Some(parent) = open.last_mut() {
                    parent.1 = false;
                }
            }
            _ => {}
        }
    }
    Some(nodes)
}

/// How many units a document holds: records under their containers, and the
/// functional deliveries under the service delivery.
fn units_in(xml: &str) -> usize {
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut stack: Vec<String> = Vec::new();
    let mut units = 0;
    loop {
        let (name, opens) = match reader.read_event().expect("well-formed") {
            Event::Eof => break,
            Event::Start(element) => (String::from_utf8_lossy(element.local_name().as_ref()).into_owned(), true),
            Event::Empty(element) => (String::from_utf8_lossy(element.local_name().as_ref()).into_owned(), false),
            Event::End(_) => {
                stack.pop();
                continue;
            }
            _ => continue,
        };
        if let Some(parent) = stack.last() {
            let record = UNITS.iter().any(|(container, unit)| container == parent && *unit == name);
            let delivery = parent == "ServiceDelivery" && !SERVICE_DELIVERY_OWN.contains(&name.as_str());
            if record || delivery {
                units += 1;
            }
        }
        if opens {
            stack.push(name);
        }
    }
    units
}

/// A small deterministic generator, so that a failing round can be re-run.
struct Dice(u64);

impl Dice {
    fn roll(&mut self, sides: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        usize::try_from(self.0 % sides as u64).expect("fits")
    }
}

/// One random damage to a document: empty a leaf, garble a leaf's text, take an
/// element out, double one, or move one to the end of its parent. A document an
/// earlier damage left ill-formed is handed back as it is.
fn damage(xml: &str, dice: &mut Dice) -> String {
    let Some(nodes) = nodes(xml) else {
        return xml.to_string();
    };
    let candidates: Vec<&Node> = nodes.iter().filter(|node| node.depth >= 2).collect();
    let node = candidates[dice.roll(candidates.len())];
    let mut out = xml.to_string();
    match dice.roll(5) {
        0 if node.leaf => out.replace_range(node.start..node.end, &format!("<{}/>", node.name)),
        1 if node.leaf => out.replace_range(node.start..node.end, &format!("<{0}>garble</{0}>", node.name)),
        2 => out.replace_range(node.start..node.end, ""),
        3 => out.insert_str(node.end, &xml[node.start..node.end]),
        _ => {
            let parent = nodes
                .iter()
                .filter(|other| other.start < node.start && other.end >= node.end && other.depth + 1 == node.depth)
                .last()
                .expect("a parent");
            let end_tag = parent.end - parent.name.len() - 3;
            let moved = xml[node.start..node.end].to_string();
            out.replace_range(node.start..node.end, "");
            out.insert_str(end_tag - (node.end - node.start), &moved);
        }
    }
    out
}

const DAMAGED_EXAMPLES: &[&str] = &[
    SX,
    "xml/sx/vdv736/SX_1022_main_message.xml",
    ET,
    SM,
    "xml/sm/exs_stopMonitoring_response_complex.xml",
    VM,
    FM,
    "xml/gm/exm_generalMessage_response.xml",
    "xml/pt/ext_productionTimetable_response.xml",
    "xml/st/exs_stopTimetable_response.xml",
    "xml/ct/exc_connectionTimetable_response.xml",
    "xml/cm/exc_connectionMonitoringFeeder_response.xml",
    "xml/cm/exc_connectionMonitoringDistributor_response.xml",
    "derived/ca/exc_controlAction_delivery.xml",
];

#[test]
fn a_randomly_damaged_document_never_panics_and_never_loses_a_unit_quietly() {
    const ROUNDS: usize = 48;
    let mut dice = Dice(0x9E37_79B9_7F4A_7C15);
    let mut read = 0;
    let mut reported = 0;
    for example in DAMAGED_EXAMPLES {
        let intact = fixture(example);
        for round in 0..ROUNDS {
            let mut damaged = intact.clone();
            for _ in 0..=dice.roll(3) {
                damaged = damage(&damaged, &mut dice);
            }
            let label = format!("{example} round {round}");
            let outcome = catch_unwind(AssertUnwindSafe(|| lenient::from_str(&damaged)))
                .unwrap_or_else(|_| panic!("{label}: the lenient reader panicked on:\n{damaged}"));
            let Ok((siri, findings)) = outcome else {
                continue;
            };
            read += 1;
            for finding in &findings {
                assert!(finding.path.starts_with("ServiceDelivery."), "{label}: {finding}");
                assert!(finding.discarded.path().starts_with("ServiceDelivery."), "{label}: {finding}");
                assert!(!finding.reason.is_empty(), "{label}: {finding}");
            }
            match siri_rs::from_str::<Siri>(&damaged) {
                Ok(strictly) => {
                    assert_eq!(siri, strictly, "{label}");
                    assert!(findings.is_empty(), "{label}: {findings:?}");
                }
                Err(_) => {
                    assert!(!findings.is_empty(), "{label}: read differently from the strict reader with nothing to report");
                    reported += 1;
                }
            }
            let written = siri_rs::to_string(&siri).expect("what was read writes");
            assert_eq!(
                units_in(&damaged),
                units_in(&written) + findings.units().count(),
                "{label}: units in the document, read, reported:\n{}",
                findings.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n")
            );
        }
    }
    assert!(read > 100, "damaged documents read: {read}");
    assert!(reported > 50, "damaged documents read with findings: {reported}");
    let _ = insert_before;
}
