//! A SIRI-SX producer served over HTTP with axum.
//!
//! The library knows nothing about HTTP, so running an endpoint is a matter of
//! deciding two things:
//!
//! * which route feeds incoming messages to [`Producer::handle`], and what to do
//!   with the reply it hands back — here a POST route, answering in the response;
//! * what drives [`Producer::poll`], and where the messages it hands back are sent —
//!   here a timer, posting to the address each consumer named when it subscribed.
//!
//! Everything else — subscription bookkeeping, when a delivery is owed, when a
//! heartbeat is due — is the producer's business.
//!
//! Two routes are served so that both delivery methods are visible in one run. They
//! hold the same situations and differ only in configuration:
//!
//! | Route | Delivery |
//! |---|---|
//! | `/siri/direct` | the delivery is pushed to the consumer as soon as it is owed |
//! | `/siri/fetched` | the consumer is told data is ready and fetches it |
//!
//! Run with `cargo run --example sx_producer_axum`, then point
//! `cargo run --example sx_consumer_reqwest` at one of the routes. The address to
//! listen on can be set with `SIRI_PRODUCER_ADDRESS`.

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration as StdDuration;

use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use chrono::{DateTime, Duration, FixedOffset, Utc};

use siri_rs::enumerations::{AlertCause, Severity, SituationSourceType, WorkflowStatus};
use siri_rs::pubsub::{Outbound, Producer, ProducerConfig, SituationExchange, SituationSource};
use siri_rs::sx::situation::SituationSource as Source;
use siri_rs::sx::{PtSituationElement, SituationExchangeRequest};
use siri_rs::types::{DefaultedText, Duration as SiriDuration, HalfOpenTimestampOutputRange};
use siri_rs::Siri;

/// The media type SIRI travels as.
const XML: &str = "application/xml";
/// How often the producer is asked for the messages that have become due.
const POLL_INTERVAL: StdDuration = StdDuration::from_millis(500);
/// How long the endpoint runs before it publishes a further situation, so that a
/// subscribed consumer can be seen receiving an update.
const UPDATE_AFTER: StdDuration = StdDuration::from_secs(10);

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
            .filter(|situation| match (request.severity, situation.severity) {
                (Some(wanted), Some(actual)) => actual >= wanted,
                (Some(_), None) => false,
                (None, _) => true,
            })
            .cloned()
            .collect()
    }
}

/// A producer shared between the route that answers requests and the timer that
/// sends what is due.
type SharedProducer = Arc<Mutex<Producer<Disruptions, SituationExchange>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = std::env::var("SIRI_PRODUCER_ADDRESS")
        .unwrap_or_else(|_| "127.0.0.1:8000".to_owned())
        .parse()?;
    let started = Utc::now().fixed_offset();
    let heartbeat = SiriDuration::parse("PT15S")?;

    let direct = producer(
        ProducerConfig::new("MY-AGENCY").with_heartbeat(heartbeat.clone()),
        started,
    );
    let fetched = producer(
        ProducerConfig::new("MY-AGENCY")
            .with_fetched_delivery()
            .with_heartbeat(heartbeat),
        started,
    );

    for shared in [direct.clone(), fetched.clone()] {
        tokio::spawn(send_what_is_due(shared));
    }
    tokio::spawn(publish_an_update([direct.clone(), fetched.clone()]));

    let app = Router::new()
        .route("/siri/direct", post(answer).with_state(direct))
        .route("/siri/fetched", post(answer).with_state(fetched));

    println!("SIRI-SX endpoint listening on http://{address}");
    println!("  POST http://{address}/siri/direct   — deliveries are pushed");
    println!("  POST http://{address}/siri/fetched  — deliveries are announced, then fetched");
    axum::serve(tokio::net::TcpListener::bind(address).await?, app).await?;
    Ok(())
}

fn producer(config: ProducerConfig, started: DateTime<FixedOffset>) -> SharedProducer {
    let source = Disruptions {
        situations: vec![lift_out_of_service(started)],
    };
    Arc::new(Mutex::new(
        Producer::new(config, source).started_at(started),
    ))
}

/// Answers a message a consumer posted.
///
/// Subscription requests, termination requests, data supply requests, check-status
/// requests and plain service requests all arrive here; the producer decides which
/// of them is answered, and with what.
async fn answer(State(producer): State<SharedProducer>, body: String) -> Response {
    let now = Utc::now().fixed_offset();
    let message: Siri = match siri_rs::from_str(&body) {
        Ok(message) => message,
        Err(complaint) => {
            return (
                StatusCode::BAD_REQUEST,
                format!("unreadable SIRI: {complaint}\n"),
            )
                .into_response()
        }
    };
    show("consumer → producer", &message);

    let reply = producer
        .lock()
        .expect("the producer is usable")
        .handle(&message, now);
    match reply {
        Ok(Some(reply)) => match siri_rs::to_string(&reply) {
            Ok(body) => {
                show("producer → consumer", &reply);
                ([(header::CONTENT_TYPE, XML)], body).into_response()
            }
            Err(complaint) => {
                eprintln!("! a reply could not be written: {complaint}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        },
        // Acknowledgements are answered by silence.
        Ok(None) => StatusCode::NO_CONTENT.into_response(),
        Err(complaint) => (
            StatusCode::BAD_REQUEST,
            format!("cannot answer that message: {complaint}\n"),
        )
            .into_response(),
    }
}

/// Sends the messages the producer says are due: deliveries, data-ready
/// notifications, heartbeats and the notices that end a lapsed subscription.
async fn send_what_is_due(producer: SharedProducer) {
    let client = reqwest::Client::new();
    let mut ticker = tokio::time::interval(POLL_INTERVAL);
    loop {
        ticker.tick().await;
        let now = Utc::now().fixed_offset();
        // The lock is released before anything is sent: a consumer that is slow to
        // answer must not hold up the route that answers its next request.
        let due = producer.lock().expect("the producer is usable").poll(now);
        for outbound in due {
            send(&producer, &client, &outbound).await;
        }
    }
}

/// Sends one message to the address its recipient asked to be reached at, and feeds
/// the acknowledgement that comes back to the producer.
async fn send(producer: &SharedProducer, client: &reqwest::Client, outbound: &Outbound) {
    let Some(address) = outbound.address.as_ref() else {
        eprintln!(
            "! {} named no address, so nothing can be sent to it",
            outbound.recipient
        );
        return;
    };
    let body = match siri_rs::to_string(&outbound.message) {
        Ok(body) => body,
        Err(complaint) => {
            eprintln!("! a message could not be written: {complaint}");
            return;
        }
    };

    show(&format!("producer → {address}"), &outbound.message);
    let answer = client
        .post(address.as_str())
        .header(header::CONTENT_TYPE, XML)
        .body(body)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status);
    let answer = match answer {
        Ok(answer) => answer.text().await.unwrap_or_default(),
        Err(complaint) => {
            eprintln!("! {address} did not accept the message: {complaint}");
            return;
        }
    };
    if answer.trim().is_empty() {
        return;
    }

    // A consumer acknowledges a push in the response body. The producer wants to see
    // it, even though it answers acknowledgements with silence.
    match siri_rs::from_str::<Siri>(&answer) {
        Ok(acknowledgement) => {
            show(&format!("{address} → producer"), &acknowledgement);
            let now = Utc::now().fixed_offset();
            if let Err(complaint) = producer
                .lock()
                .expect("the producer is usable")
                .handle(&acknowledgement, now)
            {
                eprintln!("! the acknowledgement made no sense: {complaint}");
            }
        }
        Err(complaint) => eprintln!("! unreadable acknowledgement from {address}: {complaint}"),
    }
}

/// Publishes a further situation once the endpoint has been running for a while.
///
/// Telling the producer that the situations changed is all an application has to do:
/// every subscription then owes its consumer a fresh delivery, which the timer above
/// sends.
async fn publish_an_update(producers: [SharedProducer; 2]) {
    tokio::time::sleep(UPDATE_AFTER).await;
    let now = Utc::now().fixed_offset();
    println!("* a new situation was reported");
    for producer in producers {
        let mut producer = producer.lock().expect("the producer is usable");
        producer.source_mut().situations.push(bridge_works(now));
        producer.data_changed();
    }
}

/// Prints a message, so that running the example shows the whole exchange.
fn show(direction: &str, message: &Siri) {
    match siri_rs::to_string_pretty(message) {
        Ok(xml) => println!("\n=== {direction} ===\n{xml}"),
        Err(complaint) => eprintln!("! a message could not be written: {complaint}"),
    }
}

fn lift_out_of_service(now: DateTime<FixedOffset>) -> PtSituationElement {
    situation(
        now,
        "MY-AGENCY-2026-0041",
        AlertCause::LiftFailure,
        Severity::Normal,
        "The lift to platform 3 is out of service",
        "Passengers needing step-free access should use the ramp at the western entrance.",
    )
}

fn bridge_works(now: DateTime<FixedOffset>) -> PtSituationElement {
    situation(
        now,
        "MY-AGENCY-2026-0042",
        AlertCause::ConstructionWork,
        Severity::Severe,
        "Line 10 is diverted while the bridge is rebuilt",
        "Services run via the eastern loop and do not call at the two stops before the bridge.",
    )
}

fn situation(
    now: DateTime<FixedOffset>,
    number: &str,
    cause: AlertCause,
    severity: Severity,
    summary: &str,
    description: &str,
) -> PtSituationElement {
    let mut situation = PtSituationElement::new(
        now,
        number,
        Source::new(SituationSourceType::DirectReport),
        HalfOpenTimestampOutputRange::between(now, now + Duration::days(2)),
        cause,
    );
    situation.participant_ref = Some("MY-AGENCY".into());
    situation.progress = Some(WorkflowStatus::Published);
    situation.severity = Some(severity);
    situation.summary = vec![DefaultedText::with_lang("EN", summary)];
    situation.description = vec![DefaultedText::with_lang("EN", description)];
    situation
}
