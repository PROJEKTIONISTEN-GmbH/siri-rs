//! A SIRI-VM producer served over HTTP with axum.
//!
//! The wiring is the one `sx_producer_axum` documents in full. What differs is the
//! source: implementing [`VehicleMonitoringSource`] is all it takes to publish
//! vehicle positions instead of situations, and here a timer moves the vehicle every
//! few seconds so that a subscribed consumer can be watched following it.
//!
//! Run with `cargo run --example vm_producer_axum`, then point
//! `cargo run --example vm_consumer_reqwest` at it. The address to listen on can be
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

use siri_rs::enumerations::ProgressRate;
use siri_rs::model::{Location, MonitoredVehicleJourney};
use siri_rs::pubsub::{Outbound, Producer, ProducerConfig, VehicleMonitoring, VehicleMonitoringSource};
use siri_rs::vm::{VehicleActivity, VehicleMonitoringRequest};
use siri_rs::Siri;

/// The media type SIRI travels as.
const XML: &str = "application/xml";
/// The largest request body the endpoint reads. The largest official request example
/// is 3 KB; the reader bounds nesting, but only the transport can bound size.
const REQUEST_BODY_LIMIT: usize = 1024 * 1024;
/// How often the producer is asked for the messages that have become due.
const POLL_INTERVAL: StdDuration = StdDuration::from_millis(500);
/// How often the vehicle moves.
const MOVE_EVERY: StdDuration = StdDuration::from_secs(5);
/// How far east the vehicle travels per step, in degrees.
const STEP: f64 = 0.002;

/// Whatever a tracking system already keeps its vehicles in.
struct TrackedVehicles {
    vehicles: Vec<VehicleActivity>,
}

impl VehicleMonitoringSource for TrackedVehicles {
    fn vehicles(&self, request: &VehicleMonitoringRequest) -> Vec<VehicleActivity> {
        // A real source would apply every filter the request carries. This one
        // honours the vehicle filter and publishes the rest.
        match request.vehicle_ref.as_ref() {
            Some(wanted) => self
                .vehicles
                .iter()
                .filter(|activity| {
                    activity.monitored_vehicle_journey.vehicle_ref.as_ref() == Some(wanted)
                })
                .cloned()
                .collect(),
            None => self.vehicles.clone(),
        }
    }
}

/// A producer shared between the route that answers requests and the timers.
type SharedProducer = Arc<Mutex<Producer<TrackedVehicles, VehicleMonitoring>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = std::env::var("SIRI_PRODUCER_ADDRESS")
        .unwrap_or_else(|_| "127.0.0.1:8000".to_owned())
        .parse()?;
    let started = Utc::now().fixed_offset();

    let producer: SharedProducer = Arc::new(Mutex::new(
        Producer::new(
            ProducerConfig::new("MY-AGENCY"),
            TrackedVehicles {
                vehicles: vec![vehicle_at(started, 9.7411)],
            },
        )
        .started_at(started),
    ));

    tokio::spawn(send_what_is_due(producer.clone()));
    tokio::spawn(the_vehicle_moves(producer.clone()));

    let app = Router::new()
        .route("/siri", post(answer).with_state(producer))
        .layer(DefaultBodyLimit::max(REQUEST_BODY_LIMIT));
    println!("SIRI-VM endpoint listening on http://{address}/siri");
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

/// Moves the vehicle a little every few seconds.
///
/// Telling the producer that the data changed is all an application has to do:
/// every subscription then owes its consumer a fresh delivery.
async fn the_vehicle_moves(producer: SharedProducer) {
    let mut ticker = tokio::time::interval(MOVE_EVERY);
    let mut longitude = 9.7411;
    loop {
        ticker.tick().await;
        longitude += STEP;
        let now = Utc::now().fixed_offset();
        let mut producer = producer.lock().expect("the producer is usable");
        producer.source_mut().vehicles = vec![vehicle_at(now, longitude)];
        producer.data_changed();
    }
}

/// The tracked vehicle, at the given longitude.
fn vehicle_at(now: DateTime<FixedOffset>, longitude: f64) -> VehicleActivity {
    VehicleActivity {
        vehicle_monitoring_ref: Some("HANNOVER".into()),
        ..VehicleActivity::new(
            now,
            now + Duration::minutes(5),
            MonitoredVehicleJourney {
                monitored: Some(true),
                vehicle_location: Some(Location::wgs84(longitude, 52.3759)),
                bearing: Some(90.0),
                progress_rate: Some(ProgressRate::NormalProgress),
                vehicle_ref: Some("VEH-4711".into()),
                ..MonitoredVehicleJourney::on_line("10")
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
