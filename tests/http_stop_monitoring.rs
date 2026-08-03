//! A departure board over real HTTP, both ways a consumer can ask for one.
//!
//! Stop Monitoring is the service most applications reach for first, and the one
//! where both of SIRI's interaction patterns are used in earnest: a display polls
//! for the board it is about to show, while a system that keeps its own copy
//! subscribes and is told when the board changes. Both run here against a real
//! socket, with every body validated against the official schemas as it passes.

mod support;

use chrono::{DateTime, Duration, FixedOffset, Utc};

use siri_rs::model::{MonitoredCall, MonitoredVehicleJourney};
use siri_rs::pubsub::{
    Consumer, ConsumerEvent, ProducerConfig, StopMonitoring, StopMonitoringSource,
};
use siri_rs::sm::{MonitoredStopVisit, StopMonitoringRequest};
use siri_rs::types::NaturalLanguageString;
use support::http::{ConsumerEndpoint, ProducerEndpoint, Wire};
use support::{validator_available, VALIDATOR_MISSING};

/// The stop the board in these tests is about.
const KROEPCKE: &str = "de:03241:101";

/// Polling: a display asks for the board it is about to show, and shows the answer.
#[tokio::test]
async fn a_departure_board_is_answered_over_http_without_a_subscription() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let wire = Wire::default();
    let mut consumer =
        ConsumerEndpoint::start(Consumer::<StopMonitoring>::new("DISPLAY"), wire.clone()).await;
    let producer = ProducerEndpoint::start(
        ProducerConfig::new("MY-AGENCY"),
        DepartureBoard(board()),
        wire.clone(),
    )
    .await;

    let now = Utc::now().fixed_offset();
    let request = consumer.request(StopMonitoringRequest::at_stop(now, KROEPCKE), now);
    let delivery = consumer.post(&producer.url, &request).await;
    let ConsumerEvent::Delivered { items, .. } = consumer.interpret(&delivery) else {
        panic!("a service request is answered with a delivery");
    };

    assert_eq!(departures(&items), [("10", "Ahlem"), ("17", "Wallensteinstr")]);
    assert_eq!(
        producer.subscriptions(),
        0,
        "polling opens no subscription on either side"
    );
    assert_eq!(consumer.subscriptions(), 0);

    wire.assert_every_message_was_valid();
    assert_eq!(
        wire.exchanged(),
        [
            "consumer → producer: ServiceRequest",
            "producer → consumer: ServiceDelivery",
        ]
    );
}

/// Polling, narrowed: the request names a line, and only that line comes back.
#[tokio::test]
async fn a_polled_board_is_narrowed_to_the_line_the_request_names() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let wire = Wire::default();
    let mut consumer =
        ConsumerEndpoint::start(Consumer::<StopMonitoring>::new("DISPLAY"), wire.clone()).await;
    let producer = ProducerEndpoint::start(
        ProducerConfig::new("MY-AGENCY"),
        DepartureBoard(board()),
        wire.clone(),
    )
    .await;

    let now = Utc::now().fixed_offset();
    let request = consumer.request(
        StopMonitoringRequest {
            line_ref: Some("17".into()),
            ..StopMonitoringRequest::at_stop(now, KROEPCKE)
        },
        now,
    );
    let delivery = consumer.post(&producer.url, &request).await;
    let ConsumerEvent::Delivered { items, .. } = consumer.interpret(&delivery) else {
        panic!("a service request is answered with a delivery");
    };

    assert_eq!(departures(&items), [("17", "Wallensteinstr")]);
    wire.assert_every_message_was_valid();
}

/// Subscribing: the board is pushed when it is first matched and again when it moves.
#[tokio::test]
async fn a_stop_monitoring_subscription_is_pushed_and_updated_over_http() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let wire = Wire::default();
    let mut consumer = ConsumerEndpoint::start(
        Consumer::<StopMonitoring>::new("PASSENGER-APP").confirming_deliveries(),
        wire.clone(),
    )
    .await;
    let producer = ProducerEndpoint::start(
        ProducerConfig::new("MY-AGENCY"),
        DepartureBoard(board()),
        wire.clone(),
    )
    .await;

    let now = Utc::now().fixed_offset();
    let subscribe = consumer.subscribe(
        "departures",
        now + Duration::hours(1),
        StopMonitoringRequest::at_stop(now, KROEPCKE),
        now,
    );
    let response = consumer.post(&producer.url, &subscribe).await;
    let ConsumerEvent::Subscribed { outcomes } = consumer.interpret(&response) else {
        panic!("a subscription request is answered with a subscription outcome");
    };
    assert!(outcomes[0].accepted);

    let ConsumerEvent::Delivered { items, reply } = consumer.next_event().await else {
        panic!("a direct-delivery producer pushes the board it holds");
    };
    assert!(reply.is_some(), "this consumer confirms what it receives");
    assert_eq!(expected_departure(&items[0]), aimed("08:17:00"));

    // The tram loses two minutes; telling the producer is all an application has to do.
    {
        let mut held = producer.producer.lock().expect("the producer is usable");
        held.source_mut().0[0] = departure("10", "Ahlem", "08:17:00", "08:19:00");
        held.data_changed();
    }
    let ConsumerEvent::Delivered { items, .. } = consumer.next_event().await else {
        panic!("a changed board is delivered again");
    };
    assert_eq!(expected_departure(&items[0]), aimed("08:19:00"));

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
            "producer → consumer: ServiceDelivery",
            "consumer → producer: DataReceivedAcknowledgement",
            "producer → consumer: ServiceDelivery",
            "consumer → producer: DataReceivedAcknowledgement",
            "consumer → producer: TerminateSubscriptionRequest",
            "producer → consumer: TerminateSubscriptionResponse",
        ]
    );
}

/// Whatever a real-time system already keeps the services due at a stop in.
struct DepartureBoard(Vec<MonitoredStopVisit>);

impl StopMonitoringSource for DepartureBoard {
    fn visits(&self, request: &StopMonitoringRequest) -> Vec<MonitoredStopVisit> {
        // A real source would honour every filter the request carries. This one
        // honours the two a departure board is usually asked for.
        self.0
            .iter()
            .filter(|visit| {
                visit.monitoring_ref.as_ref() == Some(&request.monitoring_ref)
                    && match &request.line_ref {
                        Some(wanted) => {
                            visit.monitored_vehicle_journey.line_ref.as_ref() == Some(wanted)
                        }
                        None => true,
                    }
            })
            .cloned()
            .collect()
    }
}

fn board() -> Vec<MonitoredStopVisit> {
    vec![
        departure("10", "Ahlem", "08:17:00", "08:17:00"),
        departure("17", "Wallensteinstr", "08:21:00", "08:23:00"),
        // Another stop's departure, so that the monitoring-point filter has work to do.
        MonitoredStopVisit {
            monitoring_ref: Some("de:03241:102".into()),
            ..departure("100", "Hbf", "08:18:00", "08:18:00")
        },
    ]
}

fn departure(line: &str, destination: &str, aimed_at: &str, expected_at: &str) -> MonitoredStopVisit {
    MonitoredStopVisit {
        monitoring_ref: Some(KROEPCKE.into()),
        ..MonitoredStopVisit::new(
            aimed("08:15:00"),
            MonitoredVehicleJourney {
                destination_name: vec![NaturalLanguageString::new(destination)],
                monitored_call: Some(MonitoredCall {
                    stop_point_ref: Some(KROEPCKE.into()),
                    aimed_departure_time: Some(aimed(aimed_at)),
                    expected_departure_time: Some(aimed(expected_at)),
                    ..MonitoredCall::default()
                }),
                ..MonitoredVehicleJourney::on_line(line)
            },
        )
    }
}

/// The line and destination of each visit, in the order they were delivered.
fn departures(visits: &[MonitoredStopVisit]) -> Vec<(&str, &str)> {
    visits
        .iter()
        .map(|visit| {
            let journey = &visit.monitored_vehicle_journey;
            (
                journey
                    .line_ref
                    .as_ref()
                    .map(|line| line.as_str())
                    .unwrap_or_default(),
                journey
                    .destination_name
                    .first()
                    .map(|name| name.value.as_str())
                    .unwrap_or_default(),
            )
        })
        .collect()
}

fn expected_departure(visit: &MonitoredStopVisit) -> DateTime<FixedOffset> {
    visit
        .monitored_vehicle_journey
        .monitored_call
        .as_ref()
        .and_then(|call| call.expected_departure_time)
        .expect("a monitored departure has a real-time time")
}

fn aimed(time: &str) -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339(&format!("2026-03-04T{time}+01:00")).expect("valid timestamp")
}
