//! Error types returned by this crate.

use std::fmt;

/// Errors raised while reading or writing SIRI XML.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The document is not well-formed XML.
    Xml(quick_xml::Error),
    /// The document is well-formed XML but does not match the expected SIRI structure.
    Deserialize {
        /// Where in the document the reader was: the elements from the root down
        /// to the one that could not be read, `.`-separated, with `[n]` for the
        /// n-th of a repeated element — `ServiceDelivery.StopMonitoringDelivery.\
        /// MonitoredStopVisit[1].RecordedAtTime`. Empty when the failure has no
        /// place, such as a root element of the wrong kind.
        path: String,
        /// How far into the document, in bytes, the reader had got when it
        /// failed. `None` when the document was rewritten before it was read —
        /// one that binds the SIRI namespace to a prefix is — since an offset
        /// into text the caller never saw would mislead.
        offset: Option<u64>,
        /// What the reader could not do there.
        source: quick_xml::DeError,
    },
    /// A value could not be written as SIRI XML.
    Serialize(quick_xml::SeError),
    /// The document root element is not the one the requested type describes.
    ///
    /// SIRI declares every message as a global element, so the root element name
    /// carries the message kind. Reading a `CheckStatusResponse` out of a document
    /// rooted at `<HeartbeatNotification>` is a caller error, not a parse error.
    UnexpectedRoot {
        /// Element name the requested type expects.
        expected: &'static str,
        /// Element name actually found (without namespace prefix).
        found: String,
    },
    /// A lexical value did not match the XML Schema datatype it is declared as.
    InvalidValue {
        /// The XML Schema datatype the value was read as, e.g. `xsd:duration`.
        datatype: &'static str,
        /// The offending lexical form.
        value: String,
    },
    /// The message is valid SIRI, but not one this side of the exchange can act on.
    ///
    /// A producer handed a `SubscriptionResponse`, or a `ServiceRequest` for a
    /// service other than the one it serves, has nothing to answer with. The
    /// document was read correctly; it is the conversation that is wrong.
    UnexpectedMessage {
        /// What this side can act on.
        expected: &'static str,
        /// The message, or the element inside it, that arrived instead.
        found: &'static str,
    },
    /// The message is valid SIRI, but lacks an element the recipient needs.
    ///
    /// The schema leaves `ConsumerRef` optional on a `DataSupplyRequest`, yet a
    /// producer cannot tell whose subscriptions to supply without it. The message is
    /// well-formed, and still unanswerable.
    MissingElement {
        /// The message that lacks the element.
        message: &'static str,
        /// The element it lacks.
        element: &'static str,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Xml(e) => write!(f, "malformed XML: {e}"),
            Error::Deserialize {
                path,
                offset,
                source,
            } => {
                write!(f, "cannot read SIRI document")?;
                if !path.is_empty() {
                    write!(f, " at {path}")?;
                }
                if let Some(offset) = offset {
                    write!(f, " (byte {offset})")?;
                }
                write!(f, ": {source}")
            }
            Error::Serialize(e) => write!(f, "cannot write SIRI document: {e}"),
            Error::UnexpectedRoot { expected, found } => {
                write!(f, "expected root element <{expected}>, found <{found}>")
            }
            Error::InvalidValue { datatype, value } => {
                write!(f, "{value:?} is not a valid {datatype}")
            }
            Error::UnexpectedMessage { expected, found } => {
                write!(f, "expected {expected}, found <{found}>")
            }
            Error::MissingElement { message, element } => {
                write!(f, "<{message}> carries no <{element}>")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Xml(e) => Some(e),
            Error::Deserialize { source, .. } => Some(source),
            Error::Serialize(e) => Some(e),
            Error::UnexpectedRoot { .. }
            | Error::InvalidValue { .. }
            | Error::UnexpectedMessage { .. }
            | Error::MissingElement { .. } => None,
        }
    }
}

impl From<quick_xml::Error> for Error {
    fn from(e: quick_xml::Error) -> Self {
        Error::Xml(e)
    }
}

impl From<quick_xml::SeError> for Error {
    fn from(e: quick_xml::SeError) -> Self {
        Error::Serialize(e)
    }
}

/// Result alias for [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
