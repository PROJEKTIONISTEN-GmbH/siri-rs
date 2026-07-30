//! The publish/subscribe hub carrying the journey services over real HTTP.
//!
//! `tests/http_endpoint.rs` drives the same machinery for Situation Exchange. This
//! one does it for the two real-time journey services, which is where the hub's
//! move from one service to any of them is actually put to the test: the state
//! machines are the same code, only the [`Service`] differs. As there, every body
//! that crosses the wire is validated against the official schemas as it passes.

mod support;

use std::sync::{Arc, Mutex};
use std::time::Duration as StdDuration;

use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use chrono::{DateTime, Duration, FixedOffset, Utc};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use siri_rs::et::EstimatedTimetableRequest;
use siri_rs::model::{
    EstimatedCall, EstimatedVehicleJourney, Location, MonitoredVehicleJourney, Position,
};
use siri_rs::pubsub::{
    Consumer, ConsumerEvent, EstimatedTimetable, EstimatedTimetableSource, Outbound, Producer,
    ProducerConfig, Service, Source, VehicleMonitoring, VehicleMonitoringSource,
};
use siri_rs::vm::{VehicleActivity, VehicleMonitoringRequest};
use siri_rs::Siri;
use support::{validate, validator_available, VALIDATOR_MISSING};

/// The media type SIRI travels as.
const XML: &str = "application/xml";
/// How often the producer looks for messages that have become due.
const POLL_INTERVAL: StdDuration = StdDuration::from_millis(20);
/// How long a test waits for a message it expects before giving up.
const PATIENCE: StdDuration = StdDuration::from_secs(10);

#[tokio::test]
async fn an_estimated_timetable_subscription_runs_its_full_cycle_over_http() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let wire = Wire::default();
    let mut consumer =
        ConsumerEndpoint::start(Consumer::<EstimatedTimetable>::new("PASSENGER-APP"), wire.clone())
            .await;
    let producer = ProducerEndpoint::start(
        ProducerConfig::new("MY-AGENCY").with_fetched_delivery(),
        RunningJourneys(vec![delayed_journey(), cancelled_journey()]),
        wire.clone(),
    )
    .await;

    let now = Utc::now().fixed_offset();
    let subscribe = consumer.subscribe(
        "departures",
        now + Duration::hours(1),
        EstimatedTimetableRequest::new(now),
        now,
    );
    let response = consumer.post(&producer.url, &subscribe).await;
    let ConsumerEvent::Subscribed { outcomes } = consumer.interpret(&response) else {
        panic!("a subscription request is answered with a subscription outcome");
    };
    assert_eq!(outcomes.len(), 1);
    assert!(outcomes[0].accepted);

    // Fetched delivery: the producer announces the journeys and the consumer collects.
    let ConsumerEvent::DataReady { fetch, .. } = consumer.next_event().await else {
        panic!("a fetched-delivery producer announces its data");
    };
    let delivery = consumer.post(&producer.url, &fetch).await;
    let ConsumerEvent::Delivered { items, .. } = consumer.interpret(&delivery) else {
        panic!("a data supply request is answered with the journeys");
    };

    assert_eq!(
        items.iter().map(|j| j.line_ref.as_str()).collect::<Vec<_>>(),
        ["10", "10"]
    );
    assert_eq!(
        items[0].estimated_calls()[0].expected_departure_time,
        items[0].estimated_calls()[0]
            .aimed_departure_time
            .map(|aimed| aimed + Duration::minutes(3)),
        "the first journey is the one running three minutes late"
    );
    assert_eq!(items[1].cancellation, Some(true));

    let terminate = consumer.terminate_all(now);
    let confirmation = consumer.post(&producer.url, &terminate).await;
    let ConsumerEvent::Terminated { subscription_refs } = consumer.interpret(&confirmation) else {
        panic!("a termination request is answered with a confirmation");
    };
    assert_eq!(subscription_refs.len(), 1);
    assert_eq!(producer.subscriptions(), 0);

    wire.assert_every_message_was_valid();
    assert_eq!(
        wire.exchanged(),
        [
            "consumer → producer: SubscriptionRequest",
            "producer → consumer: SubscriptionResponse",
            "producer → consumer: DataReadyNotification",
            "consumer → producer: DataReadyAcknowledgement",
            "consumer → producer: DataSupplyRequest",
            "producer → consumer: ServiceDelivery",
            "consumer → producer: TerminateSubscriptionRequest",
            "producer → consumer: TerminateSubscriptionResponse",
        ]
    );
}

#[tokio::test]
async fn a_vehicle_monitoring_subscription_is_pushed_and_updated_over_http() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let wire = Wire::default();
    let mut consumer = ConsumerEndpoint::start(
        Consumer::<VehicleMonitoring>::new("MAP").confirming_deliveries(),
        wire.clone(),
    )
    .await;
    let producer = ProducerEndpoint::start(
        ProducerConfig::new("MY-AGENCY"),
        TrackedVehicles(vec![vehicle_at(9.7411, 52.3759)]),
        wire.clone(),
    )
    .await;

    let now = Utc::now().fixed_offset();
    let subscribe = consumer.subscribe(
        "fleet",
        now + Duration::hours(1),
        VehicleMonitoringRequest::new(now),
        now,
    );
    let response = consumer.post(&producer.url, &subscribe).await;
    consumer.interpret(&response);

    // Direct delivery: the vehicle arrives at the consumer's address unasked.
    let ConsumerEvent::Delivered { items, reply } = consumer.next_event().await else {
        panic!("a direct-delivery producer pushes what it holds");
    };
    assert!(reply.is_some(), "this consumer confirms what it receives");
    assert_eq!(position_of(&items[0]), (9.7411, 52.3759));

    // The vehicle moves; telling the producer is all an application has to do.
    {
        let mut held = producer.producer.lock().expect("the producer is usable");
        held.source_mut().0 = vec![vehicle_at(9.7500, 52.3800)];
        held.data_changed();
    }
    let ConsumerEvent::Delivered { items, .. } = consumer.next_event().await else {
        panic!("a changed source is delivered again");
    };
    assert_eq!(position_of(&items[0]), (9.7500, 52.3800));

    let terminate = consumer.terminate_all(now);
    let confirmation = consumer.post(&producer.url, &terminate).await;
    consumer.interpret(&confirmation);
    assert_eq!(producer.subscriptions(), 0);

    wire.assert_every_message_was_valid();
    assert_eq!(
        wire.exchanged(),
        [
            "consumer → producer: SubscriptionRequest",
            "producer → consumer: SubscriptionResponse",
            "producer → consumer: ServiceDelivery",
            "consumer → producer: DataReceivedAcknowledgement",
            "producer → consumer: ServiceDelivery",
            "consumer → producer: DataReceivedAcknowledgement",
            "consumer → producer: TerminateSubscriptionRequest",
            "producer → consumer: TerminateSubscriptionResponse",
        ]
    );
}

/// A producer answering a service request without any subscription at all.
#[tokio::test]
async fn a_vehicle_monitoring_request_is_answered_over_http_without_a_subscription() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let wire = Wire::default();
    let mut consumer =
        ConsumerEndpoint::start(Consumer::<VehicleMonitoring>::new("MAP"), wire.clone()).await;
    let producer = ProducerEndpoint::start(
        ProducerConfig::new("MY-AGENCY"),
        TrackedVehicles(vec![vehicle_at(9.7411, 52.3759)]),
        wire.clone(),
    )
    .await;

    let now = Utc::now().fixed_offset();
    let request = consumer.request(VehicleMonitoringRequest::new(now), now);
    let delivery = consumer.post(&producer.url, &request).await;
    let ConsumerEvent::Delivered { items, .. } = consumer.interpret(&delivery) else {
        panic!("a service request is answered with a delivery");
    };

    assert_eq!(position_of(&items[0]), (9.7411, 52.3759));
    assert_eq!(producer.subscriptions(), 0, "a request opens no subscription");
    wire.assert_every_message_was_valid();
    assert_eq!(
        wire.exchanged(),
        [
            "consumer → producer: ServiceRequest",
            "producer → consumer: ServiceDelivery",
        ]
    );
}

/// Whatever an operator already keeps its running journeys in.
struct RunningJourneys(Vec<EstimatedVehicleJourney>);

impl EstimatedTimetableSource for RunningJourneys {
    fn journeys(&self, request: &EstimatedTimetableRequest) -> Vec<EstimatedVehicleJourney> {
        // A real source would apply every filter the request carries. This one
        // honours the line filter and publishes the rest.
        let wanted: Vec<&str> = request
            .line_directions()
            .iter()
            .map(|line| line.line_ref.as_str())
            .collect();
        self.0
            .iter()
            .filter(|journey| wanted.is_empty() || wanted.contains(&journey.line_ref.as_str()))
            .cloned()
            .collect()
    }
}

/// Whatever a tracking system already keeps its vehicles in.
struct TrackedVehicles(Vec<VehicleActivity>);

impl VehicleMonitoringSource for TrackedVehicles {
    fn vehicles(&self, request: &VehicleMonitoringRequest) -> Vec<VehicleActivity> {
        match request.vehicle_ref.as_ref() {
            Some(wanted) => self
                .0
                .iter()
                .filter(|activity| {
                    activity.monitored_vehicle_journey.vehicle_ref.as_ref() == Some(wanted)
                })
                .cloned()
                .collect(),
            None => self.0.clone(),
        }
    }
}

fn delayed_journey() -> EstimatedVehicleJourney {
    let aimed = timestamp("2026-03-04T08:20:00+01:00");
    EstimatedVehicleJourney::dated("10", "OUT", "10-0815").with_estimated_calls(vec![EstimatedCall {
        aimed_departure_time: Some(aimed),
        expected_departure_time: Some(aimed + Duration::minutes(3)),
        ..EstimatedCall::at("de:03241:101")
    }])
}

fn cancelled_journey() -> EstimatedVehicleJourney {
    EstimatedVehicleJourney::dated("10", "OUT", "10-0845").cancelled()
}

fn vehicle_at(longitude: f64, latitude: f64) -> VehicleActivity {
    let recorded = timestamp("2026-03-04T08:15:00+01:00");
    VehicleActivity::new(
        recorded,
        recorded + Duration::minutes(5),
        MonitoredVehicleJourney {
            vehicle_location: Some(Location::wgs84(longitude, latitude)),
            vehicle_ref: Some("VEH-4711".into()),
            ..MonitoredVehicleJourney::on_line("10")
        },
    )
}

fn position_of(activity: &VehicleActivity) -> (f64, f64) {
    match activity
        .monitored_vehicle_journey
        .vehicle_location
        .as_ref()
        .and_then(Location::position)
    {
        Some(Position::Wgs84 {
            longitude,
            latitude,
            ..
        }) => (longitude, latitude),
        other => panic!("a tracked vehicle has a WGS 84 position, got {other:?}"),
    }
}

fn timestamp(rfc3339: &str) -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339(rfc3339).expect("valid timestamp")
}

/// Everything that crossed the wire, and whether it was valid SIRI when it did.
///
/// A message is recorded once, by whichever side put it on the wire, so that the
/// order recorded is the order the protocol took.
#[derive(Clone, Default)]
struct Wire(Arc<Mutex<WireLog>>);

#[derive(Default)]
struct WireLog {
    exchanged: Vec<String>,
    invalid: Vec<String>,
}

impl Wire {
    /// Records one message and validates it against the official schemas.
    fn saw(&self, direction: &str, xml: &str) {
        let mut log = self.0.lock().expect("the log is usable");
        log.exchanged
            .push(format!("{direction}: {}", payload_name(xml)));
        if let Err(complaint) = validate(xml) {
            log.invalid.push(format!("{direction}:\n{xml}\n{complaint}"));
        }
    }

    fn exchanged(&self) -> Vec<String> {
        self.0.lock().expect("the log is usable").exchanged.clone()
    }

    fn assert_every_message_was_valid(&self) {
        let invalid = &self.0.lock().expect("the log is usable").invalid;
        assert!(
            invalid.is_empty(),
            "{} message(s) did not validate:\n{}",
            invalid.len(),
            invalid.join("\n")
        );
    }
}

/// The name of the message a document carries, i.e. the child of `<Siri>`.
fn payload_name(xml: &str) -> String {
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut depth = 0;
    loop {
        match reader.read_event().expect("well-formed XML") {
            quick_xml::events::Event::Start(e) => {
                depth += 1;
                if depth == 2 {
                    return String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                }
            }
            quick_xml::events::Event::Eof => return "(empty)".to_owned(),
            _ => {}
        }
    }
}

/// A producer served on a real port.
struct ProducerEndpoint<Src: Source<Svc> + Send + 'static, Svc: Service> {
    url: String,
    producer: Arc<Mutex<Producer<Src, Svc>>>,
}

/// What the producer's route and its timer share.
struct ProducerState<Src: Source<Svc> + Send + 'static, Svc: Service> {
    producer: Arc<Mutex<Producer<Src, Svc>>>,
    wire: Wire,
}

// The derived `Clone` would demand `Src: Clone`; only the handles are cloned.
impl<Src: Source<Svc> + Send + 'static, Svc: Service> Clone for ProducerState<Src, Svc> {
    fn clone(&self) -> Self {
        Self {
            producer: self.producer.clone(),
            wire: self.wire.clone(),
        }
    }
}

impl<Src, Svc> ProducerEndpoint<Src, Svc>
where
    Src: Source<Svc> + Send + 'static,
    Svc: Service + Send + 'static,
    Svc::Request: Send,
{
    async fn start(config: ProducerConfig, source: Src, wire: Wire) -> Self {
        let started = Utc::now().fixed_offset();
        let state = ProducerState {
            producer: Arc::new(Mutex::new(Producer::new(config, source).started_at(started))),
            wire,
        };
        let (address, listener) = bind().await;
        let app = Router::new().route("/siri", post(answer).with_state(state.clone()));
        tokio::spawn(async move { axum::serve(listener, app).await });
        tokio::spawn(deliver_when_due(state.clone()));

        Self {
            url: format!("http://{address}/siri"),
            producer: state.producer,
        }
    }

    fn subscriptions(&self) -> usize {
        self.producer
            .lock()
            .expect("the producer is usable")
            .subscriptions()
            .len()
    }
}

async fn answer<Src, Svc>(State(state): State<ProducerState<Src, Svc>>, body: String) -> Response
where
    Src: Source<Svc> + Send + 'static,
    Svc: Service,
{
    let message: Siri = match siri_rs::from_str(&body) {
        Ok(message) => message,
        Err(complaint) => return refused(&state.wire, complaint.to_string()),
    };

    let now = Utc::now().fixed_offset();
    let reply = state
        .producer
        .lock()
        .expect("the producer is usable")
        .handle(&message, now);
    match reply {
        Ok(Some(reply)) => {
            let body = siri_rs::to_string(&reply).expect("a reply is writable");
            state.wire.saw("producer → consumer", &body);
            xml_response(body)
        }
        Ok(None) => StatusCode::NO_CONTENT.into_response(),
        Err(complaint) => refused(&state.wire, complaint.to_string()),
    }
}

/// Sends what the producer says is due, and feeds back what comes of it.
async fn deliver_when_due<Src, Svc>(state: ProducerState<Src, Svc>)
where
    Src: Source<Svc> + Send + 'static,
    Svc: Service,
{
    let client = reqwest::Client::new();
    let mut ticker = tokio::time::interval(POLL_INTERVAL);
    loop {
        ticker.tick().await;
        let now = Utc::now().fixed_offset();
        let due = state
            .producer
            .lock()
            .expect("the producer is usable")
            .poll(now);
        for outbound in due {
            send(&state, &client, &outbound).await;
        }
    }
}

async fn send<Src, Svc>(
    state: &ProducerState<Src, Svc>,
    client: &reqwest::Client,
    outbound: &Outbound,
) where
    Src: Source<Svc> + Send + 'static,
    Svc: Service,
{
    let Some(address) = outbound.address.as_ref() else {
        return;
    };
    let body = siri_rs::to_string(&outbound.message).expect("a message is writable");
    state.wire.saw("producer → consumer", &body);

    let answer = client
        .post(address.as_str())
        .header(header::CONTENT_TYPE, XML)
        .body(body)
        .send()
        .await
        .expect("the consumer answers");
    let answer = answer.text().await.unwrap_or_default();
    if answer.trim().is_empty() {
        return;
    }

    // The acknowledgement was recorded by the consumer as it wrote it.
    let acknowledgement: Siri = siri_rs::from_str(&answer).expect("an acknowledgement is readable");
    let now = Utc::now().fixed_offset();
    state
        .producer
        .lock()
        .expect("the producer is usable")
        .handle(&acknowledgement, now)
        .expect("an acknowledgement is understood");
}

/// A consumer served on a real port, with the events it saw.
struct ConsumerEndpoint<Svc: Service> {
    consumer: Arc<Mutex<Consumer<Svc>>>,
    client: reqwest::Client,
    events: mpsc::UnboundedReceiver<ConsumerEvent<Svc>>,
    wire: Wire,
}

/// What the consumer's route needs to do its work.
struct ConsumerState<Svc: Service> {
    consumer: Arc<Mutex<Consumer<Svc>>>,
    events: mpsc::UnboundedSender<ConsumerEvent<Svc>>,
    wire: Wire,
}

impl<Svc: Service> Clone for ConsumerState<Svc> {
    fn clone(&self) -> Self {
        Self {
            consumer: self.consumer.clone(),
            events: self.events.clone(),
            wire: self.wire.clone(),
        }
    }
}

impl<Svc> ConsumerEndpoint<Svc>
where
    Svc: Service + Send + 'static,
    Svc::Item: Send,
{
    async fn start(consumer: Consumer<Svc>, wire: Wire) -> Self {
        let (address, listener) = bind().await;
        let (sender, events) = mpsc::unbounded_channel();
        let consumer = Arc::new(Mutex::new(
            consumer.at_address(format!("http://{address}/siri")),
        ));
        let state = ConsumerState {
            consumer: consumer.clone(),
            events: sender,
            wire: wire.clone(),
        };
        let app = Router::new().route("/siri", post(receive).with_state(state));
        tokio::spawn(async move { axum::serve(listener, app).await });

        Self {
            consumer,
            client: reqwest::Client::new(),
            events,
            wire,
        }
    }

    fn subscribe(
        &mut self,
        identifier: &str,
        until: DateTime<FixedOffset>,
        request: Svc::Request,
        now: DateTime<FixedOffset>,
    ) -> Siri {
        self.consumer
            .lock()
            .expect("the consumer is usable")
            .subscribe(identifier, until, request, now)
    }

    fn request(&mut self, request: Svc::Request, now: DateTime<FixedOffset>) -> Siri {
        self.consumer
            .lock()
            .expect("the consumer is usable")
            .request(request, now)
    }

    fn terminate_all(&mut self, now: DateTime<FixedOffset>) -> Siri {
        self.consumer
            .lock()
            .expect("the consumer is usable")
            .terminate_all(now)
    }

    fn interpret(&self, message: &Siri) -> ConsumerEvent<Svc> {
        self.consumer
            .lock()
            .expect("the consumer is usable")
            .handle(message, Utc::now().fixed_offset())
            .expect("the message is understood")
    }

    /// Posts a message to the producer and reads the answer.
    async fn post(&self, url: &str, message: &Siri) -> Siri {
        exchange(&self.client, url, message, &self.wire).await
    }

    /// The next message the producer pushed to this consumer's own route.
    async fn next_event(&mut self) -> ConsumerEvent<Svc> {
        tokio::time::timeout(PATIENCE, self.events.recv())
            .await
            .expect("the producer sends something within the patience")
            .expect("the route that receives pushes is still running")
    }
}

async fn receive<Svc>(State(state): State<ConsumerState<Svc>>, body: String) -> Response
where
    Svc: Service,
{
    let message: Siri = match siri_rs::from_str(&body) {
        Ok(message) => message,
        Err(complaint) => return refused(&state.wire, complaint.to_string()),
    };

    let now = Utc::now().fixed_offset();
    let event = state
        .consumer
        .lock()
        .expect("the consumer is usable")
        .handle(&message, now)
        .expect("the message is understood");

    let answer = match &event {
        ConsumerEvent::DataReady { reply, .. } => Some((**reply).clone()),
        ConsumerEvent::Delivered { reply, .. } => reply.as_ref().map(|reply| (**reply).clone()),
        _ => None,
    };
    // The acknowledgement is recorded before the event is handed to the test, so
    // that what the log shows is the order the protocol actually took.
    let answer = answer.map(|reply| {
        let body = siri_rs::to_string(&reply).expect("an acknowledgement is writable");
        state.wire.saw("consumer → producer", &body);
        body
    });
    let _ = state.events.send(event);

    match answer {
        Some(body) => xml_response(body),
        None => StatusCode::NO_CONTENT.into_response(),
    }
}

async fn exchange(client: &reqwest::Client, url: &str, message: &Siri, wire: &Wire) -> Siri {
    let body = siri_rs::to_string(message).expect("a message is writable");
    wire.saw("consumer → producer", &body);
    let answer = client
        .post(url)
        .header(header::CONTENT_TYPE, XML)
        .body(body)
        .send()
        .await
        .expect("the producer answers")
        .text()
        .await
        .expect("the answer is readable");
    siri_rs::from_str(&answer).expect("the answer is a SIRI document")
}

fn xml_response(body: String) -> Response {
    ([(header::CONTENT_TYPE, XML)], body).into_response()
}

fn refused(wire: &Wire, complaint: String) -> Response {
    wire.saw("refused", &complaint);
    (StatusCode::BAD_REQUEST, complaint).into_response()
}

async fn bind() -> (std::net::SocketAddr, TcpListener) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a free port is available");
    let address = listener.local_addr().expect("the listener has an address");
    (address, listener)
}
