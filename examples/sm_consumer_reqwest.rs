//! A SIRI-SM consumer driven over HTTP with reqwest, both ways round.
//!
//! Stop Monitoring is where SIRI's two interaction patterns are both worth having,
//! so this example uses each in turn:
//!
//! 1. **Polling** — one `ServiceRequest`, one `ServiceDelivery`, the board as it
//!    stands. This is what a departure display does before it redraws itself: no
//!    subscription, no state, no address to be reached at.
//! 2. **Subscribing** — the board arrives again whenever the producer says it has
//!    changed, until the subscription is closed. This is what a system keeping its
//!    own copy of the board does.
//!
//! Run `cargo run --example sm_producer_axum` in one terminal and this in another.
//! `SIRI_PRODUCER_URL` (default `http://127.0.0.1:8000/siri`) chooses the endpoint,
//! `SIRI_CONSUMER_ADDRESS` (default `127.0.0.1:8001`) the address to be reached at
//! while subscribed.

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration as StdDuration;

use axum::extract::{DefaultBodyLimit, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use chrono::{Duration, Utc};
use tokio::sync::mpsc;

use siri_rs::pubsub::{Consumer, ConsumerEvent, StopMonitoring};
use siri_rs::sm::{MonitoredStopVisit, StopMonitoringRequest};
use siri_rs::Siri;

/// The media type SIRI travels as.
const XML: &str = "application/xml";
/// The largest delivery body the endpoint reads: sized to the largest delivery the
/// producer is expected to send. The reader bounds nesting, but only the transport
/// can bound size.
const DELIVERY_BODY_LIMIT: usize = 16 * 1024 * 1024;
/// How long to stay subscribed and print what arrives before unsubscribing.
const LISTEN_FOR: StdDuration = StdDuration::from_secs(20);
/// The stop to ask about.
const KROEPCKE: &str = "de:03241:101";

/// A consumer shared between the route that receives pushes and the code that posts
/// requests.
type SharedConsumer = Arc<Mutex<Consumer<StopMonitoring>>>;

/// What the route that receives pushes needs to do its work.
#[derive(Clone)]
struct Receiving {
    consumer: SharedConsumer,
    delivered: mpsc::UnboundedSender<Vec<MonitoredStopVisit>>,
}

#[tokio::main]
async fn main() -> Result<(), Failure> {
    let producer_url = std::env::var("SIRI_PRODUCER_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8000/siri".to_owned());
    let address: SocketAddr = std::env::var("SIRI_CONSUMER_ADDRESS")
        .unwrap_or_else(|_| "127.0.0.1:8001".to_owned())
        .parse()?;
    let client = reqwest::Client::new();

    // 1. Polling. A display that only wants to redraw itself needs nothing else.
    let consumer: SharedConsumer = Arc::new(Mutex::new(Consumer::<StopMonitoring>::new("DISPLAY")));
    let now = Utc::now().fixed_offset();
    let poll = consumer.lock().expect("the consumer is usable").request(
        StopMonitoringRequest {
            maximum_stop_visits: Some(4),
            ..StopMonitoringRequest::at_stop(now, KROEPCKE)
        },
        now,
    );
    let delivery = request(&client, &producer_url, &poll).await?;
    match interpret(&consumer, &delivery)? {
        ConsumerEvent::Delivered { items, .. } => {
            println!("the board at {KROEPCKE}, as it stands:");
            print_board(&items);
        }
        other => return Err(format!("expected a delivery, got {other:?}").into()),
    }

    // 2. Subscribing. The board now needs an address to be delivered to, so this
    //    consumer serves a route of its own.
    let consumer: SharedConsumer = Arc::new(Mutex::new(
        Consumer::<StopMonitoring>::new("PASSENGER-APP")
            .at_address(format!("http://{address}/siri")),
    ));
    let (sender, mut delivered) = mpsc::unbounded_channel();
    let app = Router::new()
        .route(
            "/siri",
            post(receive).with_state(Receiving {
                consumer: consumer.clone(),
                delivered: sender,
            }),
        )
        .layer(DefaultBodyLimit::max(DELIVERY_BODY_LIMIT));
    let listener = tokio::net::TcpListener::bind(address).await?;
    tokio::spawn(async move {
        if let Err(complaint) = axum::serve(listener, app).await {
            eprintln!("! the consumer stopped listening: {complaint}");
        }
    });
    println!("\nlistening for deliveries on http://{address}/siri");

    let now = Utc::now().fixed_offset();
    let subscribe = consumer.lock().expect("the consumer is usable").subscribe(
        "departures",
        now + Duration::hours(1),
        StopMonitoringRequest::at_stop(now, KROEPCKE),
        now,
    );
    let response = request(&client, &producer_url, &subscribe).await?;
    match interpret(&consumer, &response)? {
        ConsumerEvent::Subscribed { outcomes } => {
            for outcome in outcomes {
                println!(
                    "subscription {} was {}",
                    outcome.subscription_ref,
                    if outcome.accepted {
                        "accepted"
                    } else {
                        "refused"
                    }
                );
            }
        }
        other => return Err(format!("expected a subscription outcome, got {other:?}").into()),
    }

    let listening_until = tokio::time::Instant::now() + LISTEN_FOR;
    let mut boards = 0;
    while let Ok(delivery) = tokio::time::timeout_at(listening_until, delivered.recv()).await {
        let Some(visits) = delivery else {
            return Err("the route that receives deliveries stopped".into());
        };
        boards += 1;
        println!("the board changed:");
        print_board(&visits);
    }
    if boards == 0 {
        println!("\nthe producer delivered nothing within {LISTEN_FOR:?}");
    }

    let now = Utc::now().fixed_offset();
    let terminate = consumer
        .lock()
        .expect("the consumer is usable")
        .terminate_all(now);
    let confirmation = request(&client, &producer_url, &terminate).await?;
    match interpret(&consumer, &confirmation)? {
        ConsumerEvent::Terminated { subscription_refs } => {
            for subscription_ref in subscription_refs {
                println!("subscription {subscription_ref} was closed");
            }
        }
        other => return Err(format!("expected a termination confirmation, got {other:?}").into()),
    }
    Ok(())
}

/// Prints the board the way a display would show it.
fn print_board(visits: &[MonitoredStopVisit]) {
    if visits.is_empty() {
        println!("  (nothing due)");
    }
    for visit in visits {
        println!("  {}", describe(visit));
    }
}

/// One line about a service due: which line, where to, when, and how late.
fn describe(visit: &MonitoredStopVisit) -> String {
    let journey = &visit.monitored_vehicle_journey;
    let line = journey
        .line_ref
        .as_ref()
        .map(|reference| reference.as_str())
        .unwrap_or("(unnamed line)");
    let destination = journey
        .destination_name
        .first()
        .map(|name| name.value.as_str())
        .unwrap_or("(unknown destination)");
    let Some(call) = journey.monitored_call.as_ref() else {
        return format!("{line} to {destination}, no departure time given");
    };
    match (call.aimed_departure_time, call.expected_departure_time) {
        (Some(aimed), Some(expected)) if expected > aimed => format!(
            "{line} to {destination} at {}, {} min late",
            expected.time().format("%H:%M"),
            (expected - aimed).num_minutes()
        ),
        (_, Some(expected)) => format!(
            "{line} to {destination} at {}, on time",
            expected.time().format("%H:%M")
        ),
        (Some(aimed), None) => format!(
            "{line} to {destination} at {} (timetabled)",
            aimed.time().format("%H:%M")
        ),
        (None, None) => format!("{line} to {destination}, no departure time given"),
    }
}

/// Receives what the producer pushes: deliveries, heartbeats and the notice that
/// ends a subscription.
async fn receive(State(state): State<Receiving>, body: String) -> Response {
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

    let event = match interpret(&state.consumer, &message) {
        Ok(event) => event,
        Err(complaint) => {
            return (
                StatusCode::BAD_REQUEST,
                format!("cannot interpret that message: {complaint}\n"),
            )
                .into_response()
        }
    };
    match event {
        ConsumerEvent::Delivered { items, .. } => {
            let _ = state.delivered.send(items);
            StatusCode::NO_CONTENT.into_response()
        }
        ConsumerEvent::SubscriptionEnded { subscription_ref } => {
            println!("the producer ended subscription {subscription_ref}");
            StatusCode::NO_CONTENT.into_response()
        }
        _ => StatusCode::NO_CONTENT.into_response(),
    }
}

/// Hands a message to the consumer and says what it meant.
fn interpret(
    consumer: &SharedConsumer,
    message: &Siri,
) -> siri_rs::Result<ConsumerEvent<StopMonitoring>> {
    consumer
        .lock()
        .expect("the consumer is usable")
        .handle(message, Utc::now().fixed_offset())
}

/// Posts a message the producer is expected to answer, and reads the answer.
async fn request(client: &reqwest::Client, url: &str, message: &Siri) -> Result<Siri, Failure> {
    let body = siri_rs::to_string(message)?;
    let answer = client
        .post(url)
        .header(header::CONTENT_TYPE, XML)
        .body(body)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    if answer.trim().is_empty() {
        return Err(format!("{url} answered nothing").into());
    }
    Ok(siri_rs::from_str(&answer)?)
}

/// Anything that can stop the exchange.
type Failure = Box<dyn std::error::Error + Send + Sync>;
