//! What the hub needs to know about a functional service.
//!
//! The subscription conversation is the same whichever service carries the payload:
//! the messages that open, keep and close a subscription are framework messages, and
//! only the request, the delivery and the records inside it differ. [`Service`] is
//! that difference, stated once per service, so that [`Producer`](super::Producer)
//! and [`Consumer`](super::Consumer) can drive any of them.
//!
//! An application does not implement [`Service`] — the crate does, once per service
//! it models. What an application implements is a source: [`SituationSource`],
//! [`StopMonitoringSource`], [`FacilityMonitoringSource`] and so on, depending on
//! what it publishes.
//!
//! One delivery is not carried here: the distributor half of Connection Monitoring
//! answers with three kinds of decision rather than one kind of record — see
//! [`ConnectionMonitoringFeeder`].

use std::fmt;

use chrono::{DateTime, FixedOffset};

use crate::cm::{
    ConnectionMonitoringFeederDelivery, ConnectionMonitoringRequest,
    ConnectionMonitoringSubscriptionRequest, MonitoredFeederArrival,
};
use crate::ct::{
    ConnectionTimetableDelivery, ConnectionTimetableRequest,
    ConnectionTimetableSubscriptionRequest, TimetabledFeederArrival,
};
use crate::et::{
    EstimatedTimetableDelivery, EstimatedTimetableRequest, EstimatedTimetableSubscriptionRequest,
};
use crate::fm::{
    FacilityMonitoringDelivery, FacilityMonitoringRequest, FacilityMonitoringSubscriptionRequest,
};
use crate::framework::{ServiceDeliveryPayload, ServiceRequestPayload, SubscriptionRequestPayload};
use crate::gm::{
    GeneralMessageDelivery, GeneralMessageRequest, GeneralMessageSubscriptionRequest, InfoMessage,
};
use crate::model::{EstimatedVehicleJourney, FacilityCondition};
use crate::pt::{
    DatedTimetableVersionFrame, ProductionTimetableDelivery, ProductionTimetableRequest,
    ProductionTimetableSubscriptionRequest,
};
use crate::sm::{
    MonitoredStopVisit, StopMonitoringDelivery, StopMonitoringRequest,
    StopMonitoringSubscriptionRequest,
};
use crate::st::{
    StopTimetableDelivery, StopTimetableRequest, StopTimetableSubscriptionRequest,
    TimetabledStopVisit,
};
use crate::sx::{
    PtSituationElement, SituationExchangeDelivery, SituationExchangeRequest,
    SituationExchangeSubscriptionRequest,
};
use crate::types::{ParticipantRef, SubscriptionQualifier, SubscriptionRef};
use crate::vm::{
    VehicleActivity, VehicleMonitoringDelivery, VehicleMonitoringRequest,
    VehicleMonitoringSubscriptionRequest,
};

/// One SIRI functional service, as the publish/subscribe hub sees it.
///
/// Implemented by this crate for each service it models; the marker types
/// [`SituationExchange`], [`ProductionTimetable`], [`EstimatedTimetable`],
/// [`StopTimetable`], [`StopMonitoring`], [`VehicleMonitoring`],
/// [`ConnectionTimetable`], [`ConnectionMonitoringFeeder`], [`GeneralMessage`] and
/// [`FacilityMonitoring`] are the implementations.
///
/// The trait is sealed: the set of services is the standard's, and a further one is
/// added here without that being a breaking change for anyone. An implementation
/// outside the crate does not compile, however complete it is:
///
/// ```compile_fail,E0277
/// use chrono::{DateTime, FixedOffset};
/// use siri_rs::framework::{
///     ServiceDeliveryPayload, ServiceRequestPayload, SubscriptionRequestPayload,
/// };
/// use siri_rs::pubsub::{Service, SubscriptionParts};
/// use siri_rs::types::{ParticipantRef, SubscriptionRef};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// struct Bespoke;
///
/// impl Service for Bespoke {
///     type Request = ();
///     type Delivery = ();
///     type Item = ();
///
///     fn service_request(_: ()) -> ServiceRequestPayload {
///         unimplemented!()
///     }
///     fn request_of(_: &ServiceRequestPayload) -> Option<&()> {
///         unimplemented!()
///     }
///     fn subscription_request(_: SubscriptionParts<Self>) -> SubscriptionRequestPayload {
///         unimplemented!()
///     }
///     fn subscription_of(_: &SubscriptionRequestPayload) -> Option<SubscriptionParts<Self>> {
///         unimplemented!()
///     }
///     fn delivery(_: DateTime<FixedOffset>, _: Vec<()>) {
///         unimplemented!()
///     }
///     fn attribute(_: &mut (), _: ParticipantRef, _: SubscriptionRef) {
///         unimplemented!()
///     }
///     fn service_delivery(_: ()) -> ServiceDeliveryPayload {
///         unimplemented!()
///     }
///     fn items_of(_: &ServiceDeliveryPayload) -> Option<Vec<()>> {
///         unimplemented!()
///     }
/// }
/// ```
pub trait Service: Sized + Copy + fmt::Debug + Eq {
    /// What a consumer asks for.
    type Request: Clone + fmt::Debug + PartialEq;
    /// What a producer answers with.
    type Delivery;
    /// One record inside a delivery.
    type Item: Clone + fmt::Debug + PartialEq;

    /// Wraps a request as the element a `ServiceRequest` carries.
    fn service_request(request: Self::Request) -> ServiceRequestPayload;

    /// The request inside a `ServiceRequest` element, when it is this service's.
    fn request_of(payload: &ServiceRequestPayload) -> Option<&Self::Request>;

    /// Wraps the parts of a subscription as the element a `SubscriptionRequest`
    /// carries.
    fn subscription_request(parts: SubscriptionParts<Self>) -> SubscriptionRequestPayload;

    /// The parts of a subscription inside a `SubscriptionRequest` element, when it
    /// is this service's.
    fn subscription_of(payload: &SubscriptionRequestPayload) -> Option<SubscriptionParts<Self>>;

    /// A delivery carrying the given records.
    fn delivery(
        response_timestamp: DateTime<FixedOffset>,
        items: Vec<Self::Item>,
    ) -> Self::Delivery;

    /// Marks a delivery as satisfying a subscription rather than answering a request.
    fn attribute(
        delivery: &mut Self::Delivery,
        subscriber_ref: ParticipantRef,
        subscription_ref: SubscriptionRef,
    );

    /// Wraps a delivery as the element a `ServiceDelivery` carries.
    fn service_delivery(delivery: Self::Delivery) -> ServiceDeliveryPayload;

    /// The records inside a `ServiceDelivery` element, when it is this service's.
    fn items_of(payload: &ServiceDeliveryPayload) -> Option<Vec<Self::Item>>;
}

/// The parts of a subscription request the hub works with.
///
/// Every service's subscription element carries these; what else it carries is the
/// service's own business and is left to whoever builds the element by hand.
#[derive(Debug, Clone, PartialEq)]
pub struct SubscriptionParts<S: Service> {
    /// Who is subscribing, when different from the requestor of the enclosing message.
    pub subscriber_ref: Option<ParticipantRef>,
    /// The subscriber's name for this subscription.
    pub subscription_identifier: SubscriptionQualifier,
    /// When the subscription lapses unless renewed.
    pub initial_termination_time: DateTime<FixedOffset>,
    /// What was subscribed to.
    pub request: S::Request,
    /// Whether the consumer asked for changes only.
    pub incremental_updates: Option<bool>,
}

/// Where a producer gets the records it publishes, for one service.
///
/// Applications implement one of the per-service traits below rather than this one;
/// each of those gives its implementors a [`Source`] for the matching service.
pub trait Source<S: Service> {
    /// The records matching `request`.
    ///
    /// Applying the request's filters is the implementation's job — it knows its own
    /// data. A source that ignores a filter simply publishes more than was asked
    /// for, which the schema allows but consumers will not thank it for.
    fn items(&self, request: &S::Request) -> Vec<S::Item>;
}

/// Where a producer gets the situations it publishes.
///
/// Implementing this is the only thing an application has to do to become a SIRI-SX
/// producer; everything else about the exchange is handled by
/// [`Producer`](super::Producer).
pub trait SituationSource {
    /// The situations matching `request`.
    fn situations(&self, request: &SituationExchangeRequest) -> Vec<PtSituationElement>;
}

/// Where a producer gets the real-time journeys it publishes.
pub trait EstimatedTimetableSource {
    /// The journeys matching `request`.
    fn journeys(&self, request: &EstimatedTimetableRequest) -> Vec<EstimatedVehicleJourney>;
}

/// Where a producer gets the planned timetable it publishes.
pub trait ProductionTimetableSource {
    /// The timetable frames matching `request`.
    fn frames(&self, request: &ProductionTimetableRequest) -> Vec<DatedTimetableVersionFrame>;
}

/// Where a producer gets the vehicle positions it publishes.
pub trait VehicleMonitoringSource {
    /// The vehicles matching `request`.
    fn vehicles(&self, request: &VehicleMonitoringRequest) -> Vec<VehicleActivity>;
}

/// Where a producer gets the timetabled visits to a stop it publishes.
pub trait StopTimetableSource {
    /// The visits matching `request`.
    fn visits(&self, request: &StopTimetableRequest) -> Vec<TimetabledStopVisit>;
}

/// Where a producer gets the departure board it publishes.
pub trait StopMonitoringSource {
    /// The visits matching `request`.
    fn visits(&self, request: &StopMonitoringRequest) -> Vec<MonitoredStopVisit>;
}

/// Where a producer gets the planned interchanges it publishes.
pub trait ConnectionTimetableSource {
    /// The planned feeder arrivals matching `request`.
    fn arrivals(&self, request: &ConnectionTimetableRequest) -> Vec<TimetabledFeederArrival>;
}

/// Where the feeder side of an interchange gets the arrivals it publishes.
pub trait ConnectionMonitoringFeederSource {
    /// The arrivals matching `request`.
    fn arrivals(&self, request: &ConnectionMonitoringRequest) -> Vec<MonitoredFeederArrival>;
}

/// Where a producer gets the messages it publishes.
pub trait GeneralMessageSource {
    /// The messages matching `request`.
    fn messages(&self, request: &GeneralMessageRequest) -> Vec<InfoMessage>;
}

/// Where a producer gets the state of the facilities it publishes.
pub trait FacilityMonitoringSource {
    /// The facility conditions matching `request`.
    fn conditions(&self, request: &FacilityMonitoringRequest) -> Vec<FacilityCondition>;
}

/// The Situation Exchange service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SituationExchange;

/// The Estimated Timetable service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EstimatedTimetable;

/// The Production Timetable service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProductionTimetable;

/// The Vehicle Monitoring service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VehicleMonitoring;

/// The Stop Timetable service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StopTimetable;

/// The Stop Monitoring service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StopMonitoring;

/// The Connection Timetable service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionTimetable;

/// The feeder half of the Connection Monitoring service.
///
/// A connection-monitoring producer runs one side of an interchange. The feeder
/// side publishes arrivals, one record per feeder, which is the shape the hub
/// carries. The distributor side publishes decisions instead — that it will wait,
/// that it has moved, that it is not going — three kinds of record in one delivery,
/// so a distributor builds its
/// [`ConnectionMonitoringDistributorDelivery`](crate::cm::ConnectionMonitoringDistributorDelivery)
/// itself and carries it in a `ServiceDelivery` the framework types provide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionMonitoringFeeder;

/// The General Message service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeneralMessage;

/// The Facility Monitoring service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FacilityMonitoring;

impl<T: SituationSource> Source<SituationExchange> for T {
    fn items(&self, request: &SituationExchangeRequest) -> Vec<PtSituationElement> {
        self.situations(request)
    }
}

impl<T: EstimatedTimetableSource> Source<EstimatedTimetable> for T {
    fn items(&self, request: &EstimatedTimetableRequest) -> Vec<EstimatedVehicleJourney> {
        self.journeys(request)
    }
}

impl<T: ProductionTimetableSource> Source<ProductionTimetable> for T {
    fn items(&self, request: &ProductionTimetableRequest) -> Vec<DatedTimetableVersionFrame> {
        self.frames(request)
    }
}

impl<T: VehicleMonitoringSource> Source<VehicleMonitoring> for T {
    fn items(&self, request: &VehicleMonitoringRequest) -> Vec<VehicleActivity> {
        self.vehicles(request)
    }
}

impl<T: StopTimetableSource> Source<StopTimetable> for T {
    fn items(&self, request: &StopTimetableRequest) -> Vec<TimetabledStopVisit> {
        self.visits(request)
    }
}

impl<T: StopMonitoringSource> Source<StopMonitoring> for T {
    fn items(&self, request: &StopMonitoringRequest) -> Vec<MonitoredStopVisit> {
        self.visits(request)
    }
}

impl<T: ConnectionTimetableSource> Source<ConnectionTimetable> for T {
    fn items(&self, request: &ConnectionTimetableRequest) -> Vec<TimetabledFeederArrival> {
        self.arrivals(request)
    }
}

impl<T: ConnectionMonitoringFeederSource> Source<ConnectionMonitoringFeeder> for T {
    fn items(&self, request: &ConnectionMonitoringRequest) -> Vec<MonitoredFeederArrival> {
        self.arrivals(request)
    }
}

impl<T: GeneralMessageSource> Source<GeneralMessage> for T {
    fn items(&self, request: &GeneralMessageRequest) -> Vec<InfoMessage> {
        self.messages(request)
    }
}

impl<T: FacilityMonitoringSource> Source<FacilityMonitoring> for T {
    fn items(&self, request: &FacilityMonitoringRequest) -> Vec<FacilityCondition> {
        self.conditions(request)
    }
}

impl Service for SituationExchange {
    type Request = SituationExchangeRequest;
    type Delivery = SituationExchangeDelivery;
    type Item = PtSituationElement;

    fn service_request(request: Self::Request) -> ServiceRequestPayload {
        ServiceRequestPayload::SituationExchangeRequest(Box::new(request))
    }

    fn request_of(payload: &ServiceRequestPayload) -> Option<&Self::Request> {
        match payload {
            ServiceRequestPayload::SituationExchangeRequest(request) => Some(request),
            _ => None,
        }
    }

    fn subscription_request(parts: SubscriptionParts<Self>) -> SubscriptionRequestPayload {
        SubscriptionRequestPayload::SituationExchangeSubscriptionRequest(Box::new(
            SituationExchangeSubscriptionRequest {
                subscriber_ref: parts.subscriber_ref,
                incremental_updates: parts.incremental_updates,
                ..SituationExchangeSubscriptionRequest::new(
                    parts.subscription_identifier,
                    parts.initial_termination_time,
                    parts.request,
                )
            },
        ))
    }

    fn subscription_of(payload: &SubscriptionRequestPayload) -> Option<SubscriptionParts<Self>> {
        match payload {
            SubscriptionRequestPayload::SituationExchangeSubscriptionRequest(subscription) => {
                Some(SubscriptionParts {
                    subscriber_ref: subscription.subscriber_ref.clone(),
                    subscription_identifier: subscription.subscription_identifier.clone(),
                    initial_termination_time: subscription.initial_termination_time,
                    request: subscription.situation_exchange_request.clone(),
                    incremental_updates: subscription.incremental_updates,
                })
            }
            _ => None,
        }
    }

    fn delivery(
        response_timestamp: DateTime<FixedOffset>,
        items: Vec<Self::Item>,
    ) -> Self::Delivery {
        SituationExchangeDelivery::new(response_timestamp, items)
    }

    fn attribute(
        delivery: &mut Self::Delivery,
        subscriber_ref: ParticipantRef,
        subscription_ref: SubscriptionRef,
    ) {
        delivery.subscriber_ref = Some(subscriber_ref);
        delivery.subscription_ref = Some(subscription_ref);
        delivery.status = Some(true);
    }

    fn service_delivery(delivery: Self::Delivery) -> ServiceDeliveryPayload {
        ServiceDeliveryPayload::SituationExchangeDelivery(Box::new(delivery))
    }

    fn items_of(payload: &ServiceDeliveryPayload) -> Option<Vec<Self::Item>> {
        match payload {
            ServiceDeliveryPayload::SituationExchangeDelivery(delivery) => {
                Some(delivery.pt_situations().to_vec())
            }
            _ => None,
        }
    }
}

impl Service for EstimatedTimetable {
    type Request = EstimatedTimetableRequest;
    type Delivery = EstimatedTimetableDelivery;
    type Item = EstimatedVehicleJourney;

    fn service_request(request: Self::Request) -> ServiceRequestPayload {
        ServiceRequestPayload::EstimatedTimetableRequest(Box::new(request))
    }

    fn request_of(payload: &ServiceRequestPayload) -> Option<&Self::Request> {
        match payload {
            ServiceRequestPayload::EstimatedTimetableRequest(request) => Some(request),
            _ => None,
        }
    }

    fn subscription_request(parts: SubscriptionParts<Self>) -> SubscriptionRequestPayload {
        SubscriptionRequestPayload::EstimatedTimetableSubscriptionRequest(Box::new(
            EstimatedTimetableSubscriptionRequest {
                subscriber_ref: parts.subscriber_ref,
                incremental_updates: parts.incremental_updates,
                ..EstimatedTimetableSubscriptionRequest::new(
                    parts.subscription_identifier,
                    parts.initial_termination_time,
                    parts.request,
                )
            },
        ))
    }

    fn subscription_of(payload: &SubscriptionRequestPayload) -> Option<SubscriptionParts<Self>> {
        match payload {
            SubscriptionRequestPayload::EstimatedTimetableSubscriptionRequest(subscription) => {
                Some(SubscriptionParts {
                    subscriber_ref: subscription.subscriber_ref.clone(),
                    subscription_identifier: subscription.subscription_identifier.clone(),
                    initial_termination_time: subscription.initial_termination_time,
                    request: subscription.estimated_timetable_request.clone(),
                    incremental_updates: subscription.incremental_updates,
                })
            }
            _ => None,
        }
    }

    /// The journeys go into one version frame, recorded at the delivery's own
    /// timestamp: a producer that needs several frames, or a stated timetable
    /// version, builds the delivery itself.
    fn delivery(
        response_timestamp: DateTime<FixedOffset>,
        items: Vec<Self::Item>,
    ) -> Self::Delivery {
        EstimatedTimetableDelivery::new(response_timestamp, items)
    }

    fn attribute(
        delivery: &mut Self::Delivery,
        subscriber_ref: ParticipantRef,
        subscription_ref: SubscriptionRef,
    ) {
        delivery.subscriber_ref = Some(subscriber_ref);
        delivery.subscription_ref = Some(subscription_ref);
        delivery.status = Some(true);
    }

    fn service_delivery(delivery: Self::Delivery) -> ServiceDeliveryPayload {
        ServiceDeliveryPayload::EstimatedTimetableDelivery(Box::new(delivery))
    }

    fn items_of(payload: &ServiceDeliveryPayload) -> Option<Vec<Self::Item>> {
        match payload {
            ServiceDeliveryPayload::EstimatedTimetableDelivery(delivery) => {
                Some(delivery.journeys().cloned().collect())
            }
            _ => None,
        }
    }
}

impl Service for ProductionTimetable {
    type Request = ProductionTimetableRequest;
    type Delivery = ProductionTimetableDelivery;
    type Item = DatedTimetableVersionFrame;

    fn service_request(request: Self::Request) -> ServiceRequestPayload {
        ServiceRequestPayload::ProductionTimetableRequest(Box::new(request))
    }

    fn request_of(payload: &ServiceRequestPayload) -> Option<&Self::Request> {
        match payload {
            ServiceRequestPayload::ProductionTimetableRequest(request) => Some(request),
            _ => None,
        }
    }

    fn subscription_request(parts: SubscriptionParts<Self>) -> SubscriptionRequestPayload {
        SubscriptionRequestPayload::ProductionTimetableSubscriptionRequest(Box::new(
            ProductionTimetableSubscriptionRequest {
                subscriber_ref: parts.subscriber_ref,
                incremental_updates: parts.incremental_updates,
                ..ProductionTimetableSubscriptionRequest::new(
                    parts.subscription_identifier,
                    parts.initial_termination_time,
                    parts.request,
                )
            },
        ))
    }

    fn subscription_of(payload: &SubscriptionRequestPayload) -> Option<SubscriptionParts<Self>> {
        match payload {
            SubscriptionRequestPayload::ProductionTimetableSubscriptionRequest(subscription) => {
                Some(SubscriptionParts {
                    subscriber_ref: subscription.subscriber_ref.clone(),
                    subscription_identifier: subscription.subscription_identifier.clone(),
                    initial_termination_time: subscription.initial_termination_time,
                    request: subscription.production_timetable_request.clone(),
                    incremental_updates: subscription.incremental_updates,
                })
            }
            _ => None,
        }
    }

    fn delivery(
        response_timestamp: DateTime<FixedOffset>,
        items: Vec<Self::Item>,
    ) -> Self::Delivery {
        ProductionTimetableDelivery::new(response_timestamp, items)
    }

    fn attribute(
        delivery: &mut Self::Delivery,
        subscriber_ref: ParticipantRef,
        subscription_ref: SubscriptionRef,
    ) {
        delivery.subscriber_ref = Some(subscriber_ref);
        delivery.subscription_ref = Some(subscription_ref);
        delivery.status = Some(true);
    }

    fn service_delivery(delivery: Self::Delivery) -> ServiceDeliveryPayload {
        ServiceDeliveryPayload::ProductionTimetableDelivery(Box::new(delivery))
    }

    fn items_of(payload: &ServiceDeliveryPayload) -> Option<Vec<Self::Item>> {
        match payload {
            ServiceDeliveryPayload::ProductionTimetableDelivery(delivery) => {
                Some(delivery.dated_timetable_version_frame.clone())
            }
            _ => None,
        }
    }
}

impl Service for VehicleMonitoring {
    type Request = VehicleMonitoringRequest;
    type Delivery = VehicleMonitoringDelivery;
    type Item = VehicleActivity;

    fn service_request(request: Self::Request) -> ServiceRequestPayload {
        ServiceRequestPayload::VehicleMonitoringRequest(Box::new(request))
    }

    fn request_of(payload: &ServiceRequestPayload) -> Option<&Self::Request> {
        match payload {
            ServiceRequestPayload::VehicleMonitoringRequest(request) => Some(request),
            _ => None,
        }
    }

    fn subscription_request(parts: SubscriptionParts<Self>) -> SubscriptionRequestPayload {
        SubscriptionRequestPayload::VehicleMonitoringSubscriptionRequest(Box::new(
            VehicleMonitoringSubscriptionRequest {
                subscriber_ref: parts.subscriber_ref,
                incremental_updates: parts.incremental_updates,
                ..VehicleMonitoringSubscriptionRequest::new(
                    parts.subscription_identifier,
                    parts.initial_termination_time,
                    parts.request,
                )
            },
        ))
    }

    fn subscription_of(payload: &SubscriptionRequestPayload) -> Option<SubscriptionParts<Self>> {
        match payload {
            SubscriptionRequestPayload::VehicleMonitoringSubscriptionRequest(subscription) => {
                Some(SubscriptionParts {
                    subscriber_ref: subscription.subscriber_ref.clone(),
                    subscription_identifier: subscription.subscription_identifier.clone(),
                    initial_termination_time: subscription.initial_termination_time,
                    request: subscription.vehicle_monitoring_request.clone(),
                    incremental_updates: subscription.incremental_updates,
                })
            }
            _ => None,
        }
    }

    fn delivery(
        response_timestamp: DateTime<FixedOffset>,
        items: Vec<Self::Item>,
    ) -> Self::Delivery {
        VehicleMonitoringDelivery::new(response_timestamp, items)
    }

    fn attribute(
        delivery: &mut Self::Delivery,
        subscriber_ref: ParticipantRef,
        subscription_ref: SubscriptionRef,
    ) {
        delivery.subscriber_ref = Some(subscriber_ref);
        delivery.subscription_ref = Some(subscription_ref);
        delivery.status = Some(true);
    }

    fn service_delivery(delivery: Self::Delivery) -> ServiceDeliveryPayload {
        ServiceDeliveryPayload::VehicleMonitoringDelivery(Box::new(delivery))
    }

    fn items_of(payload: &ServiceDeliveryPayload) -> Option<Vec<Self::Item>> {
        match payload {
            ServiceDeliveryPayload::VehicleMonitoringDelivery(delivery) => {
                Some(delivery.vehicle_activity.clone())
            }
            _ => None,
        }
    }
}

impl Service for StopTimetable {
    type Request = StopTimetableRequest;
    type Delivery = StopTimetableDelivery;
    type Item = TimetabledStopVisit;

    fn service_request(request: Self::Request) -> ServiceRequestPayload {
        ServiceRequestPayload::StopTimetableRequest(Box::new(request))
    }

    fn request_of(payload: &ServiceRequestPayload) -> Option<&Self::Request> {
        match payload {
            ServiceRequestPayload::StopTimetableRequest(request) => Some(request),
            _ => None,
        }
    }

    fn subscription_request(parts: SubscriptionParts<Self>) -> SubscriptionRequestPayload {
        SubscriptionRequestPayload::StopTimetableSubscriptionRequest(Box::new(
            StopTimetableSubscriptionRequest {
                subscriber_ref: parts.subscriber_ref,
                incremental_updates: parts.incremental_updates,
                ..StopTimetableSubscriptionRequest::new(
                    parts.subscription_identifier,
                    parts.initial_termination_time,
                    parts.request,
                )
            },
        ))
    }

    fn subscription_of(payload: &SubscriptionRequestPayload) -> Option<SubscriptionParts<Self>> {
        match payload {
            SubscriptionRequestPayload::StopTimetableSubscriptionRequest(subscription) => {
                Some(SubscriptionParts {
                    subscriber_ref: subscription.subscriber_ref.clone(),
                    subscription_identifier: subscription.subscription_identifier.clone(),
                    initial_termination_time: subscription.initial_termination_time,
                    request: subscription.stop_timetable_request.clone(),
                    incremental_updates: subscription.incremental_updates,
                })
            }
            _ => None,
        }
    }

    fn delivery(
        response_timestamp: DateTime<FixedOffset>,
        items: Vec<Self::Item>,
    ) -> Self::Delivery {
        StopTimetableDelivery::new(response_timestamp, items)
    }

    fn attribute(
        delivery: &mut Self::Delivery,
        subscriber_ref: ParticipantRef,
        subscription_ref: SubscriptionRef,
    ) {
        delivery.subscriber_ref = Some(subscriber_ref);
        delivery.subscription_ref = Some(subscription_ref);
        delivery.status = Some(true);
    }

    fn service_delivery(delivery: Self::Delivery) -> ServiceDeliveryPayload {
        ServiceDeliveryPayload::StopTimetableDelivery(Box::new(delivery))
    }

    fn items_of(payload: &ServiceDeliveryPayload) -> Option<Vec<Self::Item>> {
        match payload {
            ServiceDeliveryPayload::StopTimetableDelivery(delivery) => {
                Some(delivery.timetabled_stop_visit.clone())
            }
            _ => None,
        }
    }
}

impl Service for StopMonitoring {
    type Request = StopMonitoringRequest;
    type Delivery = StopMonitoringDelivery;
    type Item = MonitoredStopVisit;

    fn service_request(request: Self::Request) -> ServiceRequestPayload {
        ServiceRequestPayload::StopMonitoringRequest(Box::new(request))
    }

    /// The request for several stops at once is a message of its own and is not read
    /// here: a hub consumer subscribes to one monitoring point per subscription, and
    /// a producer answering the multiple form builds the deliveries itself.
    fn request_of(payload: &ServiceRequestPayload) -> Option<&Self::Request> {
        match payload {
            ServiceRequestPayload::StopMonitoringRequest(request) => Some(request),
            _ => None,
        }
    }

    fn subscription_request(parts: SubscriptionParts<Self>) -> SubscriptionRequestPayload {
        SubscriptionRequestPayload::StopMonitoringSubscriptionRequest(Box::new(
            StopMonitoringSubscriptionRequest {
                subscriber_ref: parts.subscriber_ref,
                incremental_updates: parts.incremental_updates,
                ..StopMonitoringSubscriptionRequest::new(
                    parts.subscription_identifier,
                    parts.initial_termination_time,
                    parts.request,
                )
            },
        ))
    }

    fn subscription_of(payload: &SubscriptionRequestPayload) -> Option<SubscriptionParts<Self>> {
        match payload {
            SubscriptionRequestPayload::StopMonitoringSubscriptionRequest(subscription) => {
                Some(SubscriptionParts {
                    subscriber_ref: subscription.subscriber_ref.clone(),
                    subscription_identifier: subscription.subscription_identifier.clone(),
                    initial_termination_time: subscription.initial_termination_time,
                    request: subscription.stop_monitoring_request.clone(),
                    incremental_updates: subscription.incremental_updates,
                })
            }
            _ => None,
        }
    }

    fn delivery(
        response_timestamp: DateTime<FixedOffset>,
        items: Vec<Self::Item>,
    ) -> Self::Delivery {
        StopMonitoringDelivery::new(response_timestamp, items)
    }

    fn attribute(
        delivery: &mut Self::Delivery,
        subscriber_ref: ParticipantRef,
        subscription_ref: SubscriptionRef,
    ) {
        delivery.subscriber_ref = Some(subscriber_ref);
        delivery.subscription_ref = Some(subscription_ref);
        delivery.status = Some(true);
    }

    fn service_delivery(delivery: Self::Delivery) -> ServiceDeliveryPayload {
        ServiceDeliveryPayload::StopMonitoringDelivery(Box::new(delivery))
    }

    fn items_of(payload: &ServiceDeliveryPayload) -> Option<Vec<Self::Item>> {
        match payload {
            ServiceDeliveryPayload::StopMonitoringDelivery(delivery) => {
                Some(delivery.monitored_stop_visit.clone())
            }
            _ => None,
        }
    }
}

impl Service for ConnectionTimetable {
    type Request = ConnectionTimetableRequest;
    type Delivery = ConnectionTimetableDelivery;
    type Item = TimetabledFeederArrival;

    fn service_request(request: Self::Request) -> ServiceRequestPayload {
        ServiceRequestPayload::ConnectionTimetableRequest(Box::new(request))
    }

    fn request_of(payload: &ServiceRequestPayload) -> Option<&Self::Request> {
        match payload {
            ServiceRequestPayload::ConnectionTimetableRequest(request) => Some(request),
            _ => None,
        }
    }

    /// The schema gives this service's subscription no incremental-updates flag, so
    /// a consumer asking for one is served the full set each time.
    fn subscription_request(parts: SubscriptionParts<Self>) -> SubscriptionRequestPayload {
        SubscriptionRequestPayload::ConnectionTimetableSubscriptionRequest(Box::new(
            ConnectionTimetableSubscriptionRequest {
                subscriber_ref: parts.subscriber_ref,
                ..ConnectionTimetableSubscriptionRequest::new(
                    parts.subscription_identifier,
                    parts.initial_termination_time,
                    parts.request,
                )
            },
        ))
    }

    fn subscription_of(payload: &SubscriptionRequestPayload) -> Option<SubscriptionParts<Self>> {
        match payload {
            SubscriptionRequestPayload::ConnectionTimetableSubscriptionRequest(subscription) => {
                Some(SubscriptionParts {
                    subscriber_ref: subscription.subscriber_ref.clone(),
                    subscription_identifier: subscription.subscription_identifier.clone(),
                    initial_termination_time: subscription.initial_termination_time,
                    request: subscription.connection_timetable_request.clone(),
                    incremental_updates: None,
                })
            }
            _ => None,
        }
    }

    fn delivery(
        response_timestamp: DateTime<FixedOffset>,
        items: Vec<Self::Item>,
    ) -> Self::Delivery {
        ConnectionTimetableDelivery::new(response_timestamp, items)
    }

    fn attribute(
        delivery: &mut Self::Delivery,
        subscriber_ref: ParticipantRef,
        subscription_ref: SubscriptionRef,
    ) {
        delivery.subscriber_ref = Some(subscriber_ref);
        delivery.subscription_ref = Some(subscription_ref);
        delivery.status = Some(true);
    }

    fn service_delivery(delivery: Self::Delivery) -> ServiceDeliveryPayload {
        ServiceDeliveryPayload::ConnectionTimetableDelivery(Box::new(delivery))
    }

    fn items_of(payload: &ServiceDeliveryPayload) -> Option<Vec<Self::Item>> {
        match payload {
            ServiceDeliveryPayload::ConnectionTimetableDelivery(delivery) => {
                Some(delivery.timetabled_feeder_arrival.clone())
            }
            _ => None,
        }
    }
}

impl Service for ConnectionMonitoringFeeder {
    type Request = ConnectionMonitoringRequest;
    type Delivery = ConnectionMonitoringFeederDelivery;
    type Item = MonitoredFeederArrival;

    fn service_request(request: Self::Request) -> ServiceRequestPayload {
        ServiceRequestPayload::ConnectionMonitoringRequest(Box::new(request))
    }

    fn request_of(payload: &ServiceRequestPayload) -> Option<&Self::Request> {
        match payload {
            ServiceRequestPayload::ConnectionMonitoringRequest(request) => Some(request),
            _ => None,
        }
    }

    /// The schema gives this service's subscription no incremental-updates flag, so
    /// a consumer asking for one is served the full set each time.
    fn subscription_request(parts: SubscriptionParts<Self>) -> SubscriptionRequestPayload {
        SubscriptionRequestPayload::ConnectionMonitoringSubscriptionRequest(Box::new(
            ConnectionMonitoringSubscriptionRequest {
                subscriber_ref: parts.subscriber_ref,
                ..ConnectionMonitoringSubscriptionRequest::new(
                    parts.subscription_identifier,
                    parts.initial_termination_time,
                    parts.request,
                )
            },
        ))
    }

    fn subscription_of(payload: &SubscriptionRequestPayload) -> Option<SubscriptionParts<Self>> {
        match payload {
            SubscriptionRequestPayload::ConnectionMonitoringSubscriptionRequest(subscription) => {
                Some(SubscriptionParts {
                    subscriber_ref: subscription.subscriber_ref.clone(),
                    subscription_identifier: subscription.subscription_identifier.clone(),
                    initial_termination_time: subscription.initial_termination_time,
                    request: subscription.connection_monitoring_request.clone(),
                    incremental_updates: None,
                })
            }
            _ => None,
        }
    }

    fn delivery(
        response_timestamp: DateTime<FixedOffset>,
        items: Vec<Self::Item>,
    ) -> Self::Delivery {
        ConnectionMonitoringFeederDelivery {
            monitored_feeder_arrival: items,
            ..ConnectionMonitoringFeederDelivery::new(response_timestamp)
        }
    }

    fn attribute(
        delivery: &mut Self::Delivery,
        subscriber_ref: ParticipantRef,
        subscription_ref: SubscriptionRef,
    ) {
        delivery.subscriber_ref = Some(subscriber_ref);
        delivery.subscription_ref = Some(subscription_ref);
        delivery.status = Some(true);
    }

    fn service_delivery(delivery: Self::Delivery) -> ServiceDeliveryPayload {
        ServiceDeliveryPayload::ConnectionMonitoringFeederDelivery(Box::new(delivery))
    }

    fn items_of(payload: &ServiceDeliveryPayload) -> Option<Vec<Self::Item>> {
        match payload {
            ServiceDeliveryPayload::ConnectionMonitoringFeederDelivery(delivery) => {
                Some(delivery.monitored_feeder_arrival.clone())
            }
            _ => None,
        }
    }
}

impl Service for GeneralMessage {
    type Request = GeneralMessageRequest;
    type Delivery = GeneralMessageDelivery;
    type Item = InfoMessage;

    fn service_request(request: Self::Request) -> ServiceRequestPayload {
        ServiceRequestPayload::GeneralMessageRequest(Box::new(request))
    }

    fn request_of(payload: &ServiceRequestPayload) -> Option<&Self::Request> {
        match payload {
            ServiceRequestPayload::GeneralMessageRequest(request) => Some(request),
            _ => None,
        }
    }

    /// The schema gives this service's subscription no incremental-updates flag, so
    /// a consumer asking for one is served the full set each time.
    fn subscription_request(parts: SubscriptionParts<Self>) -> SubscriptionRequestPayload {
        SubscriptionRequestPayload::GeneralMessageSubscriptionRequest(Box::new(
            GeneralMessageSubscriptionRequest {
                subscriber_ref: parts.subscriber_ref,
                ..GeneralMessageSubscriptionRequest::new(
                    parts.subscription_identifier,
                    parts.initial_termination_time,
                    parts.request,
                )
            },
        ))
    }

    fn subscription_of(payload: &SubscriptionRequestPayload) -> Option<SubscriptionParts<Self>> {
        match payload {
            SubscriptionRequestPayload::GeneralMessageSubscriptionRequest(subscription) => {
                Some(SubscriptionParts {
                    subscriber_ref: subscription.subscriber_ref.clone(),
                    subscription_identifier: subscription.subscription_identifier.clone(),
                    initial_termination_time: subscription.initial_termination_time,
                    request: subscription.general_message_request.clone(),
                    incremental_updates: None,
                })
            }
            _ => None,
        }
    }

    fn delivery(
        response_timestamp: DateTime<FixedOffset>,
        items: Vec<Self::Item>,
    ) -> Self::Delivery {
        GeneralMessageDelivery::new(response_timestamp, items)
    }

    fn attribute(
        delivery: &mut Self::Delivery,
        subscriber_ref: ParticipantRef,
        subscription_ref: SubscriptionRef,
    ) {
        delivery.subscriber_ref = Some(subscriber_ref);
        delivery.subscription_ref = Some(subscription_ref);
        delivery.status = Some(true);
    }

    fn service_delivery(delivery: Self::Delivery) -> ServiceDeliveryPayload {
        ServiceDeliveryPayload::GeneralMessageDelivery(Box::new(delivery))
    }

    fn items_of(payload: &ServiceDeliveryPayload) -> Option<Vec<Self::Item>> {
        match payload {
            ServiceDeliveryPayload::GeneralMessageDelivery(delivery) => {
                Some(delivery.general_message.clone())
            }
            _ => None,
        }
    }
}

impl Service for FacilityMonitoring {
    type Request = FacilityMonitoringRequest;
    type Delivery = FacilityMonitoringDelivery;
    type Item = FacilityCondition;

    fn service_request(request: Self::Request) -> ServiceRequestPayload {
        ServiceRequestPayload::FacilityMonitoringRequest(Box::new(request))
    }

    fn request_of(payload: &ServiceRequestPayload) -> Option<&Self::Request> {
        match payload {
            ServiceRequestPayload::FacilityMonitoringRequest(request) => Some(request),
            _ => None,
        }
    }

    fn subscription_request(parts: SubscriptionParts<Self>) -> SubscriptionRequestPayload {
        SubscriptionRequestPayload::FacilityMonitoringSubscriptionRequest(Box::new(
            FacilityMonitoringSubscriptionRequest {
                subscriber_ref: parts.subscriber_ref,
                incremental_updates: parts.incremental_updates,
                ..FacilityMonitoringSubscriptionRequest::new(
                    parts.subscription_identifier,
                    parts.initial_termination_time,
                    parts.request,
                )
            },
        ))
    }

    fn subscription_of(payload: &SubscriptionRequestPayload) -> Option<SubscriptionParts<Self>> {
        match payload {
            SubscriptionRequestPayload::FacilityMonitoringSubscriptionRequest(subscription) => {
                Some(SubscriptionParts {
                    subscriber_ref: subscription.subscriber_ref.clone(),
                    subscription_identifier: subscription.subscription_identifier.clone(),
                    initial_termination_time: subscription.initial_termination_time,
                    request: subscription.facility_monitoring_request.clone(),
                    incremental_updates: subscription.incremental_updates,
                })
            }
            _ => None,
        }
    }

    fn delivery(
        response_timestamp: DateTime<FixedOffset>,
        items: Vec<Self::Item>,
    ) -> Self::Delivery {
        FacilityMonitoringDelivery::new(response_timestamp, items)
    }

    fn attribute(
        delivery: &mut Self::Delivery,
        subscriber_ref: ParticipantRef,
        subscription_ref: SubscriptionRef,
    ) {
        delivery.subscriber_ref = Some(subscriber_ref);
        delivery.subscription_ref = Some(subscription_ref);
        delivery.status = Some(true);
    }

    fn service_delivery(delivery: Self::Delivery) -> ServiceDeliveryPayload {
        ServiceDeliveryPayload::FacilityMonitoringDelivery(Box::new(delivery))
    }

    fn items_of(payload: &ServiceDeliveryPayload) -> Option<Vec<Self::Item>> {
        match payload {
            ServiceDeliveryPayload::FacilityMonitoringDelivery(delivery) => {
                Some(delivery.facility_condition.clone())
            }
            _ => None,
        }
    }
}
