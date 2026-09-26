//! The document as text: where the element a fault names is, which units a
//! fragment holds, what identifies one, and the mark an element leaves when it
//! is taken out.
//!
//! The reader works on text and hands back a path; everything here turns a path
//! back into a place in the text, or reads the text for the little a finding
//! needs beyond the path. Subtrees off the path are skipped rather than read.

use std::collections::BTreeMap;
use std::ops::Range;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use serde_path_to_error::{Path, Segment};

use super::units::{Container, Record};

/// What an element leaves behind when it is taken out: a comment naming it, so
/// that its place still counts when a later path names a neighbour. The reader
/// passes a comment over as if it were not there.
const LEFT_OUT: &str = "left out: ";

/// The comment an element named `name` leaves behind.
pub(super) fn marker(name: &str) -> String {
    format!("<!--{LEFT_OUT}{name}-->")
}

/// The element name a comment marks as left out, if it is such a comment.
fn left_out(comment: &[u8]) -> Option<&str> {
    std::str::from_utf8(comment).ok()?.strip_prefix(LEFT_OUT)
}

/// Whether a fault the strict reader found lies inside a functional service
/// delivery — which is where the lenient reader may leave something out.
pub(super) fn within_a_delivery(path: &Path) -> bool {
    let mut segments = path.iter();
    matches!(segments.next(), Some(Segment::Map { key }) if key == "$value")
        && matches!(segments.next(), Some(Segment::Enum { variant }) if variant == "ServiceDelivery")
        && matches!(segments.next(), Some(Segment::Map { key }) if key == "$value")
        && matches!(segments.next(), Some(Segment::Seq { .. }))
}

/// One step down from an element to one of its children, as a path names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Step {
    /// The child's element name.
    pub(super) name: String,
    /// The child's position among its parent's children of that name, when the
    /// element is a repeated one; `None` for one the schema allows once.
    pub(super) index: Option<usize>,
}

/// The steps from a fragment's root to the element a fault names, as far as the
/// path can be followed on the text.
///
/// A path in serde's terms may end in what is not an element: the `$text` of
/// one, an attribute, or the `$value` of a choice the reader could not make. The
/// steps stop at the element holding it, which is what a reader of the document
/// can find. A fragment read as a choice of deliveries names its root as the
/// variant first, which is no step.
pub(super) fn steps(path: &Path) -> Vec<Step> {
    let mut steps = Vec::new();
    let mut segments = path.iter().peekable();
    if matches!(segments.peek(), Some(Segment::Enum { .. })) {
        segments.next();
    }
    while let Some(segment) = segments.next() {
        match segment {
            Segment::Map { key } if !key.starts_with('$') && !key.starts_with('@') => {
                let index = match segments.peek() {
                    Some(Segment::Seq { index }) => {
                        segments.next();
                        Some(*index)
                    }
                    _ => None,
                };
                steps.push(Step {
                    name: key.clone(),
                    index,
                });
            }
            _ => break,
        }
    }
    steps
}

/// An element found by its steps.
pub(super) struct Located {
    /// From the `<` of its start tag to the `>` of its end tag.
    pub(super) range: Range<usize>,
    /// The steps that lead to it, with the index of each counting the elements
    /// left out before it, so that the path names the document as it arrived.
    pub(super) steps: Vec<Step>,
    /// The element it is a child of.
    pub(super) parent: String,
}

/// How many children of one name an element has had so far: those the reader
/// sees, and those with the ones left out earlier.
#[derive(Default)]
struct Seen {
    read: usize,
    all: usize,
}

/// Follows `steps` down from the root of `xml` and returns the element they
/// lead to — or, where the text runs out of them, the deepest element reached.
/// `None` is the root itself: no step could be followed.
///
/// A document the tokeniser refuses locates nothing.
pub(super) fn locate(xml: &str, steps: &[Step]) -> Option<Located> {
    let mut reader = Reader::from_str(xml);
    let mut root = None;
    let mut matched: Vec<(String, usize, Step)> = Vec::new();
    let mut seen: BTreeMap<String, Seen> = BTreeMap::new();

    let located = |matched: &[(String, usize, Step)], root: &Option<String>, end: usize| {
        let (_, start, _) = matched.last()?;
        let parent = match matched.len() {
            1 => root.clone()?,
            deeper => matched[deeper - 2].0.clone(),
        };
        Some(Located {
            range: *start..end,
            steps: matched.iter().map(|(_, _, step)| step.clone()).collect(),
            parent,
        })
    };

    loop {
        let at = position(&reader);
        let event = reader.read_event().ok()?;
        let (element, empty) = match event {
            Event::Start(element) => (element, false),
            Event::Empty(element) => (element, true),
            Event::End(_) | Event::Eof => return located(&matched, &root, position(&reader)),
            Event::Comment(comment) => {
                if let Some(name) = left_out(&comment) {
                    seen.entry(name.to_string()).or_default().all += 1;
                }
                continue;
            }
            _ => continue,
        };
        if root.is_none() {
            root = Some(local_name(&element));
            continue;
        }
        let Some(step) = steps.get(matched.len()) else {
            return located(&matched, &root, position(&reader));
        };
        let name = local_name(&element);
        let count = seen.entry(name.clone()).or_default();
        let hit = name == step.name && step.index.map_or(true, |index| index == count.read);
        let place = count.all;
        count.read += 1;
        count.all += 1;
        if !hit {
            if !empty {
                reader.read_to_end(element.name()).ok()?;
            }
            continue;
        }
        let original = Step {
            name,
            index: step.index.map(|_| place),
        };
        if empty || matched.len() + 1 == steps.len() {
            if !empty {
                reader.read_to_end(element.name()).ok()?;
            }
            matched.push((original.name.clone(), at, original));
            return located(&matched, &root, position(&reader));
        }
        matched.push((original.name.clone(), at, original));
        seen.clear();
    }
}

/// One child of an element.
pub(super) struct Child {
    /// The element name, or the name of the element a comment marks as left out.
    pub(super) name: String,
    /// From the `<` of its start tag to the `>` of its end tag.
    pub(super) range: Range<usize>,
    /// Its position among the children of that name, the ones left out counted.
    pub(super) place: usize,
    /// Whether it is the mark of an element left out rather than an element.
    pub(super) left_out: bool,
}

/// The children of the element spanning `of`, in document order.
pub(super) fn children(xml: &str, of: Range<usize>) -> Vec<Child> {
    let mut reader = Reader::from_str(&xml[of.clone()]);
    let mut children = Vec::new();
    let mut places: BTreeMap<String, usize> = BTreeMap::new();
    let mut opened = false;
    loop {
        let at = of.start + position(&reader);
        let Ok(event) = reader.read_event() else {
            return children;
        };
        let (name, left_out) = match event {
            Event::Start(_) if !opened => {
                opened = true;
                continue;
            }
            Event::Start(element) => {
                if reader.read_to_end(element.name()).is_err() {
                    return children;
                }
                (local_name(&element), false)
            }
            Event::Empty(element) => (local_name(&element), false),
            Event::Comment(comment) => match left_out(&comment) {
                Some(name) => (name.to_string(), true),
                None => continue,
            },
            Event::End(_) | Event::Eof => return children,
            _ => continue,
        };
        let place = places.entry(name.clone()).or_default();
        children.push(Child {
            name,
            range: at..of.start + position(&reader),
            place: *place,
            left_out,
        });
        *place += 1;
    }
}

/// A record found in a fragment.
pub(super) struct Found<'r> {
    /// What kind of record it is.
    pub(super) record: &'r Record,
    /// From the `<` of its start tag to the `>` of its end tag.
    pub(super) range: Range<usize>,
    /// Its path from the document root, in the terms the reader's errors use.
    pub(super) path: String,
}

/// Every record of the given kinds in `xml`, a delivery whose path from the
/// document root is `base`, in document order.
pub(super) fn records_in<'r>(records: &[&'r Record], xml: &str, base: &str) -> Vec<Found<'r>> {
    let mut found = Vec::new();
    if records.is_empty() {
        return found;
    }
    let mut reader = Reader::from_str(xml);
    loop {
        match reader.read_event() {
            Ok(Event::Start(_)) => break,
            Ok(Event::Eof) | Err(_) => return found,
            Ok(_) => {}
        }
    }
    walk(&mut reader, records, &[], base, &mut found);
    found
}

/// Walks the children of the element just opened, descending into the
/// containers the records sit in and collecting the records themselves.
fn walk<'r>(
    reader: &mut Reader<&[u8]>,
    records: &[&'r Record],
    chain: &[&str],
    base: &str,
    found: &mut Vec<Found<'r>>,
) -> Option<()> {
    let mut places: BTreeMap<String, usize> = BTreeMap::new();
    loop {
        let at = position(reader);
        let (element, empty) = match reader.read_event().ok()? {
            Event::Start(element) => (element, false),
            Event::Empty(element) => (element, true),
            Event::Comment(comment) => {
                if let Some(name) = left_out(&comment) {
                    *places.entry(name.to_string()).or_default() += 1;
                }
                continue;
            }
            Event::End(_) | Event::Eof => return Some(()),
            _ => continue,
        };
        let name = local_name(&element);
        let place = places.entry(name.clone()).or_default();
        let here = *place;
        *place += 1;
        if let Some(container) = container_at(records, chain, &name) {
            if empty {
                continue;
            }
            let path = if container.repeated {
                format!("{base}.{name}[{here}]")
            } else {
                format!("{base}.{name}")
            };
            let deeper: Vec<&str> = chain.iter().copied().chain(std::iter::once(name.as_str())).collect();
            walk(reader, records, &deeper, &path, found)?;
        } else {
            if !empty {
                reader.read_to_end(element.name()).ok()?;
            }
            if let Some(record) = record_at(records, chain, &name) {
                found.push(Found {
                    record,
                    range: at..position(reader),
                    path: format!("{base}.{name}[{here}]"),
                });
            }
        }
    }
}

/// The container named `name` that follows `chain` on the way to some record.
fn container_at<'r>(records: &[&'r Record], chain: &[&str], name: &str) -> Option<&'r Container> {
    records.iter().find_map(|record| {
        let next = record.containers.get(chain.len())?;
        (next.name == name && leads_through(record, chain)).then_some(next)
    })
}

/// The record named `name` that sits right below `chain`.
fn record_at<'r>(records: &[&'r Record], chain: &[&str], name: &str) -> Option<&'r Record> {
    records
        .iter()
        .find(|record| record.name == name && record.containers.len() == chain.len() && leads_through(record, chain))
        .copied()
}

/// Whether the record's containers begin with `chain`.
fn leads_through(record: &Record, chain: &[&str]) -> bool {
    record.containers.iter().zip(chain).all(|(container, name)| container.name == *name)
}

/// The text of the first element of the most telling name in `candidates` that
/// `xml` carries, wherever it is nested: what tells a reader which unit was left
/// out.
pub(super) fn identifier(xml: &str, candidates: &[&str]) -> Option<String> {
    let mut reader = Reader::from_str(xml);
    let mut best: Option<(usize, String)> = None;
    loop {
        let element = match reader.read_event() {
            Ok(Event::Start(element)) => element,
            Ok(Event::Eof) | Err(_) => break,
            Ok(_) => continue,
        };
        let name = local_name(&element);
        let Some(rank) = candidates.iter().position(|candidate| *candidate == name) else {
            continue;
        };
        if best.as_ref().is_some_and(|(found, _)| *found <= rank) {
            continue;
        }
        let Ok(text) = reader.read_text(element.name()) else {
            break;
        };
        let text = text.trim();
        if text.is_empty() {
            continue;
        }
        best = Some((rank, text.to_string()));
        if rank == 0 {
            break;
        }
    }
    best.map(|(_, text)| text)
}

/// The element name without a namespace prefix.
fn local_name(element: &BytesStart<'_>) -> String {
    String::from_utf8_lossy(element.local_name().as_ref()).into_owned()
}

/// How far into its text a reader has got, as an offset.
fn position(reader: &Reader<&[u8]>) -> usize {
    usize::try_from(reader.buffer_position()).unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "<Root>\n\t<A><X>1</X></A>\n\t<!--left out: B-->\n\t<B><Y/></B>\n\t<B><Y>2</Y><Y>3</Y></B>\n</Root>";

    fn step(name: &str, index: Option<usize>) -> Step {
        Step {
            name: name.into(),
            index,
        }
    }

    #[test]
    fn an_element_is_found_by_its_steps() {
        let located = locate(DOC, &[step("B", Some(1)), step("Y", Some(0))]).expect("found");
        assert_eq!(&DOC[located.range], "<Y>2</Y>");
        assert_eq!(located.parent, "B");
        assert_eq!(located.steps, vec![step("B", Some(2)), step("Y", Some(0))]);
    }

    #[test]
    fn the_index_of_a_step_counts_what_was_left_out_before_it() {
        let located = locate(DOC, &[step("B", Some(0))]).expect("found");
        assert_eq!(&DOC[located.range], "<B><Y/></B>");
        assert_eq!(located.steps, vec![step("B", Some(1))]);
        assert_eq!(located.parent, "Root");
    }

    #[test]
    fn steps_the_text_runs_out_of_locate_the_deepest_element_reached() {
        let located = locate(DOC, &[step("A", None), step("X", None), step("Z", None)]).expect("found");
        assert_eq!(&DOC[located.range], "<X>1</X>");
        assert_eq!(located.steps, vec![step("A", None), step("X", None)]);
        assert!(locate(DOC, &[step("Q", None)]).is_none());
        assert!(locate(DOC, &[]).is_none());
    }

    #[test]
    fn an_empty_element_on_the_way_is_the_element_located() {
        let located = locate(DOC, &[step("B", Some(0)), step("Y", Some(0)), step("Z", None)]).expect("found");
        assert_eq!(&DOC[located.range], "<Y/>");
    }

    #[test]
    fn children_are_listed_with_their_places() {
        let all = children(DOC, 0..DOC.len());
        let listed: Vec<(&str, usize, bool)> = all
            .iter()
            .map(|child| (child.name.as_str(), child.place, child.left_out))
            .collect();
        assert_eq!(listed, vec![("A", 0, false), ("B", 0, true), ("B", 1, false), ("B", 2, false)]);
        assert_eq!(&DOC[all[3].range.clone()], "<B><Y>2</Y><Y>3</Y></B>");
    }

    #[test]
    fn a_path_in_serdes_terms_becomes_steps() {
        let path = read_path("<A><B><C><D>x</D></C></B></A>");
        assert_eq!(path, vec![step("B", None), step("C", None), step("D", None)]);
    }

    /// The steps of the path a reader reports for a document whose deepest leaf
    /// is not the integer the type asks for.
    fn read_path(xml: &str) -> Vec<Step> {
        #[derive(serde::Deserialize)]
        struct A {
            #[serde(rename = "B")]
            _b: B,
        }
        #[derive(serde::Deserialize)]
        struct B {
            #[serde(rename = "C")]
            _c: C,
        }
        #[derive(serde::Deserialize)]
        struct C {
            #[serde(rename = "D")]
            _d: u32,
        }
        let fault = crate::xml::read::<A>(xml).err().expect("x is no integer");
        steps(&fault.path)
    }

    #[test]
    fn the_most_telling_identifier_wins_over_the_first() {
        let xml = "<R><Second>two</Second><Inner><First>one</First></Inner></R>";
        assert_eq!(identifier(xml, &["First", "Second"]).as_deref(), Some("one"));
        assert_eq!(identifier(xml, &["Third", "Second"]).as_deref(), Some("two"));
        assert_eq!(identifier(xml, &["Third"]), None);
        assert_eq!(identifier("<R><First>  </First></R>", &["First"]), None);
    }

    #[test]
    fn a_fault_inside_a_delivery_is_told_from_one_in_the_envelope() {
        let inside = crate::xml::read::<crate::Siri>(
            r#"<Siri><ServiceDelivery><ResponseTimestamp>2004-12-17T09:30:46-05:00</ResponseTimestamp><StopMonitoringDelivery><ResponseTimestamp/></StopMonitoringDelivery></ServiceDelivery></Siri>"#,
        )
        .expect_err("the delivery's timestamp is empty");
        assert!(within_a_delivery(&inside.path));
        let envelope = crate::xml::read::<crate::Siri>(
            r#"<Siri><ServiceDelivery><ResponseTimestamp/></ServiceDelivery></Siri>"#,
        )
        .expect_err("the envelope's timestamp is empty");
        assert!(!within_a_delivery(&envelope.path));
    }
}
