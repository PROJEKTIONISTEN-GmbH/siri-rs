//! Shared machinery for the conformance tests: fixtures, semantic XML comparison,
//! schema validation and — in [`http`] — a producer and a consumer on a real port.
//!
//! Cargo compiles this module separately into each integration-test binary, and no
//! single binary uses all of it.
#![allow(dead_code)]

pub mod http;

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::{DateTime, FixedOffset};
use quick_xml::events::Event;
use quick_xml::name::ResolveResult;
use quick_xml::NsReader;

/// Directory holding the official example documents and schemas.
pub fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// The root schema every SIRI document is validated against.
pub fn schema_path() -> PathBuf {
    fixtures_dir().join("xsd/siri.xsd")
}

/// One official example document.
pub struct Fixture {
    /// Path relative to `tests/fixtures/xml`, used as the test's label.
    pub name: String,
    /// Absolute path on disk.
    pub path: PathBuf,
    /// The document itself.
    pub xml: String,
}

/// Every example document under `tests/fixtures/xml`, in a stable order.
pub fn fixtures() -> Vec<Fixture> {
    let root = fixtures_dir().join("xml");
    let mut paths = Vec::new();
    collect_xml(&root, &mut paths);
    paths.sort();
    assert!(!paths.is_empty(), "no fixtures found under {}", root.display());
    paths
        .into_iter()
        .map(|path| Fixture {
            name: path
                .strip_prefix(&root)
                .expect("fixture lives under the fixture root")
                .to_string_lossy()
                .into_owned(),
            xml: std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display())),
            path,
        })
        .collect()
}

fn collect_xml(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
    {
        let path = entry.expect("readable directory entry").path();
        if path.is_dir() {
            collect_xml(&path, out);
        } else if path.extension().is_some_and(|e| e == "xml") {
            out.push(path);
        }
    }
}

/// Name of a document's root element, without its namespace prefix.
pub fn root_element(xml: &str) -> String {
    let mut reader = quick_xml::Reader::from_str(xml);
    loop {
        match reader.read_event().expect("fixture is well-formed XML") {
            Event::Start(e) | Event::Empty(e) => {
                return String::from_utf8_lossy(e.local_name().as_ref()).into_owned()
            }
            Event::Eof => panic!("document has no elements"),
            _ => {}
        }
    }
}

/// An XML element reduced to what the schema treats as content.
///
/// Namespace declarations and `xsi:*` attributes are dropped: they carry no
/// content, and `xsi:schemaLocation` in particular is a hint pointing at wherever
/// the author happened to keep the schemas. Everything that does carry content —
/// the resolved namespace and local name of each element, its attributes, its
/// character data and the order of its children — is kept.
#[derive(Debug, PartialEq)]
pub struct Element {
    name: String,
    attributes: BTreeMap<String, String>,
    text: String,
    children: Vec<Element>,
}

const XSI_NAMESPACE: &[u8] = b"http://www.w3.org/2001/XMLSchema-instance";

/// Parses a document into its content tree.
pub fn parse(xml: &str) -> Element {
    let mut reader = NsReader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut stack: Vec<Element> = Vec::new();
    let mut root = None;

    loop {
        let (namespace, event) = reader.read_resolved_event().expect("well-formed XML");
        match event {
            Event::Eof => break,
            Event::Start(ref start) => stack.push(start_element(&namespace, start)),
            Event::Empty(ref start) => {
                let element = start_element(&namespace, start);
                push(&mut stack, &mut root, element);
            }
            Event::End(_) => {
                let element = stack.pop().expect("end tag matches a start tag");
                push(&mut stack, &mut root, element);
            }
            Event::Text(text) => {
                if let Some(current) = stack.last_mut() {
                    let decoded = text.unescape().expect("decodable text").into_owned();
                    current.text.push_str(decoded.trim());
                }
            }
            Event::CData(cdata) => {
                if let Some(current) = stack.last_mut() {
                    let decoded = String::from_utf8_lossy(&cdata).into_owned();
                    current.text.push_str(decoded.trim());
                }
            }
            _ => {}
        }
    }

    root.expect("document has a root element")
}

fn push(stack: &mut [Element], root: &mut Option<Element>, element: Element) {
    match stack.last_mut() {
        Some(parent) => parent.children.push(element),
        None => *root = Some(element),
    }
}

fn start_element(namespace: &ResolveResult<'_>, start: &quick_xml::events::BytesStart<'_>) -> Element {
    let local = String::from_utf8_lossy(start.local_name().as_ref()).into_owned();
    let name = match namespace {
        ResolveResult::Bound(ns) => format!("{{{}}}{local}", String::from_utf8_lossy(ns.as_ref())),
        _ => local,
    };

    let mut attributes = BTreeMap::new();
    for attribute in start.attributes().with_checks(false) {
        let attribute = attribute.expect("well-formed attribute");
        let key = attribute.key;
        if key.as_ref() == b"xmlns" || key.prefix().is_some_and(|p| p.as_ref() == b"xmlns") {
            continue;
        }
        let (resolved, local) = namespace_of_attribute(key);
        if resolved == Some(XSI_NAMESPACE) {
            continue;
        }
        let value = attribute.unescape_value().expect("decodable attribute value");
        attributes.insert(local, value.trim().to_owned());
    }

    Element {
        name,
        attributes,
        text: String::new(),
        children: Vec::new(),
    }
}

/// Attribute namespaces are not inherited from the default declaration, so only a
/// prefix can put an attribute into one. The two prefixes SIRI documents use are
/// `xsi` and `xml`; both are bound by specification.
fn namespace_of_attribute(key: quick_xml::name::QName<'_>) -> (Option<&'static [u8]>, String) {
    let local = String::from_utf8_lossy(key.local_name().as_ref()).into_owned();
    match key.prefix().map(|p| p.as_ref().to_vec()) {
        Some(prefix) if prefix == b"xsi" => (Some(XSI_NAMESPACE), local),
        Some(prefix) => (None, format!("{}:{local}", String::from_utf8_lossy(&prefix))),
        None => (None, local),
    }
}

/// Compares two documents for content equality, returning a description of the
/// first difference.
///
/// Character data is compared by value rather than by spelling: two `xsd:dateTime`
/// literals denoting the same instant match even when written with different
/// fraction digits or time-zone spellings, and two numbers with the same value
/// match regardless of trailing zeroes. Any other difference is a difference.
pub fn compare(expected: &Element, actual: &Element) -> Result<(), String> {
    let mut path = String::new();
    compare_at(expected, actual, &mut path)
}

fn compare_at(expected: &Element, actual: &Element, path: &mut String) -> Result<(), String> {
    let _ = write!(path, "/{}", short_name(&expected.name));
    let here = path.clone();

    if expected.name != actual.name {
        return Err(format!(
            "{here}: element is <{}>, expected <{}>",
            actual.name, expected.name
        ));
    }
    if expected.attributes != actual.attributes {
        return Err(format!(
            "{here}: attributes are {:?}, expected {:?}",
            actual.attributes, expected.attributes
        ));
    }
    if !text_matches(&expected.text, &actual.text) {
        return Err(format!(
            "{here}: content is {:?}, expected {:?}",
            actual.text, expected.text
        ));
    }
    if expected.children.len() != actual.children.len() {
        return Err(format!(
            "{here}: has {} children {:?}, expected {} children {:?}",
            actual.children.len(),
            child_names(actual),
            expected.children.len(),
            child_names(expected),
        ));
    }

    for (expected_child, actual_child) in expected.children.iter().zip(&actual.children) {
        let mut child_path = here.clone();
        compare_at(expected_child, actual_child, &mut child_path)?;
    }
    Ok(())
}

fn short_name(name: &str) -> &str {
    name.rsplit('}').next().unwrap_or(name)
}

fn child_names(element: &Element) -> Vec<&str> {
    element.children.iter().map(|c| short_name(&c.name)).collect()
}

fn text_matches(expected: &str, actual: &str) -> bool {
    if expected == actual {
        return true;
    }
    if let (Ok(a), Ok(b)) = (
        DateTime::<FixedOffset>::parse_from_rfc3339(expected),
        DateTime::<FixedOffset>::parse_from_rfc3339(actual),
    ) {
        return a == b;
    }
    match (expected.parse::<f64>(), actual.parse::<f64>()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

/// Whether an XML Schema validator is available.
pub fn validator_available() -> bool {
    Command::new("xmllint")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

/// Message pointing a contributor at the missing validator.
pub const VALIDATOR_MISSING: &str = "\
`xmllint` is not on PATH, so the schema-validation tests cannot run.
Install it (Debian/Ubuntu: `apt install libxml2-utils`, macOS: `brew install libxml2`) and re-run.";

/// Validates a document against the SIRI schema, returning the validator's
/// complaint when it does not.
pub fn validate(xml: &str) -> Result<(), String> {
    let file = temp_file(xml);
    let output = Command::new("xmllint")
        .arg("--noout")
        .arg("--schema")
        .arg(schema_path())
        .arg(&file)
        .output()
        .expect("xmllint runs");
    let _ = std::fs::remove_file(&file);

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr)
            .replace(&file.to_string_lossy().to_string(), "<document>"))
    }
}

/// Writes `xml` to a uniquely named file next to the other test temporaries.
fn temp_file(xml: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let path = std::env::temp_dir().join(format!(
        "siri-conformance-{}-{}.xml",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::write(&path, xml).expect("temporary file is writable");
    path
}
