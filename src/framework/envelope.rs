//! The `<Siri>` document envelope and the two payload-carrying messages inside it.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::{DeliveryMethod, Predictors};
use crate::framework::capabilities::{CapabilitiesRequest, CapabilitiesResponse};
use crate::framework::delivery::{
    DataReadyAcknowledgement, DataReadyNotification, DataReceivedAcknowledgement, DataSupplyRequest,
};
use crate::framework::discovery::{
    LinesDelivery, LinesRequest, ProductCategoriesDelivery, ProductCategoriesRequest,
    ServiceFeaturesDelivery, ServiceFeaturesRequest, StopPointsDelivery, StopPointsRequest,
    VehicleFeaturesDelivery, VehicleFeaturesRequest,
};
use crate::framework::error_condition::{DeliveryError, ErrorCondition};
use crate::framework::status::{CheckStatusRequest, CheckStatusResponse, HeartbeatNotification};
use crate::framework::subscription::{
    SubscriptionRequest, SubscriptionResponse, SubscriptionTerminatedNotification,
    TerminateSubscriptionRequest, TerminateSubscriptionResponse,
};
use crate::cm::{
    ConnectionMonitoringDistributorDelivery, ConnectionMonitoringFeederDelivery,
    ConnectionMonitoringRequest,
};
use crate::ct::{ConnectionTimetableDelivery, ConnectionTimetableRequest};
use crate::et::{EstimatedTimetableDelivery, EstimatedTimetableRequest};
use crate::fm::{FacilityMonitoringDelivery, FacilityMonitoringRequest};
use crate::gm::{GeneralMessageDelivery, GeneralMessageRequest};
use crate::pt::{ProductionTimetableDelivery, ProductionTimetableRequest};
use crate::sm::{StopMonitoringDelivery, StopMonitoringMultipleRequest, StopMonitoringRequest};
use crate::st::{StopTimetableDelivery, StopTimetableRequest};
use crate::sx::{SituationExchangeDelivery, SituationExchangeRequest};
use crate::vm::{VehicleMonitoringDelivery, VehicleMonitoringRequest};
use crate::types::{
    Duration, EndpointAddress, Empty, Extensions, MessageQualifier, MessageRef, ParticipantRef,
};
use crate::xml::SiriRoot;

/// The document element every SIRI exchange is wrapped in.
///
/// A SIRI document carries exactly one message; which one is the [`SiriPayload`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Siri {
    /// Version of SIRI the document conforms to, e.g. `2.1`.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// The message this document carries.
    #[serde(rename = "$value")]
    pub payload: SiriPayload,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl Siri {
    /// Wraps a message in a `<Siri>` envelope declaring the given SIRI version.
    pub fn new(version: impl Into<String>, payload: impl Into<SiriPayload>) -> Self {
        Self {
            version: Some(version.into()),
            payload: payload.into(),
            extensions: None,
        }
    }
}

impl SiriRoot for Siri {
    const ELEMENT_NAME: &'static str = "Siri";
}

/// Declares the payload enum together with a `From` impl and an accessor per variant.
macro_rules! siri_payload {
    ($($(#[$meta:meta])* $variant:ident($ty:ty) => $accessor:ident),* $(,)?) => {
        /// The message a [`Siri`] document carries.
        ///
        /// The variants are the global elements the schema allows directly under
        /// `<Siri>`; the variant name is the element name.
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub enum SiriPayload {
            $($(#[$meta])* $variant($ty),)*
        }

        impl SiriPayload {
            $(
                #[doc = concat!("The payload as a `", stringify!($variant), "`, or `None` if it is another message.")]
                pub fn $accessor(&self) -> Option<&$ty> {
                    match self {
                        Self::$variant(inner) => Some(inner),
                        _ => None,
                    }
                }
            )*
        }

        $(
            impl From<$ty> for SiriPayload {
                fn from(inner: $ty) -> Self {
                    Self::$variant(inner)
                }
            }
        )*
    };
}

siri_payload! {
    /// A direct request for data from one or more functional services.
    ServiceRequest(ServiceRequest) => as_service_request,
    /// A request to open one or more subscriptions.
    SubscriptionRequest(SubscriptionRequest) => as_subscription_request,
    /// A request to close subscriptions.
    TerminateSubscriptionRequest(TerminateSubscriptionRequest) => as_terminate_subscription_request,
    /// A producer telling a consumer that data is waiting to be fetched.
    DataReadyNotification(DataReadyNotification) => as_data_ready_notification,
    /// A consumer fetching the data a producer announced.
    DataSupplyRequest(DataSupplyRequest) => as_data_supply_request,
    /// A request for the current operational status of a service.
    CheckStatusRequest(CheckStatusRequest) => as_check_status_request,
    /// A producer's unsolicited sign of life.
    HeartbeatNotification(HeartbeatNotification) => as_heartbeat_notification,
    /// A request for the capabilities of one or more functional services.
    CapabilitiesRequest(CapabilitiesRequest) => as_capabilities_request,
    /// A request for the stop points a producer serves.
    StopPointsRequest(StopPointsRequest) => as_stop_points_request,
    /// A request for the lines a producer serves.
    LinesRequest(LinesRequest) => as_lines_request,
    /// A request for the service features a producer uses.
    ServiceFeaturesRequest(ServiceFeaturesRequest) => as_service_features_request,
    /// A request for the vehicle features a producer uses.
    VehicleFeaturesRequest(VehicleFeaturesRequest) => as_vehicle_features_request,
    /// A request for the product categories a producer uses.
    ProductCategoriesRequest(ProductCategoriesRequest) => as_product_categories_request,
    /// The outcome of a subscription request, one status per subscription.
    SubscriptionResponse(SubscriptionResponse) => as_subscription_response,
    /// The outcome of a termination request, one status per subscription.
    TerminateSubscriptionResponse(TerminateSubscriptionResponse) => as_terminate_subscription_response,
    /// A producer telling a consumer that it has ended a subscription.
    SubscriptionTerminatedNotification(SubscriptionTerminatedNotification) => as_subscription_terminated_notification,
    /// A consumer acknowledging a data-ready notification.
    DataReadyAcknowledgement(DataReadyAcknowledgement) => as_data_ready_acknowledgement,
    /// Payload data, either answering a request or satisfying a subscription.
    ServiceDelivery(ServiceDelivery) => as_service_delivery,
    /// A consumer acknowledging a delivery.
    DataReceivedAcknowledgement(DataReceivedAcknowledgement) => as_data_received_acknowledgement,
    /// The current operational status of a service.
    CheckStatusResponse(CheckStatusResponse) => as_check_status_response,
    /// The capabilities of one or more functional services.
    CapabilitiesResponse(CapabilitiesResponse) => as_capabilities_response,
    /// The stop points a producer serves.
    StopPointsDelivery(StopPointsDelivery) => as_stop_points_delivery,
    /// The lines a producer serves.
    LinesDelivery(LinesDelivery) => as_lines_delivery,
    /// The service features a producer uses.
    ServiceFeaturesDelivery(ServiceFeaturesDelivery) => as_service_features_delivery,
    /// The vehicle features a producer uses.
    VehicleFeaturesDelivery(VehicleFeaturesDelivery) => as_vehicle_features_delivery,
    /// The product categories a producer uses.
    ProductCategoriesDelivery(ProductCategoriesDelivery) => as_product_categories_delivery,
}

/// A direct request for data, answered by a [`ServiceDelivery`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceRequest {
    /// Defaults the requestor asks the producer to apply to the whole exchange.
    #[serde(rename = "ServiceRequestContext", default, skip_serializing_if = "Option::is_none")]
    pub service_request_context: Option<ServiceRequestContext>,
    /// When the request was made.
    #[serde(rename = "RequestTimestamp")]
    pub request_timestamp: DateTime<FixedOffset>,
    /// Account the requestor authenticates as.
    #[serde(rename = "AccountId", default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    /// Secret authenticating the account.
    #[serde(rename = "AccountKey", default, skip_serializing_if = "Option::is_none")]
    pub account_key: Option<String>,
    /// Address to send the answer to.
    #[serde(rename = "Address", default, skip_serializing_if = "Option::is_none")]
    pub address: Option<EndpointAddress>,
    /// Who is asking.
    #[serde(rename = "RequestorRef")]
    pub requestor_ref: ParticipantRef,
    /// Identifier the requestor puts on this message.
    #[serde(rename = "MessageIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub message_identifier: Option<MessageQualifier>,
    /// Address of the participant this request is made on behalf of.
    #[serde(rename = "DelegatorAddress", default, skip_serializing_if = "Option::is_none")]
    pub delegator_address: Option<EndpointAddress>,
    /// Participant this request is made on behalf of.
    #[serde(rename = "DelegatorRef", default, skip_serializing_if = "Option::is_none")]
    pub delegator_ref: Option<ParticipantRef>,
    /// The functional service requests, all of the same service.
    #[serde(rename = "$value")]
    pub requests: Vec<ServiceRequestPayload>,
}

impl ServiceRequest {
    /// A request from `requestor_ref` carrying the given functional service requests.
    pub fn new(
        request_timestamp: DateTime<FixedOffset>,
        requestor_ref: impl Into<ParticipantRef>,
        requests: Vec<ServiceRequestPayload>,
    ) -> Self {
        Self {
            service_request_context: None,
            request_timestamp,
            account_id: None,
            account_key: None,
            address: None,
            requestor_ref: requestor_ref.into(),
            message_identifier: None,
            delegator_address: None,
            delegator_ref: None,
            requests,
        }
    }
}

/// A functional service request inside a [`ServiceRequest`].
///
/// The schema requires every request in one `ServiceRequest` to address the same
/// functional service. The enum is non-exhaustive so that the services this crate
/// does not implement yet can be added without a breaking change.
///
/// The requests differ widely in size — a vehicle-monitoring request is a handful
/// of fields, an estimated-timetable one several dozen — so each is boxed, keeping
/// a `Vec` of them from costing the largest variant per element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ServiceRequestPayload {
    /// A request for a planned timetable.
    ProductionTimetableRequest(Box<ProductionTimetableRequest>),
    /// A request for a real-time timetable.
    EstimatedTimetableRequest(Box<EstimatedTimetableRequest>),
    /// A request for the timetable at one stop.
    StopTimetableRequest(Box<StopTimetableRequest>),
    /// A request for what is due at one stop.
    StopMonitoringRequest(Box<StopMonitoringRequest>),
    /// A request for what is due at several stops at once.
    StopMonitoringMultipleRequest(Box<StopMonitoringMultipleRequest>),
    /// A request for vehicle positions.
    VehicleMonitoringRequest(Box<VehicleMonitoringRequest>),
    /// A request for the connections planned over a connection link.
    ConnectionTimetableRequest(Box<ConnectionTimetableRequest>),
    /// A request for how the connections over a connection link are going.
    ConnectionMonitoringRequest(Box<ConnectionMonitoringRequest>),
    /// A request for free-form messages.
    GeneralMessageRequest(Box<GeneralMessageRequest>),
    /// A request for the state of passenger facilities.
    FacilityMonitoringRequest(Box<FacilityMonitoringRequest>),
    /// A request for situations.
    SituationExchangeRequest(Box<SituationExchangeRequest>),
}

impl From<ProductionTimetableRequest> for ServiceRequestPayload {
    fn from(request: ProductionTimetableRequest) -> Self {
        Self::ProductionTimetableRequest(Box::new(request))
    }
}

impl From<EstimatedTimetableRequest> for ServiceRequestPayload {
    fn from(request: EstimatedTimetableRequest) -> Self {
        Self::EstimatedTimetableRequest(Box::new(request))
    }
}

impl From<StopTimetableRequest> for ServiceRequestPayload {
    fn from(request: StopTimetableRequest) -> Self {
        Self::StopTimetableRequest(Box::new(request))
    }
}

impl From<StopMonitoringRequest> for ServiceRequestPayload {
    fn from(request: StopMonitoringRequest) -> Self {
        Self::StopMonitoringRequest(Box::new(request))
    }
}

impl From<StopMonitoringMultipleRequest> for ServiceRequestPayload {
    fn from(request: StopMonitoringMultipleRequest) -> Self {
        Self::StopMonitoringMultipleRequest(Box::new(request))
    }
}

impl From<VehicleMonitoringRequest> for ServiceRequestPayload {
    fn from(request: VehicleMonitoringRequest) -> Self {
        Self::VehicleMonitoringRequest(Box::new(request))
    }
}

impl From<ConnectionTimetableRequest> for ServiceRequestPayload {
    fn from(request: ConnectionTimetableRequest) -> Self {
        Self::ConnectionTimetableRequest(Box::new(request))
    }
}

impl From<ConnectionMonitoringRequest> for ServiceRequestPayload {
    fn from(request: ConnectionMonitoringRequest) -> Self {
        Self::ConnectionMonitoringRequest(Box::new(request))
    }
}

impl From<GeneralMessageRequest> for ServiceRequestPayload {
    fn from(request: GeneralMessageRequest) -> Self {
        Self::GeneralMessageRequest(Box::new(request))
    }
}

impl From<FacilityMonitoringRequest> for ServiceRequestPayload {
    fn from(request: FacilityMonitoringRequest) -> Self {
        Self::FacilityMonitoringRequest(Box::new(request))
    }
}

impl From<SituationExchangeRequest> for ServiceRequestPayload {
    fn from(request: SituationExchangeRequest) -> Self {
        Self::SituationExchangeRequest(Box::new(request))
    }
}

/// Payload data from a producer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceDelivery {
    /// Name of the spatial reference system positions in this delivery use.
    #[serde(rename = "@srsName", default, skip_serializing_if = "Option::is_none")]
    pub srs_name: Option<String>,
    /// When the delivery was made.
    #[serde(rename = "ResponseTimestamp")]
    pub response_timestamp: DateTime<FixedOffset>,
    /// Who produced the data.
    #[serde(rename = "ProducerRef", default, skip_serializing_if = "Option::is_none")]
    pub producer_ref: Option<ParticipantRef>,
    /// Address of the producer.
    #[serde(rename = "Address", default, skip_serializing_if = "Option::is_none")]
    pub address: Option<EndpointAddress>,
    /// Identifier the producer puts on this message.
    #[serde(rename = "ResponseMessageIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub response_message_identifier: Option<MessageQualifier>,
    /// The request this delivery answers.
    #[serde(rename = "RequestMessageRef", default, skip_serializing_if = "Option::is_none")]
    pub request_message_ref: Option<MessageRef>,
    /// Address of the participant the data is delivered on behalf of.
    #[serde(rename = "DelegatorAddress", default, skip_serializing_if = "Option::is_none")]
    pub delegator_address: Option<EndpointAddress>,
    /// Participant the data is delivered on behalf of.
    #[serde(rename = "DelegatorRef", default, skip_serializing_if = "Option::is_none")]
    pub delegator_ref: Option<ParticipantRef>,
    /// Whether the request was processed successfully.
    #[serde(rename = "Status", default, skip_serializing_if = "Option::is_none")]
    pub status: Option<bool>,
    /// Why the request could not be processed.
    #[serde(rename = "ErrorCondition", default, skip_serializing_if = "Option::is_none")]
    pub error_condition: Option<ErrorCondition<DeliveryError>>,
    /// Whether further parts of this delivery follow.
    #[serde(rename = "MoreData", default, skip_serializing_if = "Option::is_none")]
    pub more_data: Option<bool>,
    /// The functional service deliveries.
    #[serde(rename = "$value")]
    pub deliveries: Vec<ServiceDeliveryPayload>,
}

impl ServiceDelivery {
    /// A successful delivery carrying the given functional service deliveries.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        producer_ref: impl Into<ParticipantRef>,
        deliveries: Vec<ServiceDeliveryPayload>,
    ) -> Self {
        Self {
            srs_name: None,
            response_timestamp,
            producer_ref: Some(producer_ref.into()),
            address: None,
            response_message_identifier: None,
            request_message_ref: None,
            delegator_address: None,
            delegator_ref: None,
            status: None,
            error_condition: None,
            more_data: None,
            deliveries,
        }
    }
}

/// A functional service delivery inside a [`ServiceDelivery`].
///
/// Non-exhaustive, and boxed, for the same reasons as [`ServiceRequestPayload`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ServiceDeliveryPayload {
    /// A planned timetable from the Production Timetable service.
    ProductionTimetableDelivery(Box<ProductionTimetableDelivery>),
    /// A real-time timetable from the Estimated Timetable service.
    EstimatedTimetableDelivery(Box<EstimatedTimetableDelivery>),
    /// A timetable at a stop from the Stop Timetable service.
    StopTimetableDelivery(Box<StopTimetableDelivery>),
    /// A departure board from the Stop Monitoring service.
    StopMonitoringDelivery(Box<StopMonitoringDelivery>),
    /// Vehicle positions from the Vehicle Monitoring service.
    VehicleMonitoringDelivery(Box<VehicleMonitoringDelivery>),
    /// Planned connections from the Connection Timetable service.
    ConnectionTimetableDelivery(Box<ConnectionTimetableDelivery>),
    /// Feeder arrivals from the Connection Monitoring service.
    ConnectionMonitoringFeederDelivery(Box<ConnectionMonitoringFeederDelivery>),
    /// Distributor decisions from the Connection Monitoring service.
    ConnectionMonitoringDistributorDelivery(Box<ConnectionMonitoringDistributorDelivery>),
    /// Free-form messages from the General Message service.
    GeneralMessageDelivery(Box<GeneralMessageDelivery>),
    /// Facility states from the Facility Monitoring service.
    FacilityMonitoringDelivery(Box<FacilityMonitoringDelivery>),
    /// Situations from the Situation Exchange service.
    SituationExchangeDelivery(Box<SituationExchangeDelivery>),
}

impl From<ProductionTimetableDelivery> for ServiceDeliveryPayload {
    fn from(delivery: ProductionTimetableDelivery) -> Self {
        Self::ProductionTimetableDelivery(Box::new(delivery))
    }
}

impl From<EstimatedTimetableDelivery> for ServiceDeliveryPayload {
    fn from(delivery: EstimatedTimetableDelivery) -> Self {
        Self::EstimatedTimetableDelivery(Box::new(delivery))
    }
}

impl From<StopTimetableDelivery> for ServiceDeliveryPayload {
    fn from(delivery: StopTimetableDelivery) -> Self {
        Self::StopTimetableDelivery(Box::new(delivery))
    }
}

impl From<StopMonitoringDelivery> for ServiceDeliveryPayload {
    fn from(delivery: StopMonitoringDelivery) -> Self {
        Self::StopMonitoringDelivery(Box::new(delivery))
    }
}

impl From<VehicleMonitoringDelivery> for ServiceDeliveryPayload {
    fn from(delivery: VehicleMonitoringDelivery) -> Self {
        Self::VehicleMonitoringDelivery(Box::new(delivery))
    }
}

impl From<ConnectionTimetableDelivery> for ServiceDeliveryPayload {
    fn from(delivery: ConnectionTimetableDelivery) -> Self {
        Self::ConnectionTimetableDelivery(Box::new(delivery))
    }
}

impl From<ConnectionMonitoringFeederDelivery> for ServiceDeliveryPayload {
    fn from(delivery: ConnectionMonitoringFeederDelivery) -> Self {
        Self::ConnectionMonitoringFeederDelivery(Box::new(delivery))
    }
}

impl From<ConnectionMonitoringDistributorDelivery> for ServiceDeliveryPayload {
    fn from(delivery: ConnectionMonitoringDistributorDelivery) -> Self {
        Self::ConnectionMonitoringDistributorDelivery(Box::new(delivery))
    }
}

impl From<GeneralMessageDelivery> for ServiceDeliveryPayload {
    fn from(delivery: GeneralMessageDelivery) -> Self {
        Self::GeneralMessageDelivery(Box::new(delivery))
    }
}

impl From<FacilityMonitoringDelivery> for ServiceDeliveryPayload {
    fn from(delivery: FacilityMonitoringDelivery) -> Self {
        Self::FacilityMonitoringDelivery(Box::new(delivery))
    }
}

impl From<SituationExchangeDelivery> for ServiceDeliveryPayload {
    fn from(delivery: SituationExchangeDelivery) -> Self {
        Self::SituationExchangeDelivery(Box::new(delivery))
    }
}

/// Defaults a requestor asks a producer to apply to a whole exchange.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ServiceRequestContext {
    /// Where to send check-status requests.
    #[serde(rename = "CheckStatusAddress", default, skip_serializing_if = "Option::is_none")]
    pub check_status_address: Option<EndpointAddress>,
    /// Where to send subscription requests.
    #[serde(rename = "SubscribeAddress", default, skip_serializing_if = "Option::is_none")]
    pub subscribe_address: Option<EndpointAddress>,
    /// Where to send subscription management requests.
    #[serde(rename = "ManageSubscriptionAddress", default, skip_serializing_if = "Option::is_none")]
    pub manage_subscription_address: Option<EndpointAddress>,
    /// Where to fetch data from.
    #[serde(rename = "GetDataAddress", default, skip_serializing_if = "Option::is_none")]
    pub get_data_address: Option<EndpointAddress>,
    /// Where the client receives status responses.
    #[serde(rename = "StatusResponseAddress", default, skip_serializing_if = "Option::is_none")]
    pub status_response_address: Option<EndpointAddress>,
    /// Where the client receives subscription responses.
    #[serde(rename = "SubscriberAddress", default, skip_serializing_if = "Option::is_none")]
    pub subscriber_address: Option<EndpointAddress>,
    /// Where the client receives notifications.
    #[serde(rename = "NotifyAddress", default, skip_serializing_if = "Option::is_none")]
    pub notify_address: Option<EndpointAddress>,
    /// Where the client receives deliveries.
    #[serde(rename = "ConsumerAddress", default, skip_serializing_if = "Option::is_none")]
    pub consumer_address: Option<EndpointAddress>,
    /// Namespaces the identifiers in this exchange are drawn from.
    #[serde(rename = "DataNameSpaces", default, skip_serializing_if = "Option::is_none")]
    pub data_name_spaces: Option<DataNameSpaces>,
    /// Languages the requestor would like texts in, most preferred first.
    #[serde(rename = "Language", default, skip_serializing_if = "Vec::is_empty")]
    pub language: Vec<String>,
    /// Requests that positions be given as WGS 84 decimal degrees.
    #[serde(rename = "WgsDecimalDegrees", default, skip_serializing_if = "Option::is_none")]
    pub wgs_decimal_degrees: Option<Empty>,
    /// Requests that positions be given in the named GML coordinate system.
    #[serde(rename = "GmlCoordinateFormat", default, skip_serializing_if = "Option::is_none")]
    pub gml_coordinate_format: Option<String>,
    /// Units distances should be given in.
    #[serde(rename = "DistanceUnits", default, skip_serializing_if = "Option::is_none")]
    pub distance_units: Option<String>,
    /// Units velocities should be given in.
    #[serde(rename = "VelocityUnits", default, skip_serializing_if = "Option::is_none")]
    pub velocity_units: Option<String>,
    /// How far into the future the requestor is interested in data.
    #[serde(rename = "DataHorizon", default, skip_serializing_if = "Option::is_none")]
    pub data_horizon: Option<Duration>,
    /// How long the requestor will wait for an answer.
    #[serde(rename = "RequestTimeout", default, skip_serializing_if = "Option::is_none")]
    pub request_timeout: Option<Duration>,
    /// Whether deliveries are pushed directly or fetched after a notification.
    #[serde(rename = "DeliveryMethod", default, skip_serializing_if = "Option::is_none")]
    pub delivery_method: Option<DeliveryMethod>,
    /// Whether a delivery may be split over several messages.
    #[serde(rename = "MultipartDespatch", default, skip_serializing_if = "Option::is_none")]
    pub multipart_despatch: Option<bool>,
    /// Whether the consumer will acknowledge each delivery.
    #[serde(rename = "ConfirmDelivery", default, skip_serializing_if = "Option::is_none")]
    pub confirm_delivery: Option<bool>,
    /// How many subscriptions the requestor may hold at once.
    #[serde(rename = "MaximimumNumberOfSubscriptions", default, skip_serializing_if = "Option::is_none")]
    pub maximimum_number_of_subscriptions: Option<u64>,
    /// Which sources of prediction the requestor accepts.
    #[serde(rename = "AllowedPredictors", default, skip_serializing_if = "Option::is_none")]
    pub allowed_predictors: Option<Predictors>,
    /// Name of the prediction function the requestor asks for.
    #[serde(rename = "PredictionFunction", default, skip_serializing_if = "Option::is_none")]
    pub prediction_function: Option<String>,
}

/// Namespaces the identifiers in an exchange are drawn from.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataNameSpaces {
    /// Namespace of stop point identifiers.
    #[serde(rename = "StopPointNameSpace", default, skip_serializing_if = "Option::is_none")]
    pub stop_point_name_space: Option<String>,
    /// Namespace of line identifiers.
    #[serde(rename = "LineNameSpace", default, skip_serializing_if = "Option::is_none")]
    pub line_name_space: Option<String>,
    /// Namespace of product category identifiers.
    #[serde(rename = "ProductCategoryNameSpace", default, skip_serializing_if = "Option::is_none")]
    pub product_category_name_space: Option<String>,
    /// Namespace of service feature identifiers.
    #[serde(rename = "ServiceFeatureNameSpace", default, skip_serializing_if = "Option::is_none")]
    pub service_feature_name_space: Option<String>,
    /// Namespace of vehicle feature identifiers.
    #[serde(rename = "VehicleFeatureNameSpace", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_feature_name_space: Option<String>,
}
