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
//!
//! # Reading a feed
//!
//! A delivery arrives inside a service delivery, which may carry several; each one
//! holds the situations that matched.
//!
//! ```
//! use siri_rs::framework::ServiceDeliveryPayload;
//! use siri_rs::{Siri, SiriPayload};
//!
//! let xml = r#"<Siri xmlns="http://www.siri.org.uk/siri" version="2.0">
//!   <ServiceDelivery>
//!     <ResponseTimestamp>2026-03-04T08:15:00+01:00</ResponseTimestamp>
//!     <SituationExchangeDelivery>
//!       <ResponseTimestamp>2026-03-04T08:15:00+01:00</ResponseTimestamp>
//!       <Situations>
//!         <PtSituationElement>
//!           <CreationTime>2026-03-04T07:50:00+01:00</CreationTime>
//!           <SituationNumber>2026-0041</SituationNumber>
//!           <Source>
//!             <SourceType>feed</SourceType>
//!           </Source>
//!           <ValidityPeriod>
//!             <StartTime>2026-03-04T07:50:00+01:00</StartTime>
//!           </ValidityPeriod>
//!           <EquipmentReason>liftFailure</EquipmentReason>
//!           <Summary xml:lang="EN">Lift out of service at Central Station</Summary>
//!         </PtSituationElement>
//!       </Situations>
//!     </SituationExchangeDelivery>
//!   </ServiceDelivery>
//! </Siri>"#;
//!
//! let message: Siri = siri_rs::from_str(xml)?;
//! let SiriPayload::ServiceDelivery(delivery) = &message.payload else {
//!     panic!("the document is a service delivery")
//! };
//!
//! let mut summaries = Vec::new();
//! for payload in &delivery.deliveries {
//!     if let ServiceDeliveryPayload::SituationExchangeDelivery(sx) = payload {
//!         for situation in sx.pt_situations() {
//!             summaries.push(format!(
//!                 "{}: {}",
//!                 situation.situation_number,
//!                 situation.summary.first().map(|s| s.value.as_str()).unwrap_or("")
//!             ));
//!         }
//!     }
//! }
//!
//! assert_eq!(summaries, ["2026-0041: Lift out of service at Central Station"]);
//! # Ok::<(), siri_rs::Error>(())
//! ```

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
pub use crate::model::RequestedLines;
pub use request::{
    RoadFilter, SituationExchangeRequest, SituationExchangeSubscriptionRequest, SituationRoadFilter,
};
pub use situation::{PtSituationElement, Reason, RoadSituationElement};

impl crate::xml::SiriRoot for PtSituationElement {
    const ELEMENT_NAME: &'static str = "PtSituationElement";
}

impl crate::xml::SiriRoot for RoadSituationElement {
    const ELEMENT_NAME: &'static str = "RoadSituationElement";
}
