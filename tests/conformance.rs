//! Conformance harness.
//!
//! For every official SIRI example document under `tests/fixtures/xml` this
//! * reads it into the crate's types,
//! * writes those types back out,
//! * compares the result to the original element by element, and
//! * validates the result against the official schemas.
//!
//! A document that loses content, invents content, reorders content or fails
//! validation fails the suite. That is what this crate means by "conformant".

mod support;

use siri_rs::cm::ConnectionMonitoringCapabilitiesResponse;
use siri_rs::ct::ConnectionTimetableCapabilitiesResponse;
use siri_rs::et::EstimatedTimetableCapabilitiesResponse;
use siri_rs::fm::FacilityMonitoringCapabilitiesResponse;
use siri_rs::framework::SituationExchangeCapabilitiesResponse;
use siri_rs::gm::GeneralMessageCapabilitiesResponse;
use siri_rs::pt::ProductionTimetableCapabilitiesResponse;
use siri_rs::sm::{StopMonitoringCapabilitiesResponse, StopMonitoringPermissions};
use siri_rs::st::StopTimetableCapabilitiesResponse;
use siri_rs::sx::{PtSituationElement, RoadSituationElement};
use siri_rs::vm::VehicleMonitoringCapabilitiesResponse;
use siri_rs::Siri;
use support::{compare, parse, validate, validator_available, Fixture, VALIDATOR_MISSING};

/// Reads a document into `T` and writes it straight back out.
///
/// A token the crate does not recognise is kept and written back as it was, so a
/// field transcribed against the wrong enumeration would round-trip without a
/// trace; what is read is therefore checked for one before it is written.
fn rewrite<T: siri_rs::SiriRoot + std::fmt::Debug>(xml: &str) -> Result<String, String> {
    let document = siri_rs::from_str::<T>(xml).map_err(|error| error.to_string())?;
    let read = format!("{document:?}");
    if let Some(at) = read.find("Unrecognised(") {
        let end = read[at..].find(')').map_or(read.len(), |close| at + close + 1);
        return Err(format!("read a token the crate does not recognise: {}", &read[at..end]));
    }
    siri_rs::to_string_pretty(&document).map_err(|error| error.to_string())
}

/// Reads a document into the type its root element names, then writes it back out.
///
/// SIRI declares every message as a global element, so a document may be rooted at
/// something other than `<Siri>`. Every root the fixtures use is listed here; an
/// unlisted one is reported as a failure rather than skipped quietly.
fn round_trip(root: &str, xml: &str) -> Result<String, String> {
    let written = match root {
        "Siri" => rewrite::<Siri>(xml),
        "ConnectionMonitoringCapabilitiesResponse" => {
            rewrite::<ConnectionMonitoringCapabilitiesResponse>(xml)
        }
        "ConnectionTimetableCapabilitiesResponse" => {
            rewrite::<ConnectionTimetableCapabilitiesResponse>(xml)
        }
        "EstimatedTimetableCapabilitiesResponse" => {
            rewrite::<EstimatedTimetableCapabilitiesResponse>(xml)
        }
        "FacilityMonitoringCapabilitiesResponse" => {
            rewrite::<FacilityMonitoringCapabilitiesResponse>(xml)
        }
        "GeneralMessageCapabilitiesResponse" => rewrite::<GeneralMessageCapabilitiesResponse>(xml),
        "ProductionTimetableCapabilitiesResponse" => {
            rewrite::<ProductionTimetableCapabilitiesResponse>(xml)
        }
        "PtSituationElement" => rewrite::<PtSituationElement>(xml),
        "RoadSituationElement" => rewrite::<RoadSituationElement>(xml),
        "SituationExchangeCapabilitiesResponse" => {
            rewrite::<SituationExchangeCapabilitiesResponse>(xml)
        }
        "StopMonitoringCapabilitiesResponse" => rewrite::<StopMonitoringCapabilitiesResponse>(xml),
        "StopMonitoringPermissions" => rewrite::<StopMonitoringPermissions>(xml),
        "StopTimetableCapabilitiesResponse" => rewrite::<StopTimetableCapabilitiesResponse>(xml),
        "VehicleMonitoringCapabilitiesResponse" => {
            rewrite::<VehicleMonitoringCapabilitiesResponse>(xml)
        }
        other => {
            return Err(format!(
                "no document type is registered for root element <{other}>"
            ))
        }
    };
    written.map_err(|error| format!("cannot round-trip: {error}"))
}

/// Reads every document, writes it back and reports each one that lost, invented or
/// reordered content.
fn round_trip_differences(fixtures: Vec<Fixture>) -> Vec<String> {
    let mut failures = Vec::new();

    for fixture in fixtures {
        let root = support::root_element(&fixture.xml);
        let written = match round_trip(&root, &fixture.xml) {
            Ok(written) => written,
            Err(complaint) => {
                failures.push(format!("{}: {complaint}", fixture.name));
                continue;
            }
        };
        if let Err(difference) = compare(&parse(&fixture.xml), &parse(&written)) {
            failures.push(format!("{}: {difference}", fixture.name));
        }
    }

    failures
}

/// Reads every document, writes it back and reports each written document the
/// validator rejects.
fn written_documents_the_validator_rejects(fixtures: Vec<Fixture>) -> Vec<String> {
    let mut failures = Vec::new();

    for fixture in fixtures {
        let root = support::root_element(&fixture.xml);
        let written = match round_trip(&root, &fixture.xml) {
            Ok(written) => written,
            Err(complaint) => {
                failures.push(format!("{}: {complaint}", fixture.name));
                continue;
            }
        };
        if let Err(complaint) = validate(&written) {
            failures.push(format!("{}:\n{complaint}", fixture.name));
        }
    }

    failures
}

#[test]
fn every_official_example_round_trips_without_loss() {
    let failures = round_trip_differences(support::fixtures());
    assert!(
        failures.is_empty(),
        "{} of the official examples did not round-trip:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn every_official_example_is_written_back_as_schema_valid_xml() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let failures = written_documents_the_validator_rejects(support::fixtures());
    assert!(
        failures.is_empty(),
        "{} of the written documents did not validate:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// The derived documents cover content the official examples leave uncovered —
/// extension payloads and an embedded DATEX II record. They are held to the same
/// bar; `tests/fixtures/derived/README.md` says what each is derived from.
#[test]
fn every_derived_example_round_trips_without_loss() {
    let failures = round_trip_differences(support::derived_fixtures());
    assert!(
        failures.is_empty(),
        "{} of the derived examples did not round-trip:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn every_derived_example_is_written_back_as_schema_valid_xml() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let failures = written_documents_the_validator_rejects(support::derived_fixtures());
    assert!(
        failures.is_empty(),
        "{} of the written derived documents did not validate:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// The fixtures are valid SIRI before the crate touches them: the official ones
/// because they are the published examples unmodified — if they stopped validating,
/// the copy would have drifted from the published schemas — and the derived ones
/// because a fixture the schema rejects would prove nothing.
#[test]
fn the_fixtures_are_valid_siri_to_begin_with() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let mut failures = Vec::new();

    for fixture in support::fixtures().into_iter().chain(support::derived_fixtures()) {
        if let Err(complaint) = validate(&fixture.xml) {
            failures.push(format!("{}:\n{complaint}", fixture.name));
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The set of covered documents, so that losing one is a test failure rather than
/// a silently smaller suite. `tests/fixtures/README.md` records where they come
/// from and which official examples are deliberately not here.
#[test]
fn the_expected_documents_are_covered() {
    const EXPECTED: &[&str] = &[
        "capability/exd_allServices_capabilitiesRequest.xml",
        "cm/exc_connectionMonitoringDistributor_response.xml",
        "cm/exc_connectionMonitoringFeeder_response.xml",
        "cm/exc_connectionMonitoring_capabilitiesResponse.xml",
        "cm/exc_connectionMonitoring_request.xml",
        "cm/exc_connectionMonitoring_subscriptionRequest.xml",
        "ct/exc_connectionTimetable_capabilitiesResponse.xml",
        "ct/exc_connectionTimetable_request.xml",
        "ct/exc_connectionTimetable_response.xml",
        "ct/exc_connectionTimetable_subscriptionRequest.xml",
        "discovery/exd_lines_discoveryRequest.xml",
        "discovery/exd_lines_discoveryResponse.xml",
        "discovery/exd_productCategories_discoveryRequest.xml",
        "discovery/exd_productCategories_discoveryResponse.xml",
        "discovery/exd_serviceFeatures_discoveryRequest.xml",
        "discovery/exd_serviceFeatures_discoveryResponse.xml",
        "discovery/exd_stopPoints_discoveryRequest.xml",
        "discovery/exd_stopPoints_discoveryResponse.xml",
        "discovery/exd_vehicleFeatures_discoveryRequest.xml",
        "discovery/exd_vehicleFeatures_discoveryResponse.xml",
        "et/ext_estimatedTimetable_capabilitiesResponse.xml",
        "et/ext_estimatedTimetable_request.xml",
        "et/ext_estimatedTimetable_response.xml",
        "et/ext_estimatedTimetable_subscriptionRequest.xml",
        "fm/exf_facilityMonitoring_capabilitiesResponse.xml",
        "fm/exf_facilityMonitoring_request.xml",
        "fm/exf_facilityMonitoring_response.xml",
        "fm/exf_facilityMonitoring_subscriptionRequest.xml",
        "framework/exa_checkStatus_request.xml",
        "framework/exa_checkStatus_response.xml",
        "framework/exa_dataReady_request.xml",
        "framework/exa_dataReady_response.xml",
        "framework/exa_dataReceived_response.xml",
        "framework/exa_dataSupply_request.xml",
        "framework/exa_heartbeat_request.xml",
        "framework/exa_requestSubscription_response.xml",
        "framework/exa_subscriptionTerminated_notification.xml",
        "framework/exa_terminateSubscription_request.xml",
        "framework/exa_terminateSubscription_response.xml",
        "framework/exa_terminateSubscription_response_err.xml",
        "gm/exm_generalMessage_capabilityResponse.xml",
        "gm/exm_generalMessage_request.xml",
        "gm/exm_generalMessage_response.xml",
        "gm/exm_generalMessage_response_embed.xml",
        "gm/exm_generalMessage_subscriptionRequest.xml",
        "pt/ext_productionTimetable_capabilitiesResponse.xml",
        "pt/ext_productionTimetable_request.xml",
        "pt/ext_productionTimetable_response.xml",
        "pt/ext_productionTimetable_subscriptionRequest.xml",
        "sm/exp_stopMonitoring_permissions.xml",
        "sm/exs_stopMonitoring_capabilitiesResponse.xml",
        "sm/exs_stopMonitoring_request.xml",
        "sm/exs_stopMonitoring_request_simple.xml",
        "sm/exs_stopMonitoring_response.xml",
        "sm/exs_stopMonitoring_response_complex.xml",
        "sm/exs_stopMonitoring_response_simple.xml",
        "sm/exs_stopMonitoring_subscriptionRequest.xml",
        "sm/exs_stopMonitoring_subscriptionRequest_simple.xml",
        "st/exs_stopTimetable_capabilitiesResponse.xml",
        "st/exs_stopTimetable_request.xml",
        "st/exs_stopTimetable_response.xml",
        "st/exs_stopTimetable_subscriptionRequest.xml",
        "sx/exx_situationExchangeResponse.xml",
        "sx/exx_situationExchange_ATOC.xml",
        "sx/exx_situationExchange_Pt.xml",
        "sx/exx_situationExchange_capabilityResponse.xml",
        "sx/exx_situationExchange_request.xml",
        "sx/exx_situationExchange_request_simple.xml",
        "sx/exx_situationExchange_response.xml",
        "sx/exx_situationExchange_road.xml",
        "sx/exx_situationExchange_subscriptionRequest.xml",
        "sx/vdv736/SX_1010_first_message.xml",
        "sx/vdv736/SX_1022_main_message.xml",
        "sx/vdv736/SX_1135_main_message_update.xml",
        "sx/vdv736/SX_1247_end_message.xml",
        "vm/exv_vehicleMonitoring_capabilitiesResponse.xml",
        "vm/exv_vehicleMonitoring_request.xml",
        "vm/exv_vehicleMonitoring_request_simple.xml",
        "vm/exv_vehicleMonitoring_response.xml",
        "vm/exv_vehicleMonitoring_response_simple.xml",
        "vm/exv_vehicleMonitoring_responsex_simple.xml",
        "vm/exv_vehicleMonitoring_subscriptionRequest.xml",
    ];

    let present: Vec<String> = support::fixtures()
        .into_iter()
        .map(|fixture| fixture.name.replace('\\', "/"))
        .collect();
    assert_eq!(present, EXPECTED);
}

/// The derived documents, listed for the same reason: losing one would quietly
/// shrink the suite. `tests/fixtures/derived/README.md` records their provenance.
#[test]
fn the_expected_derived_documents_are_covered() {
    const EXPECTED: &[&str] = &[
        "framework/exa_checkStatus_request_extensions.xml",
        "sx/exx_situationExchange_road_datex.xml",
    ];

    let present: Vec<String> = support::derived_fixtures()
        .into_iter()
        .map(|fixture| fixture.name.replace('\\', "/"))
        .collect();
    assert_eq!(present, EXPECTED);
}

/// A document rooted at a different message must not be read into the wrong type.
#[test]
fn reading_the_wrong_message_type_is_an_error() {
    let heartbeat = r#"<Siri xmlns="http://www.siri.org.uk/siri">
        <HeartbeatNotification>
            <RequestTimestamp>2004-12-17T09:30:47-05:00</RequestTimestamp>
        </HeartbeatNotification>
    </Siri>"#;
    let error = siri_rs::from_str::<RoadSituationElement>(heartbeat).unwrap_err();
    assert!(
        matches!(&error, siri_rs::Error::UnexpectedRoot { expected, found }
            if *expected == "RoadSituationElement" && found == "Siri"),
        "{error}"
    );
}

mod comparator {
    use super::support::{compare, parse};

    #[test]
    fn namespace_prefixes_do_not_affect_equality() {
        let default_binding = r#"<Siri xmlns="http://www.siri.org.uk/siri"><A>1</A></Siri>"#;
        let prefixed = r#"<s:Siri xmlns:s="http://www.siri.org.uk/siri"><s:A>1</s:A></s:Siri>"#;
        assert_eq!(compare(&parse(default_binding), &parse(prefixed)), Ok(()));
    }

    #[test]
    fn namespace_uris_do_affect_equality() {
        let siri = r#"<Siri xmlns="http://www.siri.org.uk/siri"><A>1</A></Siri>"#;
        let other = r#"<Siri xmlns="urn:elsewhere"><A>1</A></Siri>"#;
        assert!(compare(&parse(siri), &parse(other)).is_err());
    }

    #[test]
    fn schema_hints_are_not_content_but_other_attributes_are() {
        let with_hint = concat!(
            r#"<Siri xmlns="http://www.siri.org.uk/siri" "#,
            r#"xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" "#,
            r#"xsi:schemaLocation="http://www.siri.org.uk/siri x.xsd" version="2.0"/>"#
        );
        let without = r#"<Siri xmlns="http://www.siri.org.uk/siri" version="2.0"/>"#;
        assert_eq!(compare(&parse(with_hint), &parse(without)), Ok(()));

        let other_version = r#"<Siri xmlns="http://www.siri.org.uk/siri" version="2.1"/>"#;
        assert!(compare(&parse(without), &parse(other_version)).is_err());
    }

    #[test]
    fn comments_and_indentation_are_not_content() {
        let pretty = "<Siri>\n\t<!-- a note -->\n\t<A>1</A>\n</Siri>";
        let dense = "<Siri><A>1</A></Siri>";
        assert_eq!(compare(&parse(pretty), &parse(dense)), Ok(()));
    }

    #[test]
    fn element_order_is_content() {
        let one = "<Siri><A>1</A><B>2</B></Siri>";
        let other = "<Siri><B>2</B><A>1</A></Siri>";
        assert!(compare(&parse(one), &parse(other)).is_err());
    }

    #[test]
    fn timestamps_are_compared_as_instants() {
        let fractional = "<Siri><T>2001-12-17T09:30:47.0Z</T></Siri>";
        let plain = "<Siri><T>2001-12-17T09:30:47+00:00</T></Siri>";
        assert_eq!(compare(&parse(fractional), &parse(plain)), Ok(()));

        let elsewhere = "<Siri><T>2001-12-17T09:30:47-05:00</T></Siri>";
        assert!(compare(&parse(plain), &parse(elsewhere)).is_err());
    }

    #[test]
    fn differing_text_is_a_difference() {
        let one = "<Siri><Progress>open</Progress></Siri>";
        let other = "<Siri><Progress>closed</Progress></Siri>";
        assert!(compare(&parse(one), &parse(other)).is_err());
    }

    #[test]
    fn a_missing_element_is_reported_with_its_path() {
        let expected = parse("<Siri><A><B>1</B><C>2</C></A></Siri>");
        let actual = parse("<Siri><A><B>1</B></A></Siri>");
        let error = compare(&expected, &actual).unwrap_err();
        assert!(error.starts_with("/Siri/A:"), "{error}");
        assert!(error.contains("expected 2 children"), "{error}");
    }
}
