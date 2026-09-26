//! Taking out what does not read, one element at a time, until a fragment reads.
//!
//! A fragment — a record, or a whole functional service delivery — is read as its
//! own type. When it does not read, the element the fault names is taken out and
//! the fragment read again; the reader is deterministic, so the next fault is the
//! next one in document order. An element the schema makes mandatory shows up on
//! the next reading as missing from its parent, which is then the element taken
//! out — and so on up to the fragment's root, which is the unit itself. Every
//! element taken out is one finding; a chain of them climbing to a parent is the
//! same finding, told where it ended up.

use std::ops::Range;

use serde::de::DeserializeOwned;

use super::document::{self, Step};
use super::units::{Record, DELIVERY_IDENTIFIERS, RECORDS, SERVICE_DELIVERY_FIELDS};
use super::{Discarded, Finding, Findings};
use crate::framework::ServiceDeliveryPayload;
use crate::xml;
use crate::{Error, Result};

/// The unit a fragment is, and what may be found inside it.
pub(super) struct Scope<'a> {
    /// The unit's element name.
    pub(super) name: String,
    /// The unit's path from the document root.
    pub(super) path: String,
    /// The elements that may identify the unit in a finding.
    pub(super) identifiers: &'static [&'static str],
    /// The records the fragment may hold — none inside a record.
    pub(super) records: &'a [&'a Record],
}

/// How a fragment came out of reading.
pub(super) enum Outcome {
    /// It reads, once whatever had to be left out was.
    Kept,
    /// It could not be read: the fault that started the chain of removals which
    /// ended in the unit itself, spelled from the document root.
    LeftOut {
        /// The element that could not be read.
        path: String,
        /// What the reader could not do there.
        source: quick_xml::DeError,
    },
}

/// Reads every functional service delivery of `document` leniently, in place:
/// first each record on its own, then the delivery as a whole. A delivery that
/// cannot be read is left out — unless none would remain, which fails the
/// document with the fault the first of them could not be read past.
pub(super) fn deliveries(document: &mut String, findings: &mut Findings) -> Result<()> {
    let service_delivery = Step {
        name: "ServiceDelivery".to_string(),
        index: None,
    };
    let Some(located) = document::locate(document, std::slice::from_ref(&service_delivery)) else {
        return Ok(());
    };
    let mut out = String::with_capacity(document.len());
    let mut copied = 0;
    let mut kept = 0;
    let mut first_left_out = None;
    let mut slot = 0;
    for child in document::children(document, located.range) {
        if SERVICE_DELIVERY_FIELDS.contains(&child.name.as_str()) {
            continue;
        }
        let index = slot;
        slot += 1;
        if child.left_out {
            continue;
        }
        let original = &document[child.range.clone()];
        let records: Vec<&Record> = RECORDS.iter().filter(|record| record.delivery == child.name).collect();
        let scope = Scope {
            name: child.name.clone(),
            path: format!("ServiceDelivery.{}[{index}]", child.name),
            identifiers: DELIVERY_IDENTIFIERS,
            records: &records,
        };
        let mut text = original.to_string();
        records_of(&mut text, &scope, findings);
        let replacement = match fragment::<ServiceDeliveryPayload>(&mut text, original, &scope, findings) {
            Outcome::Kept => {
                kept += 1;
                text
            }
            Outcome::LeftOut { path, source } => {
                first_left_out.get_or_insert((path, source));
                document::marker(&child.name)
            }
        };
        out.push_str(&document[copied..child.range.start]);
        out.push_str(&replacement);
        copied = child.range.end;
    }
    out.push_str(&document[copied..]);
    *document = out;
    match first_left_out {
        Some((path, source)) if kept == 0 => Err(Error::Deserialize {
            path,
            offset: None,
            source,
        }),
        _ => Ok(()),
    }
}

/// Reads every record of a delivery on its own, in place, leaving out each one
/// that cannot be read.
///
/// A record read alone reads as it does in place, and reading the records one by
/// one keeps the cost of a delivery with many faults linear in its size: the
/// delivery as a whole is read again only once, after them.
fn records_of(text: &mut String, scope: &Scope<'_>, findings: &mut Findings) {
    let found = document::records_in(scope.records, text, &scope.path);
    if found.is_empty() {
        return;
    }
    let mut out = String::with_capacity(text.len());
    let mut copied = 0;
    for record in found {
        let original = &text[record.range.clone()];
        let record_scope = Scope {
            name: record.record.name.to_string(),
            path: record.path,
            identifiers: record.record.identifiers,
            records: &[],
        };
        let mut fragment = original.to_string();
        let replacement = match (record.record.read)(&mut fragment, original, &record_scope, findings) {
            Outcome::Kept => fragment,
            Outcome::LeftOut { .. } => document::marker(record.record.name),
        };
        out.push_str(&text[copied..record.range.start]);
        out.push_str(&replacement);
        copied = record.range.end;
    }
    out.push_str(&text[copied..]);
    *text = out;
}

/// A chain of removals that began with one fault: the finding that tells it,
/// the element last taken out, and what the reader could not do at the start.
struct Chain {
    finding: usize,
    removed: Vec<Step>,
    source: quick_xml::DeError,
}

/// The element a fault names, found in the fragment.
struct Target {
    range: Range<usize>,
    steps: Vec<Step>,
    name: String,
    parent: String,
    path: String,
    is_root: bool,
}

impl Target {
    /// Where in `text` the fault at `path` is, as far as the text can say; the
    /// fragment's root when the path names nothing below it.
    fn of(text: &str, path: &serde_path_to_error::Path, scope: &Scope<'_>) -> Self {
        match document::locate(text, &document::steps(path)) {
            Some(located) => {
                let name = located.steps.last().map(|step| step.name.clone()).unwrap_or_default();
                Self {
                    path: spelled(&scope.path, &located.steps),
                    name,
                    parent: located.parent,
                    steps: located.steps,
                    range: located.range,
                    is_root: false,
                }
            }
            None => Self {
                range: 0..text.len(),
                steps: Vec::new(),
                name: scope.name.clone(),
                parent: String::new(),
                path: scope.path.clone(),
                is_root: true,
            },
        }
    }
}

/// Reads `text` as a `T`, taking out what does not read until it reads or nothing
/// readable is left. `original` is the text as it was before anything was taken
/// out, which is where a unit's identifier is read from once it is left out.
pub(super) fn fragment<T: DeserializeOwned>(
    text: &mut String,
    original: &str,
    scope: &Scope<'_>,
    findings: &mut Findings,
) -> Outcome {
    let mut chain: Option<Chain> = None;
    loop {
        let fault = match xml::read::<T>(text) {
            Ok(_) => return Outcome::Kept,
            Err(fault) => fault,
        };
        let reason = fault.source.to_string();
        let target = Target::of(text, &fault.path, scope);

        if let Some(field) = named_field(&reason, "duplicate field") {
            if leave_out_the_interrupted_run(text, &target, field, scope, findings) {
                chain = None;
                continue;
            }
        }

        let continues = chain.as_ref().is_some_and(|chain| {
            named_field(&reason, "missing field") == chain.removed.last().map(|step| step.name.as_str())
                && chain.removed[..chain.removed.len() - 1] == target.steps[..]
        });
        let discarded = discarded_at(text, original, &target, scope);

        let finding = match chain.take() {
            Some(chain) if continues => {
                findings.findings[chain.finding].discarded = discarded;
                chain
            }
            _ => Chain {
                finding: findings.push(Finding {
                    path: target.path.clone(),
                    reason,
                    discarded,
                }),
                removed: Vec::new(),
                source: fault.source,
            },
        };
        let origin = findings.findings[finding.finding].clone();
        report_contained(text, &target, scope, &origin, findings);

        if target.is_root {
            return Outcome::LeftOut {
                path: origin.path,
                source: finding.source,
            };
        }
        text.replace_range(target.range.clone(), &document::marker(&target.name));
        chain = Some(Chain {
            removed: target.steps,
            ..finding
        });
    }
}

/// What is left out when the element at `target` is: the unit itself at the
/// root, identified from the text as it arrived; a record where the element is
/// one; and otherwise the element.
fn discarded_at(text: &str, original: &str, target: &Target, scope: &Scope<'_>) -> Discarded {
    if target.is_root {
        return Discarded::Unit {
            path: scope.path.clone(),
            name: scope.name.clone(),
            identifier: document::identifier(original, scope.identifiers),
        };
    }
    match record_kind(scope, &target.parent, &target.name) {
        Some(record) => Discarded::Unit {
            path: target.path.clone(),
            name: target.name.clone(),
            identifier: document::identifier(&text[target.range.clone()], record.identifiers),
        },
        None => Discarded::Element {
            path: target.path.clone(),
        },
    }
}

/// Reports every record inside the element at `target` as left out with it: a
/// frame taken out takes its journeys, a delivery its records. Nothing goes
/// quietly.
fn report_contained(text: &str, target: &Target, scope: &Scope<'_>, origin: &Finding, findings: &mut Findings) {
    if scope.records.is_empty() || record_kind(scope, &target.parent, &target.name).is_some() {
        return;
    }
    for found in document::records_in(scope.records, text, &scope.path) {
        let inside = target.is_root
            || (found.range.start > target.range.start && found.range.end <= target.range.end);
        if !inside {
            continue;
        }
        findings.push(Finding {
            path: origin.path.clone(),
            reason: origin.reason.clone(),
            discarded: Discarded::Unit {
                path: found.path,
                name: found.record.name.to_string(),
                identifier: document::identifier(&text[found.range], found.record.identifiers),
            },
        });
    }
}

/// A list of records the reader found interrupted by another element — records
/// of a name it had already read a run of — is read up to the interruption; the
/// records after it are left out one by one. Answers whether that was the case.
fn leave_out_the_interrupted_run(
    text: &mut String,
    target: &Target,
    field: &str,
    scope: &Scope<'_>,
    findings: &mut Findings,
) -> bool {
    let parent = if target.is_root { scope.name.as_str() } else { target.name.as_str() };
    let Some(record) = record_kind(scope, parent, field) else {
        return false;
    };
    let children = document::children(text, target.range.clone());
    let Some(first) = children.iter().position(|child| child.name == field && !child.left_out) else {
        return false;
    };
    let Some(interruption) = children[first..]
        .iter()
        .position(|child| child.name != field && !child.left_out)
        .map(|offset| first + offset)
    else {
        return false;
    };
    let interrupted: Vec<&document::Child> = children[interruption..]
        .iter()
        .filter(|child| child.name == field && !child.left_out)
        .collect();
    if interrupted.is_empty() {
        return false;
    }
    for child in &interrupted {
        findings.push(Finding {
            path: target.path.clone(),
            reason: format!("duplicate field `{field}`"),
            discarded: Discarded::Unit {
                path: format!("{}.{field}[{}]", target.path, child.place),
                name: field.to_string(),
                identifier: document::identifier(&text[child.range.clone()], record.identifiers),
            },
        });
    }
    for child in interrupted.iter().rev() {
        text.replace_range(child.range.clone(), &document::marker(field));
    }
    true
}

/// The record kind an element named `name` under `parent` is, if it is one.
fn record_kind<'a>(scope: &Scope<'a>, parent: &str, name: &str) -> Option<&'a Record> {
    scope
        .records
        .iter()
        .find(|record| record.name == name && record.parent() == parent)
        .copied()
}

/// The field a reader's message of the form ``<what> field `Name` `` names.
fn named_field<'m>(reason: &'m str, what: &str) -> Option<&'m str> {
    reason.strip_prefix(what)?.strip_prefix(" `")?.strip_suffix('`')
}

/// A path from `base` down `steps`, in the terms the reader's errors use.
fn spelled(base: &str, steps: &[Step]) -> String {
    let mut path = base.to_string();
    for step in steps {
        path.push('.');
        path.push_str(&step.name);
        if let Some(index) = step.index {
            path.push_str(&format!("[{index}]"));
        }
    }
    path
}
