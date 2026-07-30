//! Situation Exchange (SIRI-SX): incidents and disruptions.
//!
//! A *situation* is anything that perturbs, or may perturb, normal operation — an
//! accident, a strike, engineering work, a special event. SIRI-SX exchanges them as
//! [`PtSituationElement`] records: what happened ([`situation::Reason`]), when it
//! holds, what it affects ([`affects::AffectsScope`]), what it does to the service
//! ([`consequence::Consequence`]), and how it should be published
//! ([`action::Actions`]).
//!
//! Ask for situations with a [`SituationExchangeRequest`], receive them in a
//! [`SituationExchangeDelivery`]. [`crate::pubsub`] wires those two into a
//! subscription.

pub mod action;
pub mod affects;
pub mod consequence;
pub mod delivery;
pub mod request;
pub mod situation;

pub use action::{Actions, PublishingAction};
pub use affects::AffectsScope;
pub use consequence::{Consequence, PtConsequences};
pub use delivery::{
    Network, NetworkContext, SituationContext, SituationExchangeDelivery, Situations,
};
pub use request::{
    RequestedLines, RoadFilter, SituationExchangeRequest, SituationExchangeSubscriptionRequest,
    SituationRoadFilter,
};
pub use situation::{PtSituationElement, Reason, RoadSituationElement};

impl crate::xml::SiriRoot for RoadSituationElement {
    const ELEMENT_NAME: &'static str = "RoadSituationElement";
}
