//! Facility Monitoring (SIRI-FM): whether the lifts are working.
//!
//! SIRI-FM reports the state of the things passengers use rather than the services
//! themselves: lifts, escalators, ticket machines, waiting rooms, cycle racks,
//! charging points. One [`FacilityCondition`](crate::model::FacilityCondition) per
//! facility says what state it is in, what is being done about it, and — the point
//! of the service — what that means for a passenger with restricted mobility.
//!
//! Ask for it with a [`FacilityMonitoringRequest`], receive it in a
//! [`FacilityMonitoringDelivery`]. A request may filter by accessibility need, so a
//! consumer can subscribe to just the failures that matter to its users.

pub mod capabilities;
pub mod delivery;
pub mod request;

pub use capabilities::{
    FacilityMonitoringAccessControl, FacilityMonitoringCapabilitiesResponse,
    FacilityMonitoringPermissions, FacilityMonitoringRequestPolicy,
    FacilityMonitoringResponseFeatures, FacilityMonitoringServiceCapabilities,
    FacilityMonitoringServicePermission, FacilityMonitoringTopicFiltering,
};
pub use delivery::FacilityMonitoringDelivery;
pub use request::{
    AccessibilityNeedsFilter, FacilityMonitoringRequest, FacilityMonitoringSubscriptionRequest,
};
