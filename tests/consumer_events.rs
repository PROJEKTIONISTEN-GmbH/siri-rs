//! What a consumer makes of the messages that reach it, beyond the records it asked
//! for.
//!
//! A delivery carries more than the records the hub publishes: the withdrawals of
//! records delivered earlier, the notices of a departure board, the road situations
//! next to the public-transport ones, the interchanges of an estimated timetable, and
//! the situations a delivery of another service is accompanied by. With incremental
//! updates a withdrawal is the only sign that a departure is gone, so a consumer that
//! passed on the records alone would keep showing it. And a producer that reports
//! itself unable to serve is not alive in any sense a monitor cares about.

mod support;

use chrono::{DateTime, FixedOffset};

use siri_rs::framework::{
    CheckStatusResponse, ErrorCondition, HeartbeatNotification, ServiceDelivery,
    ServiceDeliveryPayload, ServiceNotAvailableError, StatusError,
};
use siri_rs::pubsub::{
    ConnectionMonitoringFeeder, ConnectionTimetable, Consumer, ConsumerEvent, EstimatedTimetable,
    FacilityMonitoring, GeneralMessage, ProductionTimetable, Service, SituationExchange,
    StopMonitoring, StopTimetable, VehicleMonitoring, PROTOCOL_VERSION,
};
use siri_rs::Siri;
use support::{validate, validator_available, VALIDATOR_MISSING};

/// The official example at `name` under `tests/fixtures`, read.
fn fixture(name: &str) -> Siri {
    let path = support::fixtures_dir().join(name);
    let xml = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    siri_rs::from_str(&xml).unwrap_or_else(|e| panic!("{name} reads: {e}"))
}

/// A `ServiceDelivery` carrying `deliveries`, written out as a document the schemas
/// accept, and read back.
fn delivered(deliveries: &str) -> Siri {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let xml = format!(
        concat!(
            r#"<Siri xmlns="http://www.siri.org.uk/siri" version="2.0"><ServiceDelivery>"#,
            "<ResponseTimestamp>2004-12-17T09:30:46-05:00</ResponseTimestamp>",
            "<ProducerRef>KUBRICK</ProducerRef>{}</ServiceDelivery></Siri>"
        ),
        deliveries
    );
    if let Err(complaint) = validate(&xml) {
        panic!("the test builds a document the schemas reject:\n{xml}\n{complaint}");
    }
    siri_rs::from_str(&xml).unwrap_or_else(|e| panic!("{e}: {xml}"))
}

/// What a fresh consumer of service `S` makes of `message`.
fn event<S: Service>(message: &Siri) -> ConsumerEvent<S> {
    Consumer::<S>::new("PASSENGER-APP")
        .handle(message, now())
        .expect("the message is understood")
}

fn service_delivery(message: &Siri) -> &ServiceDelivery {
    message.payload.as_service_delivery().expect("the message is a delivery")
}

fn now() -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339("2004-12-17T09:30:47-05:00").expect("valid instant")
}

const RECORDED: &str = "<RecordedAtTime>2004-12-17T09:30:47-05:00</RecordedAtTime>";

/// The official departure board carries two visits and withdraws two others; the
/// consumer has to hand on all four facts, not the first two alone.
#[test]
fn a_stop_monitoring_delivery_hands_on_its_cancelled_visits() {
    let message = fixture("xml/sm/exs_stopMonitoring_response.xml");

    let ConsumerEvent::DeliveredWhole { items, delivery, outcome, reply } = event::<StopMonitoring>(&message)
    else {
        panic!("a delivery that withdraws visits is handed on whole");
    };

    assert_eq!(items.len(), 2, "the visits are the records, as they were");
    let cancelled: Vec<&str> = delivery
        .deliveries
        .iter()
        .filter_map(StopMonitoring::delivery_of)
        .flat_map(|board| &board.monitored_stop_visit_cancellation)
        .map(|cancellation| cancellation.item_ref.as_str())
        .collect();
    assert_eq!(cancelled.len(), 2, "both withdrawals come along: {cancelled:?}");
    assert_eq!(*delivery, *service_delivery(&message), "nothing of the message is lost");
    assert!(outcome.succeeded());
    assert!(reply.is_none(), "the consumer was not asked to confirm deliveries");
}

/// Every record a delivery may carry beside the ones the hub publishes, for every
/// service whose delivery has one: each is a reason to hand the delivery on whole.
#[test]
fn every_record_beside_the_published_ones_hands_the_delivery_on_whole() {
    let stop_monitoring = |records: &str| {
        delivered(&format!(
            "<StopMonitoringDelivery version=\"2.0\">\
             <ResponseTimestamp>2004-12-17T09:30:47-05:00</ResponseTimestamp>{records}</StopMonitoringDelivery>"
        ))
    };

    for (record, message) in [
        (
            "StopNotice",
            stop_monitoring(&format!(
                "<StopNotice>{RECORDED}<ItemIdentifier>notice-1</ItemIdentifier>\
                 <MonitoringRef>HLTST011</MonitoringRef><StopNote>Lift out of service</StopNote></StopNotice>"
            )),
        ),
        (
            "StopNoticeCancellation",
            stop_monitoring(&format!(
                "<StopNoticeCancellation>{RECORDED}<ItemRef>notice-1</ItemRef>\
                 <MonitoringRef>HLTST011</MonitoringRef></StopNoticeCancellation>"
            )),
        ),
        (
            "ServiceException",
            stop_monitoring(&format!(
                "<ServiceException>{RECORDED}<ServiceStatus>serviceDisrupted</ServiceStatus></ServiceException>"
            )),
        ),
    ] {
        assert!(
            matches!(event::<StopMonitoring>(&message), ConsumerEvent::DeliveredWhole { .. }),
            "a {record} is handed on"
        );
    }

    let withdrawn_visit = delivered(&format!(
        "<StopTimetableDelivery version=\"2.0\">\
         <ResponseTimestamp>2004-12-17T09:30:47-05:00</ResponseTimestamp>\
         <TimetabledStopVisitCancellation>{RECORDED}<MonitoringRef>HLTST011</MonitoringRef>\
         <LineRef>123</LineRef><DirectionRef>Out</DirectionRef></TimetabledStopVisitCancellation>\
         </StopTimetableDelivery>"
    ));
    assert!(matches!(event::<StopTimetable>(&withdrawn_visit), ConsumerEvent::DeliveredWhole { .. }));

    let interchange = delivered(&format!(
        "<EstimatedTimetableDelivery version=\"2.0\">\
         <ResponseTimestamp>2004-12-17T09:30:47-05:00</ResponseTimestamp>\
         <EstimatedJourneyVersionFrame>{RECORDED}\
         <EstimatedVehicleJourney><LineRef>123</LineRef><DirectionRef>Out</DirectionRef>\
         <DatedVehicleJourneyRef>00008</DatedVehicleJourneyRef></EstimatedVehicleJourney>\
         <EstimatedServiceJourneyInterchange><InterchangeCode>IC-1</InterchangeCode>\
         </EstimatedServiceJourneyInterchange>\
         </EstimatedJourneyVersionFrame></EstimatedTimetableDelivery>"
    ));
    assert!(matches!(event::<EstimatedTimetable>(&interchange), ConsumerEvent::DeliveredWhole { .. }));

    let official = [
        ("xml/vm/exv_vehicleMonitoring_response.xml", whole::<VehicleMonitoring> as fn(&Siri) -> bool),
        ("xml/ct/exc_connectionTimetable_response.xml", whole::<ConnectionTimetable>),
        ("xml/cm/exc_connectionMonitoringFeeder_response.xml", whole::<ConnectionMonitoringFeeder>),
        ("xml/gm/exm_generalMessage_response.xml", whole::<GeneralMessage>),
        ("xml/sx/exx_situationExchange_response.xml", whole::<SituationExchange>),
    ];
    for (name, handed_on_whole) in official {
        assert!(handed_on_whole(&fixture(name)), "{name} carries a withdrawal or a road situation");
    }
}

fn whole<S: Service>(message: &Siri) -> bool {
    matches!(event::<S>(message), ConsumerEvent::DeliveredWhole { .. })
}

/// A delivery whose every record is one the hub publishes is delivered as it always
/// was: the event a consumer already handles.
#[test]
fn a_delivery_of_published_records_alone_is_delivered_as_before() {
    let board = fixture("xml/sm/exs_stopMonitoring_response_simple.xml");
    let without_notice = {
        let mut message = board.clone();
        let siri_rs::SiriPayload::ServiceDelivery(delivery) = &mut message.payload else {
            unreachable!("the fixture is a delivery")
        };
        for payload in &mut delivery.deliveries {
            if let ServiceDeliveryPayload::StopMonitoringDelivery(board) = payload {
                board.stop_line_notice.clear();
            }
        }
        message
    };

    let ConsumerEvent::Delivered { items, .. } = event::<StopMonitoring>(&without_notice) else {
        panic!("visits alone are delivered as before");
    };
    assert_eq!(items.len(), 2);
    assert!(
        matches!(event::<StopMonitoring>(&board), ConsumerEvent::DeliveredWhole { .. }),
        "the notice the official board carries is a record of its own"
    );

    for (name, delivered_as_before) in [
        ("xml/pt/ext_productionTimetable_response.xml", delivered_as_before::<ProductionTimetable> as fn(&Siri) -> bool),
        ("xml/fm/exf_facilityMonitoring_response.xml", delivered_as_before::<FacilityMonitoring>),
    ] {
        assert!(delivered_as_before(&fixture(name)), "{name}: its records are the published ones");
    }
}

fn delivered_as_before<S: Service>(message: &Siri) -> bool {
    matches!(event::<S>(message), ConsumerEvent::Delivered { .. })
}

/// The situations a departure board is delivered with are part of what arrived.
#[test]
fn the_situations_a_delivery_comes_with_are_handed_on() {
    let message = fixture("derived/sm/exs_stopMonitoring_response_included_situation.xml");

    let ConsumerEvent::DeliveredWhole { items, delivery, .. } = event::<StopMonitoring>(&message) else {
        panic!("a delivery accompanied by situations is handed on whole");
    };

    assert_eq!(items.len(), 2);
    let situations: Vec<&str> = delivery
        .deliveries
        .iter()
        .filter_map(|payload| match payload {
            ServiceDeliveryPayload::IncludedSituationExchangeDelivery(included) => Some(included),
            _ => None,
        })
        .flat_map(|included| included.pt_situations())
        .map(|situation| situation.situation_number.as_str())
        .collect();
    assert_eq!(situations, ["HLTST011-LIFT"]);
}

/// A producer that reports itself unable to serve is unavailable, not alive, and
/// says why.
#[test]
fn a_heartbeat_reporting_an_outage_is_not_alive() {
    let started = now() - chrono::Duration::hours(2);
    let outage = ErrorCondition::new(StatusError::ServiceNotAvailableError(ServiceNotAvailableError {
        error_text: Some("timetable server restarting".into()),
        expected_restart_time: Some(now() + chrono::Duration::minutes(5)),
        ..ServiceNotAvailableError::default()
    }));
    let heartbeat = Siri::new(
        PROTOCOL_VERSION,
        HeartbeatNotification {
            status: Some(false),
            error_condition: Some(outage.clone()),
            ..HeartbeatNotification::healthy(now(), "KUBRICK", started)
        },
    );
    let status = Siri::new(
        PROTOCOL_VERSION,
        CheckStatusResponse {
            status: Some(false),
            error_condition: Some(outage.clone()),
            ..CheckStatusResponse::healthy(now(), "KUBRICK", started)
        },
    );

    for message in [heartbeat, status] {
        let xml = siri_rs::to_string(&message).expect("the message writes");
        assert!(validator_available(), "{VALIDATOR_MISSING}");
        validate(&xml).unwrap_or_else(|complaint| panic!("{xml}\n{complaint}"));
        assert_eq!(
            event::<SituationExchange>(&message),
            ConsumerEvent::Unavailable {
                service_started_time: Some(started),
                error_condition: Some(outage.clone()),
            },
            "{xml}"
        );
    }
}

/// `Status` is optional and true by default: a heartbeat that leaves it out, or says
/// true, is the producer reporting itself alive.
#[test]
fn a_heartbeat_reporting_no_outage_is_alive() {
    let started = now() - chrono::Duration::hours(2);
    let healthy = HeartbeatNotification::healthy(now(), "KUBRICK", started);
    for status in [None, Some(true)] {
        let message = Siri::new(PROTOCOL_VERSION, HeartbeatNotification { status, ..healthy.clone() });
        assert_eq!(
            event::<SituationExchange>(&message),
            ConsumerEvent::Alive {
                service_started_time: Some(started),
            },
            "Status {status:?}"
        );
    }
}
