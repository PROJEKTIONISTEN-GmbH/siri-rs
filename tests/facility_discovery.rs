//! Facility discovery: a consumer asking which facilities a producer serves, and
//! the producer's answer.
//!
//! The standard publishes no example of this exchange, so the documents read here
//! are derived ones (`tests/fixtures/derived/README.md`); the round trip and the
//! validation of those documents are the conformance suite's. What is checked here
//! is that what a document says arrives in the fields it belongs to, and that a
//! message built in code — rather than read — is one the schemas accept.

mod support;

use chrono::{DateTime, FixedOffset};

use siri_rs::enumerations::{AccessFacility, Accessibility, FacilityCategory, FacilityDetail};
use siri_rs::framework::{FacilityDelivery, FacilityRequest, FacilityScope};
use siri_rs::model::{
    AnnotatedFacility, BoundingBox, Facility, FacilityFeature, FacilityFeatureKind, FacilityFeatures,
    FacilityLimitations, FacilityLocation, FacilityRef, LineRef, Location, OperatorRef, PlaceRef, StopPointRef,
};
use siri_rs::types::NaturalLanguageString;
use siri_rs::Siri;
use support::{fixtures_dir, validate, validator_available, VALIDATOR_MISSING};

fn instant(text: &str) -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339(text).expect("valid instant")
}

fn derived(name: &str) -> Siri {
    let path = fixtures_dir().join("derived/discovery").join(name);
    let xml = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    siri_rs::from_str(&xml).unwrap_or_else(|e| panic!("{name} reads: {e}"))
}

/// Writes a document, checks that the schemas accept it, and reads it back.
fn written_valid_and_read_back(document: &Siri) -> Siri {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let xml = siri_rs::to_string(document).expect("the document is written");
    validate(&xml).unwrap_or_else(|complaint| panic!("the written document is invalid:\n{complaint}\n{xml}"));
    siri_rs::from_str(&xml).expect("the written document reads back")
}

#[test]
fn a_facility_request_carries_its_filters_and_the_detail_it_asks_for() {
    let document = derived("exd_facility_discoveryRequest.xml");
    let request = document
        .payload
        .as_facility_request()
        .expect("the document is a facility request");

    assert_eq!(request.version.as_deref(), Some("2.0"));
    assert_eq!(request.request_timestamp, instant("2004-12-17T09:30:47-05:00"));
    assert_eq!(request.requestor_ref, "NADER".into());
    let area = BoundingBox::new(Location::wgs84(0.1, 53.1), Location::wgs84(0.2, 53.2));
    match request.scope() {
        Some(FacilityScope::BoundingBox(read)) => {
            assert_eq!(read.upper_left.longitude, area.upper_left.longitude);
            assert_eq!(read.lower_right.latitude, area.lower_right.latitude);
        }
        other => panic!("expected the request to narrow to a rectangle, got {other:?}"),
    }
    assert_eq!(request.operator_ref, Some(OperatorRef::new("RATP")));
    assert_eq!(request.line_ref, Some(LineRef::new("LINE4")));
    assert_eq!(request.language, ["fr", "en"]);
    assert_eq!(request.facility_detail_level, Some(FacilityDetail::Full));
}

#[test]
fn a_facility_delivery_describes_a_facility_or_names_it_alone() {
    let document = derived("exd_facility_discoveryResponse.xml");
    let delivery = document
        .payload
        .as_facility_delivery()
        .expect("the document is a facility delivery");

    assert_eq!(delivery.status, Some(true));
    assert_eq!(delivery.valid_until, Some(instant("2004-12-18T09:30:47-05:00")));
    assert_eq!(delivery.annotated_facility.len(), 2);

    let described = &delivery.annotated_facility[0];
    assert_eq!(described.facility_ref, FacilityRef::new("134567-L4"));
    assert_eq!(described.monitored, Some(true));
    let facility = described.facility.as_ref().expect("the first facility is described");
    assert_eq!(facility.facility_class, [FacilityCategory::FixedEquipment]);
    let location = facility.facility_location.as_ref().expect("the facility is located");
    assert_eq!(location.stop_point_ref, Some(StopPointRef::new("HGALLES")));
    let limitations = facility
        .limitations
        .as_ref()
        .expect("the facility states its limitations");
    assert_eq!(limitations.wheelchair_access, Accessibility::True);
    assert_eq!(limitations.step_free_access, Some(Accessibility::True));

    let named = &delivery.annotated_facility[1];
    assert_eq!(named.facility_ref, FacilityRef::new("134567-E2"));
    assert_eq!(named.monitored, Some(false));
    assert_eq!(named.facility, None);
}

#[test]
fn a_facility_request_built_in_code_is_valid_and_reads_back() {
    let mut request = FacilityRequest::new(instant("2026-11-23T08:00:00+01:00"), "NADER");
    assert_eq!(request.scope(), None);

    request.place_ref = Some(PlaceRef::new("HANNOVER"));
    request.line_ref = Some(LineRef::new("10"));
    request.facility_detail_level = Some(FacilityDetail::Minimum);
    assert_eq!(request.scope(), Some(FacilityScope::Place(&PlaceRef::new("HANNOVER"))));

    let document = Siri::new("2.1", request.clone());
    let read = written_valid_and_read_back(&document);
    assert_eq!(read.payload.as_facility_request(), Some(&request));
}

#[test]
fn a_facility_delivery_built_in_code_is_valid_and_reads_back() {
    let mut lift = AnnotatedFacility::new("LIFT-KROEPCKE-2");
    lift.monitored = Some(true);
    lift.facility = Some(Facility {
        facility_code: Some("LIFT-KROEPCKE-2".to_owned()),
        description: vec![NaturalLanguageString::new("Aufzug Gleis 2")],
        facility_class: vec![FacilityCategory::FixedEquipment],
        features: Some(FacilityFeatures {
            feature: vec![FacilityFeature::new(FacilityFeatureKind::AccessFacility(
                AccessFacility::Lift,
            ))],
        }),
        facility_location: Some(FacilityLocation {
            stop_point_ref: Some(StopPointRef::new("de:03241:31")),
            ..FacilityLocation::default()
        }),
        limitations: Some(FacilityLimitations::new(Accessibility::True)),
        ..Facility::default()
    });
    let delivery = FacilityDelivery::new(
        instant("2026-11-23T08:00:01+01:00"),
        vec![lift, AnnotatedFacility::new("ESCALATOR-KROEPCKE-1")],
    );
    assert_eq!(delivery.status, Some(true));

    let document = Siri::new("2.1", delivery.clone());
    let read = written_valid_and_read_back(&document);
    assert_eq!(read.payload.as_facility_delivery(), Some(&delivery));
}

#[test]
fn a_facility_delivery_without_facilities_is_valid() {
    // The schema lets a producer that serves no facility say so: `AnnotatedFacility`
    // is optional, so an answer of none is an answer rather than an error.
    let delivery = FacilityDelivery::new(instant("2026-11-23T08:00:01+01:00"), Vec::new());
    let read = written_valid_and_read_back(&Siri::new("2.1", delivery.clone()));
    assert_eq!(read.payload.as_facility_delivery(), Some(&delivery));
}
