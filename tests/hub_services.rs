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
use siri_rs::enumerations::{DeliveryMethod, FacilityStatus as Availability};
use siri_rs::et::EstimatedTimetableRequest;
use siri_rs::fm::FacilityMonitoringRequest;
use siri_rs::framework::ServiceDelivery;
use siri_rs::gm::{GeneralMessageRequest, InfoMessage};
use siri_rs::model::{
    EstimatedVehicleJourney, FacilityCondition, FacilityStatus, InterchangeJourney, MonitoredVehicleJourney,
    TargetedVehicleJourney,
};
use siri_rs::pubsub::{
    ConnectionMonitoringFeeder, ConnectionMonitoringFeederSource, ConnectionTimetable, ConnectionTimetableSource,
    Consumer, ConsumerEvent, EstimatedTimetable, EstimatedTimetableSource, FacilityMonitoring,
    FacilityMonitoringSource, GeneralMessage, GeneralMessageSource, Producer, ProducerConfig, ProductionTimetable,
    Service, SituationExchange, Source, StopMonitoring, StopMonitoringSource, StopTimetable, StopTimetableSource,
    SubscriptionState, VehicleMonitoring, PROTOCOL_VERSION,
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
        ConnectionMonitoringRequest::for_line(now(), "CL-Kroepcke", ConnectingTimeFilter::new("10", "OUT")),
    );

    assert_eq!(arrivals.len(), 1);
    assert_eq!(arrivals[0].expected_arrival_time, Some(now() + Duration::minutes(14)));
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
        (
            "ProductionTimetable",
            ProductionTimetable::service_delivery(ProductionTimetable::delivery(now(), Vec::new())),
        ),
        (
            "StopTimetable",
            StopTimetable::service_delivery(StopTimetable::delivery(now(), Vec::new())),
        ),
        (
            "StopMonitoring",
            StopMonitoring::service_delivery(StopMonitoring::delivery(now(), Vec::new())),
        ),
        (
            "VehicleMonitoring",
            VehicleMonitoring::service_delivery(VehicleMonitoring::delivery(now(), Vec::new())),
        ),
        (
            "ConnectionTimetable",
            ConnectionTimetable::service_delivery(ConnectionTimetable::delivery(now(), Vec::new())),
        ),
        (
            "ConnectionMonitoringFeeder",
            ConnectionMonitoringFeeder::service_delivery(ConnectionMonitoringFeeder::delivery(now(), Vec::new())),
        ),
        (
            "GeneralMessage",
            GeneralMessage::service_delivery(GeneralMessage::delivery(now(), Vec::new())),
        ),
        (
            "FacilityMonitoring",
            FacilityMonitoring::service_delivery(FacilityMonitoring::delivery(now(), Vec::new())),
        ),
        (
            "SituationExchange",
            SituationExchange::service_delivery(SituationExchange::delivery(now(), Vec::new())),
        ),
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

/// What a producer whose source holds no journey answers a request with is the
/// frame the schema rejects, and the strict reader rejects it too: the error
/// names the frame and what it lacks. Reading it anyway is
/// the lenient reader's business (`tests/lenient_findings.rs`); the strict one
/// holds to the schema.
#[test]
fn the_strict_reader_refuses_a_frame_without_a_journey() {
    let mut producer = Producer::new(ProducerConfig::new("MY-AGENCY"), Journeys(Vec::new()));
    let mut consumer = Consumer::<EstimatedTimetable>::new("PASSENGER-APP");
    let request = consumer.request(EstimatedTimetableRequest::new(now()), now());
    let answer = producer
        .handle(&request, now())
        .expect("the request is understood")
        .expect("a request is answered");
    let xml = siri_rs::to_string(&answer).expect("a message is writable");

    let Err(siri_rs::Error::Deserialize { path, offset, source }) = siri_rs::from_str::<Siri>(&xml) else {
        panic!("the strict reader refuses a frame without a journey:\n{xml}");
    };
    assert_eq!(
        path,
        "ServiceDelivery.EstimatedTimetableDelivery[0].EstimatedJourneyVersionFrame[0]"
    );
    assert_eq!(source.to_string(), "missing field `EstimatedVehicleJourney`");
    // The reader notices the missing journey at the frame's end tag, having
    // already looked one event further: past the delivery's end tag.
    let ends = "</EstimatedJourneyVersionFrame></EstimatedTimetableDelivery>";
    let looked_to = xml.find(ends).expect("the frame ends the delivery") + ends.len();
    assert_eq!(offset, Some(u64::try_from(looked_to).expect("fits")), "{xml}");
}

/// Since the schema has no form for an estimated timetable of no journeys, a
/// producer whose source holds none for a subscription sends it nothing — in
/// either delivery method — rather than a document no consumer can read, and
/// delivers as soon as there is a journey to deliver.
#[test]
fn an_estimated_timetable_subscription_is_sent_nothing_while_there_is_no_journey() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    for config in [
        ProducerConfig::new("MY-AGENCY"),
        ProducerConfig::new("MY-AGENCY").with_fetched_delivery(),
    ] {
        let fetched = matches!(config.delivery_method(), DeliveryMethod::Fetched);
        let mut producer = Producer::new(config, Journeys(Vec::new()));
        let mut consumer = Consumer::<EstimatedTimetable>::new("PASSENGER-APP");
        let subscribe = consumer.subscribe(
            "running",
            now() + Duration::hours(1),
            EstimatedTimetableRequest::new(now()),
            now(),
        );
        producer
            .handle(&valid(&subscribe), now())
            .expect("the subscription is understood")
            .expect("a subscription request is answered");

        assert!(
            producer.poll(now()).is_empty(),
            "fetched {fetched}: nothing is sent while the source holds no journey"
        );
        assert_eq!(
            producer.subscriptions()[0].state(),
            &SubscriptionState::Idle,
            "nothing is owed"
        );

        producer
            .source_mut()
            .0
            .push(EstimatedVehicleJourney::dated("10", "OUT", "10-0815"));
        producer.data_changed();
        let outbound = producer.poll(now());
        assert_eq!(
            outbound.len(),
            1,
            "fetched {fetched}: the journey is announced or delivered"
        );
        let delivered = if fetched {
            let ConsumerEvent::DataReady { fetch, .. } = consumer
                .handle(&valid(&outbound[0].message), now())
                .expect("the notification is understood")
            else {
                panic!("a fetched subscription announces its data");
            };
            producer
                .handle(&valid(&fetch), now())
                .expect("the fetch is understood")
                .expect("a fetch is answered")
        } else {
            outbound[0].message.clone()
        };
        let ConsumerEvent::Delivered { items, .. } = consumer
            .handle(&valid(&delivered), now())
            .expect("the delivery is understood")
        else {
            panic!("fetched {fetched}: the journey is delivered");
        };
        assert_eq!(items.len(), 1);
    }
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

/// Whatever a real-time system already keeps its running journeys in.
struct Journeys(Vec<EstimatedVehicleJourney>);

impl EstimatedTimetableSource for Journeys {
    fn journeys(&self, _request: &EstimatedTimetableRequest) -> Vec<EstimatedVehicleJourney> {
        self.0.clone()
    }
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
