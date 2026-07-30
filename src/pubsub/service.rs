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
//! [`EstimatedTimetableSource`], [`ProductionTimetableSource`] or
//! [`VehicleMonitoringSource`], depending on what it publishes.

use std::fmt;

use chrono::{DateTime, FixedOffset};

use crate::et::{
    EstimatedTimetableDelivery, EstimatedTimetableRequest, EstimatedTimetableSubscriptionRequest,
};
use crate::framework::{ServiceDeliveryPayload, ServiceRequestPayload, SubscriptionRequestPayload};
use crate::model::EstimatedVehicleJourney;
use crate::pt::{
    DatedTimetableVersionFrame, ProductionTimetableDelivery, ProductionTimetableRequest,
    ProductionTimetableSubscriptionRequest,
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
/// [`SituationExchange`], [`EstimatedTimetable`], [`ProductionTimetable`] and
/// [`VehicleMonitoring`] are the implementations.
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
