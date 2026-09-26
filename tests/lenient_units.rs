//! What lenient reading leaves out, and what it never does.
//!
//! A stream from a producer one does not control carries the odd element that
//! cannot be read. Read strictly, that element fails the whole document; read
//! leniently, it costs the smallest thing the document reads without: the element
//! itself where the schema makes it optional, and the unit around it — a
//! situation, a journey, a stop visit, a vehicle, a delivery — where it does not.
//! The envelope is never left out: a fault in it is the ordinary error.
//!
//! Every document here is an official example with one element broken, and the
//! expectation is spelled as the strict reading of the example with the discarded
//! element taken out by hand — the two readings have to agree to the field.
#![cfg(feature = "lenient")]

mod support;

use siri_rs::lenient::{self, Discarded, Findings};
use siri_rs::Siri;
use support::mutation::{copy, element, emptied, insert_after, remove};

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

const SM: &str = "xml/sm/exs_stopMonitoring_response.xml";
const SX: &str = "xml/sx/exx_situationExchange_response.xml";

#[test]
fn an_optional_element_that_does_not_read_is_left_out_and_its_unit_kept() {
    let example = fixture(SM);
    // The first AimedArrivalTime is in the monitored call of the first stop visit,
    // where the schema leaves it optional.
    let broken = emptied(&example, "AimedArrivalTime", 0);

    let (read, findings) = lenient(&broken);

    assert_eq!(read, strict(&remove(&example, "AimedArrivalTime", 0)));
    assert_eq!(findings.len(), 1);
    let finding = &findings.iter().next().expect("one finding");
    let path = "ServiceDelivery.StopMonitoringDelivery[0].MonitoredStopVisit[0]\
                .MonitoredVehicleJourney.MonitoredCall.AimedArrivalTime";
    assert_eq!(finding.path, path);
    assert_eq!(finding.reason, "premature end of input");
    assert!(
        matches!(&finding.discarded, Discarded::Element { .. }),
        "an optional element is left out on its own: {:?}",
        finding.discarded
    );
    assert_eq!(finding.discarded.path(), path);
}

#[test]
fn a_mandatory_element_that_does_not_read_takes_its_unit_with_it() {
    let example = fixture(SM);
    // The second RecordedAtTime belongs to the second stop visit, and a visit
    // without one is not a visit.
    let broken = emptied(&example, "RecordedAtTime", 1);

    let (read, findings) = lenient(&broken);

    assert_eq!(read, strict(&remove(&example, "MonitoredStopVisit", 1)));
    assert_eq!(findings.len(), 1);
    let finding = &findings.iter().next().expect("one finding");
    let unit = "ServiceDelivery.StopMonitoringDelivery[0].MonitoredStopVisit[1]";
    assert_eq!(finding.path, format!("{unit}.RecordedAtTime"));
    assert_eq!(finding.reason, "premature end of input");
    assert!(finding.discarded.is_unit(), "{:?}", finding.discarded);
    assert_eq!(finding.discarded.path(), unit);
    assert_eq!(finding.discarded.unit_name(), Some("MonitoredStopVisit"));
    assert_eq!(finding.discarded.identifier(), Some("SED9843214675434"));
}

#[test]
fn a_mandatory_element_inside_an_optional_one_costs_only_the_optional_one() {
    let example = fixture(SX);
    // StartTime is mandatory in a validity period; the validity period is one of
    // a list the situation may leave empty.
    let broken = remove(&example, "StartTime", 0);

    let (read, findings) = lenient(&broken);

    assert_eq!(read, strict(&remove(&example, "ValidityPeriod", 0)));
    assert_eq!(findings.len(), 1);
    let finding = &findings.iter().next().expect("one finding");
    let period = "ServiceDelivery.SituationExchangeDelivery[0].Situations\
                  .PtSituationElement[0].ValidityPeriod[0]";
    assert_eq!(finding.path, period);
    assert_eq!(finding.reason, "missing field `StartTime`");
    assert!(matches!(&finding.discarded, Discarded::Element { .. }));
    assert_eq!(finding.discarded.path(), period);
}

/// How one unit of a service is broken: which element, counted over the whole
/// document, is emptied or taken out.
enum Fault {
    Empty(&'static str, usize),
    Remove(&'static str, usize),
}

/// One unit per service, broken in a mandatory element.
struct UnitCase {
    fixture: &'static str,
    unit: &'static str,
    nth: usize,
    fault: Fault,
    identifier: Option<&'static str>,
}

const UNIT_CASES: &[UnitCase] = &[
    UnitCase {
        fixture: SX,
        unit: "PtSituationElement",
        nth: 0,
        fault: Fault::Empty("CreationTime", 0),
        identifier: Some("000354"),
    },
    UnitCase {
        fixture: "xml/et/ext_estimatedTimetable_response.xml",
        unit: "EstimatedVehicleJourney",
        nth: 1,
        fault: Fault::Remove("LineRef", 1),
        identifier: Some("00009"),
    },
    UnitCase {
        fixture: SM,
        unit: "MonitoredStopVisit",
        nth: 1,
        fault: Fault::Empty("RecordedAtTime", 1),
        identifier: Some("SED9843214675434"),
    },
    UnitCase {
        fixture: "xml/vm/exv_vehicleMonitoring_response.xml",
        unit: "VehicleActivity",
        nth: 1,
        fault: Fault::Empty("ValidUntilTime", 1),
        identifier: Some("915468"),
    },
    UnitCase {
        fixture: "xml/fm/exf_facilityMonitoring_response.xml",
        unit: "FacilityCondition",
        nth: 0,
        fault: Fault::Remove("FacilityStatus", 0),
        identifier: Some("134567-L4"),
    },
    UnitCase {
        fixture: "xml/gm/exm_generalMessage_response.xml",
        unit: "GeneralMessage",
        nth: 0,
        fault: Fault::Empty("RecordedAtTime", 0),
        identifier: Some("00034567"),
    },
    UnitCase {
        fixture: "xml/pt/ext_productionTimetable_response.xml",
        unit: "DatedVehicleJourney",
        nth: 0,
        fault: Fault::Remove("DatedCalls", 0),
        identifier: Some("DVC0008767"),
    },
    UnitCase {
        fixture: "xml/st/exs_stopTimetable_response.xml",
        unit: "TimetabledStopVisit",
        nth: 0,
        fault: Fault::Empty("RecordedAtTime", 0),
        identifier: Some("HLTST011"),
    },
    UnitCase {
        fixture: "xml/ct/exc_connectionTimetable_response.xml",
        unit: "TimetabledFeederArrival",
        nth: 0,
        fault: Fault::Empty("RecordedAtTime", 0),
        identifier: Some("98789"),
    },
    UnitCase {
        fixture: "xml/cm/exc_connectionMonitoringFeeder_response.xml",
        unit: "MonitoredFeederArrival",
        nth: 0,
        fault: Fault::Empty("RecordedAtTime", 0),
        identifier: Some("98789"),
    },
    UnitCase {
        fixture: "xml/cm/exc_connectionMonitoringDistributor_response.xml",
        unit: "WaitProlongedDeparture",
        nth: 0,
        fault: Fault::Empty("RecordedAtTime", 0),
        identifier: Some("HLKT00023"),
    },
    UnitCase {
        fixture: "derived/ca/exc_controlAction_delivery.xml",
        unit: "ControlAction",
        nth: 0,
        fault: Fault::Empty("CreationTime", 0),
        identifier: Some("CA-1"),
    },
];

#[test]
fn every_service_has_units_that_are_left_out_whole() {
    for case in UNIT_CASES {
        let example = fixture(case.fixture);
        let broken = match case.fault {
            Fault::Empty(name, nth) => emptied(&example, name, nth),
            Fault::Remove(name, nth) => remove(&example, name, nth),
        };
        assert!(
            siri_rs::from_str::<Siri>(&broken).is_err(),
            "{}: the fault fails the strict reader",
            case.fixture
        );

        let (read, findings) = lenient::from_str(&broken)
            .unwrap_or_else(|e| panic!("{}: the broken document reads leniently: {e}", case.fixture));

        assert_eq!(read, strict(&remove(&example, case.unit, case.nth)), "{}", case.fixture);
        assert_eq!(findings.len(), 1, "{}: {findings:?}", case.fixture);
        let finding = findings.iter().next().expect("one finding");
        assert_eq!(finding.discarded.unit_name(), Some(case.unit), "{}", case.fixture);
        assert!(
            finding.discarded.path().ends_with(&format!("{}[{}]", case.unit, case.nth)),
            "{}: {}",
            case.fixture,
            finding.discarded.path()
        );
        assert_eq!(finding.discarded.identifier(), case.identifier, "{}", case.fixture);
    }
}

#[test]
fn the_envelope_is_never_left_out() {
    let example = fixture(SX);
    // The first ResponseTimestamp is the service delivery's own.
    let broken = emptied(&example, "ResponseTimestamp", 0);

    let strict_error = siri_rs::from_str::<Siri>(&broken).expect_err("the envelope is broken");
    let lenient_error = lenient::from_str(&broken).expect_err("the envelope is broken");

    assert_eq!(lenient_error.to_string(), strict_error.to_string());
}

#[test]
fn a_delivery_is_left_out_when_its_own_elements_do_not_read_and_another_remains() {
    let stop_monitoring = fixture(SM);
    let situations = copy(&fixture(SX), "SituationExchangeDelivery", 0);
    let both = insert_after(&stop_monitoring, "StopMonitoringDelivery", 0, &situations);
    assert_eq!(strict(&both).payload.as_service_delivery().expect("a delivery").deliveries.len(), 2);
    // The third ResponseTimestamp is the situation exchange delivery's own.
    let broken = emptied(&both, "ResponseTimestamp", 2);

    let (read, findings) = lenient(&broken);

    assert_eq!(read, strict(&stop_monitoring));
    let delivery = findings
        .iter()
        .find(|finding| finding.discarded.unit_name() == Some("SituationExchangeDelivery"))
        .expect("the delivery is reported");
    assert_eq!(delivery.discarded.path(), "ServiceDelivery.SituationExchangeDelivery[1]");
    assert_eq!(delivery.path, "ServiceDelivery.SituationExchangeDelivery[1].ResponseTimestamp");
    assert_eq!(delivery.reason, "premature end of input");
}

#[test]
fn the_only_delivery_is_never_left_out() {
    let example = fixture(SX);
    // The second ResponseTimestamp is the situation exchange delivery's own.
    let broken = emptied(&example, "ResponseTimestamp", 1);

    let strict_error = siri_rs::from_str::<Siri>(&broken).expect_err("the delivery is broken");
    let lenient_error = lenient::from_str(&broken).expect_err("no delivery would remain");

    assert_eq!(
        without_offset(&lenient_error.to_string()),
        without_offset(&strict_error.to_string()),
        "the error is the one the strict reader gives"
    );
}

/// An error message without the byte offset: the lenient reader rewrites the
/// document before it fails, so an offset into the caller's text is not one it
/// can give.
fn without_offset(message: &str) -> String {
    match (message.find(" (byte "), message.find("): ")) {
        (Some(from), Some(to)) if from < to => format!("{}{}", &message[..from], &message[to + 1..]),
        _ => message.to_string(),
    }
}

#[test]
fn the_strict_reader_is_not_changed_by_the_feature() {
    let stop_monitoring = fixture(SM);
    let situations = fixture(SX);
    for broken in [
        emptied(&stop_monitoring, "AimedArrivalTime", 0),
        emptied(&stop_monitoring, "RecordedAtTime", 1),
        remove(&situations, "StartTime", 0),
    ] {
        let error = siri_rs::from_str::<Siri>(&broken).expect_err("the strict reader refuses the document");
        assert!(matches!(error, siri_rs::Error::Deserialize { .. }), "{error}");
    }
    let intact = fixture(SM);
    assert_eq!(strict(&intact), lenient(&intact).0);
    assert!(!element(&intact, "MonitoredStopVisit", 0).is_empty());
}
