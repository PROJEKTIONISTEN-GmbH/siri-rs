//! What the schema says an element written empty means, and that the crate reads
//! it so.
//!
//! XML Schema lets an element declaration carry a `default` or a `fixed` value. An
//! element so declared that is written with no content — `<Monitored/>` — is valid
//! and means the declared value. Those declarations are read out of the bundled
//! schemas here and rendered into `src/xml/schema_default.rs`, which is compared
//! with what is checked in; every field that transcribes such a declaration is
//! checked to read its value; and every value the crate can read is read out of a
//! real document, written back, and validated.

mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use quick_xml::events::{BytesStart, Event};
use siri_rs::enumerations::Severity;
use siri_rs::framework::{ServiceDeliveryPayload, ServiceRequestPayload};
use siri_rs::gm::GeneralMessageCapabilitiesResponse;
use siri_rs::sm::StopMonitoringRequest;
use siri_rs::{Siri, SiriRoot};
use support::{
    compare, fixtures_dir, parse, round_trip, validate_fixture, validator_available, Fixture,
    VALIDATOR_MISSING,
};

/// Where the rendered table lives, relative to the crate root.
const GENERATED: &str = "src/xml/schema_default.rs";

/// Set to rewrite the table instead of comparing it:
/// `UPDATE_SCHEMA_DEFAULT=1 cargo test --test schema_default`.
const UPDATE: &str = "UPDATE_SCHEMA_DEFAULT";

/// The module a field names to read a declared value.
const MODULE: &str = "crate::xml::schema_default";

/// The functional services, as the schema files name them and as the crate's
/// modules and the fixture directories do.
const SERVICES: &[(&str, &str)] = &[
    ("connectionMonitoring", "cm"),
    ("connectionTimetable", "ct"),
    ("controlAction", "ca"),
    ("estimatedTimetable", "et"),
    ("facilityMonitoring", "fm"),
    ("generalMessage", "gm"),
    ("productionTimetable", "pt"),
    ("situationExchange", "sx"),
    ("stopMonitoring", "sm"),
    ("stopTimetable", "st"),
    ("vehicleMonitoring", "vm"),
];

/// What kind of value a declaration's type holds, which decides how the crate reads
/// it and which Rust types can transcribe it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Class {
    Boolean,
    Integer,
    Decimal,
    Duration,
    String,
    Enumeration,
}

impl Class {
    fn name(self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
            Self::Integer => "integer",
            Self::Decimal => "decimal",
            Self::Duration => "duration",
            Self::String => "string",
            Self::Enumeration => "enumeration",
        }
    }
}

/// One element declaration that carries a value constraint.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Declaration {
    element: String,
    /// The schema file, relative to `tests/fixtures/xsd`.
    file: String,
    /// The innermost named type, group or element the declaration sits in; empty for
    /// a global element.
    context: String,
    /// The type as the declaration writes it, e.g. `xsd:boolean`.
    xsd_type: String,
    class: Class,
    fixed: bool,
    value: String,
}

impl Declaration {
    /// The type's local name, without a namespace prefix.
    fn type_name(&self) -> &str {
        local_name(&self.xsd_type)
    }

    /// The service the declaring file belongs to, when it is a service schema.
    fn service(&self) -> Option<&'static str> {
        service_of(&self.file)
    }
}

/// The service whose schema, module or fixture directory a path belongs to.
fn service_of(path: &str) -> Option<&'static str> {
    SERVICES
        .iter()
        .find(|(schema, module)| {
            path.contains(&format!("siri_{schema}_service.xsd"))
                || path.starts_with(&format!("{module}/"))
                || path.starts_with(&format!("src/{module}/"))
        })
        .map(|(_, module)| *module)
}

fn local_name(qualified: &str) -> &str {
    qualified.rsplit(':').next().unwrap_or(qualified)
}

/// A named `xsd:simpleType`: what it restricts, and its tokens when it is an
/// enumeration.
#[derive(Debug, Default)]
struct SimpleType {
    base: Option<String>,
    tokens: Vec<String>,
}

/// The named types the schemas define, as far as classifying a value needs them.
#[derive(Debug, Default)]
struct Types {
    simple: BTreeMap<String, SimpleType>,
    /// Complex types with simple content, by name, and the type their content has.
    content: BTreeMap<String, String>,
}

impl Types {
    fn class_of(&self, xsd_type: &str) -> Class {
        let name = local_name(xsd_type);
        if xsd_type.starts_with("xsd:") || xsd_type.starts_with("xs:") {
            return match name {
                "boolean" => Class::Boolean,
                "integer" | "nonNegativeInteger" | "positiveInteger" | "int" | "long"
                | "short" | "unsignedInt" | "unsignedLong" | "unsignedShort" => Class::Integer,
                "decimal" | "float" | "double" => Class::Decimal,
                "duration" => Class::Duration,
                "string" | "normalizedString" | "token" | "language" | "NMTOKEN" | "anyURI" => {
                    Class::String
                }
                other => panic!("no class for the schema primitive xsd:{other}"),
            };
        }
        if let Some(simple) = self.simple.get(name) {
            if !simple.tokens.is_empty() {
                return Class::Enumeration;
            }
            let base = simple
                .base
                .as_deref()
                .unwrap_or_else(|| panic!("simple type {name} neither enumerates nor restricts"));
            return self.class_of(base);
        }
        if let Some(base) = self.content.get(name) {
            return self.class_of(base);
        }
        panic!("cannot classify the schema type {xsd_type}: no such named type");
    }

    /// The tokens of an enumerated type, following restrictions until the list.
    fn tokens_of(&self, xsd_type: &str) -> &[String] {
        let simple = self
            .simple
            .get(local_name(xsd_type))
            .unwrap_or_else(|| panic!("{xsd_type} is not a named simple type"));
        if simple.tokens.is_empty() {
            let base = simple.base.as_deref().expect("a simple type without tokens restricts");
            self.tokens_of(base)
        } else {
            &simple.tokens
        }
    }
}

/// Every schema file the table is read from: the SIRI schema set with the
/// namespaces it builds on. Left out are the W3C schema for schemas, which is not
/// a document grammar, and DATEX II, which the crate carries as opaque content.
fn schema_files() -> Vec<PathBuf> {
    let root = fixtures_dir().join("xsd");
    let mut out = Vec::new();
    collect(&root, &mut out);
    out.sort();
    return out;

    fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("readable schema directory") {
            let path = entry.expect("readable directory entry").path();
            let name = path.file_name().map(|n| n.to_string_lossy().into_owned());
            if path.is_dir() {
                if name.as_deref() != Some("xml") && name.as_deref() != Some("datex2") {
                    collect(&path, out);
                }
            } else if path.extension().is_some_and(|e| e == "xsd") {
                out.push(path);
            }
        }
    }
}

/// Reads every element declaration that carries a `default` or a `fixed` value out
/// of the schemas, and the named types needed to say what kind of value it is.
fn read_schemas() -> (Vec<Declaration>, Types) {
    let root = fixtures_dir().join("xsd");
    let mut declarations = Vec::new();
    let mut types = Types::default();

    for path in schema_files() {
        let file = path
            .strip_prefix(&root)
            .expect("schema lives under the schema root")
            .to_string_lossy()
            .replace('\\', "/");
        let xml = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        read_schema(&file, &xml, &mut declarations, &mut types);
    }

    declarations.sort();
    declarations.dedup();
    (declarations, types)
}

fn read_schema(file: &str, xml: &str, declarations: &mut Vec<Declaration>, types: &mut Types) {
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut depth = 0usize;
    // The named types, groups and elements open at this point, with their depth.
    let mut named: Vec<(usize, String)> = Vec::new();
    // The named simple type being read, with its depth.
    let mut simple: Option<(usize, String, SimpleType)> = None;
    // The complex type whose simple content is being read.
    let mut content_of: Option<String> = None;

    loop {
        let event = reader.read_event().expect("schema is well-formed XML");
        match &event {
            Event::Eof => break,
            Event::Start(start) | Event::Empty(start) => {
                if matches!(event, Event::Start(_)) {
                    depth += 1;
                }
                let tag = String::from_utf8_lossy(start.local_name().as_ref()).into_owned();
                let name = attribute(start, "name");
                match tag.as_str() {
                    "element" => {
                        let constraint = attribute(start, "default")
                            .map(|value| (false, value))
                            .or_else(|| attribute(start, "fixed").map(|value| (true, value)));
                        if let (Some(element), Some((fixed, value))) = (name.clone(), constraint) {
                            let xsd_type = attribute(start, "type").unwrap_or_else(|| {
                                panic!("{file}: <{element}> has a default but no named type")
                            });
                            declarations.push(Declaration {
                                element,
                                file: file.to_owned(),
                                context: named.last().map(|(_, n)| n.clone()).unwrap_or_default(),
                                xsd_type,
                                class: Class::Boolean, // classified once every type is read
                                fixed,
                                value,
                            });
                        }
                    }
                    "simpleType" => {
                        if let Some(name) = name.clone() {
                            simple = Some((depth, name, SimpleType::default()));
                        }
                    }
                    "simpleContent" => content_of = named.last().map(|(_, n)| n.clone()),
                    "restriction" | "extension" | "list" => {
                        let base = attribute(start, "base").or_else(|| attribute(start, "itemType"));
                        if let (Some((_, _, simple)), Some(base)) = (simple.as_mut(), base.clone()) {
                            simple.base.get_or_insert(base);
                        } else if let (Some(complex), Some(base)) = (content_of.as_ref(), base) {
                            types.content.insert(complex.clone(), base);
                        }
                    }
                    "enumeration" => {
                        if let (Some((_, _, simple)), Some(value)) =
                            (simple.as_mut(), attribute(start, "value"))
                        {
                            if !simple.tokens.contains(&value) {
                                simple.tokens.push(value);
                            }
                        }
                    }
                    _ => {}
                }
                if matches!(event, Event::Start(_)) {
                    if let (Some(name), "complexType" | "group" | "element" | "attributeGroup") =
                        (name, tag.as_str())
                    {
                        named.push((depth, name));
                    }
                }
            }
            Event::End(end) => {
                if named.last().is_some_and(|(at, _)| *at == depth) {
                    named.pop();
                }
                if simple.as_ref().is_some_and(|(at, _, _)| *at == depth) {
                    let (_, name, definition) = simple.take().expect("checked just above");
                    types.simple.insert(name, definition);
                }
                if end.local_name().as_ref() == b"simpleContent" {
                    content_of = None;
                }
                depth -= 1;
            }
            _ => {}
        }
    }
}

fn attribute(start: &BytesStart<'_>, name: &str) -> Option<String> {
    start
        .attributes()
        .with_checks(false)
        .flatten()
        .find(|attribute| attribute.key.as_ref() == name.as_bytes())
        .map(|attribute| String::from_utf8_lossy(&attribute.value).trim().to_owned())
}

/// The declarations, classified, and the types they were classified with.
fn declarations() -> (Vec<Declaration>, Types) {
    let (mut declarations, types) = read_schemas();
    for declaration in &mut declarations {
        declaration.class = types.class_of(&declaration.xsd_type);
    }
    declarations.sort();
    (declarations, types)
}

/// The name of the function that reads one declared value.
fn function_name(class: Class, value: &str) -> String {
    let mut name = String::from(class.name());
    name.push('_');
    let mut previous_lower = false;
    for c in value.chars() {
        if c.is_ascii_alphanumeric() {
            if c.is_ascii_uppercase() && previous_lower {
                name.push('_');
            }
            name.push(c.to_ascii_lowercase());
            previous_lower = c.is_ascii_lowercase();
        } else if !name.ends_with('_') {
            name.push('_');
            previous_lower = false;
        }
    }
    name
}

/// Renders the table as the module the crate compiles.
fn render(declarations: &[Declaration], used: &BTreeSet<(Class, String)>) -> String {
    let files: BTreeSet<&str> = declarations.iter().map(|d| d.file.as_str()).collect();
    let mut by_value: BTreeMap<(Class, String), Vec<&Declaration>> = BTreeMap::new();
    for declaration in declarations {
        by_value
            .entry((declaration.class, declaration.value.clone()))
            .or_default()
            .push(declaration);
    }

    let mut out = String::new();
    let _ = write!(
        out,
        "\
//! The value the schema gives an element that is written empty, declaration by
//! declaration.
//!
//! Generated from the schemas under `tests/fixtures/xsd` by `tests/schema_default.rs`;
//! after a schema change, regenerate it with
//! `{UPDATE}=1 cargo test --test schema_default`. Do not edit by hand: the test
//! fails when this file and the schemas disagree.
//!
//! XML Schema lets an element declaration carry a `default` or a `fixed` value. An
//! element so declared that is written with no content — `<Monitored/>`,
//! `<Monitored></Monitored>` — is valid and means the declared value. Each function
//! below reads one declared value that way, through
//! [`Defaulted`](super::defaulted::Defaulted), and a field carries the function of
//! the declaration it transcribes. What the value means for an element declared with
//! and without a default, and how it is written back, is said there.
//!
//! Attribute defaults are not listed: an attribute's default applies when the
//! attribute is absent, and an absent attribute reads as `None`.
//!
//! {} declarations in {} schema files, {} distinct values.

use serde::Deserializer;

use super::defaulted::Defaulted;
",
        declarations.len(),
        files.len(),
        by_value.len(),
    );

    for ((class, value), rows) in &by_value {
        let name = function_name(*class, value);
        let _ = write!(
            out,
            "
/// The {} `{value}`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
",
            class.name()
        );
        for row in rows {
            let _ = writeln!(
                out,
                "/// | `{}` | `{}` | `{}`{}{} |",
                row.element,
                row.xsd_type,
                row.file,
                if row.context.is_empty() {
                    String::new()
                } else {
                    format!(", `{}`", row.context)
                },
                if row.fixed { " (fixed)" } else { "" },
            );
        }
        if !used.contains(&(*class, value.clone())) {
            out.push_str(
                "// No field transcribes a declaration with this value; the function is kept so\n\
                 // that the table is the schema's rather than the crate's.\n\
                 #[allow(dead_code)]\n",
            );
        }
        let _ = write!(
            out,
            "pub(crate) fn {name}<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{{
    T::read(deserializer, {value:?})
}}
"
        );
    }
    out
}

/// One struct field of the crate that reads an element.
#[derive(Debug)]
struct Field {
    /// The source file, relative to the crate root.
    file: String,
    line: usize,
    element: String,
    rust_type: String,
    /// The kind of value the field's type holds, when it can hold a declared one.
    class: Option<Class>,
    /// For an enumeration, the schema type the Rust enumeration transcribes.
    enum_xsd_type: Option<String>,
    deserialize_with: Option<String>,
}

/// Every field in `src/` that reads an element, with the attributes it carries.
fn crate_fields() -> Vec<Field> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let enum_types = enumeration_types();
    let mut sources = Vec::new();
    collect_sources(&root.join("src"), &mut sources);
    sources.sort();

    let mut fields = Vec::new();
    for path in sources {
        let file = path
            .strip_prefix(root)
            .expect("source lives under the crate root")
            .to_string_lossy()
            .replace('\\', "/");
        let source = std::fs::read_to_string(&path).expect("readable source");
        let lines: Vec<&str> = source.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            if !lines[i].trim_start().starts_with("#[serde(") {
                i += 1;
                continue;
            }
            let mut attribute = String::new();
            while i < lines.len() {
                attribute.push_str(lines[i].trim());
                attribute.push(' ');
                i += 1;
                if attribute.trim_end().ends_with(")]") {
                    break;
                }
            }
            while i < lines.len()
                && (lines[i].trim_start().starts_with("#[") || lines[i].trim_start().starts_with("///"))
            {
                i += 1;
            }
            let Some(declaration) = lines.get(i).map(|line| line.trim()) else {
                break;
            };
            let Some((_, rust_type)) = declaration
                .strip_prefix("pub ")
                .and_then(|rest| rest.strip_suffix(','))
                .and_then(|rest| rest.split_once(": "))
            else {
                continue;
            };
            let Some(element) = quoted(&attribute, "rename = ") else {
                continue;
            };
            if element.starts_with('@') || element.starts_with('$') {
                continue;
            }
            let inner = rust_type
                .strip_prefix("Option<")
                .or_else(|| rust_type.strip_prefix("Vec<"))
                .and_then(|rest| rest.strip_suffix('>'))
                .unwrap_or(rust_type);
            let (class, enum_xsd_type) = match inner {
                "bool" => (Some(Class::Boolean), None),
                "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" => {
                    (Some(Class::Integer), None)
                }
                "f32" | "f64" => (Some(Class::Decimal), None),
                "String" => (Some(Class::String), None),
                "Duration" => (Some(Class::Duration), None),
                other => match enum_types.get(other) {
                    Some(xsd) => (Some(Class::Enumeration), Some(xsd.clone())),
                    None => (None, None),
                },
            };
            fields.push(Field {
                file: file.clone(),
                line: i + 1,
                element,
                rust_type: rust_type.to_owned(),
                class,
                enum_xsd_type,
                deserialize_with: quoted(&attribute, "deserialize_with = "),
            });
        }
    }
    fields
}

/// The value of a `key = "…"` attribute argument.
fn quoted(attribute: &str, key: &str) -> Option<String> {
    let rest = &attribute[attribute.find(key)? + key.len()..];
    let rest = rest.strip_prefix('"')?;
    Some(rest[..rest.find('"')?].to_owned())
}

/// The crate's enumerations, by Rust name, and the schema type each transcribes.
fn enumeration_types() -> BTreeMap<String, String> {
    let source = include_str!("../src/enumerations.rs");
    source
        .lines()
        .filter_map(|line| {
            let (name, rest) = line.trim().split_once(" as \"")?;
            let (xsd_type, _) = rest.split_once('"')?;
            name.chars()
                .all(|c| c.is_ascii_alphanumeric())
                .then(|| (name.to_owned(), xsd_type.to_owned()))
        })
        .collect()
}

fn collect_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
    {
        let path = entry.expect("readable directory entry").path();
        if path.is_dir() {
            collect_sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// The declared value a field is expected to read, if the element it transcribes is
/// declared with one for a value of the field's kind.
///
/// An element declared with different values in different services —
/// `IncrementalUpdates` is `true` for Estimated Timetable and `false` elsewhere —
/// is resolved by the service the field's module belongs to.
fn expected_value(field: &Field, declarations: &[Declaration]) -> Result<Option<String>, String> {
    let Some(class) = field.class else {
        return Ok(None);
    };
    let rows: Vec<&Declaration> = declarations
        .iter()
        .filter(|d| d.element == field.element && d.class == class)
        .filter(|d| {
            class != Class::Enumeration || Some(d.type_name()) == field.enum_xsd_type.as_deref()
        })
        .collect();
    let values: BTreeSet<&str> = rows.iter().map(|d| d.value.as_str()).collect();
    match values.len() {
        0 => Ok(None),
        1 => Ok(values.into_iter().next().map(str::to_owned)),
        _ => {
            let module = service_of(&field.file);
            let values: BTreeSet<&str> = rows
                .iter()
                .filter(|d| d.service() == module)
                .map(|d| d.value.as_str())
                .collect();
            match values.len() {
                1 => Ok(values.into_iter().next().map(str::to_owned)),
                _ => Err(format!(
                    "{}:{}: <{}> is declared with several values and the field's module does not decide between them",
                    field.file, field.line, field.element
                )),
            }
        }
    }
}

/// Every field with the value it is expected to read, and every one that is expected
/// to read none: the crate's side of the table.
fn expectations<'a>(
    fields: &'a [Field],
    declarations: &[Declaration],
) -> (Vec<(&'a Field, Option<String>)>, Vec<String>) {
    let mut expected = Vec::new();
    let mut failures = Vec::new();
    for field in fields {
        match expected_value(field, declarations) {
            Ok(value) => expected.push((field, value)),
            Err(complaint) => failures.push(complaint),
        }
    }
    (expected, failures)
}

/// The values some field of the crate reads, by kind.
fn values_in_use(fields: &[Field], declarations: &[Declaration]) -> BTreeSet<(Class, String)> {
    expectations(fields, declarations)
        .0
        .into_iter()
        .filter_map(|(field, value)| Some((field.class?, value?)))
        .collect()
}

#[test]
fn the_table_is_what_the_schemas_declare() {
    let (declarations, _) = declarations();
    assert!(!declarations.is_empty(), "no element declaration carries a default");
    let rendered = render(&declarations, &values_in_use(&crate_fields(), &declarations));

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(GENERATED);
    if std::env::var_os(UPDATE).is_some() {
        std::fs::write(&path, &rendered).expect("the table is writable");
    }
    let checked_in = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("{GENERATED} cannot be read ({e}); generate it with {UPDATE}=1 cargo test --test schema_default")
    });
    assert!(
        checked_in == rendered,
        "{GENERATED} does not match the schemas; regenerate it with {UPDATE}=1 cargo test --test schema_default"
    );
}

/// What the schema copy declares, by kind, so that a change in the copy shows up
/// as a change in the numbers rather than going unnoticed.
#[test]
fn the_table_has_the_expected_shape() {
    let (declarations, _) = declarations();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for declaration in &declarations {
        *counts.entry(declaration.class.name()).or_default() += 1;
    }
    *counts.entry("fixed").or_default() = declarations.iter().filter(|d| d.fixed).count();
    *counts.entry("total").or_default() = declarations.len();
    let expected: BTreeMap<&str, usize> = [
        ("boolean", 172),
        ("decimal", 1),
        ("duration", 6),
        ("enumeration", 79),
        ("fixed", 5),
        ("integer", 2),
        ("string", 19),
        ("total", 279),
    ]
    .into_iter()
    .collect();
    assert_eq!(counts, expected);
}

#[test]
fn every_field_that_transcribes_a_declared_value_reads_it() {
    let (declarations, _) = declarations();
    let fields = crate_fields();
    assert!(fields.len() > 1000, "the source scan found only {} fields", fields.len());
    let (expected, mut failures) = expectations(&fields, &declarations);

    for (field, value) in expected {
        let expected = value.map(|value| {
            format!("{MODULE}::{}", function_name(field.class.expect("a value has a class"), &value))
        });
        let actual = field.deserialize_with.as_deref();
        match (&expected, actual) {
            (Some(expected), Some(actual)) if expected == actual => {}
            (Some(expected), _) => failures.push(format!(
                "{}:{}: <{}> as `{}` reads {} but the schema declares a value: expected deserialize_with = \"{expected}\"",
                field.file,
                field.line,
                field.element,
                field.rust_type,
                actual.map_or("plainly".to_owned(), |with| format!("with `{with}`")),
            )),
            (None, Some(actual)) if actual.starts_with(MODULE) => failures.push(format!(
                "{}:{}: <{}> as `{}` reads a declared value with `{actual}` but the schema declares none for it",
                field.file, field.line, field.element, field.rust_type,
            )),
            (None, _) => {}
        }
    }

    assert!(
        failures.is_empty(),
        "{} fields do not read what the schema declares:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// The declared elements no field reads a value for.
///
/// `Allow` is read as a [`DefaultedBoolean`](siri_rs::types::DefaultedBoolean), which
/// keeps the element as it was written — empty or spelled out — and leaves the value
/// to the reader; it is the one element the crate transcribes that way. The rest
/// have no field at all: each is a modelling gap, not a reading one, and when one is
/// transcribed it must read its value and be read out of a document, which the tests
/// above and below then demand.
#[test]
fn the_declared_elements_no_field_reads_a_value_for_are_these() {
    const KEEPS_ITS_WORDING: &[&str] = &["Allow"];
    const NOT_TRANSCRIBED: &[&str] = &[
        "AccommodationFacility",
        "AssistanceFacility",
        "AudibleSignalsAvailable",
        "BookingStatusType",
        "ConnectionLinksDetailLevel",
        "EscalatorFreeAccess",
        "FacilityDetailLevel",
        "FareClassFacility",
        "FunicularSubmode",
        "GuideDogAccess",
        "HasNames",
        "HireFacility",
        "LiftFreeAccess",
        "LuggageFacility",
        "MobilityFacility",
        "NuisanceFacility",
        "PassengerCommsFacility",
        "PassengerInformationFacility",
        "RefreshmentFacility",
        "ReservedSpaceFacility",
        "RetailFacility",
        "RoutePointType",
        "SanitaryFacility",
        "SelfDriveSubmode",
        "StepFreeAccess",
        "TaxiSubmode",
        "TicketRestrictionType",
        "TicketingFacility",
        "TimetableType",
        "VisualSignsAvailable",
        "WheelchairAccess",
    ];

    let (declarations, _) = declarations();
    let fields = crate_fields();
    let transcribed: BTreeSet<&str> = expectations(&fields, &declarations)
        .0
        .into_iter()
        .filter(|(_, value)| value.is_some())
        .map(|(field, _)| field.element.as_str())
        .collect();
    let missing: Vec<&str> = declarations
        .iter()
        .map(|d| d.element.as_str())
        .filter(|element| !transcribed.contains(element))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let mut expected: Vec<&str> = KEEPS_ITS_WORDING.iter().chain(NOT_TRANSCRIBED).copied().collect();
    expected.sort_unstable();
    assert_eq!(missing, expected);
}

/// One declared value the crate reads, and how an occurrence of its element in a
/// document is recognised as carrying a value of that kind.
struct Exercise {
    element: String,
    class: Class,
    /// The value, by the service a document belongs to when the element is declared
    /// with different values per service, else under `None`.
    values: BTreeMap<Option<&'static str>, String>,
    /// For an enumeration, the tokens an occurrence may carry.
    tokens: Vec<String>,
}

/// Every declared value some field reads, keyed by element.
fn exercises(declarations: &[Declaration], types: &Types, fields: &[Field]) -> Vec<Exercise> {
    let mut by_element: BTreeMap<(String, Class), Exercise> = BTreeMap::new();
    for (field, value) in expectations(fields, declarations).0 {
        let (Some(class), Some(value)) = (field.class, value) else {
            continue;
        };
        let exercise = by_element
            .entry((field.element.clone(), class))
            .or_insert_with(|| Exercise {
                element: field.element.clone(),
                class,
                values: BTreeMap::new(),
                tokens: match class {
                    Class::Enumeration => types
                        .tokens_of(field.enum_xsd_type.as_deref().expect("an enumeration names its type"))
                        .to_vec(),
                    _ => Vec::new(),
                },
            });
        let rows: BTreeSet<&str> = declarations
            .iter()
            .filter(|d| d.element == field.element && d.class == class)
            .map(|d| d.value.as_str())
            .collect();
        let service = if rows.len() > 1 { service_of(&field.file) } else { None };
        exercise.values.insert(service, value);
    }
    by_element.into_values().collect()
}

impl Exercise {
    /// Whether an occurrence's text is a value of this kind, so that an element name
    /// the schema also uses for another kind — `<Status>` is a boolean in a delivery
    /// and an enumeration in a facility — is only emptied where it means this one.
    fn carries(&self, text: &str) -> bool {
        match self.class {
            Class::Boolean => matches!(text, "true" | "false" | "1" | "0"),
            Class::Enumeration => self.tokens.iter().any(|token| token == text),
            Class::Integer => text.parse::<u64>().is_ok(),
            Class::Decimal => text.parse::<f64>().is_ok(),
            Class::Duration => text.starts_with('P'),
            Class::String => !text.is_empty(),
        }
    }
}

/// A document with every occurrence of the exercised elements emptied, the same
/// document with each of them carrying its declared value instead, and the values
/// so exercised.
fn empty_out(fixture: &Fixture, exercises: &[Exercise]) -> (String, String, BTreeSet<(String, String)>) {
    let service = service_of(&fixture.name.replace('\\', "/"));
    let mut emptied = fixture.xml.clone();
    let mut expected = fixture.xml.clone();
    let mut exercised = BTreeSet::new();

    for exercise in exercises {
        let Some(value) = exercise
            .values
            .get(&service)
            .or_else(|| exercise.values.get(&None))
        else {
            continue;
        };
        for (open, text, close) in leaf_occurrences(&fixture.xml, &exercise.element) {
            if !exercise.carries(&text) {
                continue;
            }
            let original = format!("{open}{text}{close}");
            let element = open.trim_start_matches('<').trim_end_matches('>');
            emptied = emptied.replace(&original, &format!("<{element}/>"));
            expected = expected.replace(&original, &format!("{open}{value}{close}"));
            exercised.insert((exercise.element.clone(), value.clone()));
        }
    }
    (emptied, expected, exercised)
}

/// Every `<Element>text</Element>` in a document — the element written without
/// attributes and with text alone inside it — as its start tag, its text and its
/// end tag, allowing for a namespace prefix.
///
/// An occurrence inside open content is left out: a general-message body or an
/// extension payload is carried as it was written, so an element in it that
/// happens to share a name with a declared one means nothing to the schema.
fn leaf_occurrences(xml: &str, element: &str) -> Vec<(String, String, String)> {
    let opaque = open_content(xml);
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(at) = xml[from..].find(&format!("{element}>")) {
        let name_at = from + at;
        from = name_at + element.len();
        if opaque.iter().any(|range| range.contains(&name_at)) {
            continue;
        }
        let tag_at = xml[..name_at]
            .rfind('<')
            .filter(|&lt| xml[lt + 1..name_at].chars().all(|c| c.is_ascii_alphanumeric() || c == ':'));
        let Some(tag_at) = tag_at else {
            continue;
        };
        let qualified = &xml[tag_at + 1..name_at + element.len()];
        if qualified.starts_with('/') || qualified.rsplit(':').next() != Some(element) {
            continue;
        }
        let open = format!("<{qualified}>");
        let close = format!("</{qualified}>");
        let text_at = tag_at + open.len();
        let Some(len) = xml[text_at..].find('<') else {
            continue;
        };
        if xml[text_at + len..].starts_with(&close) {
            out.push((open, xml[text_at..text_at + len].to_owned(), close));
        }
    }
    out
}

/// Elements the crate writes where the schema does not declare them, so that no
/// valid document can carry them: `ControlActionResponseFeatures` transcribes
/// `HasDriverMessages` and `HasVehicleDetectings`, which the schema declares in
/// the request policy of the same capabilities structure. Moving them changes a
/// public struct, which is left to a major release; until then they read their
/// value but cannot be read out of a document.
const NOT_EXERCISABLE: &[&str] = &["HasDriverMessages", "HasVehicleDetectings"];

/// The byte ranges of the elements whose content SIRI leaves to the participants:
/// a general message's `Content` and every `Extensions`.
fn open_content(xml: &str) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    for name in ["Content", "Extensions"] {
        let mut from = 0;
        while let Some(at) = xml[from..].find(&format!("<{name}>")) {
            let start = from + at;
            let Some(len) = xml[start..].find(&format!("</{name}>")) else {
                break;
            };
            ranges.push(start..start + len);
            from = start + len;
        }
    }
    ranges
}

#[test]
fn every_declared_value_the_crate_reads_is_read_out_of_an_empty_element() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let (declarations, types) = declarations();
    let exercises = exercises(&declarations, &types, &crate_fields());
    let mut attempted = BTreeSet::new();
    let mut failures = Vec::new();

    for fixture in support::fixtures().into_iter().chain(support::derived_fixtures()) {
        let (emptied, expected, hit) = empty_out(&fixture, &exercises);
        if hit.is_empty() {
            continue;
        }
        let elements: Vec<String> = hit.iter().map(|(element, _)| element.clone()).collect();
        attempted.extend(hit);
        let written = match round_trip(&support::root_element(&emptied), &emptied) {
            Ok(written) => written,
            Err(complaint) => {
                failures.push(format!("{} with {elements:?} emptied: {complaint}", fixture.name));
                continue;
            }
        };
        if let Err(difference) = compare(&parse(&expected), &parse(&written)) {
            failures.push(format!(
                "{} with {elements:?} emptied is not read as the declared values: {difference}",
                fixture.name
            ));
            continue;
        }
        if let Err(complaint) = validate_fixture(&fixture.name, &written) {
            failures.push(format!(
                "{} with {elements:?} emptied is written back invalid:\n{complaint}",
                fixture.name
            ));
        }
    }

    let unexercised: Vec<String> = exercises
        .iter()
        .flat_map(|exercise| {
            exercise
                .values
                .values()
                .map(|value| (exercise.element.clone(), value.clone()))
        })
        .filter(|pair| !attempted.contains(pair) && !NOT_EXERCISABLE.contains(&pair.0.as_str()))
        .map(|(element, value)| format!("<{element}> = {value}"))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    assert!(
        failures.is_empty() && unexercised.is_empty(),
        "{} documents did not read an empty element as its declared value:\n{}\n\n{} declared values occur in no document:\n{}",
        failures.len(),
        failures.join("\n"),
        unexercised.len(),
        unexercised.join("\n"),
    );
}

/// An official example with one element emptied, read into the type its root names.
fn official_with_empty<T: SiriRoot>(path: &str, original: &str) -> T {
    let xml = std::fs::read_to_string(fixtures_dir().join("xml").join(path))
        .unwrap_or_else(|e| panic!("cannot read fixture {path}: {e}"));
    assert!(xml.contains(original), "{path} carries {original}");
    let element = original[1..original.find('>').expect("a start tag")].to_owned();
    let emptied = xml.replacen(original, &format!("<{element}/>"), 1);
    siri_rs::from_str(&emptied).expect("the document reads")
}

fn deliveries(document: &Siri) -> &[ServiceDeliveryPayload] {
    &document
        .payload
        .as_service_delivery()
        .expect("the document is a service delivery")
        .deliveries
}

/// The first stop monitoring request a `<Siri>` service request carries.
fn stop_monitoring_request(document: &Siri) -> &StopMonitoringRequest {
    document
        .payload
        .as_service_request()
        .expect("the document is a service request")
        .requests
        .iter()
        .find_map(|request| match request {
            ServiceRequestPayload::StopMonitoringRequest(request) => Some(request.as_ref()),
            _ => None,
        })
        .expect("the request asks for a stop")
}

#[test]
fn an_empty_boolean_reads_as_the_declared_value() {
    let document: Siri = official_with_empty(
        "sm/exs_stopMonitoring_response_complex.xml",
        "<Monitored>true</Monitored>",
    );
    let Some(ServiceDeliveryPayload::StopMonitoringDelivery(delivery)) = deliveries(&document).first()
    else {
        panic!("the delivery is a stop monitoring delivery");
    };
    let visit = delivery.monitored_stop_visit.first().expect("the board has a visit");
    assert_eq!(visit.monitored_vehicle_journey.monitored, Some(true));
}

#[test]
fn an_empty_enumeration_reads_as_the_declared_value_and_is_written_in_full() {
    let document: Siri = official_with_empty(
        "sx/exx_situationExchange_response.xml",
        "<Severity>severe</Severity>",
    );
    let Some(ServiceDeliveryPayload::SituationExchangeDelivery(delivery)) = deliveries(&document).first()
    else {
        panic!("the delivery is a situation exchange delivery");
    };
    let situation = delivery.pt_situations().first().expect("the delivery carries a situation");
    assert_eq!(situation.severity, Some(Severity::Normal));

    let written = siri_rs::to_string(&document).expect("the document writes");
    assert!(
        written.contains("<Severity>normal</Severity>"),
        "the declared value is written out in full"
    );
}

#[test]
fn an_empty_integer_reads_as_the_declared_value() {
    let document: Siri = official_with_empty(
        "sm/exs_stopMonitoring_request.xml",
        "<MaximumTextLength>20</MaximumTextLength>",
    );
    assert_eq!(stop_monitoring_request(&document).maximum_text_length, Some(30));
}

#[test]
fn an_empty_decimal_reads_as_the_declared_value() {
    let document: Siri = official_with_empty(
        "sm/exs_stopMonitoring_response_complex.xml",
        "<Percentile>0.9</Percentile>",
    );
    let Some(ServiceDeliveryPayload::StopMonitoringDelivery(delivery)) = deliveries(&document).first()
    else {
        panic!("the delivery is a stop monitoring delivery");
    };
    let percentile = delivery
        .monitored_stop_visit
        .iter()
        .flat_map(|visit| &visit.monitored_vehicle_journey.onward_calls)
        .flat_map(|calls| &calls.onward_call)
        .flat_map(|call| &call.expected_departure_prediction_quality)
        .filter_map(|quality| quality.percentile)
        .next();
    assert_eq!(percentile, Some(0.9));
}

#[test]
fn an_empty_duration_reads_as_the_declared_value() {
    let response: GeneralMessageCapabilitiesResponse = official_with_empty(
        "gm/exm_generalMessage_capabilityResponse.xml",
        "<DefaultPreviewInterval>PT60M</DefaultPreviewInterval>",
    );
    let filtering = response
        .general_message_service_capabilities
        .as_ref()
        .and_then(|capabilities| capabilities.topic_filtering.as_ref())
        .expect("the response says what can be filtered");
    assert_eq!(filtering.default_preview_interval.as_str(), "PT60M");
}

#[test]
fn an_empty_language_reads_as_the_declared_value() {
    // The first language in the example is the one the request context names.
    let document: Siri =
        official_with_empty("sm/exs_stopMonitoring_request.xml", "<Language>en</Language>");
    let context = document
        .payload
        .as_service_request()
        .expect("the document is a service request")
        .service_request_context
        .as_ref()
        .expect("the request names a context");
    assert_eq!(context.language, ["en"]);
}

#[test]
fn an_element_of_whitespace_alone_is_not_an_empty_element() {
    // Under the schema an element holding only whitespace is not empty: it carries
    // that text, which is no boolean. Such a document is refused, as it was.
    let xml = std::fs::read_to_string(
        fixtures_dir().join("xml/sm/exs_stopMonitoring_response_complex.xml"),
    )
    .expect("readable fixture");
    let blank = xml.replacen("<Monitored>true</Monitored>", "<Monitored>   </Monitored>", 1);
    let error = siri_rs::from_str::<Siri>(&blank).expect_err("whitespace is not a boolean");
    assert!(error.to_string().contains("Monitored"), "{error}");
}

#[test]
fn the_function_names_are_spelled_from_the_values() {
    assert_eq!(function_name(Class::Boolean, "true"), "boolean_true");
    assert_eq!(function_name(Class::Enumeration, "everyDay"), "enumeration_every_day");
    assert_eq!(function_name(Class::Enumeration, "httpPost"), "enumeration_http_post");
    assert_eq!(function_name(Class::Duration, "PT60M"), "duration_pt60m");
    assert_eq!(function_name(Class::Decimal, "0.9"), "decimal_0_9");
    assert_eq!(function_name(Class::Integer, "30"), "integer_30");
}

#[test]
fn leaf_occurrences_find_the_element_and_only_the_element() {
    let xml = r#"<Siri><Status>true</Status><VehicleStatus>x</VehicleStatus><siri:Status>false</siri:Status><Status><A/></Status><Content><Status>true</Status></Content></Siri>"#;
    assert_eq!(
        leaf_occurrences(xml, "Status"),
        [
            ("<Status>".to_owned(), "true".to_owned(), "</Status>".to_owned()),
            ("<siri:Status>".to_owned(), "false".to_owned(), "</siri:Status>".to_owned()),
        ]
    );
}
