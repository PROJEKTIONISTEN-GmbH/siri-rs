//! Elements a producer leaves empty, and what the reader makes of them.
//!
//! An empty position list is valid GML — `gml:posList` is a list type, and a list
//! may hold nothing. An empty free text is not valid SIRI — the text types require
//! at least one character — but real feeds carry one now and then. Neither is a
//! reason to lose the document around it: each is read as an empty `value`, and
//! written back exactly as it stands.
//!
//! Every case below is one of the official example documents with a single element
//! emptied, so that "the rest of the document is read" can be checked against the
//! whole of it rather than against the field next door.

mod support;

use siri_rs::framework::ServiceDeliveryPayload;
use siri_rs::model::PosList;
use siri_rs::types::DefaultedText;
use siri_rs::{NaturalLanguageString, Siri};
use support::{compare, fixtures_dir, parse, validate, validator_available, VALIDATOR_MISSING};

/// The summary of the first situation in the Situation Exchange example response.
const SUMMARY: &str = r#"<Summary overridden="true" xml:lang="EN">Bomb at Barchester station</Summary>"#;

/// The line name of the first vehicle in the simple Vehicle Monitoring example response.
const PUBLISHED_LINE_NAME: &str = r#"<PublishedLineName xml:lang="EN">123</PublishedLineName>"#;

/// The arrival platform of the second call in the Estimated Timetable example
/// response, after which the schema lets a stop assignment follow.
const ARRIVAL_PLATFORM: &str = r#"<ArrivalPlatformName xml:lang="EN">4</ArrivalPlatformName>"#;

/// The three ways of writing an element with no content, and the language tag each
/// of them carries.
fn empty_spellings(element: &str) -> [(String, Option<&'static str>); 3] {
    [
        (format!("<{element}/>"), None),
        (format!("<{element}></{element}>"), None),
        (format!(r#"<{element} xml:lang="DE"/>"#), Some("DE")),
    ]
}

/// An official example document with one element replaced.
fn fixture_with(path: &str, original: &str, replacement: &str) -> String {
    let xml = std::fs::read_to_string(fixtures_dir().join("xml").join(path))
        .unwrap_or_else(|e| panic!("cannot read fixture {path}: {e}"));
    assert!(xml.contains(original), "{path} carries {original}");
    xml.replacen(original, replacement, 1)
}

/// The Estimated Timetable example with a stop assignment whose flexible area is a
/// polygon drawn with the given position list.
fn estimated_timetable_with_positions(pos_list: &str) -> String {
    let assignment = format!(
        "{ARRIVAL_PLATFORM}<ArrivalStopAssignment><AimedFlexibleArea>\
         <gml:Polygon xmlns:gml=\"http://www.opengis.net/gml/3.2\" gml:id=\"area-1\">\
         <gml:exterior><gml:LinearRing>{pos_list}</gml:LinearRing></gml:exterior>\
         </gml:Polygon></AimedFlexibleArea></ArrivalStopAssignment>"
    );
    fixture_with("et/ext_estimatedTimetable_response.xml", ARRIVAL_PLATFORM, &assignment)
}

/// Reads a document, writes it back, and checks that the written document carries
/// all the content the read one did.
fn read_whole(xml: &str) -> Siri {
    let document: Siri = siri_rs::from_str(xml).expect("the document reads");
    let written = siri_rs::to_string(&document).expect("the document writes");
    compare(&parse(xml), &parse(&written)).expect("nothing is lost between reading and writing");
    document
}

fn deliveries(document: &Siri) -> &[ServiceDeliveryPayload] {
    &document
        .payload
        .as_service_delivery()
        .expect("the document is a service delivery")
        .deliveries
}

/// The summaries of every situation in the delivery, public-transport ones first.
fn summaries(document: &Siri) -> Vec<&DefaultedText> {
    let Some(ServiceDeliveryPayload::SituationExchangeDelivery(delivery)) = deliveries(document).first()
    else {
        panic!("the delivery is a situation exchange delivery");
    };
    let situations = delivery.situations.as_ref().expect("the delivery carries situations");
    let pt = situations.pt_situation_element.iter().flat_map(|situation| &situation.summary);
    let road = situations.road_situation_element.iter().flat_map(|situation| &situation.summary);
    pt.chain(road).collect()
}

fn published_line_names(document: &Siri) -> Vec<&NaturalLanguageString> {
    let Some(ServiceDeliveryPayload::VehicleMonitoringDelivery(delivery)) = deliveries(document).first()
    else {
        panic!("the delivery is a vehicle monitoring delivery");
    };
    delivery
        .vehicle_activity
        .iter()
        .flat_map(|activity| &activity.monitored_vehicle_journey.published_line_name)
        .collect()
}

fn flexible_area_positions(document: &Siri) -> &PosList {
    let Some(ServiceDeliveryPayload::EstimatedTimetableDelivery(delivery)) = deliveries(document).first()
    else {
        panic!("the delivery is an estimated timetable delivery");
    };
    let call = delivery
        .estimated_journey_version_frame
        .iter()
        .flat_map(|frame| &frame.estimated_vehicle_journey)
        .flat_map(|journey| journey.estimated_calls.iter().flat_map(|calls| &calls.estimated_call))
        .find(|call| !call.arrival_stop_assignment.is_empty())
        .expect("one call carries a stop assignment");
    let polygon = call.arrival_stop_assignment[0]
        .aimed_flexible_area
        .as_ref()
        .and_then(|area| area.polygon.as_ref())
        .expect("the assignment names a polygon");
    &polygon
        .exterior
        .as_ref()
        .expect("the polygon has an outer boundary")
        .linear_ring
        .pos_list
}

#[test]
fn an_empty_summary_is_read_as_empty_text_and_the_situations_around_it_are_kept() {
    for (empty, lang) in empty_spellings("Summary") {
        let xml = fixture_with("sx/exx_situationExchange_response.xml", SUMMARY, &empty);
        let document = read_whole(&xml);

        let summaries = summaries(&document);
        assert_eq!(summaries.len(), 2, "{empty}: the road situation keeps its summary too");
        assert_eq!(summaries[0].value, "", "{empty}");
        assert_eq!(summaries[0].lang.as_deref(), lang, "{empty}");
        assert_eq!(summaries[0].overridden, None, "{empty}");
        assert_eq!(summaries[1].value, "Jam on the access road to Barchester Station", "{empty}");
    }
}

#[test]
fn an_empty_line_name_is_read_as_empty_text_and_the_vehicles_around_it_are_kept() {
    for (empty, lang) in empty_spellings("PublishedLineName") {
        let xml = fixture_with(
            "vm/exv_vehicleMonitoring_response_simple.xml",
            PUBLISHED_LINE_NAME,
            &empty,
        );
        let document = read_whole(&xml);

        let names = published_line_names(&document);
        assert_eq!(names.len(), 1, "{empty}: only the first vehicle names its line");
        assert_eq!(names[0].value, "", "{empty}");
        assert_eq!(names[0].lang.as_deref(), lang, "{empty}");
    }
}

#[test]
fn an_empty_position_list_is_valid_and_is_read_and_written_as_one() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");

    let spellings = [
        ("<gml:posList/>", None),
        ("<gml:posList></gml:posList>", None),
        (r#"<gml:posList srsDimension="2"/>"#, Some(2)),
    ];
    for (empty, srs_dimension) in spellings {
        let xml = estimated_timetable_with_positions(empty);
        validate(&xml).unwrap_or_else(|e| panic!("{empty}: an empty list is valid GML: {e}"));

        let document = read_whole(&xml);
        let positions = flexible_area_positions(&document);
        assert_eq!(positions.value, "", "{empty}");
        assert_eq!(positions.srs_dimension, srs_dimension, "{empty}");
        assert_eq!(positions.count, None, "{empty}");

        let written = siri_rs::to_string(&document).expect("the document writes");
        validate(&written).unwrap_or_else(|e| panic!("{empty}: what was valid stays valid: {e}"));
    }
}

#[test]
fn an_empty_text_is_written_as_it_stands_even_though_the_schema_refuses_it() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");

    let xml = fixture_with("sx/exx_situationExchange_response.xml", SUMMARY, "<Summary/>");
    let document = read_whole(&xml);
    let written = siri_rs::to_string(&document).expect("the document writes");

    assert_eq!(summaries(&siri_rs::from_str(&written).expect("the written document reads"))[0].value, "");
    let complaint = validate(&written).expect_err("an empty text is not valid SIRI");
    assert!(complaint.contains("minLength"), "the schema names the rule: {complaint}");
}

#[test]
fn text_of_whitespace_alone_reads_like_an_empty_element() {
    // The XML reader drops character data that is nothing but whitespace, so there
    // is no text left to read: the element is as good as empty.
    let xml = fixture_with(
        "sx/exx_situationExchange_response.xml",
        SUMMARY,
        "<Summary>   </Summary>",
    );
    assert_eq!(summaries(&read_whole(&xml))[0].value, "");
}

#[test]
fn text_with_surrounding_whitespace_keeps_what_it_read_before() {
    // The XML reader trims character data at the element's edges and leaves the
    // inside alone. Reading an empty element tolerantly adds no trimming of its own.
    let xml = fixture_with(
        "sx/exx_situationExchange_response.xml",
        SUMMARY,
        "<Summary>  Bomb  at Barchester  </Summary>",
    );
    assert_eq!(summaries(&read_whole(&xml))[0].value, "Bomb  at Barchester");
}
