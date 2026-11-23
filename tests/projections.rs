//! Projections — a link, a point or a zone drawn onto a map — are IFOPT's types,
//! and the schema puts their content in the IFOPT namespace even where a SIRI
//! element carries them.
//!
//! `LinkProjectionToNextStopPoint` is a SIRI element of type
//! `ifopt:LinkProjectionStructure`, so its children (`Features`, `Line`, the
//! points) are `ifopt:` elements; a stop place component names its projections
//! with the IFOPT elements themselves (`ifopt:PointProjection`, …). Releases up to
//! 2.4 wrote all of it in the SIRI namespace, which the schema rejects. Such
//! documents are still read; what is written is the schema's form.

mod support;

use quick_xml::events::Event;
use quick_xml::name::ResolveResult;
use quick_xml::NsReader;

use siri_rs::Siri;
use support::{compare, fixtures_dir, parse, validate, validator_available, VALIDATOR_MISSING};

const IFOPT: &str = "http://www.ifopt.org.uk/ifopt";
const SIRI: &str = "http://www.siri.org.uk/siri";

/// The derived documents that carry projections, by path under `derived/`.
const WITH_PROJECTIONS: &[&str] = &[
    "discovery/exd_lines_discoveryResponse_projection.xml",
    "sx/exx_situationExchange_response_projection.xml",
];

/// The elements a projection is carried in, whichever namespace they are in.
const PROJECTIONS: &[&str] = &[
    "LinkProjectionToNextStopPoint",
    "LinkProjection",
    "PointProjection",
    "ZoneProjection",
];

fn derived(name: &str) -> String {
    let path = fixtures_dir().join("derived").join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// The document as releases up to 2.4 wrote it: every projection element in the
/// SIRI namespace, the IFOPT prefix nowhere.
fn as_written_before(xml: &str) -> String {
    xml.replace("<ifopt:", "<").replace("</ifopt:", "</")
}

/// Every element of a document as `(path of local names, namespace)`.
fn elements(xml: &str) -> Vec<(String, String)> {
    let mut reader = NsReader::from_str(xml);
    let mut path: Vec<String> = Vec::new();
    let mut out = Vec::new();
    loop {
        let (namespace, event) = reader.read_resolved_event().expect("well-formed XML");
        let (start, empty) = match &event {
            Event::Start(start) => (start, false),
            Event::Empty(start) => (start, true),
            Event::End(_) => {
                path.pop();
                continue;
            }
            Event::Eof => break,
            _ => continue,
        };
        let namespace = match namespace {
            ResolveResult::Bound(namespace) => String::from_utf8_lossy(namespace.as_ref()).into_owned(),
            _ => String::new(),
        };
        path.push(String::from_utf8_lossy(start.local_name().as_ref()).into_owned());
        out.push((path.join("/"), namespace));
        if empty {
            path.pop();
        }
    }
    out
}

/// Whether the element at `path` is inside a projection — below the element
/// carrying it, not the element itself.
fn inside_a_projection(path: &str) -> bool {
    let mut names: Vec<&str> = path.split('/').collect();
    names.pop();
    names.iter().any(|name| PROJECTIONS.contains(name))
}

/// Checks the namespaces a written document puts its projections in: everything
/// inside one in IFOPT's, and the element carrying it in the namespace the schema
/// declares it in — IFOPT's for the elements a stop place component borrows from
/// IFOPT, SIRI's for every other.
fn assert_projections_in_the_schemas_namespaces(name: &str, xml: &str) {
    let mut projections = 0;
    for (path, namespace) in elements(xml) {
        let local = path.rsplit('/').next().expect("a path has a last name");
        if inside_a_projection(&path) {
            assert_eq!(
                namespace, IFOPT,
                "{name}: {path} is written outside the IFOPT namespace"
            );
        } else if PROJECTIONS.contains(&local) {
            projections += 1;
            let expected = if path.ends_with(&format!("AffectedComponent/{local}")) {
                IFOPT
            } else {
                SIRI
            };
            assert_eq!(namespace, expected, "{name}: {path} is written in the wrong namespace");
        }
    }
    assert!(projections > 0, "{name} carries no projection to check");
}

#[test]
fn a_projection_is_written_in_the_ifopt_namespace() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    for name in WITH_PROJECTIONS {
        let document: Siri = siri_rs::from_str(&derived(name)).unwrap_or_else(|e| panic!("{name} reads: {e}"));
        let written = siri_rs::to_string(&document).expect("the document is written");

        assert_projections_in_the_schemas_namespaces(name, &written);
        validate(&written).unwrap_or_else(|complaint| panic!("{name} is written invalid:\n{complaint}"));
    }
}

#[test]
fn a_projection_in_the_siri_namespace_still_reads_and_is_written_in_the_ifopt_one() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    for name in WITH_PROJECTIONS {
        let correct = derived(name);
        let before = as_written_before(&correct);
        assert!(
            validate(&before).is_err(),
            "{name}: the form written before should be one the schema rejects"
        );

        let from_before: Siri = siri_rs::from_str(&before).unwrap_or_else(|e| panic!("{name} as written before: {e}"));
        let from_correct: Siri = siri_rs::from_str(&correct).expect("the correct form reads");
        assert_eq!(from_before, from_correct, "{name}: the two forms read differently");

        let written = siri_rs::to_string(&from_before).expect("the document is written");
        compare(&parse(&correct), &parse(&written))
            .unwrap_or_else(|difference| panic!("{name} is not written in the schema's form: {difference}"));
        validate(&written).unwrap_or_else(|complaint| panic!("{name} is written invalid:\n{complaint}"));
    }
}

#[cfg(feature = "lenient")]
#[test]
fn the_lenient_reader_reads_a_projection_in_either_namespace() {
    for name in WITH_PROJECTIONS {
        let correct = derived(name);
        let (from_correct, findings) = siri_rs::lenient::from_str(&correct).expect("the correct form reads");
        assert!(findings.is_empty(), "{name}: {findings:?}");
        let (from_before, findings) =
            siri_rs::lenient::from_str(&as_written_before(&correct)).expect("the form written before reads");
        assert!(findings.is_empty(), "{name}: {findings:?}");
        assert_eq!(from_before, from_correct);
    }
}
