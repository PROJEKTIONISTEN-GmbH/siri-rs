//! General Message (SIRI-GM): free-form messages on named channels.
//!
//! SIRI-GM is the service for everything the structured services cannot say. A
//! producer publishes [`InfoMessage`]s on named channels — `WARNINGS`, `EMERGENCY`,
//! whatever the two ends have agreed — and a consumer subscribes to the channels it
//! wants. The body is deliberately unconstrained: the schema declares it as
//! `xsd:anyType`, so it may be a sentence or a whole document in another vocabulary,
//! and this crate keeps it as [`AnyContent`](crate::types::AnyContent) either way.
//!
//! Ask for it with a [`GeneralMessageRequest`], receive it in a
//! [`GeneralMessageDelivery`].

pub mod capabilities;
pub mod delivery;
pub mod request;

pub use capabilities::{
    GeneralMessageAccessControl, GeneralMessageCapabilitiesResponse, GeneralMessagePermissions,
    GeneralMessageServiceCapabilities, GeneralMessageServicePermission,
    GeneralMessageTopicFiltering, InfoChannelPermission, InfoChannelPermissionItem,
    InfoChannelPermissions,
};
pub use delivery::{GeneralMessageDelivery, InfoMessage, InfoMessageCancellation};
pub use request::{GeneralMessageRequest, GeneralMessageSubscriptionRequest};

siri_ref! {
    /// Identifies a channel a producer publishes general messages on.
    InfoChannelRef;
    /// Identifies one general message, so that it can be revised or withdrawn.
    InfoMessageRef;
}
