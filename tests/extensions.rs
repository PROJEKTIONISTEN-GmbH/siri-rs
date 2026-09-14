//! What survives a round trip through an `<Extensions>` element, and through the
//! DATEX II records SIRI-SX embeds.
//!
//! Both are content the schema leaves to the participants: `ExtensionsStructure` is
//! an `xsd:any` wildcard, and the road-situation elements are typed by the imported
//! DATEX II schema. The crate models neither, so what it owes them is to carry them
//! through unchanged — which is what this checks, at the level of a whole document.

mod support;

use serde::{Deserialize, Serialize};
use siri_rs::sx::RoadSituationElement;
use siri_rs::types::Extensions;
use siri_rs::{CheckStatusRequest, Siri};
use support::{compare, parse, validate, validator_available, VALIDATOR_MISSING};

/// The derived fixture whose `<Extensions>` element carries a payload.
fn extensions_fixture() -> String {
    read_fixture("derived/framework/exa_checkStatus_request_extensions.xml")
}

/// The derived fixture whose `<Road>` element carries a DATEX II location.
fn datex_fixture() -> String {
    read_fixture("derived/sx/exx_situationExchange_road_datex.xml")
}

fn read_fixture(name: &str) -> String {
    let path = support::fixtures_dir().join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// The payload of the fixture's `<Extensions>`, as the crate reads it.
fn extensions_of_the_fixture() -> Extensions {
    let message: Siri = siri_rs::from_str(&extensions_fixture()).expect("the fixture reads");
    message
        .payload
        .as_check_status_request()
        .expect("the fixture is a status request")
        .extensions
        .clone()
        .expect("the request carries extensions")
}

#[test]
fn an_extension_payload_reaches_the_reader_whole() {
    let extensions = extensions_of_the_fixture();

    assert_eq!(
        extensions
            .children_named("ProfileVersion")
            .next()
            .map(|version| version.text.as_str()),
        Some("1.4")
    );

    let settings = extensions
        .children_named("OperatorSettings")
        .next()
        .expect("the payload carries the operator settings");
    assert_eq!(settings.attribute("scope"), Some("regional"));
    let values: Vec<(&str, &str)> = settings
        .children_named("Setting")
        .map(|setting| {
            (
                setting.attribute("name").expect("a setting is named"),
                setting.text.as_str(),
            )
        })
        .collect();
    assert_eq!(values, [("MaximumAge", "15"), ("Language", "EN")]);
}

#[test]
fn an_extension_payload_survives_the_round_trip_the_conformance_suite_measures() {
    let original = extensions_fixture();
    let written = siri_rs::to_string_pretty(
        &siri_rs::from_str::<Siri>(&original).expect("the fixture reads"),
    )
    .expect("the message writes");

    compare(&parse(&original), &parse(&written)).expect("the payload comes back unchanged");
    assert!(written.contains("<Setting name=\"MaximumAge\" unit=\"minutes\">15</Setting>"));
}

/// A payload may be qualified — by a prefix or by a default declaration — and the
/// namespace is part of what it means, so it has to come back resolving the same way.
#[test]
fn a_qualified_extension_payload_keeps_its_namespace() {
    let extensions = extensions_of_the_fixture();

    let diagnostics = extensions
        .children_named("Diagnostics")
        .next()
        .expect("the payload carries the prefixed subtree");
    assert_eq!(
        diagnostics.attribute("xmlns"),
        Some("http://example.org/siri/extension"),
        "the prefix binding has to survive as a namespace the writer can restate"
    );
    assert_eq!(
        diagnostics
            .children_named("Counter")
            .next()
            .map(|counter| counter.text.as_str()),
        Some("17")
    );

    let telemetry = extensions
        .children_named("Telemetry")
        .next()
        .expect("the payload carries the default-declared subtree");
    assert_eq!(
        telemetry.attribute("xmlns"),
        Some("http://example.org/siri/telemetry")
    );
}

#[test]
fn an_embedded_datex_record_reaches_the_reader_whole() {
    let situation: RoadSituationElement =
        siri_rs::from_str(&datex_fixture()).expect("the fixture reads");

    let road = situation
        .affects
        .as_ref()
        .and_then(|affects| affects.roads.as_ref())
        .and_then(|roads| roads.affected_road.first())
        .and_then(|affected| affected.road.as_ref())
        .expect("the situation affects a road");

    let location = road
        .children_named("roadsideReferencePointPrimaryLocation")
        .next()
        .expect("the DATEX record names a primary location");
    assert_eq!(
        location.attribute("xmlns"),
        Some("http://datex2.eu/schema/2_0RC1/2_0"),
        "the DATEX namespace is what makes this a DATEX record"
    );

    let primary = location
        .children_named("roadsideReferencePoint")
        .next()
        .expect("the primary location names a reference point");
    assert_eq!(
        primary.attribute("xmlns"),
        None,
        "a descendant in the same namespace inherits the declaration above it"
    );
    let identifiers: Vec<&str> = primary
        .children_named("roadsideReferencePointIdentifier")
        .map(|id| id.text.as_str())
        .collect();
    assert_eq!(identifiers, ["A255-KM-3"]);
}

#[test]
fn an_embedded_datex_record_survives_the_round_trip_and_stays_schema_valid() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");

    let original = datex_fixture();
    let written = siri_rs::to_string_pretty(
        &siri_rs::from_str::<RoadSituationElement>(&original).expect("the fixture reads"),
    )
    .expect("the situation writes");

    compare(&parse(&original), &parse(&written)).expect("the DATEX record comes back unchanged");
    validate(&written).expect("the written document is valid SIRI");
}

/// The settings the derived fixture's payload carries, as a consumer that knows the
/// profile would declare them. Nothing in the crate knows this type: the profile is
/// the participants' business, and the seam is all the crate owes it.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct OperatorSettings {
    #[serde(rename = "@scope")]
    scope: String,
    #[serde(rename = "Setting")]
    settings: Vec<Setting>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Setting {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@unit", default, skip_serializing_if = "Option::is_none")]
    unit: Option<String>,
    #[serde(rename = "$text")]
    value: String,
}

fn operator_settings() -> OperatorSettings {
    OperatorSettings {
        scope: "regional".to_owned(),
        settings: vec![
            Setting {
                name: "MaximumAge".to_owned(),
                unit: Some("minutes".to_owned()),
                value: "15".to_owned(),
            },
            Setting {
                name: "Language".to_owned(),
                unit: None,
                value: "EN".to_owned(),
            },
        ],
    }
}

#[test]
fn a_payload_a_consumer_knows_reads_into_a_type_of_its_own() {
    let extensions = extensions_of_the_fixture();
    let subtree = extensions
        .children_named("OperatorSettings")
        .next()
        .expect("the payload carries the operator settings");

    let settings: OperatorSettings = subtree.parse().expect("the payload reads into the type");
    assert_eq!(settings, operator_settings());

    assert_eq!(
        &Extensions::from_payload(&settings).expect("the value writes back into a subtree"),
        subtree,
        "what the typed value holds is what the subtree held"
    );
}

/// The seam in the producing direction: a payload built from a consumer's own type
/// reaches the wire as the elements that type describes.
#[test]
fn a_payload_built_from_a_consumers_type_is_written_into_the_document() {
    let mut request = CheckStatusRequest::new(
        "2004-12-17T09:30:47-05:00".parse().expect("a valid instant"),
        "EREWHON",
    );
    let mut extensions = Extensions::default();
    extensions.children.push((
        "OperatorSettings".to_owned(),
        Extensions::from_payload(&operator_settings()).expect("the settings write into a subtree"),
    ));
    request.extensions = Some(extensions);

    let written = siri_rs::to_string(&Siri::new("2.0", request)).expect("the message writes");

    assert!(
        written.contains(concat!(
            r#"<Extensions><OperatorSettings scope="regional">"#,
            r#"<Setting name="MaximumAge" unit="minutes">15</Setting>"#,
            r#"<Setting name="Language">EN</Setting>"#,
            "</OperatorSettings></Extensions>"
        )),
        "{written}"
    );
}

/// A prefix on an attribute inside a payload is the one thing that does not come
/// back: the reader reports an attribute by its local name, so `xsi:type="…"` is
/// indistinguishable from `type="…"` by the time the crate sees it. This pins that
/// boundary — a release that closes it should change this test rather than discover
/// the behaviour in the field. `types::AnyContent` documents it too.
#[test]
fn a_prefix_on_an_attribute_inside_a_payload_is_not_carried_back() {
    let document = concat!(
        r#"<Siri xmlns="http://www.siri.org.uk/siri" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">"#,
        r#"<CheckStatusRequest><RequestTimestamp>2004-12-17T09:30:47-05:00</RequestTimestamp>"#,
        r#"<RequestorRef>EREWHON</RequestorRef>"#,
        r#"<Extensions><Record xsi:type="Accident"/></Extensions>"#,
        "</CheckStatusRequest></Siri>"
    );

    let written = siri_rs::to_string(&siri_rs::from_str::<Siri>(document).expect("the document reads"))
        .expect("the message writes");

    assert!(written.contains(r#"<Record type="Accident"/>"#), "{written}");
}

/// The schema makes `<Extensions>` one of the alternatives directly under `<Siri>`,
/// not a trailer after a message: a document carrying only extensions is valid, and
/// one carrying a message *and* extensions is not.
#[test]
fn a_document_that_carries_only_extensions_is_read() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let document = concat!(
        r#"<Siri xmlns="http://www.siri.org.uk/siri" version="2.0">"#,
        r#"<Extensions><Diagnostics level="verbose"><Counter>17</Counter></Diagnostics></Extensions>"#,
        "</Siri>"
    );
    validate(document).expect("the schema admits a document of extensions alone");

    let message: Siri = siri_rs::from_str(document).expect("the document reads");
    let written = siri_rs::to_string(&message).expect("the message writes");
    validate(&written).expect("what is written back is still valid");
    assert!(written.contains("<Counter>17</Counter>"), "{written}");
}
