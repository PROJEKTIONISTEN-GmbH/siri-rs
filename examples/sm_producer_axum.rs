//! A SIRI-SM producer served over HTTP with axum.
//!
//! The wiring is the one `sx_producer_axum` documents in full. What differs is the
//! source: implementing [`StopMonitoringSource`] is all it takes to publish a
//! departure board, and a producer serves both ways of asking for one — a consumer
//! that polls gets the board as it stands, a consumer that subscribes gets it again
//! whenever it changes. Here the tram loses a minute every few seconds, so a
//! subscribed consumer can be watched receiving the growing delay.
//!
//! Run with `cargo run --example sm_producer_axum`, then point
//! `cargo run --example sm_consumer_reqwest` at it. The address to listen on can be
//! set with `SIRI_PRODUCER_ADDRESS`.

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration as StdDuration;

use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use chrono::{DateTime, Duration, FixedOffset, Utc};

use siri_rs::model::{MonitoredCall, MonitoredVehicleJourney, MonitoringRef};
use siri_rs::pubsub::{Outbound, Producer, ProducerConfig, StopMonitoring, StopMonitoringSource};
use siri_rs::sm::{MonitoredStopVisit, StopMonitoringRequest};
use siri_rs::types::NaturalLanguageString;
use siri_rs::Siri;

/// The media type SIRI travels as.
const XML: &str = "application/xml";
/// How often the producer is asked for the messages that have become due.
const POLL_INTERVAL: StdDuration = StdDuration::from_millis(500);
/// How often the first tram loses another minute.
const DELAY_GROWS_EVERY: StdDuration = StdDuration::from_secs(5);
/// The stop this producer publishes a board for.
const KROEPCKE: &str = "de:03241:101";

/// Whatever a real-time system already keeps the services due at a stop in.
struct DepartureBoard {
    visits: Vec<MonitoredStopVisit>,
}

impl StopMonitoringSource for DepartureBoard {
    fn visits(&self, request: &StopMonitoringRequest) -> Vec<MonitoredStopVisit> {
        // A real source would honour every filter the request carries. This one
        // honours the monitoring point and the line, and caps the board as asked.
        let mut board: Vec<MonitoredStopVisit> = self
            .visits
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
            .collect();
        if let Some(most) = request.maximum_stop_visits {
            board.truncate(most as usize);
        }
        board
    }
}

/// A producer shared between the route that answers requests and the timers.
type SharedProducer = Arc<Mutex<Producer<DepartureBoard, StopMonitoring>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = std::env::var("SIRI_PRODUCER_ADDRESS")
        .unwrap_or_else(|_| "127.0.0.1:8000".to_owned())
        .parse()?;
    let started = Utc::now().fixed_offset();

    let producer: SharedProducer = Arc::new(Mutex::new(
        Producer::new(
            ProducerConfig::new("MY-AGENCY"),
            DepartureBoard {
                visits: board(started, Duration::zero()),
            },
        )
        .started_at(started),
    ));

    tokio::spawn(send_what_is_due(producer.clone()));
    tokio::spawn(the_tram_falls_behind(producer.clone()));

    let app = Router::new().route("/siri", post(answer).with_state(producer));
    println!("SIRI-SM endpoint listening on http://{address}/siri");
    axum::serve(tokio::net::TcpListener::bind(address).await?, app).await?;
    Ok(())
}

/// Answers a message a consumer posted — a request for the board, or a subscription
/// to it, or an acknowledgement of something already sent.
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

/// Lets the first tram lose another minute every few seconds.
///
/// Telling the producer that the data changed is all an application has to do:
/// every subscription then owes its consumer a fresh board. A consumer that polls
/// instead simply sees the delay of the moment it asked.
async fn the_tram_falls_behind(producer: SharedProducer) {
    let mut ticker = tokio::time::interval(DELAY_GROWS_EVERY);
    let mut delay = Duration::zero();
    loop {
        ticker.tick().await;
        delay += Duration::minutes(1);
        let now = Utc::now().fixed_offset();
        let mut producer = producer.lock().expect("the producer is usable");
        producer.source_mut().visits = board(now, delay);
        producer.data_changed();
    }
}

/// The board at Kröpcke: a tram running `delay` late, a bus on time, and a departure
/// from the next stop along, which the monitoring-point filter has to leave out.
fn board(now: DateTime<FixedOffset>, delay: Duration) -> Vec<MonitoredStopVisit> {
    vec![
        departure(now, KROEPCKE, "10", "Ahlem", Duration::minutes(4), delay),
        departure(
            now,
            KROEPCKE,
            "17",
            "Wallensteinstr",
            Duration::minutes(9),
            Duration::zero(),
        ),
        departure(
            now,
            "de:03241:102",
            "100",
            "Hauptbahnhof",
            Duration::minutes(6),
            Duration::zero(),
        ),
    ]
}

/// One service due at a stop, planned `in_from_now` from now and running `delay` late.
fn departure(
    now: DateTime<FixedOffset>,
    stop: &str,
    line: &str,
    destination: &str,
    in_from_now: Duration,
    delay: Duration,
) -> MonitoredStopVisit {
    let aimed = now + in_from_now;
    MonitoredStopVisit {
        monitoring_ref: Some(MonitoringRef::from(stop)),
        ..MonitoredStopVisit::new(
            now,
            MonitoredVehicleJourney {
                monitored: Some(true),
                destination_name: vec![NaturalLanguageString::new(destination)],
                monitored_call: Some(MonitoredCall {
                    stop_point_ref: Some(stop.into()),
                    aimed_departure_time: Some(aimed),
                    expected_departure_time: Some(aimed + delay),
                    ..MonitoredCall::default()
                }),
                ..MonitoredVehicleJourney::on_line(line)
            },
        )
    }
}

/// Prints a message, so that running the example shows the whole exchange.
fn show(direction: &str, message: &Siri) {
    match siri_rs::to_string_pretty(message) {
        Ok(xml) => println!("\n=== {direction} ===\n{xml}"),
        Err(complaint) => eprintln!("! a message could not be written: {complaint}"),
    }
}
