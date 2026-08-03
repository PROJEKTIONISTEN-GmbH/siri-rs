//! A producer and a consumer, each served on a real port, for whichever service.
//!
//! The hub's state machines carry no bytes, so a test that wants to see them work
//! over HTTP has to supply a transport. This is that transport — the same wiring the
//! `*_producer_axum` and `*_consumer_reqwest` examples show, with one addition the
//! examples do not need: every body that crosses the wire is recorded and validated
//! against the official schemas as it passes, so a cycle cannot be "working" here
//! while exchanging documents no other SIRI implementation would accept, or while
//! taking a different route through the protocol than it claims to.
//!
//! It is generic over [`Service`]: what a test brings is a source, a request and the
//! assertions about what came back.

use std::sync::{Arc, Mutex};
use std::time::Duration as StdDuration;

use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use chrono::{DateTime, FixedOffset, Utc};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use siri_rs::pubsub::{Consumer, ConsumerEvent, Outbound, Producer, ProducerConfig, Service, Source};
use siri_rs::Siri;

use super::validate;

/// The media type SIRI travels as.
const XML: &str = "application/xml";
/// How often the producer looks for messages that have become due.
const POLL_INTERVAL: StdDuration = StdDuration::from_millis(20);
/// How long a test waits for a message it expects before giving up.
const PATIENCE: StdDuration = StdDuration::from_secs(10);

/// Everything that crossed the wire, and whether it was valid SIRI when it did.
///
/// A message is recorded once, by whichever side put it on the wire, so that the
/// order recorded is the order the protocol took.
#[derive(Clone, Default)]
pub struct Wire(Arc<Mutex<WireLog>>);

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

    /// The messages exchanged so far, as `sender → receiver: MessageName`.
    pub fn exchanged(&self) -> Vec<String> {
        self.0.lock().expect("the log is usable").exchanged.clone()
    }

    /// Fails unless the schemas accepted every message that crossed the wire.
    pub fn assert_every_message_was_valid(&self) {
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
pub struct ProducerEndpoint<Src: Source<Svc> + Send + 'static, Svc: Service> {
    /// Where the consumer posts to.
    pub url: String,
    /// The producer itself, for a test that changes what it publishes.
    pub producer: Arc<Mutex<Producer<Src, Svc>>>,
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
    /// Serves a producer over the given source on a port the system picks.
    pub async fn start(config: ProducerConfig, source: Src, wire: Wire) -> Self {
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

    /// How many subscriptions the producer is holding.
    pub fn subscriptions(&self) -> usize {
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
pub struct ConsumerEndpoint<Svc: Service> {
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
    /// Serves the given consumer on a port the system picks, and tells it its address.
    pub async fn start(consumer: Consumer<Svc>, wire: Wire) -> Self {
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

    /// The message opening one subscription.
    pub fn subscribe(
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

    /// The message asking for data once, outside any subscription.
    pub fn request(&mut self, request: Svc::Request, now: DateTime<FixedOffset>) -> Siri {
        self.consumer
            .lock()
            .expect("the consumer is usable")
            .request(request, now)
    }

    /// The message closing every subscription this consumer holds.
    pub fn terminate_all(&mut self, now: DateTime<FixedOffset>) -> Siri {
        self.consumer
            .lock()
            .expect("the consumer is usable")
            .terminate_all(now)
    }

    /// What the consumer makes of a message that reached it as an answer.
    pub fn interpret(&self, message: &Siri) -> ConsumerEvent<Svc> {
        self.consumer
            .lock()
            .expect("the consumer is usable")
            .handle(message, Utc::now().fixed_offset())
            .expect("the message is understood")
    }

    /// How many subscriptions the consumer believes it holds.
    pub fn subscriptions(&self) -> usize {
        self.consumer
            .lock()
            .expect("the consumer is usable")
            .subscriptions()
            .len()
    }

    /// Posts a message to the producer and reads the answer.
    pub async fn post(&self, url: &str, message: &Siri) -> Siri {
        let body = siri_rs::to_string(message).expect("a message is writable");
        self.wire.saw("consumer → producer", &body);
        let answer = self
            .client
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

    /// The next message the producer pushed to this consumer's own route.
    pub async fn next_event(&mut self) -> ConsumerEvent<Svc> {
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
