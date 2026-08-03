//! What one turn of the publish/subscribe cycle costs.
//!
//! A situation-exchange producer publishes the situations of an official example
//! document to a growing number of subscribers. `handle` measures answering an
//! incoming message, `poll` the two states a subscription can be polled in: owing
//! its consumer a delivery, or owing nothing and merely being looked at.

// `criterion_group!` expands to a function that has nowhere to carry documentation,
// so the crate's `missing_docs` lint is lifted for the benchmarks.
#![allow(missing_docs)]

mod support;

use chrono::{DateTime, Duration, FixedOffset};
use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use siri_rs::framework::SubscriptionRequest;
use siri_rs::pubsub::{
    Consumer, ConsumerEvent, Producer, ProducerConfig, Service, SituationExchange, SituationSource,
    SubscriptionParts,
};
use siri_rs::sx::{PtSituationElement, SituationExchangeRequest};
use siri_rs::Siri;

/// How many subscriptions a producer is measured holding.
const HELD: [usize; 3] = [1, 16, 128];

fn handle(c: &mut Criterion) {
    let situations = published_situations();
    let mut group = c.benchmark_group("handle");

    let opening = subscription_request(1);
    for held in HELD {
        group.bench_with_input(BenchmarkId::new("subscription-request", held), &held, |b, &held| {
            b.iter_batched_ref(
                || producer_holding(&situations, held),
                |producer| {
                    producer
                        .handle(&opening, now())
                        .expect("a subscription request is answered")
                },
                BatchSize::SmallInput,
            );
        });
    }

    let request = service_request();
    group.bench_function("service-request", |b| {
        b.iter_batched_ref(
            || producer_holding(&situations, 0),
            |producer| {
                producer
                    .handle(&request, now())
                    .expect("a service request is answered")
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn poll(c: &mut Criterion) {
    let situations = published_situations();
    let mut group = c.benchmark_group("poll");

    for held in HELD {
        group.bench_with_input(BenchmarkId::new("delivery-due", held), &held, |b, &held| {
            b.iter_batched_ref(
                || producer_holding(&situations, held),
                |producer| producer.poll(now()),
                BatchSize::SmallInput,
            );
        });

        group.bench_with_input(BenchmarkId::new("idle", held), &held, |b, &held| {
            b.iter_batched_ref(
                || {
                    let mut producer = producer_holding(&situations, held);
                    producer.poll(now());
                    producer
                },
                |producer| producer.poll(now()),
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

/// What the producer publishes: the situations of an official delivery, read back
/// out of it with a consumer.
fn published_situations() -> Vec<PtSituationElement> {
    let delivery: Siri =
        siri_rs::from_str(&support::default_bound_situation().xml).expect("the fixture reads");
    let mut consumer = Consumer::<SituationExchange>::new("BENCH");
    let ConsumerEvent::Delivered { items, .. } = consumer
        .handle(&delivery, now())
        .expect("the delivery is understood")
    else {
        panic!("the fixture is a situation delivery");
    };
    assert!(!items.is_empty(), "the fixture carries situations");
    items
}

/// The situations a producer publishes, whatever it is asked for.
struct Published(Vec<PtSituationElement>);

impl SituationSource for Published {
    fn situations(&self, _request: &SituationExchangeRequest) -> Vec<PtSituationElement> {
        self.0.clone()
    }
}

/// A producer holding `held` open subscriptions, each owed a delivery.
fn producer_holding(
    situations: &[PtSituationElement],
    held: usize,
) -> Producer<Published, SituationExchange> {
    let mut producer = Producer::new(
        ProducerConfig::new("HUB"),
        Published(situations.to_owned()),
    );
    if held > 0 {
        producer
            .handle(&subscription_request(held), now())
            .expect("the subscriptions are accepted");
    }
    producer
}

/// A request opening `count` subscriptions in one message.
fn subscription_request(count: usize) -> Siri {
    let subscriptions = (0..count)
        .map(|index| {
            SituationExchange::subscription_request(SubscriptionParts {
                subscriber_ref: None,
                subscription_identifier: format!("subscription-{index}").into(),
                initial_termination_time: now() + Duration::hours(1),
                request: SituationExchangeRequest::new(now()),
                incremental_updates: None,
            })
        })
        .collect();
    Siri::new(
        "2.1",
        SubscriptionRequest::new(now(), "PASSENGER-APP", subscriptions),
    )
}

/// A direct request for the situations, outside any subscription.
fn service_request() -> Siri {
    let mut consumer = Consumer::<SituationExchange>::new("PASSENGER-APP");
    consumer.request(SituationExchangeRequest::new(now()), now())
}

fn now() -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339("2026-03-14T08:00:00+01:00").expect("a valid instant")
}

criterion_group!(benches, handle, poll);
criterion_main!(benches);
