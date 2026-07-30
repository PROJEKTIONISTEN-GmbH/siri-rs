//! The publish/subscribe hub over real HTTP.
//!
//! `examples/sx_producer_axum.rs` and `examples/sx_consumer_reqwest.rs` show how the
//! two state machines are wired to axum and reqwest. This test runs that wiring on a
//! real port and drives full subscription cycles through it, with one addition the
//! examples do not need: every body that crosses the wire is validated against the
//! official schemas as it passes, and the order of the messages is recorded. A cycle
//! cannot be "working" here while exchanging documents no other SIRI implementation
//! would accept, or while taking a different route through the protocol than it
//! claims to.

mod support;

use std::sync::{Arc, Mutex};
use std::time::Duration as StdDuration;

use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use chrono::{DateTime, Duration, FixedOffset, Utc};
use quick_xml::events::Event;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use siri_rs::enumerations::{AlertCause, Severity, SituationSourceType, WorkflowStatus};
use siri_rs::pubsub::{Consumer, ConsumerEvent, Outbound, Producer, ProducerConfig, SituationSource};
use siri_rs::sx::situation::{HalfOpenTimestampOutputRange, SituationSource as Source};
use siri_rs::sx::{PtSituationElement, SituationExchangeRequest};
use siri_rs::types::{DefaultedText, Duration as SiriDuration};
use siri_rs::Siri;
use support::{validate, validator_available, VALIDATOR_MISSING};

/// The media type SIRI travels as.
const XML: &str = "application/xml";
/// How often the producer looks for messages that have become due.
const POLL_INTERVAL: StdDuration = StdDuration::from_millis(20);
/// How long a test waits for a message it expects before giving up.
const PATIENCE: StdDuration = StdDuration::from_secs(10);

#[tokio::test]
async fn a_fetched_delivery_subscription_runs_its_full_cycle_over_http() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let wire = Wire::default();
    let mut consumer = ConsumerEndpoint::start(Consumer::new("PASSENGER-APP"), wire.clone()).await;
    let producer = ProducerEndpoint::start(
        ProducerConfig::new("MY-AGENCY").with_fetched_delivery(),
        wire.clone(),
    )
    .await;

    let now = Utc::now().fixed_offset();
    let subscribe = consumer.subscribe("disruptions", now + Duration::hours(1), now);
    let response = consumer.post(&producer.url, &subscribe).await;
    let ConsumerEvent::Subscribed { outcomes } = consumer.interpret(&response) else {
        panic!("a subscription request is answered with a subscription outcome");
    };
    assert_eq!(outcomes.len(), 1);
    assert!(outcomes[0].accepted);
    assert_eq!(outcomes[0].subscription_ref.as_str(), "disruptions");

    // The producer owes the new subscription the situations it matched. Using fetched
    // delivery it announces them, and the announcement reaches the address the
    // consumer named in its request.
    let ConsumerEvent::DataReady { fetch, .. } = consumer.next_event().await else {
        panic!("a fetched-delivery producer announces its data");
    };

    let delivery = consumer.post(&producer.url, &fetch).await;
    let ConsumerEvent::Delivered { situations, .. } = consumer.interpret(&delivery) else {
        panic!("a data supply request is answered with the situations");
    };
    assert_eq!(numbers(&situations), ["2026-0041", "2026-0042"]);
    assert_eq!(
        summaries(&situations),
        [
            "The lift to platform 3 is out of service",
            "Line 10 is diverted while the bridge is rebuilt",
        ]
    );

    let terminate = consumer.terminate_all(now);
    let confirmation = consumer.post(&producer.url, &terminate).await;
    let ConsumerEvent::Terminated { subscription_refs } = consumer.interpret(&confirmation) else {
        panic!("a termination request is answered with a confirmation");
    };
    assert_eq!(subscription_refs.len(), 1);
    assert_eq!(consumer.subscriptions(), 0);
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
async fn a_direct_delivery_producer_pushes_and_then_beats_over_http() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let wire = Wire::default();
    let mut consumer = ConsumerEndpoint::start(
        Consumer::new("PASSENGER-APP").confirming_deliveries(),
        wire.clone(),
    )
    .await;
    let producer = ProducerEndpoint::start(
        ProducerConfig::new("MY-AGENCY")
            .with_heartbeat(SiriDuration::parse("PT1S").expect("valid duration")),
        wire.clone(),
    )
    .await;

    let now = Utc::now().fixed_offset();
    let subscribe = consumer.subscribe("disruptions", now + Duration::hours(1), now);
    let response = consumer.post(&producer.url, &subscribe).await;
    consumer.interpret(&response);

    // This producer pushes: the delivery arrives at the consumer's address without
    // being asked for, and the consumer answers it with an acknowledgement.
    let ConsumerEvent::Delivered { situations, reply } = consumer.next_event().await else {
        panic!("a direct-delivery producer pushes its situations");
    };
    assert_eq!(numbers(&situations), ["2026-0041", "2026-0042"]);
    assert!(reply.is_some(), "this consumer confirms what it receives");

    let ConsumerEvent::Alive {
        service_started_time,
    } = consumer.next_event().await
    else {
        panic!("the producer's timer sends a heartbeat");
    };
    assert!(
        service_started_time.is_some(),
        "a heartbeat says when the producer's service started"
    );

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
            "producer → consumer: HeartbeatNotification",
            "consumer → producer: TerminateSubscriptionRequest",
            "producer → consumer: TerminateSubscriptionResponse",
        ]
    );
}

#[tokio::test]
async fn a_service_request_is_answered_over_http_without_a_subscription() {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let wire = Wire::default();
    let producer = ProducerEndpoint::start(ProducerConfig::new("MY-AGENCY"), wire.clone()).await;

    let now = Utc::now().fixed_offset();
    let request = Siri::new(
        siri_rs::pubsub::PROTOCOL_VERSION,
        siri_rs::framework::ServiceRequest::new(
            now,
            "PASSENGER-APP",
            vec![SituationExchangeRequest::new(now).into()],
        ),
    );
    let answer = exchange(&reqwest::Client::new(), &producer.url, &request).await;

    let ConsumerEvent::Delivered { situations, .. } = Consumer::new("PASSENGER-APP")
        .handle(&answer, now)
        .expect("the consumer reads the delivery")
    else {
        panic!("a service request is answered with a delivery");
    };
    assert_eq!(numbers(&situations), ["2026-0041", "2026-0042"]);
    assert_eq!(
        producer.subscriptions(),
        0,
        "a direct request opens nothing"
    );

    wire.assert_every_message_was_valid();
    assert_eq!(
        wire.exchanged(),
        [
            "consumer → producer: ServiceRequest",
            "producer → consumer: ServiceDelivery",
        ]
    );
}

/// The situations both endpoints in a test exchange.
struct Disruptions(Vec<PtSituationElement>);

impl SituationSource for Disruptions {
    fn situations(&self, _request: &SituationExchangeRequest) -> Vec<PtSituationElement> {
        self.0.clone()
    }
}

fn disruptions(now: DateTime<FixedOffset>) -> Disruptions {
    Disruptions(vec![
        situation(
            now,
            "2026-0041",
            AlertCause::LiftFailure,
            "The lift to platform 3 is out of service",
        ),
        situation(
            now,
            "2026-0042",
            AlertCause::ConstructionWork,
            "Line 10 is diverted while the bridge is rebuilt",
        ),
    ])
}

fn situation(
    at: DateTime<FixedOffset>,
    number: &str,
    cause: AlertCause,
    summary: &str,
) -> PtSituationElement {
    let mut situation = PtSituationElement::new(
        at,
        number,
        Source::new(SituationSourceType::DirectReport),
        HalfOpenTimestampOutputRange::between(at, at + Duration::days(2)),
        cause,
    );
    situation.participant_ref = Some("MY-AGENCY".into());
    situation.progress = Some(WorkflowStatus::Published);
    situation.severity = Some(Severity::Normal);
    situation.summary = vec![DefaultedText::with_lang("EN", summary)];
    situation
}

fn numbers(situations: &[PtSituationElement]) -> Vec<&str> {
    situations
        .iter()
        .map(|situation| situation.situation_number.as_str())
        .collect()
}

fn summaries(situations: &[PtSituationElement]) -> Vec<&str> {
    situations
        .iter()
        .map(|situation| {
            situation
                .summary
                .first()
                .map(|summary| summary.value.as_str())
                .unwrap_or_default()
        })
        .collect()
}

/// Every message that crossed the wire, in order, and every one of them that the
/// schemas rejected.
#[derive(Clone, Default)]
struct Wire(Arc<Mutex<WireLog>>);

#[derive(Default)]
struct WireLog {
    exchanged: Vec<String>,
    complaints: Vec<String>,
}

impl Wire {
    /// Validates a body as it goes past and records that it did.
    fn passing(&self, direction: &str, xml: &str) {
        let label = format!("{direction}: {}", payload_name(xml));
        let complaint = validate(xml)
            .err()
            .map(|complaint| format!("{label} is not valid SIRI:\n{complaint}\n{xml}"));
        let mut log = self.log();
        log.exchanged.push(label);
        log.complaints.extend(complaint);
    }

    /// Records that a message could not be exchanged at all.
    fn failed(&self, what: String) {
        self.log().complaints.push(what);
    }

    fn exchanged(&self) -> Vec<String> {
        self.log().exchanged.clone()
    }

    #[track_caller]
    fn assert_every_message_was_valid(&self) {
        let complaints = self.log().complaints.clone();
        assert!(complaints.is_empty(), "{}", complaints.join("\n\n"));
    }

    fn log(&self) -> std::sync::MutexGuard<'_, WireLog> {
        self.0.lock().expect("the wire log is usable")
    }
}

/// The name of the payload element inside a `<Siri>` envelope, which is what
/// distinguishes one message from another.
fn payload_name(xml: &str) -> String {
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut inside_envelope = false;
    loop {
        match reader.read_event().expect("a well-formed message") {
            Event::Start(element) | Event::Empty(element) => {
                if inside_envelope {
                    return String::from_utf8_lossy(element.local_name().as_ref()).into_owned();
                }
                inside_envelope = true;
            }
            Event::Eof => panic!("the envelope carries a payload:\n{xml}"),
            _ => {}
        }
    }
}

type SharedProducer = Arc<Mutex<Producer<Disruptions>>>;

#[derive(Clone)]
struct ProducerState {
    producer: SharedProducer,
    wire: Wire,
}

/// A producer served over HTTP: a POST route feeds [`Producer::handle`], and a timer
/// sends what [`Producer::poll`] hands back.
struct ProducerEndpoint {
    /// Where consumers post their requests.
    url: String,
    producer: SharedProducer,
}

impl ProducerEndpoint {
    async fn start(config: ProducerConfig, wire: Wire) -> Self {
        let now = Utc::now().fixed_offset();
        let producer = Arc::new(Mutex::new(
            Producer::new(config, disruptions(now)).started_at(now),
        ));
        let state = ProducerState {
            producer: producer.clone(),
            wire,
        };

        let (address, listener) = bind().await;
        let app = Router::new().route("/siri", post(answer).with_state(state.clone()));
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("the producer serves");
        });
        tokio::spawn(deliver_when_due(state));

        Self {
            url: format!("http://{address}/siri"),
            producer,
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

/// Answers a request a consumer posted.
async fn answer(State(state): State<ProducerState>, body: String) -> Response {
    let now = Utc::now().fixed_offset();
    // Validating and answering while holding the producer means the recorded order of
    // messages is the order they were exchanged in, even with the timer running.
    let mut producer = state.producer.lock().expect("the producer is usable");
    state.wire.passing("consumer → producer", &body);

    let message: Siri = match siri_rs::from_str(&body) {
        Ok(message) => message,
        Err(complaint) => return refused(&state.wire, format!("unreadable request: {complaint}")),
    };
    match producer.handle(&message, now) {
        Ok(Some(reply)) => {
            let xml = siri_rs::to_string(&reply).expect("a reply serialises");
            state.wire.passing("producer → consumer", &xml);
            xml_response(xml)
        }
        // Acknowledgements are answered by silence.
        Ok(None) => StatusCode::NO_CONTENT.into_response(),
        Err(complaint) => refused(&state.wire, format!("unanswerable request: {complaint}")),
    }
}

/// Sends the messages that have become due, and feeds the acknowledgements that come
/// back to the producer.
async fn deliver_when_due(state: ProducerState) {
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

async fn send(state: &ProducerState, client: &reqwest::Client, outbound: &Outbound) {
    let Some(address) = outbound.address.as_ref() else {
        state.wire.failed(format!(
            "{} named no address, so {} could not be sent",
            outbound.recipient,
            payload_name(&siri_rs::to_string(&outbound.message).expect("a message serialises")),
        ));
        return;
    };

    let body = siri_rs::to_string(&outbound.message).expect("a message serialises");
    let response = client
        .post(address.as_str())
        .header(header::CONTENT_TYPE, XML)
        .body(body)
        .send()
        .await;
    let acknowledgement = match response {
        Ok(response) => response.text().await.expect("a readable answer"),
        Err(complaint) => {
            state.wire.failed(format!(
                "the consumer at {address} refused a message: {complaint}"
            ));
            return;
        }
    };
    if acknowledgement.is_empty() {
        return;
    }

    let now = Utc::now().fixed_offset();
    let message: Siri = siri_rs::from_str(&acknowledgement).expect("a readable acknowledgement");
    state
        .producer
        .lock()
        .expect("the producer is usable")
        .handle(&message, now)
        .expect("the producer accepts the acknowledgement");
}

type SharedConsumer = Arc<Mutex<Consumer>>;

#[derive(Clone)]
struct ConsumerState {
    consumer: SharedConsumer,
    wire: Wire,
    events: mpsc::UnboundedSender<ConsumerEvent>,
}

/// A consumer reachable over HTTP: it answers what the producer pushes with the
/// acknowledgement [`Consumer::handle`] builds, and reports every event to the test.
struct ConsumerEndpoint {
    consumer: SharedConsumer,
    client: reqwest::Client,
    events: mpsc::UnboundedReceiver<ConsumerEvent>,
}

impl ConsumerEndpoint {
    async fn start(consumer: Consumer, wire: Wire) -> Self {
        let (address, listener) = bind().await;
        let consumer = Arc::new(Mutex::new(
            consumer.at_address(format!("http://{address}/siri")),
        ));
        let (events, received) = mpsc::unbounded_channel();
        let state = ConsumerState {
            consumer: consumer.clone(),
            wire,
            events,
        };

        let app = Router::new().route("/siri", post(receive).with_state(state));
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("the consumer serves");
        });

        Self {
            consumer,
            client: reqwest::Client::new(),
            events: received,
        }
    }

    fn subscribe(
        &self,
        identifier: &str,
        initial_termination_time: DateTime<FixedOffset>,
        now: DateTime<FixedOffset>,
    ) -> Siri {
        self.consumer
            .lock()
            .expect("the consumer is usable")
            .subscribe(
                identifier,
                initial_termination_time,
                SituationExchangeRequest::new(now),
                now,
            )
    }

    fn terminate_all(&self, now: DateTime<FixedOffset>) -> Siri {
        self.consumer
            .lock()
            .expect("the consumer is usable")
            .terminate_all(now)
    }

    /// Posts a message to the producer and reads the answer.
    async fn post(&self, url: &str, message: &Siri) -> Siri {
        exchange(&self.client, url, message).await
    }

    /// Interprets a message that arrived as an answer rather than as a push.
    fn interpret(&self, message: &Siri) -> ConsumerEvent {
        self.consumer
            .lock()
            .expect("the consumer is usable")
            .handle(message, Utc::now().fixed_offset())
            .expect("the consumer reads the message")
    }

    /// The next message the producer pushed, as the consumer understood it.
    async fn next_event(&mut self) -> ConsumerEvent {
        tokio::time::timeout(PATIENCE, self.events.recv())
            .await
            .expect("the producer sends what it owes")
            .expect("the consumer endpoint is still listening")
    }

    fn subscriptions(&self) -> usize {
        self.consumer
            .lock()
            .expect("the consumer is usable")
            .subscriptions()
            .len()
    }
}

/// Answers a message the producer pushed.
async fn receive(State(state): State<ConsumerState>, body: String) -> Response {
    let now = Utc::now().fixed_offset();
    let mut consumer = state.consumer.lock().expect("the consumer is usable");
    state.wire.passing("producer → consumer", &body);

    let message: Siri = match siri_rs::from_str(&body) {
        Ok(message) => message,
        Err(complaint) => return refused(&state.wire, format!("unreadable push: {complaint}")),
    };
    let event = match consumer.handle(&message, now) {
        Ok(event) => event,
        Err(complaint) => {
            return refused(&state.wire, format!("uninterpretable push: {complaint}"));
        }
    };

    let reply = match &event {
        ConsumerEvent::DataReady { reply, .. } => Some(reply.as_ref().clone()),
        ConsumerEvent::Delivered { reply, .. } => reply.as_deref().cloned(),
        _ => None,
    };
    let answered = reply.map(|reply| {
        let xml = siri_rs::to_string(&reply).expect("an acknowledgement serialises");
        state.wire.passing("consumer → producer", &xml);
        xml
    });
    // Reported only after the acknowledgement is on the wire, so that a test reacting
    // to the event cannot overtake it.
    let _ = state.events.send(event);

    match answered {
        Some(xml) => xml_response(xml),
        None => StatusCode::NO_CONTENT.into_response(),
    }
}

/// Posts a SIRI message and reads the message that comes back.
async fn exchange(client: &reqwest::Client, url: &str, message: &Siri) -> Siri {
    let response = client
        .post(url)
        .header(header::CONTENT_TYPE, XML)
        .body(siri_rs::to_string(message).expect("a request serialises"))
        .send()
        .await
        .expect("the endpoint answers");
    let status = response.status();
    let body = response.text().await.expect("a readable answer");
    assert!(status.is_success(), "{url} answered {status}: {body}");
    siri_rs::from_str(&body).expect("the answer is a SIRI message")
}

fn xml_response(body: String) -> Response {
    ([(header::CONTENT_TYPE, XML)], body).into_response()
}

/// Refuses a message and records why, so the test fails with the reason rather than
/// with a timeout.
fn refused(wire: &Wire, complaint: String) -> Response {
    wire.failed(complaint.clone());
    (StatusCode::BAD_REQUEST, complaint).into_response()
}

/// A listener on a port the operating system picked, so tests can run in parallel.
async fn bind() -> (std::net::SocketAddr, TcpListener) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a free local port");
    let address = listener.local_addr().expect("the listener is bound");
    (address, listener)
}
