//! Repeated elements whose content is an enumerated token must survive a round trip.
//!
//! The schema writes a repeated enumeration as a sequence of sibling elements, e.g.
//! `<StopCondition>startPoint</StopCondition><StopCondition>destination</StopCondition>`.
//! Read back naively, the deserialiser offers the *element name* as the variant and
//! rejects the document. Every field of this shape is covered here, so that a new one
//! added without the token-list reader fails a test rather than a user's feed.

use chrono::{DateTime, FixedOffset};
use siri::enumerations::{
    DayType, FacilityStatus, RoutePointType, ScopeType, ServiceCondition, WorkflowStatus,
};
use siri::model::{DataFrameRef, DatedVehicleJourneyRef, FramedVehicleJourneyRef};
use siri::sx::affects::{
    AffectedCall, AffectedFacility, AffectedStopPoint, AffectedVehicleJourney,
};
use siri::sx::consequence::Consequence;
use siri::sx::request::SituationExchangeRequest;
use siri::sx::situation::SituationRepetitions;

/// Serialises `value` under `root` and reads it back, asserting nothing changed.
fn round_trip<T>(root: &str, value: &T) -> String
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let xml = quick_xml::se::to_string_with_root(root, value).expect("value serialises");
    let read: T = quick_xml::de::from_str(&xml)
        .unwrap_or_else(|error| panic!("{root} reads back from {xml}: {error}"));
    assert_eq!(&read, value, "{root} survives the round trip: {xml}");
    xml
}

fn timestamp() -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339("2004-12-17T09:30:47-05:00").expect("valid timestamp")
}

#[test]
fn an_affected_stop_point_keeps_every_stop_condition_it_lists() {
    let mut stop = AffectedStopPoint::new("HLTST002");
    stop.stop_condition = vec![RoutePointType::StartPoint, RoutePointType::Destination];

    let xml = round_trip("AffectedStopPoint", &stop);
    assert!(
        xml.contains("<StopCondition>startPoint</StopCondition>"),
        "stop conditions are written as their tokens: {xml}"
    );
}

#[test]
fn an_affected_vehicle_journey_keeps_every_journey_condition_it_lists() {
    let mut journey = AffectedVehicleJourney::framed(FramedVehicleJourneyRef {
        data_frame_ref: DataFrameRef::new("2004-12-17"),
        dated_vehicle_journey_ref: DatedVehicleJourneyRef::new("VJ1"),
    });
    journey.journey_condition = vec![ServiceCondition::Cancelled, ServiceCondition::Delayed];

    round_trip("AffectedVehicleJourney", &journey);
}

#[test]
fn an_affected_facility_keeps_every_status_it_lists() {
    let mut facility = AffectedFacility::default();
    facility.facility_status = vec![FacilityStatus::NotAvailable, FacilityStatus::PartiallyAvailable];

    round_trip("AffectedFacility", &facility);
}

#[test]
fn an_affected_call_keeps_both_of_its_condition_lists() {
    let mut call = AffectedCall::new("HLTST002");
    call.stop_condition = vec![RoutePointType::StartPoint, RoutePointType::NotStopping];
    call.call_condition = vec![RoutePointType::Destination];

    let xml = round_trip("AffectedCall", &call);
    assert!(
        xml.contains("<CallCondition>destination</CallCondition>"),
        "the two lists stay distinct: {xml}"
    );
}

#[test]
fn a_consequence_keeps_every_service_condition_it_lists() {
    let mut consequence = Consequence::default();
    consequence.condition = vec![ServiceCondition::Disrupted, ServiceCondition::Diverted];

    round_trip("Consequence", &consequence);
}

#[test]
fn a_request_keeps_every_scope_and_progress_value_it_filters_on() {
    let mut request = SituationExchangeRequest::new(timestamp());
    request.scope = vec![ScopeType::Line, ScopeType::StopPlace];
    request.progress = vec![WorkflowStatus::Open, WorkflowStatus::Published];

    let xml = round_trip("SituationExchangeRequest", &request);
    assert!(
        xml.contains("<Scope>line</Scope><Scope>stopPlace</Scope>"),
        "scopes are written as sibling elements: {xml}"
    );
}

#[test]
fn a_repetition_keeps_every_day_type_it_lists() {
    let repetitions = SituationRepetitions::new(vec![DayType::Saturday, DayType::Sunday]);

    let xml = round_trip("Repetitions", &repetitions);
    assert!(
        xml.contains("<DayType>saturday</DayType><DayType>sunday</DayType>"),
        "day types are written as sibling elements: {xml}"
    );
}
