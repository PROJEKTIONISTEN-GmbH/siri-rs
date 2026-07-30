//! Estimated Timetable (SIRI-ET): the timetable as it is actually running.
//!
//! Where the Production Timetable says what is planned, SIRI-ET says what is now
//! expected: journeys running late, journeys cancelled, journeys added, stops being
//! skipped. It is the service a real-time feed is normally consumed from.
//!
//! Ask for it with an [`EstimatedTimetableRequest`], receive it in an
//! [`EstimatedTimetableDelivery`]. [`crate::pubsub`] wires those two into a
//! subscription.
//!
//! # Reading a feed
//!
//! ```
//! use siri_rs::framework::ServiceDeliveryPayload;
//! use siri_rs::{Siri, SiriPayload};
//!
//! let xml = r#"<Siri xmlns="http://www.siri.org.uk/siri" version="2.1">
//!   <ServiceDelivery>
//!     <ResponseTimestamp>2026-03-04T08:15:00+01:00</ResponseTimestamp>
//!     <EstimatedTimetableDelivery version="2.1">
//!       <ResponseTimestamp>2026-03-04T08:15:00+01:00</ResponseTimestamp>
//!       <EstimatedJourneyVersionFrame>
//!         <RecordedAtTime>2026-03-04T08:14:50+01:00</RecordedAtTime>
//!         <EstimatedVehicleJourney>
//!           <LineRef>10</LineRef>
//!           <DirectionRef>OUT</DirectionRef>
//!           <DatedVehicleJourneyRef>10-0815</DatedVehicleJourneyRef>
//!           <EstimatedCalls>
//!             <EstimatedCall>
//!               <StopPointRef>de:03241:101</StopPointRef>
//!               <AimedDepartureTime>2026-03-04T08:20:00+01:00</AimedDepartureTime>
//!               <ExpectedDepartureTime>2026-03-04T08:23:00+01:00</ExpectedDepartureTime>
//!             </EstimatedCall>
//!           </EstimatedCalls>
//!         </EstimatedVehicleJourney>
//!       </EstimatedJourneyVersionFrame>
//!     </EstimatedTimetableDelivery>
//!   </ServiceDelivery>
//! </Siri>"#;
//!
//! let message: Siri = siri_rs::from_str(xml)?;
//! let SiriPayload::ServiceDelivery(delivery) = &message.payload else {
//!     panic!("the document is a service delivery")
//! };
//!
//! let mut late = Vec::new();
//! for payload in &delivery.deliveries {
//!     if let ServiceDeliveryPayload::EstimatedTimetableDelivery(et) = payload {
//!         for journey in et.journeys() {
//!             for call in journey.estimated_calls() {
//!                 if call.expected_departure_time > call.aimed_departure_time {
//!                     late.push(call.stop_point_ref.as_str());
//!                 }
//!             }
//!         }
//!     }
//! }
//!
//! assert_eq!(late, ["de:03241:101"]);
//! # Ok::<(), siri_rs::Error>(())
//! ```

pub mod capabilities;
pub mod delivery;
pub mod request;

pub use capabilities::{
    EstimatedTimetableCapabilitiesResponse, EstimatedTimetablePermissions,
    EstimatedTimetableServiceCapabilities, EstimatedTimetableTopicFiltering,
};
pub use delivery::{EstimatedTimetableDelivery, EstimatedVersionFrame};
pub use request::{EstimatedTimetableRequest, EstimatedTimetableSubscriptionRequest};
