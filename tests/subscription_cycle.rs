//! End-to-end tests of the publish/subscribe data hub.
//!
//! A producer and a consumer are wired directly to each other and driven through a
//! full subscription cycle. Every message that crosses between them is validated
//! against the official schemas, so the state machines cannot be "working" while
//! emitting documents no other SIRI implementation would accept.

mod support;

use chrono::{DateTime, Duration, FixedOffset};

use siri_rs::enumerations::{AlertCause, Severity, SituationSourceType, WorkflowStatus};
use siri_rs::et::{EstimatedTimetableRequest, EstimatedTimetableSubscriptionRequest};
use siri_rs::framework::{
    DataReadyNotification, DeliveryError, ErrorCodeDetail, ErrorCondition, ServiceDelivery,
    ServiceDeliveryPayload, SubscriptionRequest, TerminateSubscriptionRequest, TerminationError,
};
use siri_rs::pubsub::{
    Consumer, ConsumerEvent, Outbound, Producer, ProducerConfig, SituationExchange,
    SituationSource, SubscriptionState,
};
use siri_rs::sx::situation::SituationSource as Source;
use siri_rs::sx::{
    PtSituationElement, SituationExchangeDelivery, SituationExchangeRequest,
    SituationExchangeSubscriptionRequest,
};
use siri_rs::types::{DefaultedText, Duration as SiriDuration, HalfOpenTimestampOutputRange};
use siri_rs::Siri;
use support::{validate, validator_available, VALIDATOR_MISSING};

struct Disruptions(Vec<PtSituationElement>);

impl SituationSource for Disruptions {
    fn situations(&self, request: &SituationExchangeRequest) -> Vec<PtSituationElement> {
        self.0
            .iter()
            .filter(|situation| match (&request.severity, &situation.severity) {
                (Some(wanted), Some(actual)) => actual >= wanted,
                (Some(_), None) => false,
                (None, _) => true,
            })
            .cloned()
            .collect()
    }
}

fn now() -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339("2026-03-14T08:00:00+01:00").expect("valid instant")
}

fn situation(at: DateTime<FixedOffset>, number: &str, severity: Severity) -> PtSituationElement {
    let mut situation = PtSituationElement::new(
        at,
        number,
        Source::new(SituationSourceType::DirectReport),
        HalfOpenTimestampOutputRange::between(at, at + Duration::days(2)),
        AlertCause::LiftFailure,
    );
    situation.participant_ref = Some("MY-AGENCY".into());
    situation.progress = Some(WorkflowStatus::Published);
    situation.severity = Some(severity);
    situation.summary = vec![DefaultedText::with_lang("EN", "The lift is out of service")];
    situation
}

/// Writes the message out and checks it against the schemas.
#[track_caller]
fn exchanged(label: &str, message: &Siri) {
    let xml = siri_rs::to_string_pretty(message).expect("message serialises");
    if let Err(complaint) = validate(&xml) {
        panic!("{label} is not valid SIRI:\n{complaint}\n{xml}");
    }
}

#[test]
fn a_direct_delivery_subscription_runs_its_full_cycle() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let now = now();

    let mut producer = Producer::new(
        ProducerConfig::new("MY-AGENCY"),
        Disruptions(vec![situation(now, "2026-0041", Severity::Normal)]),
    )
    .started_at(now - Duration::hours(6));
    let mut consumer = Consumer::<SituationExchange>::new("PASSENGER-APP")
        .at_address("https://app.example/siri")
        .confirming_deliveries();

    let subscribe = consumer.subscribe(
        "lifts",
        now + Duration::hours(12),
        SituationExchangeRequest::new(now),
        now,
    );
    exchanged("SubscriptionRequest", &subscribe);

    let response = producer
        .handle(&subscribe, now)
        .expect("the producer answers")
        .expect("a subscription request is answered");
    exchanged("SubscriptionResponse", &response);

    let ConsumerEvent::Subscribed { outcomes } = consumer
        .handle(&response, now)
        .expect("the consumer reads the answer")
    else {
        panic!("a subscription response is a subscription outcome");
    };
    assert_eq!(outcomes.len(), 1);
    assert!(outcomes[0].accepted);
    assert_eq!(outcomes[0].subscription_ref.as_str(), "lifts");
    assert_eq!(consumer.subscriptions().len(), 1);

    // The producer owes the consumer the situations it matched, and pushes them.
    let outbound = producer.poll(now);
    assert_eq!(outbound.len(), 1, "one subscription, one delivery");
    assert_eq!(outbound[0].recipient.as_str(), "PASSENGER-APP");
    assert_eq!(
        outbound[0].address.as_ref().map(|a| a.as_str()),
        Some("https://app.example/siri")
    );
    exchanged("ServiceDelivery", &outbound[0].message);

    let ConsumerEvent::Delivered { items: situations, reply } = consumer
        .handle(&outbound[0].message, now)
        .expect("the consumer reads the delivery")
    else {
        panic!("a service delivery delivers situations");
    };
    assert_eq!(situations.len(), 1);
    assert_eq!(situations[0].situation_number.as_str(), "2026-0041");
    let reply = reply.expect("a consumer that confirms deliveries replies");
    exchanged("DataReceivedAcknowledgement", &reply);
    assert!(producer
        .handle(&reply, now)
        .expect("the producer accepts the acknowledgement")
        .is_none());

    // Nothing is owed until the situations change again.
    assert!(producer.poll(now).is_empty());
    producer.data_changed();
    assert_eq!(producer.poll(now).len(), 1);

    let terminate = consumer.terminate_all(now);
    exchanged("TerminateSubscriptionRequest", &terminate);
    let confirmation = producer
        .handle(&terminate, now)
        .expect("the producer answers")
        .expect("a termination request is answered");
    exchanged("TerminateSubscriptionResponse", &confirmation);

    let ConsumerEvent::Terminated { subscription_refs } = consumer
        .handle(&confirmation, now)
        .expect("the consumer reads the confirmation")
    else {
        panic!("a termination response confirms terminations");
    };
    assert_eq!(subscription_refs.len(), 1);
    assert!(producer.subscriptions().is_empty());
    assert!(consumer.subscriptions().is_empty());
}

#[test]
fn a_fetched_delivery_subscription_announces_before_it_delivers() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let now = now();

    let mut producer = Producer::new(
        ProducerConfig::new("MY-AGENCY").with_fetched_delivery(),
        Disruptions(vec![situation(now, "2026-0041", Severity::Normal)]),
    );
    let mut consumer = Consumer::<SituationExchange>::new("PASSENGER-APP");

    let subscribe = consumer.subscribe(
        "lifts",
        now + Duration::hours(12),
        SituationExchangeRequest::new(now),
        now,
    );
    let response = producer
        .handle(&subscribe, now)
        .expect("the producer answers")
        .expect("a subscription request is answered");
    consumer.handle(&response, now).expect("the consumer reads it");

    let outbound = producer.poll(now);
    assert_eq!(outbound.len(), 1);
    exchanged("DataReadyNotification", &outbound[0].message);
    assert!(
        outbound[0]
            .message
            .payload
            .as_data_ready_notification()
            .is_some(),
        "a fetched-delivery producer announces rather than pushes"
    );

    let ConsumerEvent::DataReady { reply, fetch } = consumer
        .handle(&outbound[0].message, now)
        .expect("the consumer reads the announcement")
    else {
        panic!("a data-ready notification asks the consumer to fetch");
    };
    exchanged("DataReadyAcknowledgement", &reply);
    exchanged("DataSupplyRequest", &fetch);
    assert!(producer
        .handle(&reply, now)
        .expect("the producer accepts the acknowledgement")
        .is_none());

    let delivery = producer
        .handle(&fetch, now)
        .expect("the producer answers")
        .expect("a data supply request is answered");
    exchanged("ServiceDelivery", &delivery);

    let ConsumerEvent::Delivered { items: situations, reply } = consumer
        .handle(&delivery, now)
        .expect("the consumer reads the delivery")
    else {
        panic!("a service delivery delivers situations");
    };
    assert_eq!(situations.len(), 1);
    assert!(reply.is_none(), "this consumer did not ask to confirm");

    // The fetch settled the debt: polling again announces nothing.
    assert!(producer.poll(now).is_empty());
}

/// Opens the three subscriptions the tests below start from: two held by a departure
/// board that named an address for its deliveries, one by a passenger app that named
/// none. Returns the consumers, board first.
///
/// Both consumers call their subscription `lifts`: SIRI scopes a subscription
/// identifier to its subscriber, so two subscribers choosing the same one is the
/// ordinary case, not a collision, and a producer that keys by identifier alone
/// loses one of them.
fn subscribe_two_consumers(
    producer: &mut Producer<Disruptions, SituationExchange>,
    now: DateTime<FixedOffset>,
) -> [Consumer<SituationExchange>; 2] {
    const BOARD: usize = 0;
    const APP: usize = 1;

    let mut consumers = [
        Consumer::<SituationExchange>::new("DEPARTURE-BOARD").at_address("board"),
        Consumer::<SituationExchange>::new("PASSENGER-APP"),
    ];
    for (consumer, identifier) in [(BOARD, "lifts"), (BOARD, "escalators"), (APP, "lifts")] {
        let subscribe = consumers[consumer].subscribe(
            identifier,
            now + Duration::hours(12),
            SituationExchangeRequest::new(now),
            now,
        );
        let response = producer
            .handle(&subscribe, now)
            .expect("the producer answers")
            .expect("a subscription request is answered");
        consumers[consumer]
            .handle(&response, now)
            .expect("the consumer reads it");
    }
    consumers
}

#[test]
fn every_subscription_is_delivered_to_the_subscriber_that_opened_it() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let now = now();
    let mut producer = Producer::new(
        ProducerConfig::new("MY-AGENCY"),
        Disruptions(vec![situation(now, "2026-0041", Severity::Normal)]),
    );

    subscribe_two_consumers(&mut producer, now);
    assert_eq!(producer.subscriptions().len(), 3);

    let outbound = producer.poll(now);
    let addressed: Vec<(&str, Option<&str>, &str)> = outbound
        .iter()
        .map(|out| {
            exchanged("ServiceDelivery", &out.message);
            let delivery = out
                .message
                .payload
                .as_service_delivery()
                .expect("a due subscription is delivered to");
            let ServiceDeliveryPayload::SituationExchangeDelivery(situations) = &delivery.deliveries
                [0]
            else {
                panic!("a situation exchange producer delivers situations");
            };
            (
                out.recipient.as_str(),
                out.address.as_ref().map(|address| address.as_str()),
                situations
                    .subscription_ref
                    .as_ref()
                    .expect("a delivery says which subscription it satisfies")
                    .as_str(),
            )
        })
        .collect();

    assert_eq!(
        addressed,
        [
            ("DEPARTURE-BOARD", Some("board"), "lifts"),
            ("DEPARTURE-BOARD", Some("board"), "escalators"),
            ("PASSENGER-APP", None, "lifts"),
        ],
        "each subscription is delivered once, to its own subscriber, at the address it named"
    );

    assert!(
        producer.poll(now).is_empty(),
        "a delivery settles every subscription it went to"
    );

    producer.source_mut().0.push(situation(now, "2026-0042", Severity::Severe));
    producer.data_changed();
    assert_eq!(
        producer.poll(now).len(),
        3,
        "changed data falls due for every subscription again"
    );
}

#[test]
fn a_fetch_settles_only_the_subscriptions_of_the_consumer_that_asked() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let now = now();
    let mut producer = Producer::new(
        ProducerConfig::new("MY-AGENCY").with_fetched_delivery(),
        Disruptions(vec![situation(now, "2026-0041", Severity::Normal)]),
    );

    let [mut board, _app] = subscribe_two_consumers(&mut producer, now);

    let announcements = producer.poll(now);
    assert_eq!(announcements.len(), 3);
    let identifiers: Vec<String> = announcements
        .iter()
        .map(|out| {
            out.message
                .payload
                .as_data_ready_notification()
                .expect("a fetched-delivery producer announces")
                .message_identifier
                .as_ref()
                .expect("an announcement is identifiable")
                .as_str()
                .to_owned()
        })
        .collect();
    assert_eq!(
        identifiers,
        ["MY-AGENCY-1", "MY-AGENCY-2", "MY-AGENCY-3"],
        "each announcement carries an identifier of its own"
    );

    let ConsumerEvent::DataReady { fetch, .. } = board
        .handle(&announcements[0].message, now)
        .expect("the consumer reads the announcement")
    else {
        panic!("a data-ready notification asks the consumer to fetch");
    };
    let delivery = producer
        .handle(&fetch, now)
        .expect("the producer answers")
        .expect("a data supply request is answered");
    exchanged("ServiceDelivery", &delivery);

    let delivered = delivery
        .payload
        .as_service_delivery()
        .expect("the answer is a service delivery");
    assert_eq!(
        delivered.deliveries.len(),
        2,
        "the fetch collects everything that subscriber is owed, and nothing else"
    );

    let waiting: Vec<(&str, bool)> = producer
        .subscriptions()
        .iter()
        .map(|held| {
            (
                held.subscription_ref.as_str(),
                matches!(held.state, SubscriptionState::AwaitingFetch(_)),
            )
        })
        .collect();
    assert_eq!(
        waiting,
        [("lifts", false), ("escalators", false), ("lifts", true)],
        "the other consumer's announcement is still outstanding"
    );
}

#[test]
fn two_subscribers_may_choose_the_same_subscription_identifier() {
    let now = now();
    let mut producer = Producer::new(
        ProducerConfig::new("MY-AGENCY"),
        Disruptions(vec![situation(now, "2026-0041", Severity::Normal)]),
    );
    let [_board, mut app] = subscribe_two_consumers(&mut producer, now);

    let held: Vec<(&str, &str)> = producer
        .subscriptions()
        .iter()
        .map(|held| (held.subscriber_ref.as_str(), held.subscription_ref.as_str()))
        .collect();
    assert_eq!(
        held,
        [
            ("DEPARTURE-BOARD", "lifts"),
            ("DEPARTURE-BOARD", "escalators"),
            ("PASSENGER-APP", "lifts"),
        ],
        "a subscription is keyed by subscriber and identifier, so the app's `lifts` \
         does not displace the board's"
    );

    // Closing the app's `lifts` leaves the board's alone.
    let terminate = app.terminate_all(now);
    let confirmation = producer
        .handle(&terminate, now)
        .expect("the producer answers")
        .expect("a termination request is answered");
    exchanged("TerminateSubscriptionResponse", &confirmation);
    let held: Vec<(&str, &str)> = producer
        .subscriptions()
        .iter()
        .map(|held| (held.subscriber_ref.as_str(), held.subscription_ref.as_str()))
        .collect();
    assert_eq!(held, [("DEPARTURE-BOARD", "lifts"), ("DEPARTURE-BOARD", "escalators")]);
}

#[test]
fn a_heartbeat_reaches_every_subscriber() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let now = now();
    let mut producer = Producer::new(
        ProducerConfig::new("MY-AGENCY")
            .with_heartbeat(SiriDuration::parse("PT5M").expect("valid duration")),
        Disruptions(Vec::new()),
    );
    let [mut board, mut app] = subscribe_two_consumers(&mut producer, now);

    let heartbeats: Vec<Outbound> = producer
        .poll(now)
        .into_iter()
        .filter(|out| out.message.payload.as_heartbeat_notification().is_some())
        .collect();
    let sent: Vec<(&str, Option<&str>)> = heartbeats
        .iter()
        .map(|out| (out.recipient.as_str(), out.address.as_ref().map(|a| a.as_str())))
        .collect();
    assert_eq!(
        sent,
        [("DEPARTURE-BOARD", Some("board")), ("PASSENGER-APP", None)],
        "one heartbeat per subscriber, not one per subscription and not one in all"
    );
    for (consumer, heartbeat) in [(&mut board, &heartbeats[0]), (&mut app, &heartbeats[1])] {
        exchanged("HeartbeatNotification", &heartbeat.message);
        assert!(matches!(
            consumer.handle(&heartbeat.message, now).expect("the consumer reads it"),
            ConsumerEvent::Alive { .. }
        ));
    }
}

#[test]
fn a_request_that_mixes_services_is_answered_entry_by_entry() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let now = now();
    let mut producer = Producer::new(
        ProducerConfig::new("MY-AGENCY"),
        Disruptions(vec![situation(now, "2026-0041", Severity::Normal)]),
    );
    let mut app = Consumer::<SituationExchange>::new("PASSENGER-APP");
    let subscribe = app.subscribe("lifts", now + Duration::hours(12), SituationExchangeRequest::new(now), now);
    let response = producer
        .handle(&subscribe, now)
        .expect("the producer answers")
        .expect("a subscription request is answered");
    app.handle(&response, now).expect("the app reads it");

    // The board asks for situations, which this producer serves, and for estimated
    // journeys, which it does not, in one request. The explicit schema, `siri.xsd`,
    // admits only one service per request, so this document is not validated here:
    // the reader accepts it all the same, and what is pinned is that the producer
    // answers it entry by entry rather than holding the first and failing the rest.
    let mixed = Siri::new(
        siri_rs::pubsub::PROTOCOL_VERSION,
        SubscriptionRequest::new(
            now,
            "DEPARTURE-BOARD",
            vec![
                SituationExchangeSubscriptionRequest::new(
                    "lifts",
                    now + Duration::hours(12),
                    SituationExchangeRequest::new(now),
                )
                .into(),
                EstimatedTimetableSubscriptionRequest::new(
                    "journeys",
                    now + Duration::hours(12),
                    EstimatedTimetableRequest::new(now),
                )
                .into(),
            ],
        ),
    );

    let response = producer
        .handle(&mixed, now)
        .expect("an entry the producer cannot serve is refused, not a failure of the request")
        .expect("a subscription request is answered");
    exchanged("SubscriptionResponse", &response);
    let outcomes: Vec<(&str, bool)> = response
        .payload
        .as_subscription_response()
        .expect("the answer is a subscription response")
        .response_status
        .iter()
        .map(|status| (status.subscription_ref.as_str(), status.is_accepted()))
        .collect();
    assert_eq!(
        outcomes,
        [("lifts", true), ("journeys", false)],
        "each entry is answered on its own"
    );

    let held: Vec<(&str, &str)> = producer
        .subscriptions()
        .iter()
        .map(|held| (held.subscriber_ref.as_str(), held.subscription_ref.as_str()))
        .collect();
    assert_eq!(
        held,
        [("PASSENGER-APP", "lifts"), ("DEPARTURE-BOARD", "lifts")],
        "the entry that was accepted is held, the refused one is not, and the app's is untouched"
    );
    assert_eq!(producer.poll(now).len(), 2, "both held subscriptions are owed a delivery");
}

#[test]
fn a_fetch_that_finds_nothing_waiting_is_answered_with_a_document_the_schema_accepts() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let now = now();
    let mut producer = Producer::new(
        ProducerConfig::new("MY-AGENCY").with_fetched_delivery(),
        Disruptions(vec![situation(now, "2026-0041", Severity::Normal)]),
    );
    let [mut board, _app] = subscribe_two_consumers(&mut producer, now);

    let announcements = producer.poll(now);
    let ConsumerEvent::DataReady { fetch, .. } = board
        .handle(&announcements[0].message, now)
        .expect("the board reads the announcement")
    else {
        panic!("a data-ready notification asks the consumer to fetch");
    };
    let first = producer
        .handle(&fetch, now)
        .expect("the producer answers")
        .expect("a data supply request is answered");
    exchanged("ServiceDelivery", &first);

    // The same fetch again — a retry after a lost response, say — finds the board's
    // subscriptions settled. Whatever the answer is, it has to be SIRI.
    let again = producer
        .handle(&fetch, now)
        .expect("the producer answers")
        .expect("a data supply request is answered");
    exchanged("ServiceDelivery with nothing to supply", &again);
    let ConsumerEvent::Delivered { items, .. } = board
        .handle(&again, now)
        .expect("the board reads the answer")
    else {
        panic!("a data supply request is answered with a delivery");
    };
    assert!(items.is_empty(), "there was nothing to supply");

    let waiting: Vec<(&str, &str, bool)> = producer
        .subscriptions()
        .iter()
        .map(|held| {
            (
                held.subscriber_ref.as_str(),
                held.subscription_ref.as_str(),
                matches!(held.state, SubscriptionState::AwaitingFetch(_)),
            )
        })
        .collect();
    assert_eq!(
        waiting,
        [
            ("DEPARTURE-BOARD", "lifts", false),
            ("DEPARTURE-BOARD", "escalators", false),
            ("PASSENGER-APP", "lifts", true),
        ],
        "the app's announcement is still outstanding"
    );
}

#[test]
fn terminating_a_subscription_the_subscriber_does_not_hold_is_reported_as_unknown() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let now = now();
    let mut producer = Producer::new(ProducerConfig::new("MY-AGENCY"), Disruptions(Vec::new()));
    let [_board, mut app] = subscribe_two_consumers(&mut producer, now);

    // The board holds `escalators`; the app does not, and asks to close it.
    let terminate = Siri::new(
        siri_rs::pubsub::PROTOCOL_VERSION,
        TerminateSubscriptionRequest::subscriptions(now, "PASSENGER-APP", vec!["escalators".into()]),
    );
    exchanged("TerminateSubscriptionRequest", &terminate);
    let confirmation = producer
        .handle(&terminate, now)
        .expect("the producer answers")
        .expect("a termination request is answered");
    exchanged("TerminateSubscriptionResponse", &confirmation);

    let statuses = &confirmation
        .payload
        .as_terminate_subscription_response()
        .expect("the answer is a termination response")
        .termination_response_status;
    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses[0].subscription_ref.as_str(), "escalators");
    assert_eq!(statuses[0].status, Some(false), "nothing was closed");
    assert!(
        matches!(
            statuses[0].error_condition.as_ref().map(|condition| &condition.code),
            Some(TerminationError::UnknownSubscriptionError(_))
        ),
        "the schema names the reason: {:?}",
        statuses[0].error_condition
    );

    let ConsumerEvent::Terminated { subscription_refs } = app
        .handle(&confirmation, now)
        .expect("the app reads the answer")
    else {
        panic!("a termination response confirms terminations");
    };
    assert!(subscription_refs.is_empty(), "the app was told nothing was closed");
    assert_eq!(
        producer.subscriptions().len(),
        3,
        "the board's `escalators` is not the app's to close"
    );
}

#[test]
fn a_fetch_after_an_announcement_without_an_identifier_names_no_notification() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let now = now();
    let mut consumer = Consumer::<SituationExchange>::new("PASSENGER-APP");
    let unnamed = Siri::new(
        siri_rs::pubsub::PROTOCOL_VERSION,
        DataReadyNotification::new(now, "MY-AGENCY"),
    );
    exchanged("DataReadyNotification without MessageIdentifier", &unnamed);

    let ConsumerEvent::DataReady { fetch, .. } = consumer
        .handle(&unnamed, now)
        .expect("the consumer reads the announcement")
    else {
        panic!("a data-ready notification asks the consumer to fetch");
    };
    exchanged("DataSupplyRequest", &fetch);
    let request = fetch
        .payload
        .as_data_supply_request()
        .expect("the fetch is a data supply request");
    assert_eq!(
        request.notification_ref, None,
        "there is no notification to refer to, so the element is left out rather than written empty"
    );
}

#[test]
fn a_delivery_that_reports_a_failure_is_not_read_as_a_delivery_of_nothing() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let now = now();
    let mut consumer = Consumer::<SituationExchange>::new("PASSENGER-APP");

    let mut failed = ServiceDelivery::new(
        now,
        "MY-AGENCY",
        vec![SituationExchangeDelivery::new(now, Vec::new()).into()],
    );
    failed.status = Some(false);
    failed.error_condition = Some(ErrorCondition::with_description(
        DeliveryError::OtherError(ErrorCodeDetail::default()),
        "the situation store is being rebuilt",
    ));
    let message = Siri::new(siri_rs::pubsub::PROTOCOL_VERSION, failed);
    exchanged("ServiceDelivery reporting a failure", &message);

    let event = consumer
        .handle(&message, now)
        .expect("the consumer reads the delivery");
    let reported = format!("{event:?}");
    assert!(
        reported.contains("the situation store is being rebuilt"),
        "the failure and its reason reach the application: {reported}"
    );
}

#[test]
fn a_request_filter_narrows_what_is_delivered() {
    let now = now();
    let mut producer = Producer::new(
        ProducerConfig::new("MY-AGENCY"),
        Disruptions(vec![
            situation(now, "minor", Severity::Slight),
            situation(now, "major", Severity::Severe),
        ]),
    );
    let mut consumer = Consumer::<SituationExchange>::new("PASSENGER-APP");

    let mut request = SituationExchangeRequest::new(now);
    request.severity = Some(Severity::Severe);
    let subscribe = consumer.subscribe("severe-only", now + Duration::hours(1), request, now);
    let response = producer
        .handle(&subscribe, now)
        .expect("the producer answers")
        .expect("a subscription request is answered");
    consumer.handle(&response, now).expect("the consumer reads it");

    let outbound = producer.poll(now);
    let ConsumerEvent::Delivered { items: situations, .. } = consumer
        .handle(&outbound[0].message, now)
        .expect("the consumer reads the delivery")
    else {
        panic!("a service delivery delivers situations");
    };
    assert_eq!(situations.len(), 1);
    assert_eq!(situations[0].situation_number.as_str(), "major");
}

#[test]
fn a_subscription_that_has_already_lapsed_is_refused() {
    let now = now();
    let mut producer = Producer::new(ProducerConfig::new("MY-AGENCY"), Disruptions(Vec::new()));
    let mut consumer = Consumer::<SituationExchange>::new("PASSENGER-APP");

    let subscribe = consumer.subscribe(
        "too-late",
        now - Duration::minutes(1),
        SituationExchangeRequest::new(now),
        now,
    );
    let response = producer
        .handle(&subscribe, now)
        .expect("the producer answers")
        .expect("a subscription request is answered");
    exchanged("SubscriptionResponse", &response);

    let ConsumerEvent::Subscribed { outcomes } = consumer
        .handle(&response, now)
        .expect("the consumer reads the answer")
    else {
        panic!("a subscription response is a subscription outcome");
    };
    assert!(!outcomes[0].accepted);
    assert!(producer.subscriptions().is_empty());
    assert!(consumer.subscriptions().is_empty());
}

#[test]
fn a_lapsed_subscription_is_ended_by_the_producer() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let now = now();
    let mut producer = Producer::new(ProducerConfig::new("MY-AGENCY"), Disruptions(Vec::new()));
    let mut consumer = Consumer::<SituationExchange>::new("PASSENGER-APP");

    let subscribe = consumer.subscribe(
        "short",
        now + Duration::minutes(10),
        SituationExchangeRequest::new(now),
        now,
    );
    let response = producer
        .handle(&subscribe, now)
        .expect("the producer answers")
        .expect("a subscription request is answered");
    consumer.handle(&response, now).expect("the consumer reads it");
    assert_eq!(consumer.subscriptions().len(), 1);

    let later = now + Duration::minutes(11);
    let outbound = producer.poll(later);
    let ended = outbound
        .iter()
        .find(|out| {
            out.message
                .payload
                .as_subscription_terminated_notification()
                .is_some()
        })
        .expect("a lapsed subscription is reported to its subscriber");
    exchanged("SubscriptionTerminatedNotification", &ended.message);

    let ConsumerEvent::SubscriptionEnded { subscription_ref } = consumer
        .handle(&ended.message, later)
        .expect("the consumer reads the notification")
    else {
        panic!("the notification ends a subscription");
    };
    assert_eq!(subscription_ref.as_str(), "short");
    assert!(producer.subscriptions().is_empty());
    assert!(consumer.subscriptions().is_empty());
}

#[test]
fn a_heartbeat_is_sent_no_faster_than_its_interval() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let now = now();
    let mut producer = Producer::new(
        ProducerConfig::new("MY-AGENCY")
            .with_heartbeat(SiriDuration::parse("PT5M").expect("valid duration")),
        Disruptions(Vec::new()),
    )
    .started_at(now - Duration::hours(6));
    let mut consumer = Consumer::<SituationExchange>::new("PASSENGER-APP");

    let subscribe = consumer.subscribe(
        "lifts",
        now + Duration::hours(12),
        SituationExchangeRequest::new(now),
        now,
    );
    let response = producer
        .handle(&subscribe, now)
        .expect("the producer answers")
        .expect("a subscription request is answered");
    consumer.handle(&response, now).expect("the consumer reads it");

    let heartbeat = producer
        .poll(now)
        .into_iter()
        .find(|out| out.message.payload.as_heartbeat_notification().is_some())
        .expect("the first poll sends a heartbeat");
    exchanged("HeartbeatNotification", &heartbeat.message);

    let ConsumerEvent::Alive {
        service_started_time,
    } = consumer
        .handle(&heartbeat.message, now)
        .expect("the consumer reads the heartbeat")
    else {
        panic!("a heartbeat says the producer is alive");
    };
    assert_eq!(service_started_time, Some(now - Duration::hours(6)));

    let soon = now + Duration::minutes(1);
    assert!(
        !producer
            .poll(soon)
            .iter()
            .any(|out| out.message.payload.as_heartbeat_notification().is_some()),
        "a heartbeat must not be sent faster than its interval"
    );

    let after_interval = now + Duration::minutes(6);
    assert!(producer
        .poll(after_interval)
        .iter()
        .any(|out| out.message.payload.as_heartbeat_notification().is_some()));
}

#[test]
fn a_direct_request_is_answered_with_a_delivery() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let now = now();
    let mut producer = Producer::new(
        ProducerConfig::new("MY-AGENCY"),
        Disruptions(vec![situation(now, "2026-0041", Severity::Normal)]),
    );

    let request = Siri::new(
        siri_rs::pubsub::PROTOCOL_VERSION,
        siri_rs::framework::ServiceRequest::new(
            now,
            "PASSENGER-APP",
            vec![SituationExchangeRequest::new(now).into()],
        ),
    );
    exchanged("ServiceRequest", &request);

    let delivery = producer
        .handle(&request, now)
        .expect("the producer answers")
        .expect("a service request is answered");
    exchanged("ServiceDelivery", &delivery);

    let delivered = delivery
        .payload
        .as_service_delivery()
        .expect("the answer is a service delivery");
    assert_eq!(delivered.deliveries.len(), 1);
    assert!(producer.subscriptions().is_empty(), "a direct request opens nothing");
}

#[test]
fn a_check_status_request_reports_when_the_service_started() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let now = now();
    let started = now - Duration::hours(6);
    let mut producer =
        Producer::new(ProducerConfig::new("MY-AGENCY"), Disruptions(Vec::new())).started_at(started);

    let request = Siri::new(
        siri_rs::pubsub::PROTOCOL_VERSION,
        siri_rs::framework::CheckStatusRequest::new(now, "PASSENGER-APP"),
    );
    exchanged("CheckStatusRequest", &request);

    let response = producer
        .handle(&request, now)
        .expect("the producer answers")
        .expect("a check status request is answered");
    exchanged("CheckStatusResponse", &response);

    let status = response
        .payload
        .as_check_status_response()
        .expect("the answer is a check status response");
    assert!(status.is_healthy());
    assert_eq!(status.service_started_time, Some(started));
}
