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
/// Returns the input unchanged when no element carries a prefix for the SIRI
/// namespace, which is the common case and costs a single scan.
pub fn normalise(xml: &str) -> Result<Cow<'_, str>> {
    if !needs_normalisation(xml)? {
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
                    e.local_name().as_ref().to_vec()
                } else {
                    e.name().as_ref().to_vec()
                };
                write(
                    &mut writer,
                    Event::End(BytesEnd::new(String::from_utf8_lossy(&name))),
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
        start.local_name().as_ref().to_vec()
    } else {
        start.name().as_ref().to_vec()
    };
    let mut out = BytesStart::new(String::from_utf8_lossy(&name).into_owned());
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

/// True when at least one element resolves to the SIRI namespace through a prefix.
fn needs_normalisation(xml: &str) -> Result<bool> {
    let mut reader = NsReader::from_str(xml);
    reader.config_mut().trim_text(false);
    loop {
        match reader.read_resolved_event()? {
            (_, Event::Eof) => return Ok(false),
            (ns, Event::Start(e)) | (ns, Event::Empty(e))
                if is_siri(&ns) && e.name().prefix().is_some() =>
            {
                return Ok(true)
            }
            _ => {}
        }
    }
}

/// Writes `xmlns="http://www.siri.org.uk/siri"` onto the root element of a
/// serialised document.
///
/// The serde serialiser has no notion of namespaces, so it emits bare element
/// names. This pass adds the single declaration that puts the whole document into
/// the SIRI namespace, which is how every official example is written.
pub fn declare_default_namespace(xml: &str) -> Result<String> {
    let mut reader = quick_xml::Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::new());
    let mut root_seen = false;

    loop {
        let event = reader.read_event()?;
        match event {
            Event::Eof => break,
            Event::Start(e) if !root_seen => {
                root_seen = true;
                write(&mut writer, Event::Start(with_default_namespace(&e)))?;
            }
            Event::Empty(e) if !root_seen => {
                root_seen = true;
                write(&mut writer, Event::Empty(with_default_namespace(&e)))?;
            }
            other => write(&mut writer, other)?,
        }
    }

    into_string(writer.into_inner())
}

fn with_default_namespace(start: &BytesStart<'_>) -> BytesStart<'static> {
    let mut out = BytesStart::new(String::from_utf8_lossy(start.name().as_ref()).into_owned());
    out.push_attribute(("xmlns", NAMESPACE));
    for attr in start.attributes().with_checks(false).flatten() {
        out.push_attribute(attr);
    }
    out.into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let out = declare_default_namespace(r#"<Siri version="2.1"><Foo/></Siri>"#).unwrap();
        assert_eq!(
            out,
            r#"<Siri xmlns="http://www.siri.org.uk/siri" version="2.1"><Foo/></Siri>"#
        );
    }

    #[test]
    fn an_empty_root_element_gains_the_default_declaration() {
        let out = declare_default_namespace("<Siri/>").unwrap();
        assert_eq!(out, r#"<Siri xmlns="http://www.siri.org.uk/siri"/>"#);
    }
}
