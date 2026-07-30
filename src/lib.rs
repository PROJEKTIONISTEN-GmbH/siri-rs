//! CEN SIRI — Service Interface for Real-time Information — in Rust.
//!
//! SIRI (EN 15531 / CEN/TS 15531) is the European standard for exchanging
//! real-time public transport data. This crate implements the SIRI **framework**
//! (message envelope, request/response, discovery and capabilities), the full
//! **publish/subscribe data hub** (subscription lifecycle, data-ready
//! notifications, fetched and direct delivery, heartbeats), and the
//! **Situation Exchange** service (SIRI-SX) that carries incidents and
//! disruptions.
//!
//! Both roles are supported: use it to consume a producer's feed, or to run one.
//!
//! # Layout
//!
//! | Module | What it holds |
//! |---|---|
//! | [`framework`] | The `<Siri>` envelope and every framework message |
//! | [`sx`] | Situation Exchange requests, deliveries and `PtSituationElement` |
//! | [`pubsub`] | The subscription state machines and the transport seam |
//! | [`model`] | References, locations and features shared across services |
//! | [`types`] | Primitive datatypes: identifiers, texts, durations |
//! | [`enumerations`] | The schema's enumerated code lists |
//!
//! # Reading and writing
//!
//! ```
//! let xml = r#"<Siri xmlns="http://www.siri.org.uk/siri" version="2.0">
//!   <HeartbeatNotification>
//!     <RequestTimestamp>2004-12-17T09:30:47-05:00</RequestTimestamp>
//!     <ProducerRef>KUBRICK</ProducerRef>
//!     <Status>true</Status>
//!   </HeartbeatNotification>
//! </Siri>"#;
//!
//! let message: siri_rs::Siri = siri_rs::from_str(xml)?;
//! let heartbeat = message.payload.as_heartbeat_notification().unwrap();
//! assert_eq!(heartbeat.producer_ref.as_ref().unwrap().as_str(), "KUBRICK");
//!
//! let written = siri_rs::to_string(&message)?;
//! assert!(written.contains("<ProducerRef>KUBRICK</ProducerRef>"));
//! # Ok::<(), siri_rs::Error>(())
//! ```
//!
//! # How the schema is modelled
//!
//! The XML Schema is the authority; where idiomatic Rust and schema fidelity
//! disagree, fidelity wins and the API absorbs the awkwardness. Three conventions
//! follow from that:
//!
//! * **Field order is element order.** Serialisation writes struct fields in
//!   declaration order, so fields appear in the order the schema's `xsd:sequence`
//!   prescribes. Attributes are declared before elements.
//! * **A choice between single elements is an enum**, written as the element name —
//!   see [`framework::ErrorCondition`].
//! * **A choice between element *groups* is a set of optional fields**, because
//!   the alternatives are not single elements and cannot carry an enum's tag. Such
//!   types offer constructors for each alternative and an accessor that reports
//!   which one is present — see [`model::Location::position`].
//!
//! # Conformance
//!
//! Every message this crate models is checked against the official SIRI v2.2
//! schemas and example documents: each example is parsed, written back out,
//! compared to the original element by element, and validated against the schema.
//! `tests/fixtures/README.md` lists the documents covered, including the German
//! VDV 736 profile.

#![deny(missing_docs)]
#![forbid(unsafe_code)]

#[macro_use]
mod macros;

pub mod enumerations;
mod error;
pub mod framework;
pub mod model;
pub mod pubsub;
pub mod sx;
pub mod types;
mod xml;

pub use error::{Error, Result};
pub use framework::{
    CheckStatusRequest, CheckStatusResponse, HeartbeatNotification, ServiceDelivery, ServiceRequest,
    Siri, SiriPayload, SubscriptionRequest, SubscriptionResponse,
};
pub use types::{Duration, NaturalLanguageString, ParticipantRef, SubscriptionRef};
pub use xml::{from_str, to_string, to_string_pretty, SiriRoot, NAMESPACE};
