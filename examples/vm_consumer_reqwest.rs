//! A SIRI-VM consumer driven over HTTP with reqwest.
//!
//! The conversation is the one `sx_consumer_reqwest` documents in full, with a
//! `Consumer::<VehicleMonitoring>` in place of a situation one. What arrives is a
//! set of vehicles, and this prints where each of them is.
//!
//! Run `cargo run --example vm_producer_axum` in one terminal and this in another.
//! `SIRI_PRODUCER_URL` (default `http://127.0.0.1:8000/siri`) chooses the endpoint
//! to subscribe to, `SIRI_CONSUMER_ADDRESS` (default `127.0.0.1:8001`) the address
//! to be reached at.

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

use siri_rs::model::{Location, Position};
use siri_rs::pubsub::{Consumer, ConsumerEvent, VehicleMonitoring};
use siri_rs::vm::{VehicleActivity, VehicleMonitoringRequest};
use siri_rs::Siri;

/// The media type SIRI travels as.
const XML: &str = "application/xml";
/// The largest delivery body the endpoint reads: sized to the largest delivery the
/// producer is expected to send. The reader bounds nesting, but only the transport
/// can bound size.
const DELIVERY_BODY_LIMIT: usize = 16 * 1024 * 1024;
/// How long to stay subscribed and print what arrives before unsubscribing.
const LISTEN_FOR: StdDuration = StdDuration::from_secs(20);

/// A consumer shared between the route that receives pushes and the code that posts
/// requests.
type SharedConsumer = Arc<Mutex<Consumer<VehicleMonitoring>>>;

/// What the route that receives pushes needs to do its work.
#[derive(Clone)]
struct Receiving {
    consumer: SharedConsumer,
    delivered: mpsc::UnboundedSender<Vec<VehicleActivity>>,
}

#[tokio::main]
async fn main() -> Result<(), Failure> {
    let producer_url = std::env::var("SIRI_PRODUCER_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8000/siri".to_owned());
    let address: SocketAddr = std::env::var("SIRI_CONSUMER_ADDRESS")
        .unwrap_or_else(|_| "127.0.0.1:8001".to_owned())
        .parse()?;

    let consumer: SharedConsumer = Arc::new(Mutex::new(
        Consumer::<VehicleMonitoring>::new("MAP").at_address(format!("http://{address}/siri")),
    ));
    let client = reqwest::Client::new();
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
    println!("listening for deliveries on http://{address}/siri");

    let now = Utc::now().fixed_offset();
    let subscribe = consumer.lock().expect("the consumer is usable").subscribe(
        "fleet",
        now + Duration::hours(1),
        VehicleMonitoringRequest::new(now),
        now,
    );
    let response = request(&client, &producer_url, &subscribe).await?;
    match interpret(&consumer, &response)? {
        ConsumerEvent::Subscribed { outcomes } => {
            for outcome in outcomes {
                println!(
                    "subscription {} was {}",
                    outcome.subscription_ref,
                    if outcome.accepted { "accepted" } else { "refused" }
                );
            }
        }
        other => return Err(format!("expected a subscription outcome, got {other:?}").into()),
    }

    let listening_until = tokio::time::Instant::now() + LISTEN_FOR;
    let mut batches = 0;
    while let Ok(delivery) = tokio::time::timeout_at(listening_until, delivered.recv()).await {
        let Some(vehicles) = delivery else {
            return Err("the route that receives deliveries stopped".into());
        };
        batches += 1;
        for vehicle in vehicles {
            println!("  {}", describe(&vehicle));
        }
    }
    if batches == 0 {
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

/// One line about a vehicle: which one it is and where it is.
fn describe(activity: &VehicleActivity) -> String {
    let journey = &activity.monitored_vehicle_journey;
    let vehicle = journey
        .vehicle_ref
        .as_ref()
        .map(|reference| reference.as_str())
        .unwrap_or("(unnamed vehicle)");
    match journey.vehicle_location.as_ref().and_then(Location::position) {
        Some(Position::Wgs84 {
            longitude,
            latitude,
            ..
        }) => format!(
            "{vehicle} at {latitude:.4}, {longitude:.4} (recorded {})",
            activity.recorded_at_time.to_rfc3339()
        ),
        Some(Position::Coordinates(coordinates)) => format!("{vehicle} at {coordinates}"),
        None => format!("{vehicle}, position unknown"),
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
) -> siri_rs::Result<ConsumerEvent<VehicleMonitoring>> {
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
