//! The publish/subscribe hub carrying the journey services over real HTTP.
//!
//! `tests/http_endpoint.rs` drives the same machinery for Situation Exchange. This
//! one does it for the two real-time journey services, which is where the hub's
//! move from one service to any of them is actually put to the test: the state
//! machines are the same code, only the service differs. As there, every body that
//! crosses the wire is validated against the official schemas as it passes — the
//! shared harness in `tests/support/http.rs` does that for both sides.

mod support;

use chrono::{DateTime, Duration, FixedOffset, Utc};

use siri_rs::et::EstimatedTimetableRequest;
use siri_rs::model::{
    EstimatedCall, EstimatedVehicleJourney, Location, MonitoredVehicleJourney, Position,
};
use siri_rs::pubsub::{
    Consumer, ConsumerEvent, EstimatedTimetable, EstimatedTimetableSource, ProducerConfig,
    VehicleMonitoring, VehicleMonitoringSource,
};
use siri_rs::vm::{VehicleActivity, VehicleMonitoringRequest};
use support::http::{ConsumerEndpoint, ProducerEndpoint, Wire};
use support::{validator_available, VALIDATOR_MISSING};

#[tokio::test]
async fn an_estimated_timetable_subscription_runs_its_full_cycle_over_http() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let wire = Wire::default();
    let mut consumer =
        ConsumerEndpoint::start(Consumer::<EstimatedTimetable>::new("PASSENGER-APP"), wire.clone())
            .await;
    let producer = ProducerEndpoint::start(
        ProducerConfig::new("MY-AGENCY").with_fetched_delivery(),
        RunningJourneys(vec![delayed_journey(), cancelled_journey()]),
        wire.clone(),
    )
    .await;

    let now = Utc::now().fixed_offset();
    let subscribe = consumer.subscribe(
        "departures",
        now + Duration::hours(1),
        EstimatedTimetableRequest::new(now),
        now,
    );
    let response = consumer.post(&producer.url, &subscribe).await;
    let ConsumerEvent::Subscribed { outcomes } = consumer.interpret(&response) else {
        panic!("a subscription request is answered with a subscription outcome");
    };
    assert_eq!(outcomes.len(), 1);
    assert!(outcomes[0].accepted);

    // Fetched delivery: the producer announces the journeys and the consumer collects.
    let ConsumerEvent::DataReady { fetch, .. } = consumer.next_event().await else {
        panic!("a fetched-delivery producer announces its data");
    };
    let delivery = consumer.post(&producer.url, &fetch).await;
    let ConsumerEvent::Delivered { items, .. } = consumer.interpret(&delivery) else {
        panic!("a data supply request is answered with the journeys");
    };

    assert_eq!(
        items.iter().map(|j| j.line_ref.as_str()).collect::<Vec<_>>(),
        ["10", "10"]
    );
    assert_eq!(
        items[0].estimated_calls()[0].expected_departure_time,
        items[0].estimated_calls()[0]
            .aimed_departure_time
            .map(|aimed| aimed + Duration::minutes(3)),
        "the first journey is the one running three minutes late"
    );
    assert_eq!(items[1].cancellation, Some(true));

    let terminate = consumer.terminate_all(now);
    let confirmation = consumer.post(&producer.url, &terminate).await;
    let ConsumerEvent::Terminated { subscription_refs } = consumer.interpret(&confirmation) else {
        panic!("a termination request is answered with a confirmation");
    };
    assert_eq!(subscription_refs.len(), 1);
    assert_eq!(producer.subscriptions(), 0);

    wire.assert_every_message_was_valid();
    assert_eq!(
        wire.exchanged(),
        [
            "consumer → producer: SubscriptionRequest",
            "producer → consumer: SubscriptionResponse",
            "producer → consumer: DataReadyNotification",
            "consumer → producer: DataReadyAcknowledgement",
            "consumer → producer: DataSupplyRequest",
            "producer → consumer: ServiceDelivery",
            "consumer → producer: TerminateSubscriptionRequest",
            "producer → consumer: TerminateSubscriptionResponse",
        ]
    );
}

#[tokio::test]
async fn a_vehicle_monitoring_subscription_is_pushed_and_updated_over_http() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let wire = Wire::default();
    let mut consumer = ConsumerEndpoint::start(
        Consumer::<VehicleMonitoring>::new("MAP").confirming_deliveries(),
        wire.clone(),
    )
    .await;
    let producer = ProducerEndpoint::start(
        ProducerConfig::new("MY-AGENCY"),
        TrackedVehicles(vec![vehicle_at(9.7411, 52.3759)]),
        wire.clone(),
    )
    .await;

    let now = Utc::now().fixed_offset();
    let subscribe = consumer.subscribe(
        "fleet",
        now + Duration::hours(1),
        VehicleMonitoringRequest::new(now),
        now,
    );
    let response = consumer.post(&producer.url, &subscribe).await;
    consumer.interpret(&response);

    // Direct delivery: the vehicle arrives at the consumer's address unasked.
    let ConsumerEvent::Delivered { items, reply } = consumer.next_event().await else {
        panic!("a direct-delivery producer pushes what it holds");
    };
    assert!(reply.is_some(), "this consumer confirms what it receives");
    assert_eq!(position_of(&items[0]), (9.7411, 52.3759));

    // The vehicle moves; telling the producer is all an application has to do.
    {
        let mut held = producer.producer.lock().expect("the producer is usable");
        held.source_mut().0 = vec![vehicle_at(9.7500, 52.3800)];
        held.data_changed();
    }
    let ConsumerEvent::Delivered { items, .. } = consumer.next_event().await else {
        panic!("a changed source is delivered again");
    };
    assert_eq!(position_of(&items[0]), (9.7500, 52.3800));

    let terminate = consumer.terminate_all(now);
    let confirmation = consumer.post(&producer.url, &terminate).await;
    consumer.interpret(&confirmation);
    assert_eq!(producer.subscriptions(), 0);

    wire.assert_every_message_was_valid();
    assert_eq!(
        wire.exchanged(),
        [
            "consumer → producer: SubscriptionRequest",
            "producer → consumer: SubscriptionResponse",
            "producer → consumer: ServiceDelivery",
            "consumer → producer: DataReceivedAcknowledgement",
            "producer → consumer: ServiceDelivery",
            "consumer → producer: DataReceivedAcknowledgement",
            "consumer → producer: TerminateSubscriptionRequest",
            "producer → consumer: TerminateSubscriptionResponse",
        ]
    );
}

/// A producer answering a service request without any subscription at all.
#[tokio::test]
async fn a_vehicle_monitoring_request_is_answered_over_http_without_a_subscription() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let wire = Wire::default();
    let mut consumer =
        ConsumerEndpoint::start(Consumer::<VehicleMonitoring>::new("MAP"), wire.clone()).await;
    let producer = ProducerEndpoint::start(
        ProducerConfig::new("MY-AGENCY"),
        TrackedVehicles(vec![vehicle_at(9.7411, 52.3759)]),
        wire.clone(),
    )
    .await;

    let now = Utc::now().fixed_offset();
    let request = consumer.request(VehicleMonitoringRequest::new(now), now);
    let delivery = consumer.post(&producer.url, &request).await;
    let ConsumerEvent::Delivered { items, .. } = consumer.interpret(&delivery) else {
        panic!("a service request is answered with a delivery");
    };

    assert_eq!(position_of(&items[0]), (9.7411, 52.3759));
    assert_eq!(producer.subscriptions(), 0, "a request opens no subscription");
    wire.assert_every_message_was_valid();
    assert_eq!(
        wire.exchanged(),
        [
            "consumer → producer: ServiceRequest",
            "producer → consumer: ServiceDelivery",
        ]
    );
}

/// Whatever an operator already keeps its running journeys in.
struct RunningJourneys(Vec<EstimatedVehicleJourney>);

impl EstimatedTimetableSource for RunningJourneys {
    fn journeys(&self, request: &EstimatedTimetableRequest) -> Vec<EstimatedVehicleJourney> {
        // A real source would apply every filter the request carries. This one
        // honours the line filter and publishes the rest.
        let wanted: Vec<&str> = request
            .line_directions()
            .iter()
            .map(|line| line.line_ref.as_str())
            .collect();
        self.0
            .iter()
            .filter(|journey| wanted.is_empty() || wanted.contains(&journey.line_ref.as_str()))
            .cloned()
            .collect()
    }
}

/// Whatever a tracking system already keeps its vehicles in.
struct TrackedVehicles(Vec<VehicleActivity>);

impl VehicleMonitoringSource for TrackedVehicles {
    fn vehicles(&self, request: &VehicleMonitoringRequest) -> Vec<VehicleActivity> {
        match request.vehicle_ref.as_ref() {
            Some(wanted) => self
                .0
                .iter()
                .filter(|activity| {
                    activity.monitored_vehicle_journey.vehicle_ref.as_ref() == Some(wanted)
                })
                .cloned()
                .collect(),
            None => self.0.clone(),
        }
    }
}

fn delayed_journey() -> EstimatedVehicleJourney {
    let aimed = timestamp("2026-03-04T08:20:00+01:00");
    EstimatedVehicleJourney::dated("10", "OUT", "10-0815").with_estimated_calls(vec![EstimatedCall {
        aimed_departure_time: Some(aimed),
        expected_departure_time: Some(aimed + Duration::minutes(3)),
        ..EstimatedCall::at("de:03241:101")
    }])
}

fn cancelled_journey() -> EstimatedVehicleJourney {
    EstimatedVehicleJourney::dated("10", "OUT", "10-0845").cancelled()
}

fn vehicle_at(longitude: f64, latitude: f64) -> VehicleActivity {
    let recorded = timestamp("2026-03-04T08:15:00+01:00");
    VehicleActivity::new(
        recorded,
        recorded + Duration::minutes(5),
        MonitoredVehicleJourney {
            vehicle_location: Some(Location::wgs84(longitude, latitude)),
            vehicle_ref: Some("VEH-4711".into()),
            ..MonitoredVehicleJourney::on_line("10")
        },
    )
}

fn position_of(activity: &VehicleActivity) -> (f64, f64) {
    match activity
        .monitored_vehicle_journey
        .vehicle_location
        .as_ref()
        .and_then(Location::position)
    {
        Some(Position::Wgs84 {
            longitude,
            latitude,
            ..
        }) => (longitude, latitude),
        other => panic!("a tracked vehicle has a WGS 84 position, got {other:?}"),
    }
}

fn timestamp(rfc3339: &str) -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339(rfc3339).expect("valid timestamp")
}
