//! Every functional service on the same publish/subscribe hub.
//!
//! The hub is service-independent: what changes from one service to the next is the
//! request, the delivery and the records inside it, which is exactly what
//! [`Service`] states. This drives one full subscription cycle per service the crate
//! models, in process, and validates every message it produces against the official
//! schemas — so a service whose hub binding names the wrong element, or builds a
//! delivery the schemas reject, fails here rather than in someone's production feed.
//!
//! `tests/http_stop_monitoring.rs` and `tests/http_journey_services.rs` run the same
//! cycles over a real socket for one service each.

mod support;

use chrono::{DateTime, Duration, FixedOffset};

use siri_rs::cm::{ConnectingTimeFilter, ConnectionMonitoringRequest, MonitoredFeederArrival};
use siri_rs::ct::{ConnectionTimetableRequest, TimetabledFeederArrival};
use siri_rs::enumerations::FacilityStatus as Availability;
use siri_rs::fm::FacilityMonitoringRequest;
use siri_rs::gm::{GeneralMessageRequest, InfoMessage};
use siri_rs::model::{
    FacilityCondition, FacilityStatus, InterchangeJourney, MonitoredVehicleJourney,
    TargetedVehicleJourney,
};
use siri_rs::framework::ServiceDelivery;
use siri_rs::pubsub::{
    ConnectionMonitoringFeeder, ConnectionMonitoringFeederSource, ConnectionTimetable,
    ConnectionTimetableSource, Consumer, ConsumerEvent, EstimatedTimetable, FacilityMonitoring,
    FacilityMonitoringSource, GeneralMessage, GeneralMessageSource, Producer, ProducerConfig,
    ProductionTimetable, Service, SituationExchange, Source, StopMonitoring,
    StopMonitoringSource, StopTimetable, StopTimetableSource, VehicleMonitoring,
    PROTOCOL_VERSION,
};
use siri_rs::sm::{MonitoredStopVisit, StopMonitoringRequest};
use siri_rs::st::{StopTimetableRequest, TimetabledStopVisit};
use siri_rs::types::AnyContent;
use siri_rs::Siri;
use support::{validate, validator_available, VALIDATOR_MISSING};

#[test]
fn the_hub_carries_a_stop_monitoring_subscription() {
    let visits = subscribe_and_collect::<_, StopMonitoring>(
        DepartureBoard(vec![MonitoredStopVisit::new(
            now(),
            MonitoredVehicleJourney::on_line("10"),
        )]),
        StopMonitoringRequest::at_stop(now(), "de:03241:101"),
    );

    assert_eq!(visits.len(), 1);
    assert_eq!(
        visits[0].monitored_vehicle_journey.line_ref.as_ref(),
        Some(&"10".into())
    );
}

#[test]
fn the_hub_carries_a_stop_timetable_subscription() {
    let visits = subscribe_and_collect::<_, StopTimetable>(
        PlannedCalls(vec![TimetabledStopVisit::new(
            now(),
            "de:03241:101",
            TargetedVehicleJourney::new("10", "OUT"),
        )]),
        StopTimetableRequest::at_stop(now(), "de:03241:101"),
    );

    assert_eq!(visits.len(), 1);
    assert_eq!(visits[0].monitoring_ref.as_str(), "de:03241:101");
}

#[test]
fn the_hub_carries_a_connection_timetable_subscription() {
    let arrivals = subscribe_and_collect::<_, ConnectionTimetable>(
        PlannedConnections(vec![TimetabledFeederArrival::new(
            now(),
            "CL-Kroepcke",
            InterchangeJourney::new("10", "OUT"),
            now() + Duration::minutes(12),
        )]),
        ConnectionTimetableRequest::over_link(now(), "CL-Kroepcke"),
    );

    assert_eq!(arrivals.len(), 1);
    assert_eq!(arrivals[0].connection_link_ref.as_str(), "CL-Kroepcke");
}

#[test]
fn the_hub_carries_a_connection_monitoring_subscription() {
    let arrivals = subscribe_and_collect::<_, ConnectionMonitoringFeeder>(
        FeederArrivals(vec![MonitoredFeederArrival {
            expected_arrival_time: Some(now() + Duration::minutes(14)),
            ..MonitoredFeederArrival::new(now(), "CL-Kroepcke", InterchangeJourney::new("10", "OUT"))
        }]),
        ConnectionMonitoringRequest::for_line(
            now(),
            "CL-Kroepcke",
            ConnectingTimeFilter::new("10", "OUT"),
        ),
    );

    assert_eq!(arrivals.len(), 1);
    assert_eq!(
        arrivals[0].expected_arrival_time,
        Some(now() + Duration::minutes(14))
    );
}

#[test]
fn the_hub_carries_a_general_message_subscription() {
    let messages = subscribe_and_collect::<_, GeneralMessage>(
        Announcements(vec![InfoMessage::new(
            now(),
            "2026-0041",
            AnyContent::text("The kiosk on platform 3 is closed today"),
        )]),
        GeneralMessageRequest::new(now()),
    );

    assert_eq!(messages.len(), 1);
    assert_eq!(
        messages[0].info_message_identifier.as_str(),
        "2026-0041",
        "the message the producer holds is the one delivered"
    );
}

#[test]
fn the_hub_carries_a_facility_monitoring_subscription() {
    let conditions = subscribe_and_collect::<_, FacilityMonitoring>(
        Lifts(vec![FacilityCondition::for_reference(
            "lift-platform-3",
            FacilityStatus::new(Availability::NotAvailable),
        )]),
        FacilityMonitoringRequest::new(now()),
    );

    assert_eq!(conditions.len(), 1);
    assert_eq!(conditions[0].facility_status.status, Availability::NotAvailable);
}

/// A fetch that finds nothing waiting is answered with a delivery of the service
/// carrying no records, because that is the form the schema provides for "nothing":
/// a `ServiceDelivery` must carry at least one functional-service delivery. So that
/// form has to be a valid document for every service the hub speaks — every one
/// but Estimated Timetable, which the test after this one is about.
#[test]
fn a_delivery_carrying_no_records_is_valid_for_every_service_that_can_say_so() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let empty = [
        ("ProductionTimetable", ProductionTimetable::service_delivery(ProductionTimetable::delivery(now(), Vec::new()))),
        ("StopTimetable", StopTimetable::service_delivery(StopTimetable::delivery(now(), Vec::new()))),
        ("StopMonitoring", StopMonitoring::service_delivery(StopMonitoring::delivery(now(), Vec::new()))),
        ("VehicleMonitoring", VehicleMonitoring::service_delivery(VehicleMonitoring::delivery(now(), Vec::new()))),
        ("ConnectionTimetable", ConnectionTimetable::service_delivery(ConnectionTimetable::delivery(now(), Vec::new()))),
        ("ConnectionMonitoringFeeder", ConnectionMonitoringFeeder::service_delivery(ConnectionMonitoringFeeder::delivery(now(), Vec::new()))),
        ("GeneralMessage", GeneralMessage::service_delivery(GeneralMessage::delivery(now(), Vec::new()))),
        ("FacilityMonitoring", FacilityMonitoring::service_delivery(FacilityMonitoring::delivery(now(), Vec::new()))),
        ("SituationExchange", SituationExchange::service_delivery(SituationExchange::delivery(now(), Vec::new()))),
    ];
    for (service, payload) in empty {
        let message = Siri::new(
            PROTOCOL_VERSION,
            ServiceDelivery::new(now(), "MY-AGENCY", vec![payload]),
        );
        let xml = siri_rs::to_string(&message).expect("a message is writable");
        if let Err(complaint) = validate(&xml) {
            panic!("an empty {service} delivery is not valid SIRI:\n{xml}\n{complaint}");
        }
    }
}

/// Estimated Timetable has no valid way of delivering nothing: the schema requires
/// at least one `EstimatedJourneyVersionFrame` in the delivery and at least one
/// `EstimatedVehicleJourney` in the frame. A producer of that service whose source
/// matches no journey therefore builds a document the schema rejects, whatever the
/// crate does about it; the front page says so. This pins the limitation to the
/// schema release the fixtures carry, so that a release which lifts it is noticed.
#[test]
fn an_estimated_timetable_delivery_cannot_carry_no_journeys() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let message = Siri::new(
        PROTOCOL_VERSION,
        ServiceDelivery::new(
            now(),
            "MY-AGENCY",
            vec![EstimatedTimetable::service_delivery(EstimatedTimetable::delivery(
                now(),
                Vec::new(),
            ))],
        ),
    );
    let xml = siri_rs::to_string(&message).expect("a message is writable");
    let complaint = validate(&xml).expect_err("the schema has no form for an empty frame");
    assert!(
        complaint.contains("EstimatedJourneyVersionFrame") && complaint.contains("Missing child"),
        "{complaint}"
    );
}

/// Opens a subscription, lets the producer answer it, and returns what was delivered.
///
/// Every message the two sides put out is validated against the official schemas
/// before the other side reads it.
fn subscribe_and_collect<Src, Svc>(source: Src, request: Svc::Request) -> Vec<Svc::Item>
where
    Src: Source<Svc>,
    Svc: Service,
{
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let mut producer = Producer::new(ProducerConfig::new("MY-AGENCY"), source);
    let mut consumer = Consumer::<Svc>::new("PASSENGER-APP");

    let subscribe = consumer.subscribe("one", now() + Duration::hours(1), request, now());
    let response = producer
        .handle(&valid(&subscribe), now())
        .expect("the subscription is understood")
        .expect("a subscription request is answered");
    let ConsumerEvent::Subscribed { outcomes } = consumer
        .handle(&valid(&response), now())
        .expect("the answer is understood")
    else {
        panic!("a subscription request is answered with a subscription outcome");
    };
    assert_eq!(outcomes.len(), 1);
    assert!(outcomes[0].accepted, "the producer serves this service");

    let mut delivered = Vec::new();
    for outbound in producer.poll(now()) {
        if let ConsumerEvent::Delivered { items, .. } = consumer
            .handle(&valid(&outbound.message), now())
            .expect("the delivery is understood")
        {
            delivered.extend(items);
        }
    }
    delivered
}

/// The message, once the schemas have accepted it.
fn valid(message: &Siri) -> Siri {
    let xml = siri_rs::to_string(message).expect("a message is writable");
    if let Err(complaint) = validate(&xml) {
        panic!("the hub built a document the schemas reject:\n{xml}\n{complaint}");
    }
    message.clone()
}

fn now() -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339("2026-03-04T08:15:00+01:00").expect("valid instant")
}

/// Whatever a departure board already keeps its due services in.
struct DepartureBoard(Vec<MonitoredStopVisit>);

impl StopMonitoringSource for DepartureBoard {
    fn visits(&self, _request: &StopMonitoringRequest) -> Vec<MonitoredStopVisit> {
        self.0.clone()
    }
}

/// Whatever a scheduling system already keeps the calls at a stop in.
struct PlannedCalls(Vec<TimetabledStopVisit>);

impl StopTimetableSource for PlannedCalls {
    fn visits(&self, _request: &StopTimetableRequest) -> Vec<TimetabledStopVisit> {
        self.0.clone()
    }
}

/// Whatever a scheduling system already keeps its planned interchanges in.
struct PlannedConnections(Vec<TimetabledFeederArrival>);

impl ConnectionTimetableSource for PlannedConnections {
    fn arrivals(&self, _request: &ConnectionTimetableRequest) -> Vec<TimetabledFeederArrival> {
        self.0.clone()
    }
}

/// Whatever the feeder side of an interchange already tracks its arrivals in.
struct FeederArrivals(Vec<MonitoredFeederArrival>);

impl ConnectionMonitoringFeederSource for FeederArrivals {
    fn arrivals(&self, _request: &ConnectionMonitoringRequest) -> Vec<MonitoredFeederArrival> {
        self.0.clone()
    }
}

/// Whatever an information desk already keeps its announcements in.
struct Announcements(Vec<InfoMessage>);

impl GeneralMessageSource for Announcements {
    fn messages(&self, _request: &GeneralMessageRequest) -> Vec<InfoMessage> {
        self.0.clone()
    }
}

/// Whatever a maintenance system already keeps the state of its lifts in.
struct Lifts(Vec<FacilityCondition>);

impl FacilityMonitoringSource for Lifts {
    fn conditions(&self, _request: &FacilityMonitoringRequest) -> Vec<FacilityCondition> {
        self.0.clone()
    }
}
