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
    quick_xml::de::from_str(&normalised).map_err(Error::from)
}

/// Writes a SIRI document as a single line of XML, prefixed by an XML declaration.
pub fn to_string<T: SiriRoot>(value: &T) -> Result<String> {
    let body = quick_xml::se::to_string_with_root(T::ELEMENT_NAME, value)?;
    Ok(format!(
        "{DECLARATION}\n{}",
        namespace::declare_default_namespace(&body)?
    ))
}

/// Writes a SIRI document indented with tabs, the layout the official examples use.
pub fn to_string_pretty<T: SiriRoot>(value: &T) -> Result<String> {
    let mut body = String::new();
    let mut serialiser =
        quick_xml::se::Serializer::with_root(&mut body, Some(T::ELEMENT_NAME)).map_err(Error::from)?;
    serialiser.indent('\t', 1);
    value.serialize(serialiser).map_err(Error::from)?;
    Ok(format!(
        "{DECLARATION}\n{}",
        namespace::declare_default_namespace(&body)?
    ))
}

const DECLARATION: &str = r#"<?xml version="1.0" encoding="UTF-8"?>"#;

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
