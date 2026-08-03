//! Connection Timetable (SIRI-CT): the connections planned onto a departure.
//!
//! SIRI-CT is what a distributor — the service passengers leave on — is told in
//! advance: one [`TimetabledFeederArrival`] per feeder service planned to arrive at
//! the interchange in time for it. Each carries an
//! [`InterchangeJourney`](crate::model::InterchangeJourney) describing the feeder.
//!
//! What actually happens on the day is [`crate::cm`], which reports the same
//! interchange as it is running.
//!
//! Ask for it with a [`ConnectionTimetableRequest`], receive it in a
//! [`ConnectionTimetableDelivery`].

pub mod capabilities;
pub mod delivery;
pub mod request;

pub use capabilities::{
    ConnectionTimetableCapabilitiesResponse, ConnectionTimetablePermissions,
    ConnectionTimetableRequestPolicy, ConnectionTimetableServiceCapabilities,
    ConnectionTimetableTopicFiltering,
};
pub use delivery::{
    ConnectionTimetableDelivery, TimetabledFeederArrival, TimetabledFeederArrivalCancellation,
};
pub use request::{ConnectionTimetableRequest, ConnectionTimetableSubscriptionRequest};
