//! A SIRI-ET consumer driven over HTTP with reqwest.
//!
//! The conversation is the one `sx_consumer_reqwest` documents in full — subscribe,
//! receive on a route of one's own, unsubscribe — with a
//! `Consumer::<EstimatedTimetable>` in place of a situation one. What arrives is a
//! set of journeys, and this prints how late each of them is running.
//!
//! Run `cargo run --example et_producer_axum` in one terminal and this in another.
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

use siri_rs::et::EstimatedTimetableRequest;
use siri_rs::model::{EstimatedVehicleJourney, JourneyAlteration};
use siri_rs::pubsub::{Consumer, ConsumerEvent, EstimatedTimetable};
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
type SharedConsumer = Arc<Mutex<Consumer<EstimatedTimetable>>>;

/// What the route that receives pushes needs to do its work.
#[derive(Clone)]
struct Receiving {
    consumer: SharedConsumer,
    client: reqwest::Client,
    producer_url: String,
    delivered: mpsc::UnboundedSender<Vec<EstimatedVehicleJourney>>,
}

#[tokio::main]
async fn main() -> Result<(), Failure> {
    let producer_url = std::env::var("SIRI_PRODUCER_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8000/siri".to_owned());
    let address: SocketAddr = std::env::var("SIRI_CONSUMER_ADDRESS")
        .unwrap_or_else(|_| "127.0.0.1:8001".to_owned())
        .parse()?;

    let consumer: SharedConsumer = Arc::new(Mutex::new(
        Consumer::<EstimatedTimetable>::new("PASSENGER-APP")
            .at_address(format!("http://{address}/siri"))
            .confirming_deliveries(),
    ));
    let client = reqwest::Client::new();
    let (sender, mut delivered) = mpsc::unbounded_channel();

    let app = Router::new()
        .route(
            "/siri",
            post(receive).with_state(Receiving {
                consumer: consumer.clone(),
                client: client.clone(),
                producer_url: producer_url.clone(),
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
        "departures",
        now + Duration::hours(1),
        EstimatedTimetableRequest::new(now),
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
        let Some(journeys) = delivery else {
            return Err("the route that receives deliveries stopped".into());
        };
        batches += 1;
        println!("\n{} journey(s) delivered:", journeys.len());
        for journey in journeys {
            println!("  {}", describe(&journey));
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

/// One line about a journey: which run it is, and what is happening to it.
fn describe(journey: &EstimatedVehicleJourney) -> String {
    let line = journey.line_ref.as_str();
    match journey.alteration() {
        Some(JourneyAlteration::Cancelled) => format!("line {line}: cancelled"),
        Some(JourneyAlteration::Extra) => format!("line {line}: an extra journey"),
        // A later schema may add a kind of alteration; the crate will add the
        // variant without a major release, so the arm is expected here.
        Some(other) => format!("line {line}: {other:?}"),
        None => {
            let late = journey.estimated_calls().iter().find_map(|call| {
                let aimed = call.aimed_departure_time?;
                let expected = call.expected_departure_time?;
                Some((call.stop_point_ref.as_str(), expected - aimed))
            });
            match late {
                Some((stop, behind)) if behind.num_seconds() > 0 => format!(
                    "line {line}: {} minute(s) late leaving {stop}",
                    behind.num_minutes()
                ),
                _ => format!("line {line}: on time"),
            }
        }
    }
}

/// Receives what the producer pushes: data-ready notifications, deliveries,
/// heartbeats and the notice that ends a subscription.
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
        ConsumerEvent::DataReady { reply, fetch } => {
            // The producer is waiting for the acknowledgement, and the fetch is a new
            // request in the other direction — so it is made once this has answered.
            tokio::spawn(fetch_the_data(state.clone(), *fetch));
            answer_with(&reply)
        }
        ConsumerEvent::Delivered { items, reply, .. } => {
            let _ = state.delivered.send(items);
            match reply {
                Some(reply) => answer_with(&reply),
                None => StatusCode::NO_CONTENT.into_response(),
            }
        }
        ConsumerEvent::Alive { .. } => {
            println!("the producer is alive");
            StatusCode::NO_CONTENT.into_response()
        }
        ConsumerEvent::SubscriptionEnded { subscription_ref } => {
            println!("the producer ended subscription {subscription_ref}");
            StatusCode::NO_CONTENT.into_response()
        }
        _ => StatusCode::NO_CONTENT.into_response(),
    }
}

/// Collects the data a notification announced.
async fn fetch_the_data(state: Receiving, fetch: Siri) {
    let delivery = match request(&state.client, &state.producer_url, &fetch).await {
        Ok(delivery) => delivery,
        Err(complaint) => {
            eprintln!("! the data could not be fetched: {complaint}");
            return;
        }
    };
    match interpret(&state.consumer, &delivery) {
        Ok(ConsumerEvent::Delivered { items, reply, .. }) => {
            let _ = state.delivered.send(items);
            // A producer that asked for confirmation gets it as a fresh request, since
            // this delivery arrived as an answer rather than as a push.
            if let Some(reply) = reply {
                if let Err(complaint) = post_to(&state.client, &state.producer_url, &reply).await {
                    eprintln!("! the delivery could not be acknowledged: {complaint}");
                }
            }
        }
        Ok(other) => eprintln!("! expected a delivery, got {other:?}"),
        Err(complaint) => eprintln!("! the delivery could not be read: {complaint}"),
    }
}

/// Hands a message to the consumer and says what it meant.
fn interpret(
    consumer: &SharedConsumer,
    message: &Siri,
) -> siri_rs::Result<ConsumerEvent<EstimatedTimetable>> {
    consumer
        .lock()
        .expect("the consumer is usable")
        .handle(message, Utc::now().fixed_offset())
}

/// Posts a message to the producer and reads the message that comes back, if any: a
/// producer answers acknowledgements with silence.
async fn post_to(
    client: &reqwest::Client,
    url: &str,
    message: &Siri,
) -> Result<Option<Siri>, Failure> {
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
        return Ok(None);
    }
    Ok(Some(siri_rs::from_str(&answer)?))
}

/// Posts a message the producer is expected to answer, and reads the answer.
async fn request(client: &reqwest::Client, url: &str, message: &Siri) -> Result<Siri, Failure> {
    post_to(client, url, message)
        .await?
        .ok_or_else(|| format!("{url} answered nothing").into())
}

/// Anything that can stop the exchange.
type Failure = Box<dyn std::error::Error + Send + Sync>;

fn answer_with(message: &Siri) -> Response {
    match siri_rs::to_string(message) {
        Ok(body) => ([(header::CONTENT_TYPE, XML)], body).into_response(),
        Err(complaint) => {
            eprintln!("! an acknowledgement could not be written: {complaint}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
