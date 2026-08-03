//! Stop Timetable (SIRI-ST): what the timetable says will call at a stop.
//!
//! SIRI-ST is the printed departure board rather than the live one: one
//! [`TimetabledStopVisit`] per service planned to call at a monitoring point within
//! a stated window, each carrying a
//! [`TargetedVehicleJourney`](crate::model::TargetedVehicleJourney) — the journey as
//! the timetable has it, with only the call at this stop.
//!
//! Ask for it with a [`StopTimetableRequest`], receive it in a
//! [`StopTimetableDelivery`]. [`crate::pubsub`] wires those two into a subscription,
//! which is how a producer tells a stop display that tomorrow's timetable has
//! changed.

pub mod capabilities;
pub mod delivery;
pub mod request;

pub use capabilities::{
    StopTimetableCapabilitiesResponse, StopTimetablePermissions, StopTimetableRequestPolicy,
    StopTimetableServiceCapabilities, StopTimetableServicePermission, StopTimetableTopicFiltering,
};
pub use delivery::{StopTimetableDelivery, TimetabledStopVisit, TimetabledStopVisitCancellation};
pub use request::{StopTimetableRequest, StopTimetableSubscriptionRequest};
