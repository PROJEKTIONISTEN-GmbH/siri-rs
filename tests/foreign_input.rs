//! What a document from the other side may do to a reader, and what the reader owes
//! whoever runs it when it cannot read one.
//!
//! None of the documents here is SIRI a producer would send on purpose: one nests
//! deeper than any path through the schemas, one carries an enumeration token this
//! crate's schema release does not list, one leaves a time-zone offset out, one
//! lacks a field ten records in. A library at an open port meets all of them, and
//! the answer to each has to be an `Err` that says where it came from — never an
//! abort, never a document read thinner than it was written.

mod support;

use chrono::{DateTime, FixedOffset};

use siri_rs::framework::SubscriptionResponse;
use siri_rs::pubsub::{Producer, ProducerConfig, SituationSource};
use siri_rs::sx::{PtSituationElement, SituationExchangeRequest};
use siri_rs::Siri;
use support::{validate, validator_available, VALIDATOR_MISSING};

/// The declarations every root element below opens with.
const ROOT: &str = r#"xmlns="http://www.siri.org.uk/siri" version="2.0""#;

/// The stack a tokio worker thread runs on by default, which is what an endpoint
/// built on it reads documents with.
const WORKER_STACK: usize = 2 * 1024 * 1024;

/// How deep the review's document nested: 4 000 elements in under 30 KB.
const HOSTILE_DEPTH: usize = 4_000;

fn now() -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339("2026-03-14T08:00:00+01:00").expect("valid instant")
}

/// A well-formed check-status request whose extension payload nests `depth` deep.
fn nested_extensions(depth: usize) -> String {
    let mut xml = format!(
        "<Siri {ROOT}><CheckStatusRequest>\
         <RequestTimestamp>2004-12-17T09:30:47-05:00</RequestTimestamp>\
         <RequestorRef>EREWHON</RequestorRef><Extensions>"
    );
    xml.extend(std::iter::repeat("<a>").take(depth));
    xml.extend(std::iter::repeat("</a>").take(depth));
    xml.push_str("</Extensions></CheckStatusRequest></Siri>");
    xml
}

#[test]
fn a_document_nested_deeper_than_any_schema_path_is_refused_not_read() {
    let xml = nested_extensions(HOSTILE_DEPTH);
    let outcome = std::thread::Builder::new()
        .stack_size(WORKER_STACK)
        .spawn(move || siri_rs::from_str::<Siri>(&xml).map(|_| ()))
        .expect("the thread starts")
        .join()
        .expect("the reader returns rather than overflowing its stack");
    assert!(
        outcome.is_err(),
        "a document {HOSTILE_DEPTH} elements deep is not one any schema describes"
    );
}

#[test]
fn a_token_the_schema_does_not_list_survives_the_round_trip_unchanged() {
    // A situation whose severity a later schema release, or a national profile,
    // may well add. Losing the whole document over it would lose the 499 other
    // situations in a delivery with it.
    let xml = format!(
        "<PtSituationElement {ROOT}>\
         <CreationTime>2026-03-04T07:50:00+01:00</CreationTime>\
         <SituationNumber>2026-0041</SituationNumber>\
         <Source><SourceType>feed</SourceType></Source>\
         <ValidityPeriod><StartTime>2026-03-04T07:50:00+01:00</StartTime></ValidityPeriod>\
         <EquipmentReason>liftFailure</EquipmentReason>\
         <Severity>apocalyptic</Severity>\
         </PtSituationElement>"
    );

    let situation: PtSituationElement = siri_rs::from_str(&xml)
        .expect("one token the schema does not list does not cost the document");
    let written = siri_rs::to_string(&situation).expect("the situation is writable");
    assert!(
        written.contains("<Severity>apocalyptic</Severity>"),
        "the token is written back as it was read:\n{written}"
    );
}

#[test]
fn a_missing_field_is_reported_with_the_path_to_the_record_that_lacks_it() {
    // Two visits at a stop; the second has no RecordedAtTime. "missing field
    // RecordedAtTime" alone is no help in a delivery of twenty thousand visits.
    let visit = |recorded_at: &str| {
        format!(
            "<MonitoredStopVisit>{recorded_at}\
             <MonitoringRef>HLTST011</MonitoringRef>\
             <MonitoredVehicleJourney><LineRef>10</LineRef></MonitoredVehicleJourney>\
             </MonitoredStopVisit>"
        )
    };
    let xml = format!(
        "<Siri {ROOT}><ServiceDelivery>\
         <ResponseTimestamp>2026-03-04T08:15:00+01:00</ResponseTimestamp>\
         <StopMonitoringDelivery>\
         <ResponseTimestamp>2026-03-04T08:15:00+01:00</ResponseTimestamp>\
         {}{}\
         </StopMonitoringDelivery></ServiceDelivery></Siri>",
        visit("<RecordedAtTime>2026-03-04T08:14:00+01:00</RecordedAtTime>"),
        visit(""),
    );

    let error = siri_rs::from_str::<Siri>(&xml).expect_err("a visit without RecordedAtTime");
    let message = error.to_string();
    assert!(message.contains("RecordedAtTime"), "names the field: {message}");
    assert!(
        message.contains("MonitoredStopVisit[1]"),
        "names the record that lacks it, and which one: {message}"
    );
}

#[test]
fn a_timestamp_without_an_offset_is_refused_and_the_refusal_names_the_field() {
    // `xsd:dateTime` lets the offset be left out, and a document that does so is
    // schema-valid; a RequestTimestamp without one names no instant, though, and
    // the crate refuses it. What it owes the reader is to say which field.
    let xml = format!(
        "<Siri {ROOT}><CheckStatusRequest>\
         <RequestTimestamp>2004-12-17T09:30:47</RequestTimestamp>\
         <RequestorRef>EREWHON</RequestorRef>\
         </CheckStatusRequest></Siri>"
    );
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    validate(&xml).expect("the schema admits a dateTime without an offset");

    let error = siri_rs::from_str::<Siri>(&xml).expect_err("no offset, no instant");
    let message = error.to_string();
    assert!(message.contains("RequestTimestamp"), "names the field: {message}");
}

struct Nothing;

impl SituationSource for Nothing {
    fn situations(&self, _request: &SituationExchangeRequest) -> Vec<PtSituationElement> {
        Vec::new()
    }
}

#[test]
fn a_fetch_that_names_no_consumer_is_refused_as_a_missing_element() {
    let mut producer = Producer::new(ProducerConfig::new("MY-AGENCY"), Nothing);
    let fetch: Siri = siri_rs::from_str(&format!(
        "<Siri {ROOT}><DataSupplyRequest>\
         <RequestTimestamp>2026-03-14T08:00:00+01:00</RequestTimestamp>\
         </DataSupplyRequest></Siri>"
    ))
    .expect("the schema leaves ConsumerRef optional");

    let error = producer
        .handle(&fetch, now())
        .expect_err("a producer cannot tell whose subscriptions to supply");
    let message = error.to_string();
    assert!(
        message.contains("DataSupplyRequest") && message.contains("ConsumerRef"),
        "names the message and the element it lacks: {message}"
    );
    assert!(
        !message.contains("not a valid"),
        "an element that is absent is not a lexical value that is wrong: {message}"
    );
}

#[test]
fn a_message_a_producer_cannot_answer_is_named_as_a_message_not_a_root_element() {
    let mut producer = Producer::new(ProducerConfig::new("MY-AGENCY"), Nothing);
    // A subscription response is what a producer sends, never what it receives.
    let response = Siri::new(
        "2.1",
        SubscriptionResponse::new(now(), "OTHER-AGENCY", Vec::new()),
    );

    let error = producer
        .handle(&response, now())
        .expect_err("a producer has no answer to another producer's answer");
    let message = error.to_string();
    assert!(message.contains("SubscriptionResponse"), "names the message: {message}");
    assert!(
        !message.contains("root element"),
        "the document's root was <Siri>, which is the right one: {message}"
    );
}
