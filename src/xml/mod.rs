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

pub mod namespace;
pub(crate) mod token_list;

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
/// element is not the one `T` describes.
pub fn from_str<T: SiriRoot>(xml: &str) -> Result<T> {
    let normalised = namespace::normalise(xml)?;
    check_root::<T>(&normalised)?;
    let rewritten = matches!(normalised, std::borrow::Cow::Owned(_));
    deserialize(&normalised, !rewritten)
}

/// Reads a value out of `xml`, reporting where in the document a failure was.
///
/// The path is tracked as the deserialiser descends; `offsets_apply` says whether
/// `xml` is the text the caller handed over, which is what a byte offset has to
/// count in to be of any use.
pub(crate) fn deserialize<T: DeserializeOwned>(xml: &str, offsets_apply: bool) -> Result<T> {
    // Tracking the path costs something on every field of every document, and what
    // it buys is only ever spent on one that fails. So a document is read without it
    // first, and only a failure is read a second time to find out where it was: the
    // reader is deterministic, so the second read fails in the same place as the
    // first. A document that parses pays nothing; one that does not is already lost.
    let mut untracked = quick_xml::de::Deserializer::from_str(xml);
    if let Ok(value) = T::deserialize(&mut untracked) {
        return Ok(value);
    }
    let mut deserializer = quick_xml::de::Deserializer::from_str(xml);
    serde_path_to_error::deserialize(&mut deserializer).map_err(|failure| Error::Deserialize {
        path: document_path(failure.path()),
        offset: offsets_apply.then(|| deserializer.get_ref().get_ref().buffer_position()),
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
fn document_path(path: &serde_path_to_error::Path) -> String {
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
