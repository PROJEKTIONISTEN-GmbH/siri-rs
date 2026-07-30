//! Production Timetable (SIRI-PT): the timetable as it is planned.
//!
//! SIRI-PT publishes the day's plan: which runs an operator intends to make, at
//! which stops, at which times, and which connections are planned around them. It
//! is the baseline the real-time services report against — an
//! [`EstimatedTimetableDelivery`](crate::et::EstimatedTimetableDelivery) says how
//! today differs from what a production timetable said it would be.
//!
//! Ask for it with a [`ProductionTimetableRequest`], receive it in a
//! [`ProductionTimetableDelivery`]. [`crate::pubsub`] wires those two into a
//! subscription.
//!
//! # Reading a timetable
//!
//! ```
//! use siri_rs::framework::ServiceDeliveryPayload;
//! use siri_rs::{Siri, SiriPayload};
//!
//! let xml = r#"<Siri xmlns="http://www.siri.org.uk/siri" version="2.1">
//!   <ServiceDelivery>
//!     <ResponseTimestamp>2026-03-04T04:00:00+01:00</ResponseTimestamp>
//!     <ProductionTimetableDelivery version="2.1">
//!       <ResponseTimestamp>2026-03-04T04:00:00+01:00</ResponseTimestamp>
//!       <DatedTimetableVersionFrame>
//!         <RecordedAtTime>2026-03-04T04:00:00+01:00</RecordedAtTime>
//!         <LineRef>10</LineRef>
//!         <DirectionRef>OUT</DirectionRef>
//!         <DatedVehicleJourney>
//!           <DatedVehicleJourneyCode>10-0815</DatedVehicleJourneyCode>
//!           <DatedCalls>
//!             <DatedCall>
//!               <StopPointRef>de:03241:101</StopPointRef>
//!               <AimedDepartureTime>2026-03-04T08:15:00+01:00</AimedDepartureTime>
//!             </DatedCall>
//!             <DatedCall>
//!               <StopPointRef>de:03241:102</StopPointRef>
//!               <AimedArrivalTime>2026-03-04T08:19:00+01:00</AimedArrivalTime>
//!             </DatedCall>
//!           </DatedCalls>
//!         </DatedVehicleJourney>
//!       </DatedTimetableVersionFrame>
//!     </ProductionTimetableDelivery>
//!   </ServiceDelivery>
//! </Siri>"#;
//!
//! let message: Siri = siri_rs::from_str(xml)?;
//! let SiriPayload::ServiceDelivery(delivery) = &message.payload else {
//!     panic!("the document is a service delivery")
//! };
//!
//! let mut stops = Vec::new();
//! for payload in &delivery.deliveries {
//!     if let ServiceDeliveryPayload::ProductionTimetableDelivery(pt) = payload {
//!         for journey in pt.journeys() {
//!             stops.extend(journey.dated_calls().iter().map(|call| call.stop_point_ref.as_str()));
//!         }
//!     }
//! }
//!
//! assert_eq!(stops, ["de:03241:101", "de:03241:102"]);
//! # Ok::<(), siri_rs::Error>(())
//! ```

pub mod capabilities;
pub mod delivery;
pub mod request;

pub use capabilities::{
    ProductionTimetableCapabilitiesResponse, ProductionTimetablePermissions,
    ProductionTimetableServiceCapabilities, ProductionTimetableSubscriptionPolicy,
    ProductionTimetableTopicFiltering,
};
pub use delivery::{DatedTimetableVersionFrame, ProductionTimetableDelivery};
pub use request::{
    ProductionTimetableRequest, ProductionTimetableSubscriptionRequest, TimetableValidityPeriod,
};
