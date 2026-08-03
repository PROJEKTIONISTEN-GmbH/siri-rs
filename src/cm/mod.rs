//! Connection Monitoring (SIRI-CM): whether a connection will be made.
//!
//! SIRI-CM is the running counterpart of [`crate::ct`], and it is two feeds rather
//! than one, because the two sides of an interchange need different things:
//!
//! * the **feeder** side reports arrivals ([`MonitoredFeederArrival`]) — when the
//!   incoming service is now expected and how many passengers want to change — so
//!   that the distributor can decide whether to wait;
//! * the **distributor** side reports its decision — that it will wait
//!   ([`WaitProlongedDeparture`]), that it has moved
//!   ([`StoppingPositionChangedDeparture`]) or that it is not going at all
//!   ([`DistributorDepartureCancellation`]).
//!
//! Ask with a [`ConnectionMonitoringRequest`]; the answer comes as a
//! [`ConnectionMonitoringFeederDelivery`] or a
//! [`ConnectionMonitoringDistributorDelivery`], depending on which side the producer
//! runs.

pub mod capabilities;
pub mod delivery;
pub mod request;

pub use capabilities::{
    ConnectionMonitoringCapabilitiesResponse, ConnectionMonitoringPermissions,
    ConnectionMonitoringRequestPolicy, ConnectionMonitoringServiceCapabilities,
    ConnectionMonitoringTopicFiltering,
};
pub use delivery::{
    ConnectionMonitoringDistributorDelivery, ConnectionMonitoringFeederDelivery,
    DistributorDepartureCancellation, MonitoredFeederArrival, MonitoredFeederArrivalCancellation,
    StoppingPositionChangedDeparture, WaitProlongedDeparture,
};
pub use request::{
    ConnectingJourneyFilter, ConnectingTimeFilter, ConnectionMonitoringRequest,
    ConnectionMonitoringSubscriptionRequest, ConnectionScope,
};
