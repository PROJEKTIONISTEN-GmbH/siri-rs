//! The publish/subscribe data hub: subscriptions, delivery and liveness.
//!
//! SIRI's data hub is a conversation with a well-defined shape. A consumer opens
//! subscriptions; the producer answers them, then keeps sending until they are
//! closed. Whether it *pushes* deliveries or announces them for the consumer to
//! *fetch* is the producer's choice, and both patterns are the same conversation
//! with an extra round trip.
//!
//! ```text
//!   consumer                                     producer
//!      | ---- SubscriptionRequest ------------------> |
//!      | <--- SubscriptionResponse ------------------ |
//!      |                                              |
//!      |   direct delivery                            |
//!      | <--- ServiceDelivery ----------------------- |   whenever data changes
//!      | ---- DataReceivedAcknowledgement ----------> |   if confirmation was asked for
//!      |                                              |
//!      |   fetched delivery                           |
//!      | <--- DataReadyNotification ----------------- |   whenever data changes
//!      | ---- DataReadyAcknowledgement -------------> |
//!      | ---- DataSupplyRequest --------------------> |
//!      | <--- ServiceDelivery ----------------------- |
//!      |                                              |
//!      | <--- HeartbeatNotification ----------------- |   on a timer
//!      | ---- TerminateSubscriptionRequest ---------> |
//!      | <--- TerminateSubscriptionResponse --------- |
//! ```
//!
//! [`Producer`] and [`Consumer`] implement the two sides of that conversation.
//! Neither knows anything about HTTP: they turn an incoming [`Siri`] message into
//! the messages that should go out, and a clock reading into the messages that are
//! due. Carrying those messages is the caller's job — see the `sx_endpoint`
//! example for a sketch.
//!
//! # A full cycle, in process
//!
//! ```
//! use chrono::{DateTime, FixedOffset};
//! use siri::pubsub::{Consumer, ConsumerEvent, Producer, ProducerConfig, SituationSource};
//! use siri::sx::{PtSituationElement, SituationExchangeRequest};
//!
//! struct OneSituation(PtSituationElement);
//! impl SituationSource for OneSituation {
//!     fn situations(&self, _request: &SituationExchangeRequest) -> Vec<PtSituationElement> {
//!         vec![self.0.clone()]
//!     }
//! }
//!
//! # fn run(situation: PtSituationElement, now: DateTime<FixedOffset>) -> siri::Result<()> {
//! let mut producer = Producer::new(ProducerConfig::new("KUBRICK"), OneSituation(situation));
//! let mut consumer = Consumer::new("NADER");
//!
//! let request = consumer.subscribe(
//!     "sub-1",
//!     now + chrono::Duration::hours(1),
//!     SituationExchangeRequest::new(now),
//!     now,
//! );
//! let response = producer.handle(&request, now)?.expect("a subscription is answered");
//! assert!(matches!(consumer.handle(&response, now)?, ConsumerEvent::Subscribed { .. }));
//!
//! // The producer now owes the consumer the situations it matched.
//! for outbound in producer.poll(now) {
//!     if let ConsumerEvent::Delivered { situations, .. } = consumer.handle(&outbound.message, now)? {
//!         assert_eq!(situations.len(), 1);
//!     }
//! }
//! # Ok(())
//! # }
//! ```

mod consumer;
mod producer;

pub use consumer::{Consumer, ConsumerEvent, Subscribed};
pub use producer::{Outbound, Producer, ProducerConfig, SituationSource, Subscription, SubscriptionState};

use crate::framework::Siri;

/// The version of SIRI the state machines write into the messages they build.
pub const PROTOCOL_VERSION: &str = "2.1";

/// Wraps a message in an envelope declaring [`PROTOCOL_VERSION`].
fn envelope(payload: impl Into<crate::framework::SiriPayload>) -> Siri {
    Siri::new(PROTOCOL_VERSION, payload)
}
