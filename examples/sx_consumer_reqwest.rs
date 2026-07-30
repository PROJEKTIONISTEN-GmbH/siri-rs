//! A SIRI-SX consumer driven over HTTP with reqwest.
//!
//! Subscribing is a conversation in both directions: the consumer posts its requests
//! to the producer, and the producer posts what it owes to an address the consumer
//! named. So a consumer needs a client *and* a route of its own — here reqwest and
//! one axum route.
//!
//! What arrives is handed to [`Consumer::handle`], which says what the message meant
//! and hands back the messages that answer it. This example follows the fetched
//! delivery path all the way through:
//!
//! 1. post a `SubscriptionRequest` and read the outcome from the answer;
//! 2. receive a `DataReadyNotification` on its own route, answer it with the
//!    acknowledgement the consumer built, and post the `DataSupplyRequest` it built;
//! 3. read the situations out of the `ServiceDelivery` that comes back;
//! 4. post a `TerminateSubscriptionRequest` and stop.
//!
//! A producer configured for direct delivery pushes the `ServiceDelivery` straight to
//! the consumer's route instead, which step 2 handles as well.
//!
//! Run `cargo run --example sx_producer_axum` in one terminal and this in another.
//! `SIRI_PRODUCER_URL` (default `http://127.0.0.1:8000/siri/fetched`) chooses the
//! endpoint to subscribe to — point it at `/siri/direct` to watch the other delivery
//! method — and `SIRI_CONSUMER_ADDRESS` (default `127.0.0.1:8001`) the address to be
//! reached at.

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration as StdDuration;

use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use chrono::{Duration, Utc};
use tokio::sync::mpsc;

use siri::pubsub::{Consumer, ConsumerEvent};
use siri::sx::{PtSituationElement, SituationExchangeRequest};
use siri::Siri;

/// The media type SIRI travels as.
const XML: &str = "application/xml";
/// How long to stay subscribed and print what arrives before unsubscribing.
const LISTEN_FOR: StdDuration = StdDuration::from_secs(20);

/// A consumer shared between the route that receives pushes and the code that posts
/// requests.
type SharedConsumer = Arc<Mutex<Consumer>>;

/// What the route that receives pushes needs to do its work.
#[derive(Clone)]
struct Receiving {
    consumer: SharedConsumer,
    client: reqwest::Client,
    producer_url: String,
    delivered: mpsc::UnboundedSender<Vec<PtSituationElement>>,
}

#[tokio::main]
async fn main() -> Result<(), Failure> {
    let producer_url = std::env::var("SIRI_PRODUCER_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8000/siri/fetched".to_owned());
    let address: SocketAddr = std::env::var("SIRI_CONSUMER_ADDRESS")
        .unwrap_or_else(|_| "127.0.0.1:8001".to_owned())
        .parse()?;

    let consumer: SharedConsumer = Arc::new(Mutex::new(
        Consumer::new("PASSENGER-APP")
            .at_address(format!("http://{address}/siri"))
            .confirming_deliveries(),
    ));
    let client = reqwest::Client::new();
    let (sender, mut delivered) = mpsc::unbounded_channel();

    let app = Router::new().route(
        "/siri",
        post(receive).with_state(Receiving {
            consumer: consumer.clone(),
            client: client.clone(),
            producer_url: producer_url.clone(),
            delivered: sender,
        }),
    );
    let listener = tokio::net::TcpListener::bind(address).await?;
    tokio::spawn(async move {
        if let Err(complaint) = axum::serve(listener, app).await {
            eprintln!("! the consumer stopped listening: {complaint}");
        }
    });
    println!("listening for deliveries on http://{address}/siri");

    // 1. Subscribe. The answer says whether the producer accepted.
    let now = Utc::now().fixed_offset();
    let subscribe = consumer.lock().expect("the consumer is usable").subscribe(
        "disruptions",
        now + Duration::hours(1),
        SituationExchangeRequest::new(now),
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

    // 2. and 3. happen on the route above; the situations it collects arrive here, once
    // when the subscription opens and again whenever the producer's situations change.
    let listening_until = tokio::time::Instant::now() + LISTEN_FOR;
    let mut batches = 0;
    while let Ok(delivery) = tokio::time::timeout_at(listening_until, delivered.recv()).await {
        let Some(situations) = delivery else {
            return Err("the route that receives deliveries stopped".into());
        };
        batches += 1;
        println!("\n{} situation(s) delivered:", situations.len());
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
    if batches == 0 {
        println!("\nthe producer delivered nothing within {LISTEN_FOR:?}");
    }

    // 4. Unsubscribe, and stop once the producer has confirmed.
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

/// Receives what the producer pushes: data-ready notifications, deliveries,
/// heartbeats and the notice that ends a subscription.
///
/// The acknowledgement the consumer builds is returned as the response body, which is
/// how a producer that pushed a message gets an answer to it.
async fn receive(State(state): State<Receiving>, body: String) -> Response {
    let message: Siri = match siri::from_str(&body) {
        Ok(message) => message,
        Err(complaint) => {
            return (StatusCode::BAD_REQUEST, format!("unreadable SIRI: {complaint}\n"))
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
        ConsumerEvent::Delivered { situations, reply } => {
            let _ = state.delivered.send(situations);
            match reply {
                Some(reply) => answer_with(&reply),
                None => StatusCode::NO_CONTENT.into_response(),
            }
        }
        ConsumerEvent::Alive {
            service_started_time,
        } => {
            println!(
                "the producer is alive, serving since {}",
                service_started_time
                    .map(|started| started.to_rfc3339())
                    .unwrap_or_else(|| "an unstated time".to_owned())
            );
            StatusCode::NO_CONTENT.into_response()
        }
        ConsumerEvent::SubscriptionEnded { subscription_ref } => {
            println!("the producer ended subscription {subscription_ref}");
            StatusCode::NO_CONTENT.into_response()
        }
        _ => StatusCode::NO_CONTENT.into_response(),
    }
}

/// Collects the data a notification announced, and reports the situations it carried.
async fn fetch_the_data(state: Receiving, fetch: Siri) {
    let delivery = match request(&state.client, &state.producer_url, &fetch).await {
        Ok(delivery) => delivery,
        Err(complaint) => {
            eprintln!("! the data could not be fetched: {complaint}");
            return;
        }
    };
    match interpret(&state.consumer, &delivery) {
        Ok(ConsumerEvent::Delivered { situations, reply }) => {
            let _ = state.delivered.send(situations);
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
fn interpret(consumer: &SharedConsumer, message: &Siri) -> siri::Result<ConsumerEvent> {
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
    let body = siri::to_string(message)?;
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
    Ok(Some(siri::from_str(&answer)?))
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
    match siri::to_string(message) {
        Ok(body) => ([(header::CONTENT_TYPE, XML)], body).into_response(),
        Err(complaint) => {
            eprintln!("! an acknowledgement could not be written: {complaint}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
