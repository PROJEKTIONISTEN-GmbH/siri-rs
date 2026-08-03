//! Vehicle Monitoring (SIRI-VM): where the vehicles are.
//!
//! SIRI-VM reports vehicles rather than timetables: one
//! [`VehicleActivity`] per vehicle, carrying its position, how it is getting on and
//! the journey it is running. It is what a live map is drawn from.
//!
//! Ask for it with a [`VehicleMonitoringRequest`], receive it in a
//! [`VehicleMonitoringDelivery`]. [`crate::pubsub`] wires those two into a
//! subscription.
//!
//! # Reading a feed
//!
//! ```
//! use siri_rs::framework::ServiceDeliveryPayload;
//! use siri_rs::model::Position;
//! use siri_rs::{Siri, SiriPayload};
//!
//! let xml = r#"<Siri xmlns="http://www.siri.org.uk/siri" version="2.1">
//!   <ServiceDelivery>
//!     <ResponseTimestamp>2026-03-04T08:15:00+01:00</ResponseTimestamp>
//!     <VehicleMonitoringDelivery version="2.1">
//!       <ResponseTimestamp>2026-03-04T08:15:00+01:00</ResponseTimestamp>
//!       <VehicleActivity>
//!         <RecordedAtTime>2026-03-04T08:14:50+01:00</RecordedAtTime>
//!         <ValidUntilTime>2026-03-04T08:19:50+01:00</ValidUntilTime>
//!         <MonitoredVehicleJourney>
//!           <LineRef>10</LineRef>
//!           <VehicleLocation>
//!             <Longitude>9.7411</Longitude>
//!             <Latitude>52.3759</Latitude>
//!           </VehicleLocation>
//!           <VehicleRef>VEH-4711</VehicleRef>
//!         </MonitoredVehicleJourney>
//!       </VehicleActivity>
//!     </VehicleMonitoringDelivery>
//!   </ServiceDelivery>
//! </Siri>"#;
//!
//! let message: Siri = siri_rs::from_str(xml)?;
//! let SiriPayload::ServiceDelivery(delivery) = &message.payload else {
//!     panic!("the document is a service delivery")
//! };
//!
//! let mut seen = Vec::new();
//! for payload in &delivery.deliveries {
//!     if let ServiceDeliveryPayload::VehicleMonitoringDelivery(vm) = payload {
//!         for activity in &vm.vehicle_activity {
//!             let journey = &activity.monitored_vehicle_journey;
//!             if let Some(Position::Wgs84 { longitude, latitude, .. }) =
//!                 journey.vehicle_location.as_ref().and_then(|at| at.position())
//!             {
//!                 seen.push((journey.vehicle_ref.as_ref().unwrap().as_str().to_owned(), longitude, latitude));
//!             }
//!         }
//!     }
//! }
//!
//! assert_eq!(seen, [("VEH-4711".to_owned(), 9.7411, 52.3759)]);
//! # Ok::<(), siri_rs::Error>(())
//! ```

pub mod capabilities;
pub mod delivery;
pub mod request;

pub use capabilities::{
    MonitoringPermissionItem, MonitoringPermissions, VehicleMonitorPermission,
    VehicleMonitoringAccessControl, VehicleMonitoringCapabilitiesResponse,
    VehicleMonitoringPermissions, VehicleMonitoringRequestPolicy,
    VehicleMonitoringResponseFeatures, VehicleMonitoringServiceCapabilities,
    VehicleMonitoringServicePermission, VehicleMonitoringTopicFiltering,
};
pub use delivery::{VehicleActivity, VehicleActivityCancellation, VehicleMonitoringDelivery};
pub use request::{
    MonitoredSubject, VehicleMonitoringRequest, VehicleMonitoringSubscriptionRequest,
};

siri_ref! {
    /// Identifies one of the vehicle-monitoring services a producer publishes under.
    ///
    /// A producer that tracks several fleets, or several regions, gives each its own
    /// reference so that a consumer can subscribe to one of them.
    VehicleMonitoringRef;
}
