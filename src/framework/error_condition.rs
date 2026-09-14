//! Error conditions a SIRI responder may report.
//!
//! Every error is an element substituting for `ErrorCode`, carrying an optional
//! `number` attribute and `ErrorText`, plus whatever detail the specific error adds.
//! The schema narrows which errors may appear in which message, so this module
//! exposes one enum per message family rather than a single catch-all — a
//! `CheckStatusResponse` can only report the two errors the schema allows it to.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::types::{CapabilityRef, EndpointAddress, ParticipantRef, SubscriptionQualifier};

/// Declares one of SIRI's `ErrorCodeStructure` extensions.
macro_rules! error_payload {
    (
        $(#[$meta:meta])*
        $name:ident { $($(#[$field_meta:meta])* $field:ident : $ty:ty => $rename:literal),* $(,)? }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
        pub struct $name {
            /// Implementation-specific error number.
            #[serde(rename = "@number", default, skip_serializing_if = "Option::is_none")]
            pub number: Option<i64>,
            /// Human-readable description of what went wrong.
            #[serde(rename = "ErrorText", default, skip_serializing_if = "Option::is_none")]
            pub error_text: Option<String>,
            $(
                $(#[$field_meta])*
                #[serde(rename = $rename, default, skip_serializing_if = "Option::is_none")]
                pub $field: Option<$ty>,
            )*
        }
    };
    (
        $(#[$meta:meta])*
        $name:ident [ $(#[$list_meta:meta])* $list:ident : $list_ty:ty => $list_rename:literal ]
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
        pub struct $name {
            /// Implementation-specific error number.
            #[serde(rename = "@number", default, skip_serializing_if = "Option::is_none")]
            pub number: Option<i64>,
            /// Human-readable description of what went wrong.
            #[serde(rename = "ErrorText", default, skip_serializing_if = "Option::is_none")]
            pub error_text: Option<String>,
            $(#[$list_meta])*
            #[serde(rename = $list_rename, default, skip_serializing_if = "Vec::is_empty")]
            pub $list: Vec<$list_ty>,
        }
    };
}

error_payload! {
    /// An error that carries no detail beyond a number and a text.
    ErrorCodeDetail {}
}

error_payload! {
    /// The account key presented with the request was not approved.
    UnapprovedKeyAccessError {
        /// The key that was presented.
        key: String => "Key"
    }
}

error_payload! {
    /// The referenced participant is unknown to the responder.
    UnknownParticipantError {
        /// The participant that is not known.
        participant_ref: ParticipantRef => "ParticipantRef"
    }
}

error_payload! {
    /// The referenced endpoint is unknown, denied or unavailable.
    EndpointError {
        /// The endpoint the request named.
        endpoint: EndpointAddress => "Endpoint"
    }
}

error_payload! {
    /// The service is temporarily out of operation.
    ServiceNotAvailableError {
        /// When the service expects to be available again.
        expected_restart_time: DateTime<FixedOffset> => "ExpectedRestartTime"
    }
}

error_payload! {
    /// The request used a capability the service does not offer.
    CapabilityNotSupportedError {
        /// The capability the request relied on.
        capability_ref: CapabilityRef => "CapabilityRef"
    }
}

error_payload! {
    /// The request referenced identifiers the responder does not know.
    InvalidDataReferencesError [
        /// The identifiers the responder does not recognise.
        invalid_ref: String => "InvalidRef"
    ]
}

error_payload! {
    /// The responder ignored request parameters it does not support.
    ParametersIgnoredError [
        /// The parameters that were ignored.
        parameter_name: String => "ParameterName"
    ]
}

error_payload! {
    /// The responder did not understand extension content in the request.
    UnknownExtensionsError [
        /// The extension elements that were not understood.
        extension_name: String => "ExtensionName"
    ]
}

error_payload! {
    /// The subscriber the request names has no relationship with the responder.
    UnknownSubscriberError {
        /// The subscriber that is not known.
        subscriber_ref: ParticipantRef => "SubscriberRef"
    }
}

error_payload! {
    /// The subscription the request names does not exist.
    UnknownSubscriptionError {
        /// The subscription that is not known.
        subscription_code: SubscriptionQualifier => "SubscriptionCode"
    }
}

/// Declares the choice of error elements a message family may report.
macro_rules! error_code_enum {
    ($(#[$meta:meta])* $name:ident { $($(#[$vmeta:meta])* $variant:ident($ty:ty)),* $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[non_exhaustive]
        pub enum $name {
            $($(#[$vmeta])* $variant($ty),)*
        }
    };
}

error_code_enum! {
    /// Errors reporting that a request could not be routed or authorised, or that
    /// the service could not answer it. This is SIRI's widest error choice; it
    /// applies to subscription responses.
    ServiceRequestError {
        /// The account key presented was not approved.
        UnapprovedKeyAccessError(UnapprovedKeyAccessError),
        /// The participant named in the request is unknown.
        UnknownParticipantError(UnknownParticipantError),
        /// The endpoint named in the request is unknown.
        UnknownEndpointError(EndpointError),
        /// The endpoint refused the request.
        EndpointDeniedAccessError(EndpointError),
        /// The endpoint could not be reached.
        EndpointNotAvailableAccessError(EndpointError),
        /// The service is temporarily out of operation.
        ServiceNotAvailableError(ServiceNotAvailableError),
        /// The request used an unsupported capability.
        CapabilityNotSupportedError(CapabilityNotSupportedError),
        /// The requestor is not allowed to see the data it asked for.
        AccessNotAllowedError(ErrorCodeDetail),
        /// The request referenced unknown identifiers.
        InvalidDataReferencesError(InvalidDataReferencesError),
        /// The request asked for data outside the service's data horizon.
        BeyondDataHorizon(ErrorCodeDetail),
        /// No data matched the requested topic.
        NoInfoForTopicError(ErrorCodeDetail),
        /// Some request parameters were ignored.
        ParametersIgnoredError(ParametersIgnoredError),
        /// Extension content in the request was not understood.
        UnknownExtensionsError(UnknownExtensionsError),
        /// The requestor exceeded its allowed resource usage.
        AllowedResourceUsageExceededError(ErrorCodeDetail),
        /// An error outside the standard set.
        OtherError(ErrorCodeDetail),
    }
}

error_code_enum! {
    /// Errors a producer may report when a subscription it holds ends unexpectedly.
    ApplicationError {
        /// The service is temporarily out of operation.
        ServiceNotAvailableError(ServiceNotAvailableError),
        /// The subscription used an unsupported capability.
        CapabilityNotSupportedError(CapabilityNotSupportedError),
        /// The subscriber is not allowed to see the data it subscribed to.
        AccessNotAllowedError(ErrorCodeDetail),
        /// The subscription referenced unknown identifiers.
        InvalidDataReferencesError(InvalidDataReferencesError),
        /// The subscription asked for data outside the service's data horizon.
        BeyondDataHorizon(ErrorCodeDetail),
        /// No data matched the subscribed topic.
        NoInfoForTopicError(ErrorCodeDetail),
        /// Some subscription parameters were ignored.
        ParametersIgnoredError(ParametersIgnoredError),
        /// Extension content in the subscription was not understood.
        UnknownExtensionsError(UnknownExtensionsError),
        /// The subscriber exceeded its allowed resource usage.
        AllowedResourceUsageExceededError(ErrorCodeDetail),
        /// An error outside the standard set.
        OtherError(ErrorCodeDetail),
    }
}

error_code_enum! {
    /// Errors a `ServiceDelivery` may report about the request that produced it.
    DeliveryError {
        /// The request used an unsupported capability.
        CapabilityNotSupportedError(CapabilityNotSupportedError),
        /// An error outside the standard set.
        OtherError(ErrorCodeDetail),
    }
}

error_code_enum! {
    /// Errors a `TerminateSubscriptionResponse` may report per subscription.
    TerminationError {
        /// The request used an unsupported capability.
        CapabilityNotSupportedError(CapabilityNotSupportedError),
        /// The subscriber is unknown to the producer.
        UnknownSubscriberError(UnknownSubscriberError),
        /// The subscription is unknown to the producer.
        UnknownSubscriptionError(UnknownSubscriptionError),
        /// An error outside the standard set.
        OtherError(ErrorCodeDetail),
    }
}

error_code_enum! {
    /// Errors a consumer may report when acknowledging a data-ready notification or
    /// a delivery.
    AcknowledgementError {
        /// The subscription is unknown to the consumer.
        UnknownSubscriptionError(UnknownSubscriptionError),
        /// An error outside the standard set.
        OtherError(ErrorCodeDetail),
    }
}

error_code_enum! {
    /// Errors a `CheckStatusResponse` or `HeartbeatNotification` may report.
    StatusError {
        /// The service is temporarily out of operation.
        ServiceNotAvailableError(ServiceNotAvailableError),
        /// An error outside the standard set.
        OtherError(ErrorCodeDetail),
    }
}

/// An error code together with the responder's description of it.
///
/// `C` is the choice of error elements the containing message may report, e.g.
/// [`StatusError`] for a `CheckStatusResponse`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorCondition<C> {
    /// Which error occurred.
    #[serde(rename = "$value")]
    pub code: C,
    /// Free text describing the error.
    #[serde(rename = "Description", default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl<C> ErrorCondition<C> {
    /// An error condition without a description.
    pub fn new(code: C) -> Self {
        Self {
            code,
            description: None,
        }
    }

    /// An error condition with a description.
    pub fn with_description(code: C, description: impl Into<String>) -> Self {
        Self {
            code,
            description: Some(description.into()),
        }
    }
}
