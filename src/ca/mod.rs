//! Control Actions (SIRI-CA): what a control room has decided to do.
//!
//! The other services describe what is happening. This one describes what is being
//! done about it: a journey put on or taken off, a stop closed, an interchange held
//! or dropped, a vehicle moved to another piece of work, a message sent to a driver.
//! Each decision is a [`ControlAction`] carrying exactly one of those, which
//! [`ControlAction::kind`] reports.
//!
//! Ask for them with a [`ControlActionRequest`], receive them in a
//! [`ControlActionDelivery`].
//!
//! # What this service is proved against
//!
//! Every other service in this crate is checked against the example documents
//! published with the standard. Control Actions ships none, so there is nothing to
//! read back and compare: it is checked instead against the schemas, by building a
//! message for every control action the schema allows and validating each one — see
//! `tests/control_actions.rs`. That is a weaker guarantee than the other services
//! have, and it is stated here rather than glossed over. Example messages would
//! make it the same guarantee.
//!
//! Two further things are worth knowing before putting this service on a wire.
//! First, `siri.xsd` — the schema variant that lists the services a `<Siri>`
//! message may carry one by one — does not list this one, so a control-action
//! message validates only against the substitution-group variant, `siriSg.xsd`.
//! Second, the schema declares four types no message can reach: a stop-point
//! allocation, the point it allocates, a journeys-within-line scope and a detail
//! level. They are not modelled here.
//!
//! # A journey taken out of today's plan
//!
//! ```
//! use chrono::{DateTime, FixedOffset};
//! use siri_rs::ca::{ControlAction, ControlActionKind, JourneyScope};
//! use siri_rs::enumerations::ControlActionReasonCategory;
//! use siri_rs::ca::ControlActionReason;
//! use siri_rs::model::FramedVehicleJourneyRef;
//!
//! # fn decide(now: DateTime<FixedOffset>) {
//! let action = ControlAction {
//!     reason: Some(ControlActionReason::new(
//!         ControlActionReasonCategory::VehicleBreakdown,
//!     )),
//!     journey_cancellation: Some(JourneyScope::for_journey(FramedVehicleJourneyRef {
//!         data_frame_ref: "2026-03-04".into(),
//!         dated_vehicle_journey_ref: "10-0815".into(),
//!     })),
//!     ..ControlAction::new(now, "CA-4711")
//! };
//!
//! assert!(matches!(
//!     action.kind(),
//!     Some(ControlActionKind::JourneyCancellation(_))
//! ));
//! # }
//! ```

pub mod action;
pub mod capabilities;
pub mod delivery;
pub mod request;

pub use action::{
    CallCancellationAction, CallsBetweenPoints, CancelledConnection, ChangeOfJourneyTiming,
    ChangeOfStopPointStatus, ChangedPoint, ChangedPoints, ControlAction, ControlActionKind,
    ControlActionReason, DatedCallRef, ExtraConnection, FlexibleJourneyActivation, JourneyCreation,
    JourneyEnd, JourneyPatternModification, JourneyScope, JourneyStart, MiddleCall,
    ModifiedConnection, Place, PointInJourneyPatternRef, RelativeTime, SelectedCalls,
    SituationDescription, StopPointStatusTimeScope, StopRefs, TargetPoint, TimedCalls,
    VehicleWorkAssignment,
};
pub use capabilities::{
    ControlActionCapabilitiesResponse, ControlActionPermissions, ControlActionRequestPolicy,
    ControlActionResponseFeatures, ControlActionServiceCapabilities,
    ControlActionServicePermission, ControlActionTopicFiltering,
};
pub use delivery::{
    AddedControlActions, ControlActionDelivery, ControlActions, DriverMessage, DriverMessages,
    DriverScope, GroupOfControlActions, GroupsOfControlActions, MessageContents,
    RemovedControlActions, RevokedControlAction, RevokedControlActions, VehicleDetecting,
    VehicleDetectings,
};
pub use request::{
    ControlActionFilter, ControlActionMultipleRequest, ControlActionRequest,
    ControlActionSubscriptionRequest, LineScope,
};
