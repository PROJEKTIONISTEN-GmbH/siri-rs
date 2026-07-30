//! The SIRI framework: the document envelope and the messages every service shares.
//!
//! A SIRI exchange is built from a small set of messages that are the same
//! whichever functional service carries the payload:
//!
//! * **Direct request/response** — [`ServiceRequest`] answered by [`ServiceDelivery`].
//! * **Publish/subscribe** — [`SubscriptionRequest`] answered by
//!   [`SubscriptionResponse`], after which the producer pushes
//!   [`ServiceDelivery`] messages until a [`TerminateSubscriptionRequest`]
//!   or a [`SubscriptionTerminatedNotification`] ends the relationship.
//! * **Fetched delivery** — a producer that cannot push sends a
//!   [`DataReadyNotification`] and waits for a [`DataSupplyRequest`].
//! * **Liveness** — [`CheckStatusRequest`]/[`CheckStatusResponse`] on demand,
//!   [`HeartbeatNotification`] on a timer.
//! * **Discovery** — what a producer can do ([`CapabilitiesRequest`]) and what it
//!   knows about ([`StopPointsRequest`], [`LinesRequest`] and friends).
//!
//! [`crate::pubsub`] turns these messages into the two state machines that drive
//! an exchange.

mod capabilities;
mod delivery;
mod discovery;
mod envelope;
mod error_condition;
mod status;
mod subscription;

pub use capabilities::{
    CapabilitiesRequest, CapabilitiesRequestPayload, CapabilitiesResponse,
    CapabilitiesResponsePayload, CoordinateFormat, DeliveryCapability, GeneralInteractionCapability,
    GeneralPermissions, InteractionCapability, LinePermission, LinePermissionItem, LinePermissions,
    OperatorPermission, OperatorPermissionItem, OperatorPermissions, PermissionScope,
    PermissionVersionRef, ServiceCapabilitiesRequest, SituationExchangeAccessControl,
    SituationExchangeCapabilitiesResponse, SituationExchangePermission,
    SituationExchangePermissions, SituationExchangeRequestPolicy,
    SituationExchangeServiceCapabilities, SituationExchangeTopicFiltering,
    SubscriptionPolicyCapability, TransportDescription,
};
pub use delivery::{
    DataReadyAcknowledgement, DataReadyNotification, DataReceivedAcknowledgement, DataSupplyRequest,
};
pub use discovery::{
    AnnotatedDestination, AnnotatedLineRef, AnnotatedStopPointRef, Destinations, Directions,
    JourneyPattern, JourneyPatterns, LinesDelivery, LinesRequest, LinesScope,
    ProductCategoriesDelivery, ProductCategoriesRequest, RouteDirection, ServiceFeaturesDelivery,
    ServiceFeaturesRequest, StopPointFeature, StopPointFeatures, StopPointInPattern, StopPointLine,
    StopPointLines, StopPointsDelivery, StopPointsRequest, StopPointsScope, StopsInPattern,
    VehicleFeaturesDelivery, VehicleFeaturesRequest,
};
pub use envelope::{
    DataNameSpaces, ServiceDelivery, ServiceDeliveryPayload, ServiceRequest, ServiceRequestContext,
    ServiceRequestPayload, Siri, SiriPayload,
};
pub use error_condition::{
    AcknowledgementError, ApplicationError, CapabilityNotSupportedError, DeliveryError,
    EndpointError, ErrorCodeDetail, ErrorCondition, InvalidDataReferencesError,
    ParametersIgnoredError, ServiceNotAvailableError, ServiceRequestError, StatusError,
    TerminationError, UnapprovedKeyAccessError, UnknownExtensionsError, UnknownParticipantError,
    UnknownSubscriberError, UnknownSubscriptionError,
};
pub use status::{CheckStatusRequest, CheckStatusResponse, HeartbeatNotification};
pub use subscription::{
    StatusResponse, SubscriptionContext, SubscriptionRequest, SubscriptionRequestPayload,
    SubscriptionResponse, SubscriptionTerminatedErrorCondition, SubscriptionTerminatedNotification,
    TerminateSubscriptionRequest, TerminateSubscriptionResponse, TerminationResponseStatus,
    TerminationScope,
};
