//! The units a document is read in when it is read leniently.
//!
//! A functional service delivery carries its data as repeated records — the
//! situations of a situation exchange, the journeys of an estimated timetable,
//! the visits of a stop monitoring delivery — and a record is what a reader of the
//! stream can do without when it cannot be read: leaving one out makes nothing
//! else in the document false. The deliveries themselves are the units above
//! them. The table here names every record of every service, the elements between
//! it and its delivery, and the elements that tell a reader which record was lost.

use serde::de::DeserializeOwned;

use super::repair::{self, Outcome, Scope};
use super::Findings;
use crate::{ca, cm, ct, gm, model, sm, st, sx, vm};

/// A repeated record of a functional service delivery: what is left out whole
/// when an element of it that the schema makes mandatory cannot be read.
pub(super) struct Record {
    /// The delivery element the record belongs to.
    pub(super) delivery: &'static str,
    /// The elements between the delivery and the record, from the top down.
    pub(super) containers: &'static [Container],
    /// The record's element name.
    pub(super) name: &'static str,
    /// The elements that tell a reader of the stream which record was left out,
    /// the most telling first. The first of them the record carries names it in
    /// a finding.
    pub(super) identifiers: &'static [&'static str],
    /// Reads the record as its own type, leaving out what does not read.
    pub(super) read: fn(&mut String, &str, &Scope<'_>, &mut Findings) -> Outcome,
}

impl Record {
    /// The element the record is a direct child of.
    pub(super) fn parent(&self) -> &'static str {
        self.containers.last().map_or(self.delivery, |container| container.name)
    }
}

/// An element between a delivery and its records.
pub(super) struct Container {
    /// The element name.
    pub(super) name: &'static str,
    /// Whether the delivery may hold several, which is when a path says which one.
    pub(super) repeated: bool,
}

/// A record of type `R`.
const fn record<R: DeserializeOwned>(
    delivery: &'static str,
    containers: &'static [Container],
    name: &'static str,
    identifiers: &'static [&'static str],
) -> Record {
    Record {
        delivery,
        containers,
        name,
        identifiers,
        read: repair::fragment::<R>,
    }
}

/// The version frame of a Production Timetable delivery.
const DATED_FRAME: &[Container] = &[Container {
    name: "DatedTimetableVersionFrame",
    repeated: true,
}];

/// The version frame of an Estimated Timetable delivery.
const ESTIMATED_FRAME: &[Container] = &[Container {
    name: "EstimatedJourneyVersionFrame",
    repeated: true,
}];

/// The element a Situation Exchange delivery holds its situations in.
const SITUATIONS: &[Container] = &[Container {
    name: "Situations",
    repeated: false,
}];

/// A record straight under its delivery.
const NONE: &[Container] = &[];

/// Every record of every functional service, with the delivery it belongs to.
///
/// The names are the schema's; where a record's Rust type is shared between
/// services or lives in the journey model, it is named by its module.
pub(super) const RECORDS: &[Record] = &[
    record::<model::DatedVehicleJourney>(
        "ProductionTimetableDelivery",
        DATED_FRAME,
        "DatedVehicleJourney",
        &["DatedVehicleJourneyCode", "DatedVehicleJourneyRef", "VehicleJourneyRef"],
    ),
    record::<model::RemovedDatedVehicleJourney>(
        "ProductionTimetableDelivery",
        DATED_FRAME,
        "RemovedDatedVehicleJourney",
        &["DatedVehicleJourneyRef"],
    ),
    record::<model::ServiceJourneyInterchange>(
        "ProductionTimetableDelivery",
        DATED_FRAME,
        "ServiceJourneyInterchange",
        &["InterchangeCode", "ConnectionLinkRef"],
    ),
    record::<model::RemovedServiceJourneyInterchange>(
        "ProductionTimetableDelivery",
        DATED_FRAME,
        "RemovedServiceJourneyInterchange",
        &["InterchangeRef"],
    ),
    record::<model::EstimatedVehicleJourney>(
        "EstimatedTimetableDelivery",
        ESTIMATED_FRAME,
        "EstimatedVehicleJourney",
        &["DatedVehicleJourneyRef", "EstimatedVehicleJourneyCode"],
    ),
    record::<model::EstimatedServiceJourneyInterchange>(
        "EstimatedTimetableDelivery",
        ESTIMATED_FRAME,
        "EstimatedServiceJourneyInterchange",
        &["InterchangeRef", "InterchangeCode"],
    ),
    record::<st::TimetabledStopVisit>(
        "StopTimetableDelivery",
        NONE,
        "TimetabledStopVisit",
        &["ItemIdentifier", "DatedVehicleJourneyRef", "MonitoringRef"],
    ),
    record::<st::TimetabledStopVisitCancellation>(
        "StopTimetableDelivery",
        NONE,
        "TimetabledStopVisitCancellation",
        &["ItemRef", "DatedVehicleJourneyRef", "MonitoringRef"],
    ),
    record::<sm::MonitoredStopVisit>(
        "StopMonitoringDelivery",
        NONE,
        "MonitoredStopVisit",
        &["ItemIdentifier", "DatedVehicleJourneyRef", "MonitoringRef"],
    ),
    record::<sm::MonitoredStopVisitCancellation>(
        "StopMonitoringDelivery",
        NONE,
        "MonitoredStopVisitCancellation",
        &["ItemRef", "DatedVehicleJourneyRef", "MonitoringRef"],
    ),
    record::<sm::StopLineNotice>(
        "StopMonitoringDelivery",
        NONE,
        "StopLineNotice",
        &["ItemIdentifier", "LineRef"],
    ),
    record::<sm::StopLineNoticeCancellation>(
        "StopMonitoringDelivery",
        NONE,
        "StopLineNoticeCancellation",
        &["ItemRef", "LineRef"],
    ),
    record::<sm::StopNotice>(
        "StopMonitoringDelivery",
        NONE,
        "StopNotice",
        &["ItemIdentifier", "MonitoringRef"],
    ),
    record::<sm::StopNoticeCancellation>(
        "StopMonitoringDelivery",
        NONE,
        "StopNoticeCancellation",
        &["ItemRef", "MonitoringRef"],
    ),
    record::<sm::ServiceException>(
        "StopMonitoringDelivery",
        NONE,
        "ServiceException",
        &["MonitoringRef", "LineRef"],
    ),
    record::<vm::VehicleActivity>(
        "VehicleMonitoringDelivery",
        NONE,
        "VehicleActivity",
        &["ItemIdentifier", "VehicleRef", "VehicleMonitoringRef"],
    ),
    record::<vm::VehicleActivityCancellation>(
        "VehicleMonitoringDelivery",
        NONE,
        "VehicleActivityCancellation",
        &["ItemRef", "VehicleMonitoringRef", "DatedVehicleJourneyRef"],
    ),
    record::<ct::TimetabledFeederArrival>(
        "ConnectionTimetableDelivery",
        NONE,
        "TimetabledFeederArrival",
        &["ItemIdentifier", "InterchangeRef", "ConnectionLinkRef"],
    ),
    record::<ct::TimetabledFeederArrivalCancellation>(
        "ConnectionTimetableDelivery",
        NONE,
        "TimetabledFeederArrivalCancellation",
        &["ItemRef", "InterchangeRef", "ConnectionLinkRef"],
    ),
    record::<cm::MonitoredFeederArrival>(
        "ConnectionMonitoringFeederDelivery",
        NONE,
        "MonitoredFeederArrival",
        &["ItemIdentifier", "InterchangeRef", "ConnectionLinkRef"],
    ),
    record::<cm::MonitoredFeederArrivalCancellation>(
        "ConnectionMonitoringFeederDelivery",
        NONE,
        "MonitoredFeederArrivalCancellation",
        &["ItemIdentifier", "InterchangeRef", "ConnectionLinkRef"],
    ),
    record::<cm::WaitProlongedDeparture>(
        "ConnectionMonitoringDistributorDelivery",
        NONE,
        "WaitProlongedDeparture",
        &["ItemIdentifier", "InterchangeRef", "ConnectionLinkRef"],
    ),
    record::<cm::StoppingPositionChangedDeparture>(
        "ConnectionMonitoringDistributorDelivery",
        NONE,
        "StoppingPositionChangedDeparture",
        &["ItemIdentifier", "InterchangeRef", "ConnectionLinkRef"],
    ),
    record::<cm::DistributorDepartureCancellation>(
        "ConnectionMonitoringDistributorDelivery",
        NONE,
        "DistributorDepartureCancellation",
        &["ItemIdentifier", "InterchangeRef", "ConnectionLinkRef"],
    ),
    record::<gm::InfoMessage>(
        "GeneralMessageDelivery",
        NONE,
        "GeneralMessage",
        &["InfoMessageIdentifier", "ItemIdentifier"],
    ),
    record::<gm::InfoMessageCancellation>(
        "GeneralMessageDelivery",
        NONE,
        "GeneralMessageCancellation",
        &["InfoMessageIdentifier", "ItemRef"],
    ),
    record::<model::FacilityCondition>(
        "FacilityMonitoringDelivery",
        NONE,
        "FacilityCondition",
        &["FacilityRef", "FacilityCode"],
    ),
    record::<sx::PtSituationElement>(
        "SituationExchangeDelivery",
        SITUATIONS,
        "PtSituationElement",
        &["SituationNumber"],
    ),
    record::<sx::RoadSituationElement>(
        "SituationExchangeDelivery",
        SITUATIONS,
        "RoadSituationElement",
        &["SituationNumber"],
    ),
    record::<ca::ControlAction>(
        "ControlActionDelivery",
        &[Container {
            name: "controlActions",
            repeated: false,
        }],
        "ControlAction",
        &["ControlActionCode", "ItemIdentifier"],
    ),
    record::<ca::GroupOfControlActions>(
        "ControlActionDelivery",
        &[Container {
            name: "groupsOfControlActions",
            repeated: false,
        }],
        "GroupOfControlActions",
        &["GroupOfControlActionsCode"],
    ),
    record::<ca::RevokedControlAction>(
        "ControlActionDelivery",
        &[Container {
            name: "revokedControlActions",
            repeated: false,
        }],
        "RevokedControlAction",
        &["ControlActionRef"],
    ),
    record::<ca::DriverMessage>(
        "ControlActionDelivery",
        &[Container {
            name: "driverMessages",
            repeated: false,
        }],
        "DriverMessage",
        &["ItemIdentifier", "VehicleRef", "DriverRef"],
    ),
    record::<ca::VehicleDetecting>(
        "ControlActionDelivery",
        &[Container {
            name: "vehicleDetectings",
            repeated: false,
        }],
        "VehicleDetecting",
        &["ItemIdentifier", "VehicleRef"],
    ),
];

/// The elements of a `ServiceDelivery` that are its own. Every other child is a
/// functional service delivery, whose position among those is what a path names.
pub(super) const SERVICE_DELIVERY_FIELDS: &[&str] = &[
    "ResponseTimestamp",
    "ProducerRef",
    "Address",
    "ResponseMessageIdentifier",
    "RequestMessageRef",
    "DelegatorAddress",
    "DelegatorRef",
    "Status",
    "ErrorCondition",
    "MoreData",
];

/// The elements that tell a reader which delivery was left out.
pub(super) const DELIVERY_IDENTIFIERS: &[&str] = &["SubscriptionRef", "RequestMessageRef"];

#[cfg(test)]
mod tests {
    use super::*;

    /// The table names each record once.
    #[test]
    fn every_record_is_listed_once() {
        for (i, record) in RECORDS.iter().enumerate() {
            let again = RECORDS[i + 1..]
                .iter()
                .find(|other| other.delivery == record.delivery && other.name == record.name);
            assert!(again.is_none(), "{} is listed twice", record.name);
        }
    }

    /// The service delivery's own elements are the fields the type writes before
    /// its deliveries, so a functional delivery's position is counted right.
    #[test]
    fn the_service_delivery_fields_are_the_ones_the_type_writes() {
        use chrono::{DateTime, FixedOffset, TimeZone};

        use crate::framework::{DeliveryError, ErrorCodeDetail, ErrorCondition, ServiceDelivery};
        use crate::sm::StopMonitoringDelivery;
        use crate::types::{EndpointAddress, MessageQualifier, MessageRef, ParticipantRef};

        let when: DateTime<FixedOffset> =
            FixedOffset::east_opt(3600).expect("an offset").with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let delivery = ServiceDelivery {
            srs_name: None,
            response_timestamp: when,
            producer_ref: Some(ParticipantRef::new("P")),
            address: Some(EndpointAddress::new("http://example.org/")),
            response_message_identifier: Some(MessageQualifier::new("m")),
            request_message_ref: Some(MessageRef::new("r")),
            delegator_address: Some(EndpointAddress::new("http://example.org/d")),
            delegator_ref: Some(ParticipantRef::new("D")),
            status: Some(true),
            error_condition: Some(ErrorCondition::new(DeliveryError::OtherError(ErrorCodeDetail::default()))),
            more_data: Some(false),
            deliveries: vec![StopMonitoringDelivery::new(when, Vec::new()).into()],
        };
        let written = crate::to_string(&crate::Siri::new("2.0", delivery)).expect("writes");
        let service_delivery = super::super::document::locate(
            &written,
            &[super::super::document::Step {
                name: "ServiceDelivery".into(),
                index: None,
            }],
        )
        .expect("the service delivery is written");
        let names: Vec<String> = super::super::document::children(&written, service_delivery.range)
            .into_iter()
            .map(|child| child.name)
            .collect();
        assert_eq!(names.len(), SERVICE_DELIVERY_FIELDS.len() + 1);
        assert_eq!(&names[..SERVICE_DELIVERY_FIELDS.len()], SERVICE_DELIVERY_FIELDS);
        assert_eq!(names.last().map(String::as_str), Some("StopMonitoringDelivery"));
    }
}
