//! A complete SIRI-SX subscription, start to finish, with no network in sight.
//!
//! This is the shape of a real endpoint. A producer holds situations and answers
//! requests; a consumer subscribes and collects deliveries. Both are state
//! machines over SIRI messages, so wiring them to a web framework means little
//! more than choosing which handler feeds `handle` and where `poll` sends its
//! output — here they are wired directly to each other so the whole exchange is
//! visible.
//!
//! Run with `cargo run --example sx_endpoint`.

use chrono::{DateTime, Duration, FixedOffset};

use siri_rs::enumerations::{AlertCause, Severity, SituationSourceType, WorkflowStatus};
use siri_rs::pubsub::{Consumer, ConsumerEvent, Producer, ProducerConfig, SituationExchange, SituationSource};
use siri_rs::sx::situation::SituationSource as Source;
use siri_rs::sx::{PtSituationElement, SituationExchangeRequest};
use siri_rs::types::{DefaultedText, HalfOpenTimestampOutputRange, NaturalLanguageString};
use siri_rs::Siri;

/// Whatever an application already keeps its disruptions in.
struct Disruptions {
    situations: Vec<PtSituationElement>,
}

impl SituationSource for Disruptions {
    fn situations(&self, request: &SituationExchangeRequest) -> Vec<PtSituationElement> {
        // A real source would apply every filter the request carries. This one
        // honours the severity filter and publishes the rest.
        self.situations
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

fn main() -> siri_rs::Result<()> {
    let now = DateTime::parse_from_rfc3339("2026-03-14T08:00:00+01:00").expect("valid instant");

    let mut producer = Producer::new(
        ProducerConfig::new("MY-AGENCY").with_fetched_delivery(),
        Disruptions {
            situations: vec![lift_out_of_service(now)],
        },
    )
    .started_at(now - Duration::hours(6));
    let mut consumer = Consumer::<SituationExchange>::new("PASSENGER-APP").at_address("https://app.example/siri");

    // 1. Subscribe.
    let subscribe = consumer.subscribe(
        "lifts",
        now + Duration::hours(12),
        SituationExchangeRequest::new(now),
        now,
    );
    show("consumer → producer", &subscribe)?;

    let response = producer
        .handle(&subscribe, now)?
        .expect("a subscription request is always answered");
    show("producer → consumer", &response)?;

    match consumer.handle(&response, now)? {
        ConsumerEvent::Subscribed { outcomes } => {
            for outcome in outcomes {
                println!(
                    "  subscription {} was {}",
                    outcome.subscription_ref,
                    if outcome.accepted { "accepted" } else { "refused" }
                );
            }
        }
        other => panic!("expected a subscription outcome, got {other:?}"),
    }

    // 2. The producer owes the consumer the situations it matched. Because this
    //    producer uses fetched delivery it announces them rather than pushing.
    for outbound in producer.poll(now) {
        show("producer → consumer", &outbound.message)?;
        let ConsumerEvent::DataReady { reply, fetch } = consumer.handle(&outbound.message, now)?
        else {
            panic!("a fetched-delivery producer announces its data");
        };

        show("consumer → producer", &reply)?;
        producer.handle(&reply, now)?;

        // 3. Collect it.
        show("consumer → producer", &fetch)?;
        let delivery = producer
            .handle(&fetch, now)?
            .expect("a data supply request is always answered");
        show("producer → consumer", &delivery)?;

        if let ConsumerEvent::Delivered { items: situations, .. } = consumer.handle(&delivery, now)? {
            for situation in situations {
                println!(
                    "  {} — {}",
                    situation.situation_number,
                    situation
                        .summary
                        .first()
                        .map(|summary| summary.value.as_str())
                        .unwrap_or("(no summary)")
                );
            }
        }
    }

    // 4. Close the subscription.
    let terminate = consumer.terminate_all(now);
    show("consumer → producer", &terminate)?;
    let confirmation = producer
        .handle(&terminate, now)?
        .expect("a termination request is always answered");
    show("producer → consumer", &confirmation)?;
    consumer.handle(&confirmation, now)?;

    println!(
        "\nsubscriptions still held — producer: {}, consumer: {}",
        producer.subscriptions().len(),
        consumer.subscriptions().len()
    );
    Ok(())
}

fn show(direction: &str, message: &Siri) -> siri_rs::Result<()> {
    println!("\n=== {direction} ===\n{}", siri_rs::to_string_pretty(message)?);
    Ok(())
}

fn lift_out_of_service(now: DateTime<FixedOffset>) -> PtSituationElement {
    let mut situation = PtSituationElement::new(
        now,
        "MY-AGENCY-2026-0041",
        Source::new(SituationSourceType::DirectReport),
        HalfOpenTimestampOutputRange::between(now, now + Duration::days(2)),
        AlertCause::LiftFailure,
    );
    situation.participant_ref = Some("MY-AGENCY".into());
    situation.progress = Some(WorkflowStatus::Published);
    situation.severity = Some(Severity::Normal);
    situation.reason_name = vec![NaturalLanguageString::with_lang("EN", "Lift failure")];
    situation.summary = vec![DefaultedText::with_lang(
        "EN",
        "The lift to platform 3 is out of service",
    )];
    situation
}
