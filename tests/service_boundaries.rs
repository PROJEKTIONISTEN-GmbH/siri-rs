//! What happens when a message reaches a hub that serves a different service.
//!
//! One `ServiceRequest` addresses one functional service, and a producer serves one.
//! Now that [`Producer`] and [`Consumer`] are generic over the service, the case
//! where the two disagree is worth pinning down: a producer must refuse rather than
//! answer with something it was not asked for, and a consumer must not read another
//! service's records as its own.

use chrono::{DateTime, Duration, FixedOffset};

use siri_rs::et::EstimatedTimetableRequest;
use siri_rs::framework::{ServiceDelivery, ServiceDeliveryPayload};
use siri_rs::model::EstimatedVehicleJourney;
use siri_rs::pubsub::{
    Consumer, ConsumerEvent, EstimatedTimetable, EstimatedTimetableSource, Producer, ProducerConfig,
    SituationExchange, VehicleMonitoring,
};
use siri_rs::sx::SituationExchangeRequest;
use siri_rs::vm::{VehicleMonitoringDelivery, VehicleMonitoringRequest};
use siri_rs::{Error, Siri};

struct NoJourneys;

impl EstimatedTimetableSource for NoJourneys {
    fn journeys(&self, _request: &EstimatedTimetableRequest) -> Vec<EstimatedVehicleJourney> {
        Vec::new()
    }
}

fn now() -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339("2026-03-14T08:00:00+01:00").expect("valid instant")
}

fn estimated_timetable_producer() -> Producer<NoJourneys, EstimatedTimetable> {
    Producer::new(ProducerConfig::new("MY-AGENCY"), NoJourneys)
}

#[test]
fn a_producer_refuses_a_subscription_meant_for_another_service() {
    let mut producer = estimated_timetable_producer();
    let mut elsewhere = Consumer::<SituationExchange>::new("PASSENGER-APP");
    let subscribe = elsewhere.subscribe(
        "disruptions",
        now() + Duration::hours(1),
        SituationExchangeRequest::new(now()),
        now(),
    );

    // The request is well-formed SIRI and is answered as such: with a status per
    // subscription, saying no.
    let response = producer
        .handle(&subscribe, now())
        .expect("a subscription request is always answerable")
        .expect("a subscription request is answered");
    let statuses = &response
        .payload
        .as_subscription_response()
        .expect("the answer is a subscription response")
        .response_status;
    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses[0].subscription_ref.as_str(), "disruptions");
    assert!(!statuses[0].is_accepted(), "a producer that does not serve the service says so");
    let reason = statuses[0]
        .error_condition
        .as_ref()
        .and_then(|condition| condition.description.as_deref())
        .expect("the refusal says why");
    assert!(reason.contains("SituationExchangeSubscriptionRequest"), "{reason}");
    assert!(
        producer.subscriptions().is_empty(),
        "a refused subscription is not held"
    );

    let ConsumerEvent::Subscribed { outcomes } = elsewhere
        .handle(&response, now())
        .expect("the consumer reads the answer")
    else {
        panic!("a subscription response is a subscription outcome");
    };
    assert!(!outcomes[0].accepted);
    assert!(elsewhere.subscriptions().is_empty());
}

#[test]
fn a_producer_refuses_a_request_meant_for_another_service() {
    let mut producer = estimated_timetable_producer();
    let mut elsewhere = Consumer::<VehicleMonitoring>::new("MAP");
    let request = elsewhere.request(VehicleMonitoringRequest::new(now()), now());

    // A delivery is typed by its service, so there is no document in which an
    // estimated-timetable producer could answer a vehicle-monitoring request; the
    // request is refused as a message, not as a document.
    let refusal = producer
        .handle(&request, now())
        .expect_err("a producer that does not serve the service says so");
    assert!(
        matches!(&refusal, Error::UnexpectedMessage { expected, found }
            if expected.contains("service this producer serves")
                && *found == "VehicleMonitoringRequest"),
        "{refusal}"
    );
}

#[test]
fn a_producer_answers_a_request_for_the_service_it_does_serve() {
    let mut producer = estimated_timetable_producer();
    let mut consumer = Consumer::<EstimatedTimetable>::new("PASSENGER-APP");
    let request = consumer.request(EstimatedTimetableRequest::new(now()), now());

    let delivery = producer
        .handle(&request, now())
        .expect("the request is understood")
        .expect("a service request is answered");
    let ConsumerEvent::Delivered { items, .. } = consumer
        .handle(&delivery, now())
        .expect("the delivery is understood")
    else {
        panic!("a service request is answered with a delivery");
    };
    assert!(items.is_empty(), "this source holds nothing");
}

#[test]
fn a_consumer_ignores_a_delivery_belonging_to_another_service() {
    let mut consumer = Consumer::<EstimatedTimetable>::new("PASSENGER-APP");
    let elsewhere = Siri::new(
        "2.1",
        ServiceDelivery::new(
            now(),
            "MY-AGENCY",
            vec![ServiceDeliveryPayload::from(VehicleMonitoringDelivery::new(
                now(),
                Vec::new(),
            ))],
        ),
    );

    let ConsumerEvent::Delivered { items, .. } = consumer
        .handle(&elsewhere, now())
        .expect("the message is still a well-formed delivery")
    else {
        panic!("a service delivery is reported as a delivery");
    };
    assert!(
        items.is_empty(),
        "a vehicle-monitoring delivery holds no estimated-timetable journeys"
    );
}
