//! Stop Monitoring (SIRI-SM): what is due at a stop.
//!
//! SIRI-SM is the departure board: one [`MonitoredStopVisit`] per service due at a
//! monitoring point, each carrying the
//! [`MonitoredVehicleJourney`](crate::model::MonitoredVehicleJourney) behind it —
//! the same journey model the vehicle-tracking service uses, seen from the stop
//! rather than from the vehicle.
//!
//! Ask for it with a [`StopMonitoringRequest`], receive it in a
//! [`StopMonitoringDelivery`]. [`crate::pubsub`] wires those two into a
//! subscription.
//!
//! # Reading a departure board
//!
//! ```
//! use siri_rs::framework::ServiceDeliveryPayload;
//! use siri_rs::{Siri, SiriPayload};
//!
//! let xml = r#"<Siri xmlns="http://www.siri.org.uk/siri" version="2.1">
//!   <ServiceDelivery>
//!     <ResponseTimestamp>2026-03-04T08:15:00+01:00</ResponseTimestamp>
//!     <StopMonitoringDelivery version="2.1">
//!       <ResponseTimestamp>2026-03-04T08:15:00+01:00</ResponseTimestamp>
//!       <MonitoringRef>de:03241:101</MonitoringRef>
//!       <MonitoredStopVisit>
//!         <RecordedAtTime>2026-03-04T08:14:50+01:00</RecordedAtTime>
//!         <MonitoringRef>de:03241:101</MonitoringRef>
//!         <MonitoredVehicleJourney>
//!           <PublishedLineName>10</PublishedLineName>
//!           <DestinationName>Ahlem</DestinationName>
//!           <MonitoredCall>
//!             <AimedDepartureTime>2026-03-04T08:17:00+01:00</AimedDepartureTime>
//!             <ExpectedDepartureTime>2026-03-04T08:19:00+01:00</ExpectedDepartureTime>
//!           </MonitoredCall>
//!         </MonitoredVehicleJourney>
//!       </MonitoredStopVisit>
//!     </StopMonitoringDelivery>
//!   </ServiceDelivery>
//! </Siri>"#;
//!
//! let message: Siri = siri_rs::from_str(xml)?;
//! let SiriPayload::ServiceDelivery(delivery) = &message.payload else {
//!     panic!("the document is a service delivery")
//! };
//!
//! let mut board = Vec::new();
//! for payload in &delivery.deliveries {
//!     if let ServiceDeliveryPayload::StopMonitoringDelivery(sm) = payload {
//!         for visit in &sm.monitored_stop_visit {
//!             let journey = &visit.monitored_vehicle_journey;
//!             let call = journey.monitored_call.as_ref().expect("the visit has a call");
//!             board.push((
//!                 journey.published_line_name[0].value.clone(),
//!                 journey.destination_name[0].value.clone(),
//!                 call.expected_departure_time.expect("a real-time departure"),
//!             ));
//!         }
//!     }
//! }
//!
//! assert_eq!(board.len(), 1);
//! assert_eq!(board[0].0, "10");
//! assert_eq!(board[0].1, "Ahlem");
//! # Ok::<(), siri_rs::Error>(())
//! ```

pub mod capabilities;
pub mod delivery;
pub mod request;

pub use capabilities::{
    StopMonitoringCapabilitiesResponse, StopMonitoringPermissions, StopMonitoringRequestPolicy,
    StopMonitoringResponseFeatures, StopMonitoringServiceCapabilities,
    StopMonitoringServicePermission, StopMonitoringTopicFiltering,
};
pub use delivery::{
    DeliveryVariant, MonitoredStopVisit, MonitoredStopVisitCancellation, ServiceException,
    StopLineNotice, StopLineNoticeCancellation, StopMonitoringDelivery, StopNotice,
    StopNoticeCancellation,
};
pub use request::{
    StopMonitoringFilter, StopMonitoringMultipleRequest, StopMonitoringRequest,
    StopMonitoringSubscriptionRequest,
};
