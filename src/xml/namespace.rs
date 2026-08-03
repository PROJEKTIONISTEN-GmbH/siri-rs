//! Namespace normalisation for incoming SIRI documents.
//!
//! SIRI documents in the wild bind the SIRI namespace two ways: as the default
//! namespace (`<Siri xmlns="http://www.siri.org.uk/siri">`, used by most official
//! examples) or through a prefix (`<siri:RoadSituationElement xmlns:siri="...">`,
//! used by others). Both are the same document as far as XML is concerned, but the
//! serde layer matches element names literally and would only recognise the first.
//!
//! The same applies to content SIRI leaves to the participants — an extension
//! payload, an embedded DATEX II record — which the schema admits in any namespace
//! and a producer usually writes with a prefix. That content is kept as a subtree
//! rather than modelled, and the subtree is captured through the same serde layer,
//! which reports names without their prefix.
//!
//! [`normalise`] therefore rewrites a document so that every element is written with
//! its local name, and every element whose namespace differs from the one in force
//! states it as a default declaration — the same infoset, spelled the one way the
//! serde layer can carry. Documents that write no prefixed element at all are
//! returned borrowed and unchanged.

use std::borrow::Cow;
use std::rc::Rc;

use quick_xml::encoding::EncodingError;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::name::ResolveResult;
use quick_xml::{NsReader, Writer};

use crate::error::{Error, Result};

/// The SIRI XML namespace, `http://www.siri.org.uk/siri`.
pub const NAMESPACE: &str = "http://www.siri.org.uk/siri";

const NAMESPACE_BYTES: &[u8] = NAMESPACE.as_bytes();

/// Rewrites prefixed elements to their unprefixed local names, restating the
/// namespace they were in as a default declaration where that is not already the one
/// in force.
///
/// Returns the input unchanged when it writes no prefixed element, which is the
/// common case and costs a scan rather than a parse.
pub fn normalise(xml: &str) -> Result<Cow<'_, str>> {
    if !writes_a_prefixed_element(xml) {
        return Ok(Cow::Borrowed(xml));
    }

    let mut reader = NsReader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::new());
    // The default namespace in force at each open element of the document being
    // written — what has been written, not what the source declared. The declaration
    // the writer later puts on the root element makes that the SIRI namespace to
    // begin with, so an element in it needs no declaration of its own. Most elements
    // inherit what their parent put in force, which is a share rather than a copy.
    let mut written: Vec<Rc<[u8]>> = vec![Rc::from(NAMESPACE_BYTES)];

    loop {
        match reader.read_resolved_event()? {
            (_, Event::Eof) => break,
            (ns, Event::Start(e)) => {
                let declared = declared_namespace(&ns, in_force(&written));
                let entered = match declared {
                    Some(namespace) => Rc::from(namespace),
                    None => Rc::clone(in_force(&written)),
                };
                write(&mut writer, Event::Start(rewrite_start(&ns, &e, declared)?))?;
                written.push(entered);
            }
            (ns, Event::Empty(e)) => {
                let declared = declared_namespace(&ns, in_force(&written));
                write(&mut writer, Event::Empty(rewrite_start(&ns, &e, declared)?))?;
            }
            (ns, Event::End(e)) => {
                written.pop();
                let name = String::from_utf8_lossy(name_of(&ns, &e.name()));
                write(&mut writer, Event::End(BytesEnd::new(name)))?;
            }
            (_, event) => write(&mut writer, event)?,
        }
    }

    into_string(writer.into_inner()).map(Cow::Owned)
}

/// The default namespace the innermost element written so far put in force.
fn in_force(written: &[Rc<[u8]>]) -> &Rc<[u8]> {
    written
        .last()
        .expect("the document's own default namespace is never popped")
}

/// The namespace an element has to declare as its default, or `None` when the one in
/// force already is the one it is in.
///
/// An element whose prefix nothing binds is left alone — its namespace is not
/// knowable, so neither the reader nor this can say what to declare.
fn declared_namespace<'ns>(ns: &'ns ResolveResult<'ns>, in_force: &[u8]) -> Option<&'ns [u8]> {
    let namespace = match ns {
        ResolveResult::Bound(namespace) => namespace.as_ref(),
        ResolveResult::Unbound => b"",
        ResolveResult::Unknown(_) => return None,
    };
    (namespace != in_force).then_some(namespace)
}

/// The name to write an element under: its local name, unless its prefix is one
/// nothing binds, in which case there is nothing better to do than keep it.
fn name_of<'name>(ns: &ResolveResult<'_>, name: &quick_xml::name::QName<'name>) -> &'name [u8] {
    match ns {
        ResolveResult::Unknown(_) => name.into_inner(),
        _ => name.local_name().into_inner(),
    }
}

/// Writes an event, mapping the writer's I/O error into this crate's error type.
///
/// The sink is a `Vec<u8>`, so this never actually fails.
fn write(writer: &mut Writer<Vec<u8>>, event: Event<'_>) -> Result<()> {
    writer
        .write_event(event)
        .map_err(|e| Error::Xml(quick_xml::Error::from(e)))
}

fn into_string(bytes: Vec<u8>) -> Result<String> {
    String::from_utf8(bytes).map_err(|e| {
        Error::Xml(quick_xml::Error::Encoding(EncodingError::Utf8(
            e.utf8_error(),
        )))
    })
}

/// Copies a start tag under the element's local name, declaring `declared` as the
/// default namespace when the element is in one other than the one in force.
///
/// Two kinds of declaration are dropped. Every default declaration goes, because the
/// rewritten document states the default from what it has actually written: keeping
/// the source's as well would put an element's descendants into a namespace the
/// rewriting no longer expects. So does every binding of the SIRI namespace to a
/// prefix, because no name in the rewritten document is written with one. Every other
/// attribute — including foreign prefix declarations, so that a prefix used on an
/// attribute keeps resolving — is carried over verbatim.
fn rewrite_start(
    ns: &ResolveResult<'_>,
    start: &BytesStart<'_>,
    declared: Option<&[u8]>,
) -> Result<BytesStart<'static>> {
    let mut out = BytesStart::new(String::from_utf8_lossy(name_of(ns, &start.name())));
    if let Some(namespace) = declared {
        out.push_attribute(("xmlns", String::from_utf8_lossy(namespace).as_ref()));
    }
    for attr in start.attributes().with_checks(false) {
        let attr = attr.map_err(quick_xml::Error::InvalidAttr)?;
        let key = attr.key.as_ref();
        let restated = key == b"xmlns"
            || (key.starts_with(b"xmlns:") && attr.value.as_ref() == NAMESPACE_BYTES);
        if restated {
            continue;
        }
        out.push_attribute(attr);
    }
    Ok(out.into_owned())
}

/// True when the document writes an element with a namespace prefix.
///
/// Such an element is the only reason to rewrite anything: a document written without
/// prefixes is already spelled the way the serde layer reads, whatever its
/// declarations bind. This runs on every document read, so it looks for the one byte
/// a prefix cannot do without — the colon — and only then asks what that colon stands
/// in. A document full of timestamps, which are colons in text, costs one scan for a
/// single byte and a few bytes of backtracking each.
fn writes_a_prefixed_element(xml: &str) -> bool {
    let bytes = xml.as_bytes();
    let mut unparsed = UnparsedText::of(xml);
    xml.match_indices(':')
        .any(|(at, _)| names_a_tag(bytes, at) && !unparsed.holds(at))
}

/// Whether the colon at `at` stands inside the name of a tag, which is the only place
/// a prefix this has to act on can be.
///
/// A colon in an attribute name is one the rewriting leaves alone; a colon in a
/// timestamp, a URL or any other text is not a prefix at all. All three are told apart
/// by what precedes the name the colon ends: only a tag's name follows `<` or `</`.
fn names_a_tag(bytes: &[u8], at: usize) -> bool {
    let mut start = at;
    while start > 0 && is_a_name_byte(bytes[start - 1]) {
        start -= 1;
    }
    start < at && matches!(&bytes[..start], [.., b'<'] | [.., b'<', b'/'])
}

/// Whether this byte may stand inside an XML name. The colon may too, but a name with
/// two of them is not one this has any reason to recognise.
fn is_a_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.') || byte >= 0x80
}

/// The comments and CDATA sections of a document, walked from the front.
///
/// What such a section holds is not content — the official situation-exchange example
/// carries a whole prefixed DATEX II record inside a comment — and rewriting a
/// document for text no reader will look at would cost it the borrowed path for
/// nothing. A document holds few sections and, usually, no candidate at all, so this
/// looks for them only when it is asked and remembers where it has got to. Positions
/// have to arrive in increasing order, which is the order they are found in.
struct UnparsedText<'a> {
    xml: &'a str,
    /// How far the search has got: everything before this holds no section that has
    /// not been reported.
    searched_to: usize,
    /// Where the section reported last begins and ends.
    section: (usize, usize),
}

impl<'a> UnparsedText<'a> {
    fn of(xml: &'a str) -> Self {
        Self {
            xml,
            searched_to: 0,
            section: (0, 0),
        }
    }

    /// Whether `at` falls inside a comment or a CDATA section.
    fn holds(&mut self, at: usize) -> bool {
        while self.section.1 <= at {
            let Some(section) = self.next_section_before(at) else {
                return false;
            };
            self.section = section;
        }
        self.section.0 <= at
    }

    /// The first section beginning before `at` that has not been accounted for, if
    /// there is one.
    ///
    /// Nothing beyond `at` is searched: a section that begins after it cannot hold it,
    /// and the position asked about next is a better place to search up to than the
    /// end of a document that may be a hundred kilobytes further on.
    fn next_section_before(&mut self, at: usize) -> Option<(usize, usize)> {
        while let Some(found) = self.xml[self.searched_to..at].find(SECTION_OPENING) {
            let opens_at = self.searched_to + found;
            match length_of_unparsed_text(&self.xml[opens_at..]) {
                Some(length) => {
                    self.searched_to = opens_at + length;
                    return Some((opens_at, opens_at + length));
                }
                None => self.searched_to = opens_at + SECTION_OPENING.len(),
            }
        }
        self.searched_to = at;
        None
    }
}

/// What a comment, a CDATA section and a doctype declaration all begin with.
const SECTION_OPENING: &str = "<!";

/// The length of the comment or CDATA section beginning here, or `None` when neither
/// does. One that is never closed swallows the rest of the document, as it does for a
/// reader.
fn length_of_unparsed_text(from: &str) -> Option<usize> {
    let (opening, closing) = if from.starts_with("<!--") {
        ("<!--", "-->")
    } else if from.starts_with("<![CDATA[") {
        ("<![CDATA[", "]]>")
    } else {
        return None;
    };
    let body = &from[opening.len()..];
    Some(
        body.find(closing)
            .map_or(from.len(), |end| opening.len() + end + closing.len()),
    )
}

/// Writes `xmlns="http://www.siri.org.uk/siri"` onto the root element of a
/// serialised document, in place.
///
/// The serde serialiser has no notion of namespaces, so it emits bare element
/// names. This adds the single declaration that puts the whole document into the
/// SIRI namespace, which is how every official example is written.
///
/// `root_at` is where the root element's `<` is. The serialiser writes that element
/// first and writes the name it was handed verbatim, so the declaration's place —
/// straight after the name, ahead of the element's own attributes — follows from the
/// name's length, and the document does not have to be read back to find it.
pub(crate) fn declare_default_namespace(document: &mut String, root_at: usize, root: &str) {
    debug_assert!(
        document[root_at..].starts_with('<') && document[root_at + 1..].starts_with(root),
        "the serialiser opens the document with <{root}"
    );
    document.insert_str(root_at + 1 + root.len(), DEFAULT_BINDING);
}

/// The declaration [`declare_default_namespace`] inserts, spelled out so that it goes
/// in as one piece; `the_default_binding_names_the_namespace` keeps it in step with
/// [`NAMESPACE`].
const DEFAULT_BINDING: &str = r#" xmlns="http://www.siri.org.uk/siri""#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_prefixed_element_is_recognised_wherever_it_stands() {
        assert!(writes_a_prefixed_element(
            r#"<s:Siri xmlns:s="http://www.siri.org.uk/siri">"#
        ));
        assert!(
            writes_a_prefixed_element(concat!(
                r#"<Siri xmlns="http://www.siri.org.uk/siri"><Extensions>"#,
                r#"<d:Thing xmlns:d="urn:other"/></Extensions></Siri>"#
            )),
            "a foreign payload is as good a reason to rewrite as a prefixed SIRI element"
        );
        assert!(
            writes_a_prefixed_element("<Siri><Foo>1</s:Foo></Siri>"),
            "a closing tag names its element too"
        );
    }

    /// The official situation-exchange example carries a prefixed DATEX II record
    /// inside a comment. Nothing reads it, so it must not cost the document the
    /// borrowed path.
    #[test]
    fn a_prefixed_element_that_is_only_commented_out_is_not_written() {
        let declaration = r#"<Siri xmlns:d2="urn:datex">"#;
        assert!(!writes_a_prefixed_element(&format!(
            concat!(
                "{}<!-- <d2:SituationRecord>1</d2:SituationRecord> -->",
                "<![CDATA[ <d2:Also/> ]]><Foo/></Siri>"
            ),
            declaration
        )));
        assert!(writes_a_prefixed_element(&format!(
            "{declaration}<!-- a note --><d2:Thing/></Siri>"
        )));
        assert!(!writes_a_prefixed_element(&format!(
            "{declaration}<!-- never closed <d2:Thing/>"
        )));
    }

    /// The colon is a common byte in a SIRI document: every timestamp holds two, and
    /// every namespace declaration and schema hint holds one more. None of them is a
    /// reason to rewrite the document.
    #[test]
    fn a_colon_that_does_not_name_a_tag_is_not_a_prefix() {
        assert!(!writes_a_prefixed_element(concat!(
            r#"<Siri xmlns="http://www.siri.org.uk/siri""#,
            r#" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance""#,
            r#" xsi:schemaLocation="http://www.siri.org.uk/siri ../xsd/siri.xsd">"#,
            "<RequestTimestamp>2004-12-17T09:30:47-05:00</RequestTimestamp>",
            "<Note>see http://www.siri.org.uk/siri</Note></Siri>"
        )));
    }

    #[test]
    fn a_document_that_only_points_at_the_schema_is_left_alone() {
        let xml = concat!(
            r#"<Siri xmlns="http://www.siri.org.uk/siri""#,
            r#" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance""#,
            r#" xsi:schemaLocation="http://www.siri.org.uk/siri ../xsd/siri.xsd">"#,
            r#"<Foo>1</Foo></Siri>"#
        );
        assert!(matches!(normalise(xml).unwrap(), Cow::Borrowed(_)));
    }

    #[test]
    fn a_prefix_nothing_uses_is_left_where_it_stands() {
        let xml = concat!(
            r#"<Siri xmlns="http://www.siri.org.uk/siri""#,
            r#" xmlns:s="http://www.siri.org.uk/siri"><Foo>1</Foo></Siri>"#
        );
        assert!(matches!(normalise(xml).unwrap(), Cow::Borrowed(_)));
    }

    #[test]
    fn documents_using_the_default_binding_are_left_alone() {
        let xml = r#"<Siri xmlns="http://www.siri.org.uk/siri"><Foo>1</Foo></Siri>"#;
        assert!(matches!(normalise(xml).unwrap(), Cow::Borrowed(_)));
    }

    #[test]
    fn prefixed_siri_elements_lose_their_prefix() {
        let xml = r#"<s:Siri xmlns:s="http://www.siri.org.uk/siri"><s:Foo>1</s:Foo></s:Siri>"#;
        assert_eq!(normalise(xml).unwrap(), "<Siri><Foo>1</Foo></Siri>");
    }

    #[test]
    fn foreign_content_keeps_its_namespace_as_a_default_declaration() {
        let xml = concat!(
            r#"<s:Siri xmlns:s="http://www.siri.org.uk/siri" xmlns:d="urn:other">"#,
            r#"<s:Extensions><d:Thing d:a="1"/></s:Extensions></s:Siri>"#
        );
        assert_eq!(
            normalise(xml).unwrap(),
            concat!(
                r#"<Siri xmlns:d="urn:other"><Extensions>"#,
                r#"<Thing xmlns="urn:other" d:a="1"/></Extensions></Siri>"#
            )
        );
    }

    #[test]
    fn a_foreign_subtree_declares_its_namespace_once() {
        let xml = concat!(
            r#"<s:Siri xmlns:s="http://www.siri.org.uk/siri" xmlns:d="urn:other">"#,
            r#"<s:Extensions><d:Outer><d:Inner>1</d:Inner></d:Outer></s:Extensions></s:Siri>"#
        );
        assert_eq!(
            normalise(xml).unwrap(),
            concat!(
                r#"<Siri xmlns:d="urn:other"><Extensions>"#,
                r#"<Outer xmlns="urn:other"><Inner>1</Inner></Outer></Extensions></Siri>"#
            )
        );
    }

    #[test]
    fn returning_to_the_siri_namespace_declares_it_again() {
        let xml = concat!(
            r#"<s:Siri xmlns:s="http://www.siri.org.uk/siri" xmlns:d="urn:other">"#,
            r#"<s:Extensions><d:Outer><s:Inner>1</s:Inner></d:Outer></s:Extensions></s:Siri>"#
        );
        assert_eq!(
            normalise(xml).unwrap(),
            concat!(
                r#"<Siri xmlns:d="urn:other"><Extensions><Outer xmlns="urn:other">"#,
                r#"<Inner xmlns="http://www.siri.org.uk/siri">1</Inner>"#,
                r#"</Outer></Extensions></Siri>"#
            )
        );
    }

    /// The source's own default declarations are not copied: what an element's
    /// descendants inherit has to be what the rewriting believes it wrote, or a
    /// subtree ends up in a namespace it was never in.
    #[test]
    fn a_default_declaration_the_source_wrote_does_not_outlive_the_rewriting() {
        let xml = concat!(
            r#"<Siri xmlns="http://www.siri.org.uk/siri"><Extensions>"#,
            r#"<Wrapper xmlns="urn:other"><d:Thing xmlns:d="urn:third"/></Wrapper>"#,
            r#"</Extensions></Siri>"#
        );
        assert_eq!(
            normalise(xml).unwrap(),
            concat!(
                r#"<Siri><Extensions><Wrapper xmlns="urn:other">"#,
                r#"<Thing xmlns="urn:third" xmlns:d="urn:third"/>"#,
                r#"</Wrapper></Extensions></Siri>"#
            )
        );
    }

    #[test]
    fn non_namespace_attributes_are_preserved() {
        let xml = r#"<s:Siri xmlns:s="http://www.siri.org.uk/siri" version="2.1"><s:N xml:lang="DE">x</s:N></s:Siri>"#;
        assert_eq!(
            normalise(xml).unwrap(),
            r#"<Siri version="2.1"><N xml:lang="DE">x</N></Siri>"#
        );
    }

    #[test]
    fn the_root_element_gains_the_default_declaration() {
        let mut document = String::from(r#"<Siri version="2.1"><Foo/></Siri>"#);
        declare_default_namespace(&mut document, 0, "Siri");
        assert_eq!(
            document,
            r#"<Siri xmlns="http://www.siri.org.uk/siri" version="2.1"><Foo/></Siri>"#
        );
    }

    #[test]
    fn an_empty_root_element_gains_the_default_declaration() {
        let mut document = String::from("<Siri/>");
        declare_default_namespace(&mut document, 0, "Siri");
        assert_eq!(document, r#"<Siri xmlns="http://www.siri.org.uk/siri"/>"#);
    }

    #[test]
    fn what_precedes_the_root_element_is_left_where_it_is() {
        let mut document = String::from("<?xml version=\"1.0\"?>\n<Siri/>");
        declare_default_namespace(&mut document, 22, "Siri");
        assert_eq!(
            document,
            "<?xml version=\"1.0\"?>\n<Siri xmlns=\"http://www.siri.org.uk/siri\"/>"
        );
    }

    #[test]
    fn the_default_binding_names_the_namespace() {
        assert_eq!(DEFAULT_BINDING, format!(" xmlns=\"{NAMESPACE}\""));
    }
}
