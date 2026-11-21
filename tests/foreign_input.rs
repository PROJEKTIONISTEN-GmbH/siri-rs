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
    assert!(message.contains("time-zone offset"), "says why: {message}");
}

#[test]
fn the_end_of_a_day_written_as_hour_24_is_refused_and_the_refusal_says_why() {
    // XML Schema 1.0 spells midnight at the end of a day `24:00:00`; chrono has no
    // such hour. The refusal has to say that, not that the input is out of range.
    let xml = format!(
        "<Siri {ROOT}><CheckStatusRequest>\
         <RequestTimestamp>2004-12-17T24:00:00-05:00</RequestTimestamp>\
         <RequestorRef>EREWHON</RequestorRef>\
         </CheckStatusRequest></Siri>"
    );
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    validate(&xml).expect("the schema admits the end of a day as hour 24");

    let message = siri_rs::from_str::<Siri>(&xml).expect_err("hour 24 is not read").to_string();
    assert!(message.contains("RequestTimestamp"), "names the field: {message}");
    assert!(message.contains("24:00:00"), "says why: {message}");
}

#[test]
fn a_timestamp_that_reads_names_its_instant_as_before() {
    let xml = format!(
        "<Siri {ROOT}><CheckStatusRequest>\
         <RequestTimestamp>2004-12-17T09:30:47.250-05:00</RequestTimestamp>\
         <RequestorRef>EREWHON</RequestorRef>\
         </CheckStatusRequest></Siri>"
    );
    let siri: Siri = siri_rs::from_str(&xml).expect("a timestamp with an offset reads");
    let request = siri.payload.as_check_status_request().expect("a check-status request");
    assert_eq!(
        request.request_timestamp,
        DateTime::parse_from_rfc3339("2004-12-17T09:30:47.250-05:00").expect("valid instant")
    );
}

/// A heartbeat in the shape every document below varies, with `root` as the root
/// element's start tag.
fn heartbeat_under(root: &str, end: &str) -> String {
    format!(
        "{root}<HeartbeatNotification>\
         <RequestTimestamp>2004-12-17T09:30:47-05:00</RequestTimestamp>\
         <ProducerRef>KUBRICK</ProducerRef>\
         </HeartbeatNotification>{end}"
    )
}

#[test]
fn a_document_in_another_namespace_or_in_none_is_refused() {
    for (label, root, end) in [
        ("another namespace", r#"<Siri xmlns="urn:not-siri" version="2.0">"#, "</Siri>"),
        ("no namespace", r#"<Siri version="2.0">"#, "</Siri>"),
        ("another namespace by prefix", r#"<s:Siri xmlns:s="urn:not-siri" version="2.0">"#, "</s:Siri>"),
    ] {
        let xml = heartbeat_under(root, end);
        let error = siri_rs::from_str::<Siri>(&xml).expect_err(label);
        assert!(
            matches!(&error, siri_rs::Error::UnexpectedNamespace { element: "Siri", .. }),
            "{label}: {error}"
        );
        #[cfg(feature = "lenient")]
        assert!(
            matches!(
                siri_rs::lenient::from_str(&xml),
                Err(siri_rs::Error::UnexpectedNamespace { element: "Siri", .. })
            ),
            "{label}: the lenient reader refuses it too"
        );
    }

    let error = siri_rs::from_str::<Siri>(&heartbeat_under(r#"<Siri xmlns="urn:not-siri">"#, "</Siri>"))
        .expect_err("another namespace");
    let message = error.to_string();
    assert!(message.contains("urn:not-siri") && message.contains(siri_rs::NAMESPACE), "{message}");
}

#[test]
fn the_siri_namespace_reads_whether_it_is_the_default_or_bound_to_a_prefix() {
    let by_default = heartbeat_under(&format!("<Siri {ROOT}>"), "</Siri>");
    let by_prefix = concat!(
        r#"<s:Siri xmlns:s="http://www.siri.org.uk/siri" version="2.0"><s:HeartbeatNotification>"#,
        "<s:RequestTimestamp>2004-12-17T09:30:47-05:00</s:RequestTimestamp>",
        "<s:ProducerRef>KUBRICK</s:ProducerRef>",
        "</s:HeartbeatNotification></s:Siri>"
    );
    let expected = siri_rs::from_str::<Siri>(&by_default).expect("the default binding reads");
    assert_eq!(siri_rs::from_str::<Siri>(by_prefix).expect("a prefix binding reads"), expected);
}

#[test]
fn nothing_but_comments_and_processing_instructions_may_follow_the_root_element() {
    let document = heartbeat_under(&format!("<Siri {ROOT}>"), "</Siri>");
    for tail in ["\n", "\n<!-- delivered at 09:30 -->\n", "<?trace id=\"7\"?>\n"] {
        let xml = format!("{document}{tail}");
        siri_rs::from_str::<Siri>(&xml).unwrap_or_else(|e| panic!("{tail:?}: {e}"));
    }
    for tail in ["<Siri/>", "<Other/>", "trailing text"] {
        let xml = format!("{document}{tail}");
        let error = siri_rs::from_str::<Siri>(&xml).expect_err(tail);
        assert!(
            matches!(&error, siri_rs::Error::ContentAfterRoot { offset: Some(offset) } if *offset == document.len() as u64),
            "{tail}: {error}"
        );
        #[cfg(feature = "lenient")]
        assert!(
            matches!(siri_rs::lenient::from_str(&xml), Err(siri_rs::Error::ContentAfterRoot { .. })),
            "{tail}: the lenient reader refuses it too"
        );
    }
}

#[test]
fn a_character_xml_does_not_allow_is_not_written() {
    let started = now();
    for value in ["P\u{1}Q", "P\u{1b}Q", "P\u{fffe}Q"] {
        let heartbeat = siri_rs::framework::HeartbeatNotification::healthy(now(), value, started);
        let error = siri_rs::to_string(&Siri::new("2.0", heartbeat.clone())).expect_err(value);
        let illegal = value.chars().nth(1).expect("the second character is the illegal one");
        assert!(
            matches!(error, siri_rs::Error::IllegalCharacter { character, .. } if character == illegal),
            "{value:?}: {error}"
        );
        assert!(siri_rs::to_string_pretty(&Siri::new("2.0", heartbeat)).is_err(), "{value:?}");
    }

    for allowed in ["P\tQ\r\nR", "P\u{e9}\u{fffd}\u{10ffff}"] {
        let heartbeat = siri_rs::framework::HeartbeatNotification::healthy(now(), allowed, started);
        let xml = siri_rs::to_string(&Siri::new("2.0", heartbeat)).expect("XML allows these characters");
        assert!(xml.contains(allowed), "written as they are: {xml}");
    }
}

#[test]
fn a_character_xml_does_not_allow_is_not_read() {
    let document = |producer: &str| heartbeat_under(&format!("<Siri {ROOT}>"), "</Siri>").replace("KUBRICK", producer);
    for (producer, character) in [("P\u{1}Q", '\u{1}'), ("P&#1;Q", '\u{1}'), ("P&#x1F;Q", '\u{1f}'), ("P\u{ffff}Q", '\u{ffff}')] {
        let xml = document(producer);
        let error = siri_rs::from_str::<Siri>(&xml).expect_err(producer);
        let at = xml.find(producer).expect("the producer is in the document") as u64 + 1;
        assert!(
            matches!(error, siri_rs::Error::IllegalCharacter { character: found, offset } if found == character && offset == at),
            "{producer:?}: {error}"
        );
    }
    for producer in ["P&#9;Q", "P&#xE9;Q", "P&amp;Q"] {
        siri_rs::from_str::<Siri>(&document(producer)).unwrap_or_else(|e| panic!("{producer}: {e}"));
    }
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
