//! CEN SIRI — Service Interface for Real-time Information — in Rust.
//!
//! SIRI (EN 15531 / CEN/TS 15531) is the European standard for exchanging
//! real-time public transport data. This crate implements the SIRI **framework**
//! (message envelope, request/response, discovery and capabilities), the full
//! **publish/subscribe data hub** (subscription lifecycle, data-ready
//! notifications, fetched and direct delivery, heartbeats), and every functional
//! service on top of it: the planned timetable and the timetable as it is actually
//! running, the two of them seen from a stop, the vehicles running it, the
//! interchanges between them planned and monitored, free-form messages, the state
//! of passenger facilities, the incidents and disruptions that perturb it all, and
//! what a control room decides to do about them.
//!
//! Both roles are supported: use it to consume a producer's feed, or to run one.
//!
//! # Layout
//!
//! | Module | What it holds |
//! |---|---|
//! | [`framework`] | The `<Siri>` envelope and every framework message |
//! | [`pt`] | Production Timetable: the day's plan |
//! | [`et`] | Estimated Timetable: the plan as it is running |
//! | [`st`] | Stop Timetable: the plan at one stop |
//! | [`sm`] | Stop Monitoring: what is due at one stop now |
//! | [`vm`] | Vehicle Monitoring: where the vehicles are |
//! | [`ct`] | Connection Timetable: the interchanges planned over a link |
//! | [`cm`] | Connection Monitoring: whether those interchanges will be made |
//! | [`gm`] | General Message: free-form messages on named channels |
//! | [`fm`] | Facility Monitoring: whether the lift is working |
//! | [`ca`] | Control Actions: what the control room has decided to do |
//! | [`sx`] | Situation Exchange requests, deliveries and `PtSituationElement` |
//! | [`pubsub`] | The subscription state machines and the transport seam |
//! | [`model`] | The journey model, references and locations shared across services |
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
//! # Speed
//!
//! [`from_str`] hands the document to the deserialiser borrowed unless it writes an
//! element with a namespace prefix, and [`to_string`] produces the finished
//! document — declaration, namespace and body — into a single buffer. What is left
//! is the profile the finished program is built with, which a library cannot impose
//! on its consumer: a program that wants the calls into this crate optimised across
//! the crate boundary asks for `lto` and `codegen-units = 1` in its own
//! `[profile.release]`. `benches/` measures reading, writing and one turn of the
//! publish/subscribe cycle over the official example documents.
//!
//! # At an open port
//!
//! A document from the other side may be anything, and a reader that answers it
//! with an `Err` has done its job; one that takes the process down has not.
//! Entity expansion is refused by the XML layer, a text node of any length reads in
//! linear time, and open content — an `<Extensions>` payload, a general-message
//! body, an embedded DATEX II record — may nest at most
//! [`types::AnyContent::MAX_DEPTH`] elements deep, which stops the one recursion the
//! schema does not bound. What the reader cannot do is bound the *size* of what it
//! is given, because it is given a string: the transport in front of it should.
//! The largest official request example is 3 KB and the largest delivery
//! example 111 KB, so a producer answering requests loses nothing by refusing a
//! body over 1 MiB, and a consumer receiving deliveries should set its limit at
//! the largest delivery its producer sends — the examples take 16 MiB.
//!
//! # How the schema is modelled
//!
//! The XML Schema is the authority; where idiomatic Rust and schema fidelity
//! disagree, fidelity wins and the API absorbs the awkwardness. Four conventions
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
//! * **What the schema leaves open is kept, not interpreted.** An `<Extensions>`
//!   payload, a general-message body and an embedded DATEX II record belong to
//!   profiles outside SIRI, so they are held as the subtree they are and written
//!   back unchanged — and read into a consumer's own type where it has one. See
//!   [`types::AnyContent`].
//! * **A timestamp names an instant.** `xsd:dateTime` lets the time-zone offset
//!   be left out, and a document that leaves it out is schema-valid; but a
//!   wall-clock time without an offset names no instant — XML Schema itself
//!   ranks it as incomparable with a zoned one — and the hub compares instants:
//!   an `InitialTerminationTime` against now, a `ValidUntil` against the next
//!   poll. Every timestamp is therefore a [`chrono::DateTime<FixedOffset>`], and
//!   a document that leaves the offset out is refused with an error naming the
//!   field. The one place the official examples do leave it out — the validity
//!   period of a Production Timetable, which is a period in the timetable's own
//!   zone — is a [`types::Timestamp`], which keeps the lexical form and offers the
//!   instant only when the document gave one.
//!
//! # Conformance
//!
//! Every message this crate models is checked against the official SIRI v2.2
//! schemas and example documents: each example is parsed, written back out,
//! compared to the original element by element, and validated against the schema.
//! `tests/fixtures/README.md` lists the documents covered, including the German
//! VDV 736 profile, and the one official document left out — the capabilities
//! response for all eleven services at once, which needs a discovery structure
//! the crate does not model. [`ca`] is the one service with no example documents
//! to check against, and its own documentation says what is checked instead.

#![deny(missing_docs)]
#![forbid(unsafe_code)]

#[macro_use]
mod macros;

pub mod ca;
pub mod cm;
pub mod ct;
pub mod enumerations;
mod error;
pub mod et;
pub mod fm;
pub mod framework;
pub mod gm;
pub mod model;
pub mod pt;
pub mod pubsub;
pub mod sm;
pub mod st;
pub mod sx;
pub mod types;
pub mod vm;
mod xml;

pub use error::{Error, Result};
pub use framework::{
    CheckStatusRequest, CheckStatusResponse, HeartbeatNotification, ServiceDelivery, ServiceRequest,
    Siri, SiriPayload, SubscriptionRequest, SubscriptionResponse,
};
pub use types::{Duration, NaturalLanguageString, ParticipantRef, SubscriptionRef};
pub use xml::{from_str, to_string, to_string_pretty, SiriRoot, NAMESPACE};
