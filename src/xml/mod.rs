//! Reading and writing SIRI XML documents.
//!
//! Every SIRI message is a global XML element, so the root element name identifies
//! the message. Types that may appear as a document root implement [`SiriRoot`],
//! which is what [`from_str`] checks against and what [`to_string`] writes.
//!
//! ```
//! use siri_rs::{CheckStatusRequest, ParticipantRef};
//!
//! let doc = r#"<Siri xmlns="http://www.siri.org.uk/siri" version="2.0">
//!   <CheckStatusRequest version="2.0">
//!     <RequestTimestamp>2004-12-17T09:30:47-05:00</RequestTimestamp>
//!     <RequestorRef>EREWHON</RequestorRef>
//!   </CheckStatusRequest>
//! </Siri>"#;
//!
//! let siri: siri_rs::Siri = siri_rs::from_str(doc)?;
//! let request: &CheckStatusRequest = siri.payload.as_check_status_request().unwrap();
//! assert_eq!(request.requestor_ref, ParticipantRef::new("EREWHON"));
//! # Ok::<(), siri_rs::Error>(())
//! ```

mod characters;
pub(crate) mod date_time;
pub(crate) mod defaulted;
pub mod namespace;
pub(crate) mod schema_default;
pub(crate) mod token_list;
mod unparsed;
mod whitespace;

use std::borrow::Cow;

use serde::{de::DeserializeOwned, Serialize};

use crate::error::{Error, Result};

pub use namespace::NAMESPACE;

/// A type that can stand as the root element of a SIRI document.
pub trait SiriRoot: Serialize + DeserializeOwned {
    /// The local name of the root element, e.g. `Siri`.
    const ELEMENT_NAME: &'static str;
}

/// Reads a SIRI document.
///
/// Accepts both namespace bindings used in practice — the SIRI namespace as the
/// document default, or bound to a prefix — and rejects documents whose root
/// element is not the one `T` describes, in the SIRI namespace. An element whose
/// content is whitespace alone is read as that whitespace, which the deserialiser
/// would otherwise drop.
///
/// What XML itself does not allow is refused too: a character outside XML 1.0's
/// (a control character other than tab, line feed and carriage return, as itself
/// or as a character reference), and anything after the root element but
/// comments, processing instructions and whitespace.
pub fn from_str<T: SiriRoot>(xml: &str) -> Result<T> {
    let text = prepare::<T>(xml)?;
    let offsets_apply = matches!(text, Cow::Borrowed(_));
    read_document(&text, offsets_apply)
}

/// Makes a document ready for the reader: its characters checked, its root element
/// checked against `T` and its namespace, the SIRI namespace bound by default, and
/// whitespace-only content kept.
///
/// The text comes back borrowed when nothing had to be rewritten, which is when
/// a byte offset into it is an offset into what the caller handed over.
pub(crate) fn prepare<T: SiriRoot>(xml: &str) -> Result<Cow<'_, str>> {
    refuse_illegal_characters(xml)?;
    // The root's namespace is read off the document as it arrived: the rewriting
    // below states namespaces relative to the SIRI one it assumes.
    check_root::<T>(xml)?;
    let text = match namespace::normalise(xml)? {
        Cow::Borrowed(text) => whitespace::preserve(text),
        Cow::Owned(text) => {
            let kept = match whitespace::preserve(&text) {
                Cow::Owned(kept) => Some(kept),
                Cow::Borrowed(_) => None,
            };
            Cow::Owned(kept.unwrap_or(text))
        }
    };
    Ok(text)
}

/// Fails on the first character in `xml` that XML 1.0 does not allow, whether it
/// is written as itself or as a character reference.
fn refuse_illegal_characters(xml: &str) -> Result<()> {
    let first = [characters::first_illegal(xml), characters::first_illegal_reference(xml)]
        .into_iter()
        .flatten()
        .min_by_key(|(at, _)| *at);
    match first {
        Some((at, character)) => Err(Error::IllegalCharacter {
            character,
            offset: at as u64,
        }),
        None => Ok(()),
    }
}

/// Reads a whole prepared document: the value its root element holds, and after
/// the root element nothing XML does not allow there. `offsets_apply` is as for
/// [`deserialize`].
pub(crate) fn read_document<T: DeserializeOwned>(xml: &str, offsets_apply: bool) -> Result<T> {
    match read_whole(xml) {
        Ok((value, false)) => Ok(value),
        Ok((_, true)) => Err(content_after_root(xml, offsets_apply)),
        Err(fault) => Err(fault.into_error(offsets_apply)),
    }
}

/// The error for a document that goes on after its root element with more than
/// comments, processing instructions and whitespace, placed where it does.
pub(crate) fn content_after_root(xml: &str, offsets_apply: bool) -> Error {
    Error::ContentAfterRoot {
        offset: offsets_apply.then(|| where_the_root_is_followed(xml)).flatten(),
    }
}

/// Where the first thing after the root element of `xml` stands that is not a
/// comment, a processing instruction or whitespace, if anything does.
///
/// The reader that reads the document knows only that something follows, since it
/// reads one event ahead; this walks the document again to say where, which it is
/// asked to do only for a document that is refused.
fn where_the_root_is_followed(xml: &str) -> Option<u64> {
    use quick_xml::events::Event;

    let mut reader = quick_xml::Reader::from_str(xml);
    let mut depth = 0usize;
    let mut root_seen = false;
    loop {
        let at = reader.buffer_position();
        let event = reader.read_event();
        if root_seen && depth == 0 {
            match event {
                Ok(Event::Comment(_) | Event::PI(_)) => continue,
                Ok(Event::Text(text)) if text.iter().all(u8::is_ascii_whitespace) => continue,
                Ok(Event::Eof) => return None,
                _ => return Some(at),
            }
        }
        match event.ok()? {
            Event::Start(_) => {
                depth += 1;
                root_seen = true;
            }
            Event::Empty(_) => root_seen = true,
            Event::End(_) => depth = depth.saturating_sub(1),
            Event::Eof => return None,
            _ => {}
        }
    }
}

/// Reads a value out of `xml`, reporting where in the document a failure was.
///
/// `offsets_apply` says whether `xml` is the text the caller handed over, which is
/// what a byte offset has to count in to be of any use.
pub(crate) fn deserialize<T: DeserializeOwned>(xml: &str, offsets_apply: bool) -> Result<T> {
    read(xml).map_err(|fault| fault.into_error(offsets_apply))
}

/// Where the reader failed and what it could not do there, before it is spelled
/// out as an [`Error`]: the path as the deserialiser tracked it, for a caller
/// that has to find the element in the document rather than name it.
pub(crate) struct Fault {
    /// The path the deserialiser tracked, in serde's terms.
    pub(crate) path: serde_path_to_error::Path,
    /// How far into `xml` the reader had got when it failed, in bytes.
    pub(crate) offset: u64,
    /// What the reader could not do there.
    pub(crate) source: quick_xml::DeError,
}

impl Fault {
    /// The fault as the error the reader reports, with the offset when it counts
    /// into text the caller has seen.
    pub(crate) fn into_error(self, offsets_apply: bool) -> Error {
        Error::Deserialize {
            path: document_path(&self.path),
            offset: offsets_apply.then_some(self.offset),
            source: self.source,
        }
    }
}

/// Reads a value out of `xml`, tracking where in the document a failure was.
///
/// Tracking the path costs something on every field of every document, and what
/// it buys is only ever spent on one that fails. So a document is read without it
/// first, and only a failure is read a second time to find out where it was: the
/// reader is deterministic, so the second read fails in the same place as the
/// first. A document that parses pays nothing; one that does not is already lost.
pub(crate) fn read<T: DeserializeOwned>(xml: &str) -> std::result::Result<T, Fault> {
    read_whole(xml).map(|(value, _)| value)
}

/// Reads a value out of `xml` as [`read`] does, and says whether the element it was
/// read from is followed by anything but comments, processing instructions and
/// whitespace — which, after the root element of a document, XML does not allow.
pub(crate) fn read_whole<T: DeserializeOwned>(xml: &str) -> std::result::Result<(T, bool), Fault> {
    let mut untracked = quick_xml::de::Deserializer::from_str(xml);
    if let Ok(value) = T::deserialize(&mut untracked) {
        return Ok((value, !untracked.is_empty()));
    }
    let mut deserializer = quick_xml::de::Deserializer::from_str(xml);
    match serde_path_to_error::deserialize::<_, T>(&mut deserializer) {
        Ok(value) => Ok((value, !deserializer.is_empty())),
        Err(failure) => Err(Fault {
            path: failure.path().clone(),
            offset: deserializer.get_ref().get_ref().buffer_position(),
            source: failure.into_inner(),
        }),
    }
}

/// Spells a path the deserialiser tracked in the document's own terms.
///
/// The tracked path names serde's view: a field renamed `$value` for the element a
/// choice carries, `$text` for character data, and an index for the position in a
/// repeated field. A reader of the document knows none of those, so the `$`-names
/// are dropped and an index that belonged to one is carried onto the element it
/// selected — `$value[0].StopMonitoringDelivery` becomes
/// `StopMonitoringDelivery[0]`.
pub(crate) fn document_path(path: &serde_path_to_error::Path) -> String {
    use serde_path_to_error::Segment;

    let mut spelled = String::new();
    let mut after_dropped_name = false;
    let mut carried_index = None;
    for segment in path {
        match segment {
            Segment::Map { key } | Segment::Enum { variant: key } if key.starts_with('$') => {
                after_dropped_name = true;
            }
            Segment::Map { key } | Segment::Enum { variant: key } => {
                if !spelled.is_empty() {
                    spelled.push('.');
                }
                spelled.push_str(key);
                if let Some(index) = carried_index.take() {
                    spelled.push_str(&format!("[{index}]"));
                }
                after_dropped_name = false;
            }
            Segment::Seq { index } if after_dropped_name => carried_index = Some(*index),
            Segment::Seq { index } => spelled.push_str(&format!("[{index}]")),
            Segment::Unknown => {
                if !spelled.is_empty() {
                    spelled.push('.');
                }
                spelled.push('?');
                after_dropped_name = false;
            }
        }
    }
    spelled
}

/// Writes a SIRI document as a single line of XML, prefixed by an XML declaration.
///
/// A value holding a character XML 1.0 does not allow — a control character other
/// than tab, line feed and carriage return, U+FFFE or U+FFFF — is not written:
/// no reader would read the document.
pub fn to_string<T: SiriRoot>(value: &T) -> Result<String> {
    let mut document = String::from(PROLOGUE);
    let serialiser = quick_xml::se::Serializer::with_root(&mut document, Some(T::ELEMENT_NAME)).map_err(Error::from)?;
    value.serialize(serialiser).map_err(Error::from)?;
    finish::<T>(document)
}

/// Writes a SIRI document indented with tabs, the layout the official examples use,
/// refusing what [`to_string`] refuses.
pub fn to_string_pretty<T: SiriRoot>(value: &T) -> Result<String> {
    let mut document = String::from(PROLOGUE);
    let mut serialiser =
        quick_xml::se::Serializer::with_root(&mut document, Some(T::ELEMENT_NAME)).map_err(Error::from)?;
    serialiser.indent('\t', 1);
    value.serialize(serialiser).map_err(Error::from)?;
    finish::<T>(document)
}

/// A serialised document, checked for characters XML 1.0 does not allow and put
/// into the SIRI namespace.
fn finish<T: SiriRoot>(mut document: String) -> Result<String> {
    if let Some((at, character)) = characters::first_illegal(&document) {
        return Err(Error::IllegalCharacter {
            character,
            offset: at as u64,
        });
    }
    namespace::declare_default_namespace(&mut document, PROLOGUE.len(), T::ELEMENT_NAME);
    Ok(document)
}

/// The XML declaration every document written here opens with, and the line break
/// after it. The root element follows immediately, which is what tells
/// [`namespace::declare_default_namespace`] where to write the namespace.
const PROLOGUE: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n";

/// Fails unless the first element of `xml` is `T::ELEMENT_NAME` in the SIRI
/// namespace.
///
/// Without this check a document rooted at the wrong message would deserialise into
/// `T` field by field and quietly produce a value the sender never meant, and one in
/// another vocabulary would be read as SIRI for sharing its element names. Only what
/// fails is spelled out, so the check that passes allocates nothing.
fn check_root<T: SiriRoot>(xml: &str) -> Result<()> {
    use quick_xml::events::Event;
    use quick_xml::name::{NamespaceError, ResolveResult};

    let mut reader = quick_xml::NsReader::from_str(xml);
    loop {
        let (namespace, element) = match reader.read_resolved_event()? {
            (namespace, Event::Start(e) | Event::Empty(e)) => (namespace, e),
            (_, Event::Eof) => {
                return Err(Error::UnexpectedRoot {
                    expected: T::ELEMENT_NAME,
                    found: String::new(),
                })
            }
            _ => continue,
        };
        let found = element.local_name();
        if found.as_ref() != T::ELEMENT_NAME.as_bytes() {
            return Err(Error::UnexpectedRoot {
                expected: T::ELEMENT_NAME,
                found: String::from_utf8_lossy(found.as_ref()).into_owned(),
            });
        }
        let found = match namespace {
            ResolveResult::Bound(bound) if bound.as_ref() == NAMESPACE.as_bytes() => return Ok(()),
            ResolveResult::Bound(bound) => String::from_utf8_lossy(bound.as_ref()).into_owned(),
            ResolveResult::Unbound => String::new(),
            ResolveResult::Unknown(prefix) => {
                return Err(Error::Xml(quick_xml::Error::Namespace(NamespaceError::UnknownPrefix(
                    prefix,
                ))))
            }
        };
        return Err(Error::UnexpectedNamespace {
            element: T::ELEMENT_NAME,
            found,
        });
    }
}
