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

pub(crate) mod defaulted;
pub mod namespace;
pub(crate) mod schema_default;
pub(crate) mod token_list;
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
/// element is not the one `T` describes. An element whose content is whitespace
/// alone is read as that whitespace, which the deserialiser would otherwise drop.
pub fn from_str<T: SiriRoot>(xml: &str) -> Result<T> {
    let text = prepare::<T>(xml)?;
    let offsets_apply = matches!(text, Cow::Borrowed(_));
    deserialize(&text, offsets_apply)
}

/// Makes a document ready for the reader: the SIRI namespace bound by default,
/// whitespace-only content kept, and the root element checked against `T`.
///
/// The text comes back borrowed when nothing had to be rewritten, which is when
/// a byte offset into it is an offset into what the caller handed over.
pub(crate) fn prepare<T: SiriRoot>(xml: &str) -> Result<Cow<'_, str>> {
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
    check_root::<T>(&text)?;
    Ok(text)
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
    let mut untracked = quick_xml::de::Deserializer::from_str(xml);
    if let Ok(value) = T::deserialize(&mut untracked) {
        return Ok(value);
    }
    let mut deserializer = quick_xml::de::Deserializer::from_str(xml);
    serde_path_to_error::deserialize(&mut deserializer).map_err(|failure| Fault {
        path: failure.path().clone(),
        offset: deserializer.get_ref().get_ref().buffer_position(),
        source: failure.into_inner(),
    })
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
pub fn to_string<T: SiriRoot>(value: &T) -> Result<String> {
    let mut document = String::from(PROLOGUE);
    let serialiser = quick_xml::se::Serializer::with_root(&mut document, Some(T::ELEMENT_NAME))
        .map_err(Error::from)?;
    value.serialize(serialiser).map_err(Error::from)?;
    namespace::declare_default_namespace(&mut document, PROLOGUE.len(), T::ELEMENT_NAME);
    Ok(document)
}

/// Writes a SIRI document indented with tabs, the layout the official examples use.
pub fn to_string_pretty<T: SiriRoot>(value: &T) -> Result<String> {
    let mut document = String::from(PROLOGUE);
    let mut serialiser = quick_xml::se::Serializer::with_root(&mut document, Some(T::ELEMENT_NAME))
        .map_err(Error::from)?;
    serialiser.indent('\t', 1);
    value.serialize(serialiser).map_err(Error::from)?;
    namespace::declare_default_namespace(&mut document, PROLOGUE.len(), T::ELEMENT_NAME);
    Ok(document)
}

/// The XML declaration every document written here opens with, and the line break
/// after it. The root element follows immediately, which is what tells
/// [`namespace::declare_default_namespace`] where to write the namespace.
const PROLOGUE: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n";

/// Fails unless the first element of `xml` is `T::ELEMENT_NAME`.
///
/// Without this check a document rooted at the wrong message would deserialise into
/// `T` field by field and quietly produce a value the sender never meant. Only the
/// name that fails is spelled out, so the check that passes allocates nothing.
fn check_root<T: SiriRoot>(xml: &str) -> Result<()> {
    let mut reader = quick_xml::Reader::from_str(xml);
    loop {
        let element = match reader.read_event()? {
            quick_xml::events::Event::Start(e) | quick_xml::events::Event::Empty(e) => e,
            quick_xml::events::Event::Eof => {
                return Err(Error::UnexpectedRoot {
                    expected: T::ELEMENT_NAME,
                    found: String::new(),
                })
            }
            _ => continue,
        };
        let found = element.local_name();
        return if found.as_ref() == T::ELEMENT_NAME.as_bytes() {
            Ok(())
        } else {
            Err(Error::UnexpectedRoot {
                expected: T::ELEMENT_NAME,
                found: String::from_utf8_lossy(found.as_ref()).into_owned(),
            })
        };
    }
}
