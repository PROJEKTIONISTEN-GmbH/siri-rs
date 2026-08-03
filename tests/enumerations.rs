//! Checks every enumeration in the crate against the schema it transcribes.
//!
//! An enumeration is the one place where a typo is invisible until a document is
//! rejected in production: the Rust variant compiles, the wire token is wrong.
//! This reads the tokens back out of the official schemas and compares them with
//! the crate's, value by value and in order.

mod support;

use std::collections::BTreeMap;

use quick_xml::events::Event;

/// Collects `(XSD type name, wire tokens)` for the listed enumerations.
macro_rules! enumerations {
    ($($ty:ty),* $(,)?) => {
        vec![$((
            <$ty>::XSD_TYPE,
            <$ty>::ALL.iter().map(|value| value.as_str()).collect::<Vec<&str>>(),
        )),*]
    };
}

fn transcribed() -> Vec<(&'static str, Vec<&'static str>)> {
    use siri_rs::enumerations::*;
    enumerations![
        AccessFacility,
        AccessModes,
        Accessibility,
        AccessibilityFeature,
        AccommodationFacility,
        ActionStatus,
        AdviceType,
        AirSubmodesOfTransport,
        AlertCause,
        AreaOfInterest,
        ArrivalBoardingActivity,
        AssistanceFacility,
        Audience,
        BusSubmodesOfTransport,
        CallStatus,
        ChangeModel,
        ChangeOfJourneyTimingType,
        CoachSubmodesOfTransport,
        CommunicationsTransportMethod,
        CompressionMethod,
        ConnectionDirection,
        ConnectionMonitoringDetail,
        ControlActionReasonCategory,
        CountedFeatureUnit,
        CountingTrend,
        CountingType,
        DayType,
        DaysOfWeek,
        DelayBand,
        DelaysType,
        DeliveryMethod,
        DepartureBoardingActivity,
        Direction,
        Encumbrance,
        EndTimePrecision,
        EndTimeStatus,
        EquipmentStatus,
        EstimatedTimetableDetail,
        FacilityCategory,
        FacilityStatus,
        FareClass,
        FareClassFacility,
        FirstOrLastJourney,
        FormationChange,
        HireFacility,
        HolidayType,
        ImageContent,
        InformationStatus,
        InterchangeStatus,
        JourneyRelationType,
        LinesDetail,
        LinkContent,
        LuggageFacility,
        MedicalNeed,
        MetroSubmodesOfTransport,
        Mobility,
        MobilityFacility,
        MonitoringType,
        NuisanceFacility,
        Occupancy,
        ParkingFacility,
        PassengerCommsFacility,
        PassengerInformationFacility,
        Perspective,
        Predictability,
        PredictionInaccurateReason,
        Predictors,
        ProbabilityOfOccurrence,
        ProgressRate,
        PublicEventType,
        PyschosensoryNeed,
        QualityIndex,
        RailSubmodesOfTransport,
        RefreshmentFacility,
        RelatedTo,
        RemedyType,
        ReportType,
        ReservedSpaceFacility,
        RetailFacility,
        RoutePointType,
        SanitaryFacility,
        ScopeType,
        Sensitivity,
        ServiceCondition,
        ServiceException,
        Severity,
        SituationSourceType,
        SourceType,
        StopMonitoringDetail,
        StopPlaceComponentType,
        StopPlaceStatus,
        StopPlaceType,
        StopPointType,
        StopPointsDetail,
        StopVisitType,
        Suitability,
        TelecabinSubmodesOfTransport,
        TicketRestriction,
        TicketingFacility,
        TrainElementType,
        TrainSize,
        TramSubmodesOfTransport,
        TypeOfActivatedJourney,
        TypeOfFuel,
        TypeOfNestedQuay,
        VehicleInFormationStatus,
        VehicleModesOfTransport,
        VehicleMonitoringDetail,
        VehicleStatus,
        VerificationStatus,
        WaterSubmodesOfTransport,
        WorkflowStatus,
    ]
}

#[test]
fn every_enumeration_matches_the_schema_token_for_token() {
    let schema = schema_enumerations();
    let mut failures = Vec::new();

    for (xsd_type, tokens) in transcribed() {
        let Some(expected) = schema.get(xsd_type) else {
            failures.push(format!("{xsd_type}: no such enumerated type in the schemas"));
            continue;
        };
        if *expected != tokens {
            failures.push(format!(
                "{xsd_type}:\n  schema: {expected:?}\n  crate:  {tokens:?}"
            ));
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Guards against an enumeration being added to the crate but not to the list
/// above, which would leave it unchecked.
#[test]
fn every_enumeration_in_the_crate_is_checked() {
    let source = include_str!("../src/enumerations.rs");
    let declared = source.matches("siri_enum! {").count();
    assert_eq!(
        declared,
        transcribed().len(),
        "src/enumerations.rs declares {declared} enumerations but {} are checked here",
        transcribed().len()
    );
}

/// Reads every named enumerated `xsd:simpleType` out of the bundled schemas.
///
/// Repeated tokens are collapsed: the published schema states a few values twice
/// inside one restriction, and a repeated token is still one value.
fn schema_enumerations() -> BTreeMap<String, Vec<String>> {
    let mut out = BTreeMap::new();
    for path in schema_files() {
        let xml = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        let mut reader = quick_xml::Reader::from_str(&xml);
        let mut depth = 0usize;
        let mut current: Option<(usize, String, Vec<String>)> = None;

        loop {
            match reader.read_event().expect("schema is well-formed XML") {
                Event::Eof => break,
                // A value carrying documentation is a `Start`, one without is an
                // `Empty`; the schemas use both forms.
                Event::Start(start) => {
                    depth += 1;
                    match local_name(start.local_name().as_ref()).as_str() {
                        "simpleType" => {
                            if let Some(name) = attribute(&start, "name") {
                                current = Some((depth, name, Vec::new()));
                            }
                        }
                        "enumeration" => collect_value(current.as_mut(), &start),
                        _ => {}
                    }
                }
                Event::Empty(empty) => {
                    if local_name(empty.local_name().as_ref()) == "enumeration" {
                        collect_value(current.as_mut(), &empty);
                    }
                }
                Event::End(_) => {
                    if let Some((opened_at, name, values)) = current.take() {
                        if opened_at == depth {
                            let mut unique: Vec<String> = Vec::new();
                            for value in values {
                                if !unique.contains(&value) {
                                    unique.push(value);
                                }
                            }
                            if !unique.is_empty() {
                                out.insert(name, unique);
                            }
                        } else {
                            current = Some((opened_at, name, values));
                        }
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }
    }

    assert!(!out.is_empty(), "no enumerated types found in the schemas");
    out
}

fn collect_value(
    current: Option<&mut (usize, String, Vec<String>)>,
    element: &quick_xml::events::BytesStart<'_>,
) {
    if let (Some(entry), Some(value)) = (current, attribute(element, "value")) {
        entry.2.push(value);
    }
}

fn attribute(start: &quick_xml::events::BytesStart<'_>, name: &str) -> Option<String> {
    start
        .attributes()
        .with_checks(false)
        .flatten()
        .find(|attribute| attribute.key.as_ref() == name.as_bytes())
        .map(|attribute| {
            String::from_utf8_lossy(&attribute.value)
                .trim()
                .to_owned()
        })
}

fn local_name(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn schema_files() -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    collect(&support::fixtures_dir().join("xsd"), &mut out);
    out.sort();
    return out;

    fn collect(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("readable schema directory") {
            let path = entry.expect("readable directory entry").path();
            if path.is_dir() {
                collect(&path, out);
            } else if path.extension().is_some_and(|e| e == "xsd") {
                out.push(path);
            }
        }
    }
}
