//! Namespace normalisation for incoming SIRI documents.
//!
//! SIRI documents in the wild bind the SIRI namespace two ways: as the default
//! namespace (`<Siri xmlns="http://www.siri.org.uk/siri">`, used by most official
//! examples) or through a prefix (`<siri:RoadSituationElement xmlns:siri="...">`,
//! used by others). Both are the same document as far as XML is concerned, but the
//! serde layer matches element names literally and would only recognise the first.
//!
//! [`normalise`] rewrites a document so that every element resolving to the SIRI
//! namespace is written with its local name, leaving foreign content untouched.
//! Documents that already use the default binding are returned borrowed and unchanged.

use std::borrow::Cow;

use quick_xml::encoding::EncodingError;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::name::ResolveResult;
use quick_xml::{NsReader, Writer};

use crate::error::{Error, Result};

/// The SIRI XML namespace, `http://www.siri.org.uk/siri`.
pub const NAMESPACE: &str = "http://www.siri.org.uk/siri";

const NAMESPACE_BYTES: &[u8] = NAMESPACE.as_bytes();

/// Rewrites SIRI-namespaced elements to their unprefixed local names.
///
/// Returns the input unchanged when nothing binds the SIRI namespace to a prefix,
/// which is the common case and costs a substring search rather than a parse.
pub fn normalise(xml: &str) -> Result<Cow<'_, str>> {
    if !binds_the_namespace_to_a_prefix(xml) {
        return Ok(Cow::Borrowed(xml));
    }

    let mut reader = NsReader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::new());

    loop {
        match reader.read_resolved_event()? {
            (_, Event::Eof) => break,
            (ns, Event::Start(e)) => {
                write(&mut writer, Event::Start(rewrite_start(&ns, &e)?))?;
            }
            (ns, Event::Empty(e)) => {
                write(&mut writer, Event::Empty(rewrite_start(&ns, &e)?))?;
            }
            (ns, Event::End(e)) => {
                let name = if is_siri(&ns) {
                    e.local_name().into_inner()
                } else {
                    e.name().into_inner()
                };
                write(
                    &mut writer,
                    Event::End(BytesEnd::new(String::from_utf8_lossy(name))),
                )?;
            }
            (_, event) => write(&mut writer, event)?,
        }
    }

    into_string(writer.into_inner()).map(Cow::Owned)
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

/// Copies a start tag, dropping the prefix when the element is SIRI-namespaced and
/// dropping the namespace declarations that bind the SIRI namespace.
///
/// Every other attribute — including foreign namespace declarations, so that foreign
/// content keeps resolving — is carried over verbatim.
fn rewrite_start(ns: &ResolveResult<'_>, start: &BytesStart<'_>) -> Result<BytesStart<'static>> {
    let name = if is_siri(ns) {
        start.local_name().into_inner()
    } else {
        start.name().into_inner()
    };
    let mut out = BytesStart::new(String::from_utf8_lossy(name));
    for attr in start.attributes().with_checks(false) {
        let attr = attr.map_err(quick_xml::Error::InvalidAttr)?;
        if declares_siri_namespace(&attr) {
            continue;
        }
        out.push_attribute(attr);
    }
    Ok(out.into_owned())
}

fn is_siri(ns: &ResolveResult<'_>) -> bool {
    matches!(ns, ResolveResult::Bound(n) if n.as_ref() == NAMESPACE_BYTES)
}

fn declares_siri_namespace(attr: &quick_xml::events::attributes::Attribute<'_>) -> bool {
    let key = attr.key.as_ref();
    let is_declaration = key == b"xmlns" || key.starts_with(b"xmlns:");
    is_declaration && attr.value.as_ref() == NAMESPACE_BYTES
}

/// True when the document's text binds the SIRI namespace to a prefix.
///
/// A prefixed element can only exist where a declaration binds its prefix, and the
/// resolver binds what the declaration literally says — so a document whose text
/// holds no `xmlns:…="http://www.siri.org.uk/siri"` cannot carry one, and finding
/// that out costs a substring search instead of reading the whole document.
///
/// The other direction is deliberately approximate: a binding nothing uses, or one
/// written inside a comment, sends the document down the rewriting path, which
/// leaves such a document as it was.
fn binds_the_namespace_to_a_prefix(xml: &str) -> bool {
    xml.match_indices(NAMESPACE).any(|(at, _)| {
        xml[..at]
            .trim_end_matches(['"', '\''])
            .trim_end()
            .strip_suffix('=')
            .and_then(|attribute| attribute.split_whitespace().next_back())
            .is_some_and(|name| name.starts_with("xmlns:"))
    })
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
    fn a_prefix_binding_is_recognised_however_it_is_spaced() {
        assert!(binds_the_namespace_to_a_prefix(
            r#"<s:Siri xmlns:s="http://www.siri.org.uk/siri">"#
        ));
        assert!(binds_the_namespace_to_a_prefix(
            "<s:Siri xmlns:s = 'http://www.siri.org.uk/siri'>"
        ));
        assert!(binds_the_namespace_to_a_prefix(
            "<s:Siri\n\txmlns:s=\"http://www.siri.org.uk/siri\">"
        ));
    }

    #[test]
    fn text_that_merely_names_the_namespace_is_not_a_prefix_binding() {
        assert!(!binds_the_namespace_to_a_prefix(
            r#"<Siri xmlns="http://www.siri.org.uk/siri">"#
        ));
        assert!(!binds_the_namespace_to_a_prefix(
            r#"<Siri xsi:schemaLocation="http://www.siri.org.uk/siri ../xsd/siri.xsd">"#
        ));
        assert!(!binds_the_namespace_to_a_prefix(
            "<Note>http://www.siri.org.uk/siri</Note>"
        ));
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
    fn a_prefix_nothing_uses_costs_a_rewrite_but_changes_nothing() {
        let xml = concat!(
            r#"<Siri xmlns="http://www.siri.org.uk/siri""#,
            r#" xmlns:s="http://www.siri.org.uk/siri"><Foo>1</Foo></Siri>"#
        );
        assert_eq!(normalise(xml).unwrap(), "<Siri><Foo>1</Foo></Siri>");
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
    fn foreign_content_and_its_declarations_survive() {
        let xml = concat!(
            r#"<s:Siri xmlns:s="http://www.siri.org.uk/siri" xmlns:d="urn:other">"#,
            r#"<s:Extensions><d:Thing d:a="1"/></s:Extensions></s:Siri>"#
        );
        assert_eq!(
            normalise(xml).unwrap(),
            r#"<Siri xmlns:d="urn:other"><Extensions><d:Thing d:a="1"/></Extensions></Siri>"#
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
