//! A SIRI-ET producer served over HTTP with axum.
//!
//! The wiring is the same as in `sx_producer_axum`, which documents it in full: one
//! route feeds incoming messages to [`Producer::handle`], one timer sends what
//! [`Producer::poll`] says is due. What differs is the source. Implementing
//! [`EstimatedTimetableSource`] is all it takes to make the producer a real-time
//! timetable producer instead of a situation one; the subscription state machine
//! underneath is the same code.
//!
//! Run with `cargo run --example et_producer_axum`, then point
//! `cargo run --example et_consumer_reqwest` at it. The address to listen on can be
//! set with `SIRI_PRODUCER_ADDRESS`.

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration as StdDuration;

use axum::extract::{DefaultBodyLimit, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use chrono::{DateTime, Duration, FixedOffset, Utc};

use siri_rs::et::EstimatedTimetableRequest;
use siri_rs::model::{EstimatedCall, EstimatedVehicleJourney};
use siri_rs::pubsub::{
    EstimatedTimetable, EstimatedTimetableSource, Outbound, Producer, ProducerConfig,
};
use siri_rs::types::Duration as SiriDuration;
use siri_rs::Siri;

/// The media type SIRI travels as.
const XML: &str = "application/xml";
/// The largest request body the endpoint reads. The largest official request example
/// is 3 KB; the reader bounds nesting, but only the transport can bound size.
const REQUEST_BODY_LIMIT: usize = 1024 * 1024;
/// How often the producer is asked for the messages that have become due.
const POLL_INTERVAL: StdDuration = StdDuration::from_millis(500);
/// How long the endpoint runs before the delay grows, so that a subscribed consumer
/// can be seen receiving an update.
const UPDATE_AFTER: StdDuration = StdDuration::from_secs(10);

/// Whatever an operator already keeps its running journeys in.
struct RunningJourneys {
    journeys: Vec<EstimatedVehicleJourney>,
}

impl EstimatedTimetableSource for RunningJourneys {
    fn journeys(&self, request: &EstimatedTimetableRequest) -> Vec<EstimatedVehicleJourney> {
        // A real source would apply every filter the request carries. This one
        // honours the line filter and publishes the rest.
        let wanted: Vec<&str> = request
            .line_directions()
            .iter()
            .map(|line| line.line_ref.as_str())
            .collect();
        self.journeys
            .iter()
            .filter(|journey| wanted.is_empty() || wanted.contains(&journey.line_ref.as_str()))
            .cloned()
            .collect()
    }
}

/// A producer shared between the route that answers requests and the timer that
/// sends what is due.
type SharedProducer = Arc<Mutex<Producer<RunningJourneys, EstimatedTimetable>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = std::env::var("SIRI_PRODUCER_ADDRESS")
        .unwrap_or_else(|_| "127.0.0.1:8000".to_owned())
        .parse()?;
    let started = Utc::now().fixed_offset();

    let producer: SharedProducer = Arc::new(Mutex::new(
        Producer::new(
            ProducerConfig::new("MY-AGENCY").with_heartbeat(SiriDuration::parse("PT15S")?),
            RunningJourneys {
                journeys: vec![running_late(started, Duration::minutes(3)), cancelled()],
            },
        )
        .started_at(started),
    ));

    tokio::spawn(send_what_is_due(producer.clone()));
    tokio::spawn(the_delay_grows(producer.clone(), started));

    let app = Router::new()
        .route("/siri", post(answer).with_state(producer))
        .layer(DefaultBodyLimit::max(REQUEST_BODY_LIMIT));
    println!("SIRI-ET endpoint listening on http://{address}/siri");
    axum::serve(tokio::net::TcpListener::bind(address).await?, app).await?;
    Ok(())
}

/// Answers a message a consumer posted.
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

/// Sends the messages the producer says are due.
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
            send(&client, &outbound).await;
        }
    }
}

/// Sends one message to the address its recipient asked to be reached at.
async fn send(client: &reqwest::Client, outbound: &Outbound) {
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
    if let Err(complaint) = client
        .post(address.as_str())
        .header(header::CONTENT_TYPE, XML)
        .body(body)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
    {
        eprintln!("! {address} did not accept the message: {complaint}");
    }
}

/// Reports a larger delay once the endpoint has been running for a while.
///
/// Telling the producer that the data changed is all an application has to do:
/// every subscription then owes its consumer a fresh delivery.
async fn the_delay_grows(producer: SharedProducer, started: DateTime<FixedOffset>) {
    tokio::time::sleep(UPDATE_AFTER).await;
    println!("* line 10 has fallen further behind");
    let mut producer = producer.lock().expect("the producer is usable");
    producer.source_mut().journeys[0] = running_late(started, Duration::minutes(7));
    producer.data_changed();
}

/// A journey on line 10 that is running `delay` behind its timetable.
fn running_late(now: DateTime<FixedOffset>, delay: Duration) -> EstimatedVehicleJourney {
    let aimed = now + Duration::minutes(5);
    EstimatedVehicleJourney {
        recorded_at_time: Some(now),
        monitored: Some(true),
        delay: Some(SiriDuration::from_secs(delay.num_seconds().unsigned_abs())),
        ..EstimatedVehicleJourney::dated("10", "OUT", "10-0815").with_estimated_calls(vec![
            EstimatedCall {
                aimed_departure_time: Some(aimed),
                expected_departure_time: Some(aimed + delay),
                ..EstimatedCall::at("de:03241:101")
            },
            EstimatedCall {
                aimed_arrival_time: Some(aimed + Duration::minutes(4)),
                expected_arrival_time: Some(aimed + Duration::minutes(4) + delay),
                ..EstimatedCall::at("de:03241:102")
            },
        ])
    }
}

/// A journey that will not run at all today.
fn cancelled() -> EstimatedVehicleJourney {
    EstimatedVehicleJourney::dated("10", "OUT", "10-0845").cancelled()
}

/// Prints a message, so that running the example shows the whole exchange.
fn show(direction: &str, message: &Siri) {
    match siri_rs::to_string_pretty(message) {
        Ok(xml) => println!("\n=== {direction} ===\n{xml}"),
        Err(complaint) => eprintln!("! a message could not be written: {complaint}"),
    }
}
