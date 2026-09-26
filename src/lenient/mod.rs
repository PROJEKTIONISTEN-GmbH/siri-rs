//! Reading a document that carries faults, keeping what reads.
//!
//! [`crate::from_str`] reads a document whole or not at all: one element that
//! cannot be read fails the document, and every valid record in it with it. A
//! program that reads a producer's stream has no say in what arrives, and a
//! delivery of a thousand journeys is worth reading for the one that carries an
//! empty timestamp. [`from_str`] here reads such a document as far as it goes:
//! what cannot be read is left out, the rest is read as the strict reader would
//! read it, and every element left out is reported as a [`Finding`] — where it
//! was, why, and what went with it.
//!
//! A document the strict reader accepts reads here exactly as it does there,
//! with no findings and at the same cost: the lenient reading *is* the strict
//! reading until it fails. Only a document that fails is read again, in pieces.
//!
//! # What may be left out
//!
//! A document is read in *units*. The units are the repeated records a functional
//! service delivery carries, and the deliveries themselves:
//!
//! | Service | Records left out whole |
//! |---|---|
//! | Production Timetable | `DatedVehicleJourney`, `RemovedDatedVehicleJourney`, `ServiceJourneyInterchange`, `RemovedServiceJourneyInterchange` |
//! | Estimated Timetable | `EstimatedVehicleJourney`, `EstimatedServiceJourneyInterchange` |
//! | Stop Timetable | `TimetabledStopVisit`, `TimetabledStopVisitCancellation` |
//! | Stop Monitoring | `MonitoredStopVisit`, `MonitoredStopVisitCancellation`, `StopLineNotice`, `StopLineNoticeCancellation`, `StopNotice`, `StopNoticeCancellation`, `ServiceException` |
//! | Vehicle Monitoring | `VehicleActivity`, `VehicleActivityCancellation` |
//! | Connection Timetable | `TimetabledFeederArrival`, `TimetabledFeederArrivalCancellation` |
//! | Connection Monitoring | `MonitoredFeederArrival`, `MonitoredFeederArrivalCancellation`, `WaitProlongedDeparture`, `StoppingPositionChangedDeparture`, `DistributorDepartureCancellation` |
//! | General Message | `GeneralMessage`, `GeneralMessageCancellation` |
//! | Facility Monitoring | `FacilityCondition` |
//! | Situation Exchange | `PtSituationElement`, `RoadSituationElement` |
//! | Control Actions | `ControlAction`, `GroupOfControlActions`, `RevokedControlAction`, `DriverMessage`, `VehicleDetecting` |
//! | every service | the `…Delivery` element itself, while another remains |
//!
//! Inside a unit, an element that cannot be read costs the smallest thing the
//! document reads without:
//!
//! * where the schema leaves the element **optional** (`minOccurs="0"`), the
//!   element alone is left out and the unit kept — an empty `AimedArrivalTime`
//!   costs the timestamp, not the stop visit;
//! * where the element is **mandatory**, its parent cannot be read without it,
//!   and the same rule applies to the parent, up to the unit — an empty
//!   `RecordedAtTime` costs the stop visit. The rule is applied by reading, not
//!   from a table: the element is taken out and the fragment read again, and a
//!   parent that then lacks a mandatory element is the next thing taken out;
//! * a list of records the reader finds **interrupted** by another element — a
//!   second run of `PtSituationElement` after a `RoadSituationElement`, which the
//!   reader cannot place — keeps its first run; the records after the
//!   interruption are left out one by one;
//! * a delivery whose own elements cannot be read is left out **whole**, with
//!   every record in it reported, as long as another delivery remains. The last
//!   delivery is never left out: a `ServiceDelivery` cannot be read without one.
//!
//! A unit is never replaced by a guess. What does not read is left out; nothing
//! is put in its place.
//!
//! # What is never left out
//!
//! The envelope — `Siri`, `ServiceDelivery` and the latter's own elements such as
//! `ResponseTimestamp` and `ProducerRef` — is read strictly: a fault there fails
//! the document with the error [`crate::from_str`] would give. So does a fault
//! in the only delivery, and so does everything that is not a fault *in* the
//! document but *of* it: XML that is not well-formed, a root element that is not
//! `<Siri>`, and a document in a namespace that is not SIRI's.
//!
//! # Paths
//!
//! A finding names elements the way [`crate::Error::Deserialize`] does: the
//! elements from the document root down, `.`-separated, with `[n]` for the
//! n-th of a repeated element, counted from 0 —
//! `ServiceDelivery.StopMonitoringDelivery[0].MonitoredStopVisit[3].RecordedAtTime`.
//! The count is the document's as it arrived: a unit left out ahead of a later
//! one still counts, so that a path can be looked up in the text the producer
//! sent.
//!
//! # Cost
//!
//! Reading a document that reads strictly costs one strict reading. Reading one
//! that does not costs a few more over the same text: once to find the first
//! fault, once over each record on its own, once over each delivery, and once
//! over the whole document as it is left — so a delivery with a hundred faults
//! costs about as much as one with a single fault. A record is read on its own
//! once, and again for each element taken out of it.
//!
//! ```
//! let xml = r#"<Siri xmlns="http://www.siri.org.uk/siri" version="2.0">
//!   <ServiceDelivery>
//!     <ResponseTimestamp>2004-12-17T09:30:46-05:00</ResponseTimestamp>
//!     <StopMonitoringDelivery>
//!       <ResponseTimestamp>2004-12-17T09:30:47-05:00</ResponseTimestamp>
//!       <MonitoredStopVisit>
//!         <RecordedAtTime/>
//!         <ItemIdentifier>visit-1</ItemIdentifier>
//!         <MonitoringRef>STOP-1</MonitoringRef>
//!         <MonitoredVehicleJourney><LineRef>1</LineRef></MonitoredVehicleJourney>
//!       </MonitoredStopVisit>
//!       <MonitoredStopVisit>
//!         <RecordedAtTime>2004-12-17T09:30:47-05:00</RecordedAtTime>
//!         <ItemIdentifier>visit-2</ItemIdentifier>
//!         <MonitoringRef>STOP-1</MonitoringRef>
//!         <MonitoredVehicleJourney><LineRef>1</LineRef></MonitoredVehicleJourney>
//!       </MonitoredStopVisit>
//!     </StopMonitoringDelivery>
//!   </ServiceDelivery>
//! </Siri>"#;
//!
//! assert!(siri_rs::from_str::<siri_rs::Siri>(xml).is_err());
//!
//! let (siri, findings) = siri_rs::lenient::from_str(xml)?;
//! let delivery = siri.payload.as_service_delivery().unwrap();
//! let visits = match &delivery.deliveries[0] {
//!     siri_rs::framework::ServiceDeliveryPayload::StopMonitoringDelivery(sm) => &sm.monitored_stop_visit,
//!     _ => unreachable!(),
//! };
//! assert_eq!(visits.len(), 1);
//! assert_eq!(findings.len(), 1);
//! let finding = findings.iter().next().unwrap();
//! assert_eq!(
//!     finding.path,
//!     "ServiceDelivery.StopMonitoringDelivery[0].MonitoredStopVisit[0].RecordedAtTime"
//! );
//! assert_eq!(finding.discarded.unit_name(), Some("MonitoredStopVisit"));
//! assert_eq!(finding.discarded.identifier(), Some("visit-1"));
//! # Ok::<(), siri_rs::Error>(())
//! ```

mod document;
mod repair;
mod units;

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt;

use crate::framework::Siri;
use crate::xml;
use crate::Result;

/// Reads a SIRI document, leaving out what cannot be read.
///
/// Reads like [`crate::from_str`] — the namespace may be bound by default or to a
/// prefix, the root must be `<Siri>` — and, where that reading fails inside a
/// functional service delivery, reads on without the element or unit at fault,
/// as the module documentation describes. The document comes back with one
/// [`Finding`] per element left out; a document the strict reader accepts comes
/// back as it would from there, with none.
///
/// # Errors
///
/// The errors [`crate::from_str`] gives, for the faults it would give them for
/// outside a delivery: in the envelope, in the only delivery, or in the XML
/// itself.
pub fn from_str(xml: &str) -> Result<(Siri, Findings)> {
    let text = xml::prepare::<Siri>(xml)?;
    let fault = match xml::read::<Siri>(&text) {
        Ok(siri) => return Ok((siri, Findings::default())),
        Err(fault) => fault,
    };
    if !document::within_a_delivery(&fault.path) {
        return Err(fault.into_error(matches!(text, Cow::Borrowed(_))));
    }
    let mut document = text.into_owned();
    let mut findings = Findings::default();
    repair::deliveries(&mut document, &mut findings)?;
    let siri = xml::read::<Siri>(&document).map_err(|fault| fault.into_error(false))?;
    Ok((siri, findings))
}

/// One element the lenient reader could not read, and what was left out for it.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Finding {
    /// The element that could not be read, from the document root down, in the
    /// terms [`crate::Error::Deserialize`] uses.
    pub path: String,
    /// What the reader could not do there, as it reported it.
    pub reason: String,
    /// What was left out because of it.
    pub discarded: Discarded,
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cannot read {}: {}; left out {}", self.path, self.reason, self.discarded)
    }
}

/// What the lenient reader left out for one element it could not read.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Discarded {
    /// An element inside a unit; the unit itself was kept. The element is the
    /// one at fault or, where that one is mandatory, the smallest optional
    /// element around it.
    Element {
        /// The element's path from the document root.
        path: String,
    },
    /// A whole unit — a record of a functional service, or a delivery.
    Unit {
        /// The unit's path from the document root.
        path: String,
        /// The unit's element name, e.g. `PtSituationElement`.
        name: String,
        /// What identifies the unit to a reader of the stream — its
        /// `SituationNumber`, `DatedVehicleJourneyRef`, `ItemIdentifier` or the
        /// like — when the unit carries one.
        identifier: Option<String>,
    },
}

impl Discarded {
    /// The path of what was left out, from the document root.
    pub fn path(&self) -> &str {
        match self {
            Discarded::Element { path } | Discarded::Unit { path, .. } => path,
        }
    }

    /// Whether a whole unit was left out rather than an element inside one.
    pub fn is_unit(&self) -> bool {
        matches!(self, Discarded::Unit { .. })
    }

    /// The element name of the unit left out, if a unit was.
    pub fn unit_name(&self) -> Option<&str> {
        match self {
            Discarded::Unit { name, .. } => Some(name),
            Discarded::Element { .. } => None,
        }
    }

    /// What identifies the unit left out, if a unit was and it carried one.
    pub fn identifier(&self) -> Option<&str> {
        match self {
            Discarded::Unit { identifier, .. } => identifier.as_deref(),
            Discarded::Element { .. } => None,
        }
    }
}

impl fmt::Display for Discarded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Discarded::Element { path } => write!(f, "the element {path}"),
            Discarded::Unit {
                path,
                name,
                identifier: Some(identifier),
            } => write!(f, "the {name} {identifier} at {path}"),
            Discarded::Unit {
                path,
                name,
                identifier: None,
            } => write!(f, "the {name} at {path}"),
        }
    }
}

/// Everything the lenient reader left out of one document, in document order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Findings {
    findings: Vec<Finding>,
}

impl Findings {
    /// How many elements were left out.
    pub fn len(&self) -> usize {
        self.findings.len()
    }

    /// Whether the document read whole.
    pub fn is_empty(&self) -> bool {
        self.findings.is_empty()
    }

    /// The findings, in document order.
    pub fn iter(&self) -> std::slice::Iter<'_, Finding> {
        self.findings.iter()
    }

    /// The findings as a slice.
    pub fn as_slice(&self) -> &[Finding] {
        &self.findings
    }

    /// The findings that left out a whole unit.
    pub fn units(&self) -> impl Iterator<Item = &Finding> {
        self.findings.iter().filter(|finding| finding.discarded.is_unit())
    }

    /// The findings that left out an element inside a unit that was kept.
    pub fn elements(&self) -> impl Iterator<Item = &Finding> {
        self.findings.iter().filter(|finding| !finding.discarded.is_unit())
    }

    /// How many findings there are for each reason, so that a stream's faults can
    /// be counted by kind rather than read one by one.
    pub fn by_reason(&self) -> BTreeMap<&str, usize> {
        let mut counts = BTreeMap::new();
        for finding in &self.findings {
            *counts.entry(finding.reason.as_str()).or_insert(0) += 1;
        }
        counts
    }

    /// Adds a finding and answers its position.
    fn push(&mut self, finding: Finding) -> usize {
        self.findings.push(finding);
        self.findings.len() - 1
    }
}

impl IntoIterator for Findings {
    type Item = Finding;
    type IntoIter = std::vec::IntoIter<Finding>;

    fn into_iter(self) -> Self::IntoIter {
        self.findings.into_iter()
    }
}

impl<'a> IntoIterator for &'a Findings {
    type Item = &'a Finding;
    type IntoIter = std::slice::Iter<'a, Finding>;

    fn into_iter(self) -> Self::IntoIter {
        self.findings.iter()
    }
}
