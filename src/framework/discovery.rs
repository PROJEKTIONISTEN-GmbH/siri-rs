//! Finding out which stops, lines and code lists a producer knows about.
//!
//! Every SIRI functional service is driven by identifiers — a stop point, a line, a
//! service feature — that the consumer has to know before it can ask a useful
//! question. The discovery services publish those identifiers: five request/delivery
//! pairs, each answering "what values may I put in this field?".
//!
//! Discovery deliveries are stand-alone answers rather than payloads of a
//! `ServiceDelivery`, so each carries its own status and error condition.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::{LinesDetail, StopPointsDetail};
use crate::framework::error_condition::{ErrorCondition, ServiceRequestError};
use crate::model::{
    BoundingBox, DestinationRef, DirectionRef, JourneyPatternRef, LineDirection, LineRef, LineShape,
    LinkProjection, Location, OperatorRef, PlaceRef, ProductCategory, ServiceFeature,
    ServiceFeatureRef, StopAreaRef, StopPointRef, VehicleFeature,
};
use crate::types::{
    Duration, EndpointAddress, Extensions, MessageQualifier, NaturalLanguageString, ParticipantRef,
};

/// A request for the stop points a producer serves.
///
/// The optional filters narrow an otherwise unbounded answer. Area, circle and
/// place are alternatives to each other — see [`StopPointsRequest::scope`] — while
/// operator and line further restrict whichever of them is used.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopPointsRequest {
    /// Version of SIRI the request conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
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
    /// Rectangle the stops must lie in.
    #[serde(rename = "BoundingBox", default, skip_serializing_if = "Option::is_none")]
    pub bounding_box: Option<BoundingBox>,
    /// Circle the stops must lie in: the point is its centre and the point's
    /// precision its radius in metres.
    #[serde(rename = "Circle", default, skip_serializing_if = "Option::is_none")]
    pub circle: Option<Location>,
    /// Topographic place the stops must belong to.
    #[serde(rename = "PlaceRef", default, skip_serializing_if = "Option::is_none")]
    pub place_ref: Option<PlaceRef>,
    /// Operator whose stops are wanted.
    #[serde(rename = "OperatorRef", default, skip_serializing_if = "Option::is_none")]
    pub operator_ref: Option<OperatorRef>,
    /// Line whose stops are wanted.
    #[serde(rename = "LineRef", default, skip_serializing_if = "Option::is_none")]
    pub line_ref: Option<LineRef>,
    /// Languages the requestor would like names in, most preferred first.
    #[serde(rename = "Language", default, skip_serializing_if = "Vec::is_empty")]
    pub language: Vec<String>,
    /// How much detail each stop should be described with.
    #[serde(rename = "StopPointsDetailLevel", default, skip_serializing_if = "Option::is_none")]
    pub stop_points_detail_level: Option<StopPointsDetail>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl StopPointsRequest {
    /// An unfiltered request for every stop the producer serves.
    pub fn new(
        request_timestamp: DateTime<FixedOffset>,
        requestor_ref: impl Into<ParticipantRef>,
    ) -> Self {
        Self {
            version: None,
            request_timestamp,
            account_id: None,
            account_key: None,
            address: None,
            requestor_ref: requestor_ref.into(),
            message_identifier: None,
            bounding_box: None,
            circle: None,
            place_ref: None,
            operator_ref: None,
            line_ref: None,
            language: Vec::new(),
            stop_points_detail_level: None,
            extensions: None,
        }
    }

    /// Which alternative of the schema's area choice this request uses, or `None`
    /// when it asks about every stop.
    pub fn scope(&self) -> Option<StopPointsScope<'_>> {
        if let Some(bounding_box) = &self.bounding_box {
            Some(StopPointsScope::BoundingBox(bounding_box))
        } else if let Some(circle) = &self.circle {
            Some(StopPointsScope::Circle(circle))
        } else {
            self.place_ref.as_ref().map(StopPointsScope::Place)
        }
    }
}

/// Which way a [`StopPointsRequest`] narrows the area it asks about.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StopPointsScope<'a> {
    /// Stops inside a rectangle.
    BoundingBox(&'a BoundingBox),
    /// Stops inside a circle around a point.
    Circle(&'a Location),
    /// Stops belonging to a topographic place.
    Place(&'a PlaceRef),
}

/// The stop points a producer serves.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopPointsDelivery {
    /// Version of SIRI the delivery conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// When the delivery was made.
    #[serde(rename = "ResponseTimestamp")]
    pub response_timestamp: DateTime<FixedOffset>,
    /// Whether the request was processed successfully.
    #[serde(rename = "Status", default, skip_serializing_if = "Option::is_none")]
    pub status: Option<bool>,
    /// Why the request could not be processed.
    #[serde(rename = "ErrorCondition", default, skip_serializing_if = "Option::is_none")]
    pub error_condition: Option<ErrorCondition<ServiceRequestError>>,
    /// End of the producer's data horizon.
    #[serde(rename = "ValidUntil", default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<FixedOffset>>,
    /// The shortest interval at which the producer will send updates.
    #[serde(rename = "ShortestPossibleCycle", default, skip_serializing_if = "Option::is_none")]
    pub shortest_possible_cycle: Option<Duration>,
    /// The stops, described to the level of detail the request asked for.
    #[serde(rename = "AnnotatedStopPointRef", default, skip_serializing_if = "Vec::is_empty")]
    pub annotated_stop_point_ref: Vec<AnnotatedStopPointRef>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl StopPointsDelivery {
    /// A successful delivery of the given stops.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        annotated_stop_point_ref: Vec<AnnotatedStopPointRef>,
    ) -> Self {
        Self {
            version: None,
            response_timestamp,
            status: Some(true),
            error_condition: None,
            valid_until: None,
            shortest_possible_cycle: None,
            annotated_stop_point_ref,
            extensions: None,
        }
    }
}

/// A request for the lines a producer serves.
///
/// Area, circle, place and line-direction are alternatives to each other — see
/// [`LinesRequest::scope`] — while operator further restricts whichever is used.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinesRequest {
    /// Version of SIRI the request conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
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
    /// Rectangle the lines' stops must lie in.
    #[serde(rename = "BoundingBox", default, skip_serializing_if = "Option::is_none")]
    pub bounding_box: Option<BoundingBox>,
    /// Circle the lines' stops must lie in: the point is its centre and the point's
    /// precision its radius in metres.
    #[serde(rename = "Circle", default, skip_serializing_if = "Option::is_none")]
    pub circle: Option<Location>,
    /// Topographic place the lines' stops must belong to.
    #[serde(rename = "PlaceRef", default, skip_serializing_if = "Option::is_none")]
    pub place_ref: Option<PlaceRef>,
    /// A single line, optionally one direction of it, to be described in detail.
    #[serde(rename = "LineDirectionRef", default, skip_serializing_if = "Option::is_none")]
    pub line_direction_ref: Option<LineDirection>,
    /// Operator whose lines are wanted.
    #[serde(rename = "OperatorRef", default, skip_serializing_if = "Option::is_none")]
    pub operator_ref: Option<OperatorRef>,
    /// Languages the requestor would like names in, most preferred first.
    #[serde(rename = "Language", default, skip_serializing_if = "Vec::is_empty")]
    pub language: Vec<String>,
    /// How much detail each line should be described with.
    #[serde(rename = "LinesDetailLevel", default, skip_serializing_if = "Option::is_none")]
    pub lines_detail_level: Option<LinesDetail>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl LinesRequest {
    /// An unfiltered request for every line the producer serves.
    pub fn new(
        request_timestamp: DateTime<FixedOffset>,
        requestor_ref: impl Into<ParticipantRef>,
    ) -> Self {
        Self {
            version: None,
            request_timestamp,
            account_id: None,
            account_key: None,
            address: None,
            requestor_ref: requestor_ref.into(),
            message_identifier: None,
            bounding_box: None,
            circle: None,
            place_ref: None,
            line_direction_ref: None,
            operator_ref: None,
            language: Vec::new(),
            lines_detail_level: None,
            extensions: None,
        }
    }

    /// Which alternative of the schema's topic choice this request uses, or `None`
    /// when it asks about every line.
    pub fn scope(&self) -> Option<LinesScope<'_>> {
        if let Some(bounding_box) = &self.bounding_box {
            Some(LinesScope::BoundingBox(bounding_box))
        } else if let Some(circle) = &self.circle {
            Some(LinesScope::Circle(circle))
        } else if let Some(place_ref) = &self.place_ref {
            Some(LinesScope::Place(place_ref))
        } else {
            self.line_direction_ref
                .as_ref()
                .map(LinesScope::LineDirection)
        }
    }
}

/// Which way a [`LinesRequest`] narrows the lines it asks about.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LinesScope<'a> {
    /// Lines calling at stops inside a rectangle.
    BoundingBox(&'a BoundingBox),
    /// Lines calling at stops inside a circle around a point.
    Circle(&'a Location),
    /// Lines calling at stops belonging to a topographic place.
    Place(&'a PlaceRef),
    /// One named line, optionally narrowed to one direction.
    LineDirection(&'a LineDirection),
}

/// The lines a producer serves.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinesDelivery {
    /// Version of SIRI the delivery conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// When the delivery was made.
    #[serde(rename = "ResponseTimestamp")]
    pub response_timestamp: DateTime<FixedOffset>,
    /// Whether the request was processed successfully.
    #[serde(rename = "Status", default, skip_serializing_if = "Option::is_none")]
    pub status: Option<bool>,
    /// Why the request could not be processed.
    #[serde(rename = "ErrorCondition", default, skip_serializing_if = "Option::is_none")]
    pub error_condition: Option<ErrorCondition<ServiceRequestError>>,
    /// End of the producer's data horizon.
    #[serde(rename = "ValidUntil", default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<FixedOffset>>,
    /// The shortest interval at which the producer will send updates.
    #[serde(rename = "ShortestPossibleCycle", default, skip_serializing_if = "Option::is_none")]
    pub shortest_possible_cycle: Option<Duration>,
    /// The lines, described to the level of detail the request asked for.
    #[serde(rename = "AnnotatedLineRef", default, skip_serializing_if = "Vec::is_empty")]
    pub annotated_line_ref: Vec<AnnotatedLineRef>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl LinesDelivery {
    /// A successful delivery of the given lines.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        annotated_line_ref: Vec<AnnotatedLineRef>,
    ) -> Self {
        Self {
            version: None,
            response_timestamp,
            status: Some(true),
            error_condition: None,
            valid_until: None,
            shortest_possible_cycle: None,
            annotated_line_ref,
            extensions: None,
        }
    }
}

/// A request for the service features a producer labels its services with.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceFeaturesRequest {
    /// Version of SIRI the request conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
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
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl ServiceFeaturesRequest {
    /// A request for the producer's whole service-feature code list.
    pub fn new(
        request_timestamp: DateTime<FixedOffset>,
        requestor_ref: impl Into<ParticipantRef>,
    ) -> Self {
        Self {
            version: None,
            request_timestamp,
            account_id: None,
            account_key: None,
            address: None,
            requestor_ref: requestor_ref.into(),
            message_identifier: None,
            extensions: None,
        }
    }
}

/// The service features a producer labels its services with.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceFeaturesDelivery {
    /// Version of SIRI the delivery conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// When the delivery was made.
    #[serde(rename = "ResponseTimestamp")]
    pub response_timestamp: DateTime<FixedOffset>,
    /// Whether the request was processed successfully.
    #[serde(rename = "Status", default, skip_serializing_if = "Option::is_none")]
    pub status: Option<bool>,
    /// Why the request could not be processed.
    #[serde(rename = "ErrorCondition", default, skip_serializing_if = "Option::is_none")]
    pub error_condition: Option<ErrorCondition<ServiceRequestError>>,
    /// End of the producer's data horizon.
    #[serde(rename = "ValidUntil", default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<FixedOffset>>,
    /// The shortest interval at which the producer will send updates.
    #[serde(rename = "ShortestPossibleCycle", default, skip_serializing_if = "Option::is_none")]
    pub shortest_possible_cycle: Option<Duration>,
    /// The code list, one entry per feature.
    ///
    /// Unlike the other discovery deliveries this one has no `Extensions` element.
    #[serde(rename = "ServiceFeature", default, skip_serializing_if = "Vec::is_empty")]
    pub service_feature: Vec<ServiceFeature>,
}

impl ServiceFeaturesDelivery {
    /// A successful delivery of the given code list.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        service_feature: Vec<ServiceFeature>,
    ) -> Self {
        Self {
            version: None,
            response_timestamp,
            status: Some(true),
            error_condition: None,
            valid_until: None,
            shortest_possible_cycle: None,
            service_feature,
        }
    }
}

/// A request for the vehicle features a producer labels its vehicles with.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleFeaturesRequest {
    /// Version of SIRI the request conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
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
    /// Languages the requestor would like names in, most preferred first.
    #[serde(rename = "Language", default, skip_serializing_if = "Vec::is_empty")]
    pub language: Vec<String>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl VehicleFeaturesRequest {
    /// A request for the producer's whole vehicle-feature code list.
    pub fn new(
        request_timestamp: DateTime<FixedOffset>,
        requestor_ref: impl Into<ParticipantRef>,
    ) -> Self {
        Self {
            version: None,
            request_timestamp,
            account_id: None,
            account_key: None,
            address: None,
            requestor_ref: requestor_ref.into(),
            message_identifier: None,
            language: Vec::new(),
            extensions: None,
        }
    }
}

/// The vehicle features a producer labels its vehicles with.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleFeaturesDelivery {
    /// Version of SIRI the delivery conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// When the delivery was made.
    #[serde(rename = "ResponseTimestamp")]
    pub response_timestamp: DateTime<FixedOffset>,
    /// Whether the request was processed successfully.
    #[serde(rename = "Status", default, skip_serializing_if = "Option::is_none")]
    pub status: Option<bool>,
    /// Why the request could not be processed.
    #[serde(rename = "ErrorCondition", default, skip_serializing_if = "Option::is_none")]
    pub error_condition: Option<ErrorCondition<ServiceRequestError>>,
    /// End of the producer's data horizon.
    #[serde(rename = "ValidUntil", default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<FixedOffset>>,
    /// The shortest interval at which the producer will send updates.
    #[serde(rename = "ShortestPossibleCycle", default, skip_serializing_if = "Option::is_none")]
    pub shortest_possible_cycle: Option<Duration>,
    /// The code list, one entry per feature.
    #[serde(rename = "VehicleFeature", default, skip_serializing_if = "Vec::is_empty")]
    pub vehicle_feature: Vec<VehicleFeature>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl VehicleFeaturesDelivery {
    /// A successful delivery of the given code list.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        vehicle_feature: Vec<VehicleFeature>,
    ) -> Self {
        Self {
            version: None,
            response_timestamp,
            status: Some(true),
            error_condition: None,
            valid_until: None,
            shortest_possible_cycle: None,
            vehicle_feature,
            extensions: None,
        }
    }
}

/// A request for the product categories a producer sorts its services into.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductCategoriesRequest {
    /// Version of SIRI the request conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
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
    /// Languages the requestor would like names in, most preferred first.
    #[serde(rename = "Language", default, skip_serializing_if = "Vec::is_empty")]
    pub language: Vec<String>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl ProductCategoriesRequest {
    /// A request for the producer's whole product-category code list.
    pub fn new(
        request_timestamp: DateTime<FixedOffset>,
        requestor_ref: impl Into<ParticipantRef>,
    ) -> Self {
        Self {
            version: None,
            request_timestamp,
            account_id: None,
            account_key: None,
            address: None,
            requestor_ref: requestor_ref.into(),
            message_identifier: None,
            language: Vec::new(),
            extensions: None,
        }
    }
}

/// The product categories a producer sorts its services into.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductCategoriesDelivery {
    /// Version of SIRI the delivery conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// When the delivery was made.
    #[serde(rename = "ResponseTimestamp")]
    pub response_timestamp: DateTime<FixedOffset>,
    /// Whether the request was processed successfully.
    #[serde(rename = "Status", default, skip_serializing_if = "Option::is_none")]
    pub status: Option<bool>,
    /// Why the request could not be processed.
    #[serde(rename = "ErrorCondition", default, skip_serializing_if = "Option::is_none")]
    pub error_condition: Option<ErrorCondition<ServiceRequestError>>,
    /// End of the producer's data horizon.
    #[serde(rename = "ValidUntil", default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<FixedOffset>>,
    /// The shortest interval at which the producer will send updates.
    #[serde(rename = "ShortestPossibleCycle", default, skip_serializing_if = "Option::is_none")]
    pub shortest_possible_cycle: Option<Duration>,
    /// The code list, one entry per category.
    #[serde(rename = "ProductCategory", default, skip_serializing_if = "Vec::is_empty")]
    pub product_category: Vec<ProductCategory>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl ProductCategoriesDelivery {
    /// A successful delivery of the given code list.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        product_category: Vec<ProductCategory>,
    ) -> Self {
        Self {
            version: None,
            response_timestamp,
            status: Some(true),
            error_condition: None,
            valid_until: None,
            shortest_possible_cycle: None,
            product_category,
            extensions: None,
        }
    }
}

/// A stop point together with as much about it as the request asked for.
///
/// Only the identifier is always present. Name and position come at the `normal`
/// level of detail, features and lines at the `full` level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnnotatedStopPointRef {
    /// Identifier of the stop.
    #[serde(rename = "StopPointRef")]
    pub stop_point_ref: StopPointRef,
    /// Whether the stop is a timing point, i.e. one whose passing times are held to
    /// and published rather than merely interpolated.
    #[serde(rename = "TimingPoint", default, skip_serializing_if = "Option::is_none")]
    pub timing_point: Option<bool>,
    /// Whether real-time data is available for the stop.
    #[serde(rename = "Monitored", default, skip_serializing_if = "Option::is_none")]
    pub monitored: Option<bool>,
    /// Names of the stop, one per language.
    #[serde(rename = "StopName", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_name: Vec<NaturalLanguageString>,
    /// The group of stops this one belongs to.
    #[serde(rename = "StopAreaRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_area_ref: Option<StopAreaRef>,
    /// Features of the stop, such as a shelter or a ticket machine.
    #[serde(rename = "Features", default, skip_serializing_if = "Option::is_none")]
    pub features: Option<StopPointFeatures>,
    /// Lines that call at the stop.
    #[serde(rename = "Lines", default, skip_serializing_if = "Option::is_none")]
    pub lines: Option<StopPointLines>,
    /// Where to draw the stop on a map.
    #[serde(rename = "Location", default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    /// Web page about the stop.
    #[serde(rename = "Url", default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl AnnotatedStopPointRef {
    /// A stop described by identifier alone, the `minimum` level of detail.
    pub fn new(stop_point_ref: impl Into<StopPointRef>) -> Self {
        Self {
            stop_point_ref: stop_point_ref.into(),
            timing_point: None,
            monitored: None,
            stop_name: Vec::new(),
            stop_area_ref: None,
            features: None,
            lines: None,
            location: None,
            url: None,
        }
    }
}

/// The features of a stop, each given either in full or as a reference into the
/// producer's code list.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StopPointFeatures {
    /// The features, in the order the producer listed them.
    #[serde(rename = "$value", default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<StopPointFeature>,
}

/// One feature of a stop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StopPointFeature {
    /// The feature spelled out, with its code, names and icon.
    ServiceFeature(ServiceFeature),
    /// A reference to a feature of the producer's code list.
    ServiceFeatureRef(ServiceFeatureRef),
}

/// The lines calling at a stop, each given with or without a direction.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StopPointLines {
    /// The lines, in the order the producer listed them.
    #[serde(rename = "$value", default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<StopPointLine>,
}

/// One line calling at a stop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StopPointLine {
    /// The line as a whole, in both directions.
    LineRef(LineRef),
    /// One direction of the line.
    LineDirection(LineDirection),
}

/// A line together with as much about it as the request asked for.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnnotatedLineRef {
    /// Identifier of the line.
    #[serde(rename = "LineRef")]
    pub line_ref: LineRef,
    /// Names of the line, one per language.
    #[serde(rename = "LineName")]
    pub line_name: Vec<NaturalLanguageString>,
    /// Whether real-time data is available for the line.
    #[serde(rename = "Monitored", default, skip_serializing_if = "Option::is_none")]
    pub monitored: Option<bool>,
    /// Places the line runs to, as shown to passengers.
    #[serde(rename = "Destinations", default, skip_serializing_if = "Option::is_none")]
    pub destinations: Option<Destinations>,
    /// The directions the line is operated in, with their stopping patterns.
    #[serde(rename = "Directions", default, skip_serializing_if = "Option::is_none")]
    pub directions: Option<Directions>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl AnnotatedLineRef {
    /// A line described by identifier and name, the `minimum` level of detail.
    pub fn new(line_ref: impl Into<LineRef>, line_name: Vec<NaturalLanguageString>) -> Self {
        Self {
            line_ref: line_ref.into(),
            line_name,
            monitored: None,
            destinations: None,
            directions: None,
            extensions: None,
        }
    }
}

/// The places a line runs to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Destinations {
    /// The destinations, at least one.
    #[serde(rename = "Destination")]
    pub destination: Vec<AnnotatedDestination>,
}

/// A place a line runs to, as shown to passengers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnnotatedDestination {
    /// Identifier of the destination.
    #[serde(rename = "DestinationRef")]
    pub destination_ref: DestinationRef,
    /// Names of the destination place, one per language.
    #[serde(rename = "PlaceName")]
    pub place_name: Vec<NaturalLanguageString>,
    /// The direction of travel this destination lies in.
    #[serde(rename = "DirectionRef", default, skip_serializing_if = "Option::is_none")]
    pub direction_ref: Option<DirectionRef>,
}

/// The directions a line is operated in.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Directions {
    /// The directions, at least one.
    #[serde(rename = "Direction")]
    pub direction: Vec<RouteDirection>,
}

/// One direction of travel along a line, with the patterns operated in it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteDirection {
    /// Identifier of the direction.
    #[serde(rename = "DirectionRef")]
    pub direction_ref: DirectionRef,
    /// Names of the direction, one per language.
    #[serde(rename = "DirectionName", default, skip_serializing_if = "Vec::is_empty")]
    pub direction_name: Vec<NaturalLanguageString>,
    /// The stopping patterns operated in this direction.
    #[serde(rename = "JourneyPatterns", default, skip_serializing_if = "Option::is_none")]
    pub journey_patterns: Option<JourneyPatterns>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// The stopping patterns operated in one direction of a line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JourneyPatterns {
    /// The patterns, at least one.
    #[serde(rename = "JourneyPattern")]
    pub journey_pattern: Vec<JourneyPattern>,
}

/// One ordered sequence of stops that journeys on a line follow.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct JourneyPattern {
    /// Identifier of the pattern.
    #[serde(rename = "JourneyPatternRef", default, skip_serializing_if = "Option::is_none")]
    pub journey_pattern_ref: Option<JourneyPatternRef>,
    /// Names of the pattern, one per language.
    #[serde(rename = "Name", default, skip_serializing_if = "Vec::is_empty")]
    pub name: Vec<NaturalLanguageString>,
    /// The stops, in the order they are called at.
    #[serde(rename = "StopsInPattern", default, skip_serializing_if = "Option::is_none")]
    pub stops_in_pattern: Option<StopsInPattern>,
}

/// The stops of a journey pattern, in the order they are called at.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopsInPattern {
    /// The stops; the schema requires at least two, since a pattern has an origin
    /// and a destination.
    #[serde(rename = "StopPointInPattern")]
    pub stop_point_in_pattern: Vec<StopPointInPattern>,
}

/// A stop as it appears within a journey pattern.
///
/// Everything an [`AnnotatedStopPointRef`] carries, plus the stop's place in the
/// sequence and the shape of the run to the next stop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopPointInPattern {
    /// Identifier of the stop.
    #[serde(rename = "StopPointRef")]
    pub stop_point_ref: StopPointRef,
    /// Whether the stop is a timing point, i.e. one whose passing times are held to
    /// and published rather than merely interpolated.
    #[serde(rename = "TimingPoint", default, skip_serializing_if = "Option::is_none")]
    pub timing_point: Option<bool>,
    /// Whether real-time data is available for the stop.
    #[serde(rename = "Monitored", default, skip_serializing_if = "Option::is_none")]
    pub monitored: Option<bool>,
    /// Names of the stop, one per language.
    #[serde(rename = "StopName", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_name: Vec<NaturalLanguageString>,
    /// The group of stops this one belongs to.
    #[serde(rename = "StopAreaRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_area_ref: Option<StopAreaRef>,
    /// Features of the stop, such as a shelter or a ticket machine.
    #[serde(rename = "Features", default, skip_serializing_if = "Option::is_none")]
    pub features: Option<StopPointFeatures>,
    /// Lines that call at the stop.
    #[serde(rename = "Lines", default, skip_serializing_if = "Option::is_none")]
    pub lines: Option<StopPointLines>,
    /// Where to draw the stop on a map.
    #[serde(rename = "Location", default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    /// Web page about the stop.
    #[serde(rename = "Url", default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Position of the stop in the pattern, counting from one.
    #[serde(rename = "Order")]
    pub order: u64,
    /// The path from this stop to the next one, as a list of points.
    #[serde(rename = "OnwardLinkShape", default, skip_serializing_if = "Option::is_none")]
    pub onward_link_shape: Option<LineShape>,
    /// The path from this stop to the next one, projected onto a geographic
    /// information system's own geometry.
    #[serde(
        rename = "LinkProjectionToNextStopPoint",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub link_projection_to_next_stop_point: Option<LinkProjection>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const STOP_POINTS_DELIVERY: &str = r#"<StopPointsDelivery version="2.0">
	<ResponseTimestamp>2004-12-17T09:30:47-05:00</ResponseTimestamp>
	<Status>true</Status>
	<AnnotatedStopPointRef>
		<StopPointRef>HLTS0001</StopPointRef>
		<Monitored>true</Monitored>
		<StopName xml:lang="EN">Stop 101</StopName>
		<Features>
			<ServiceFeature>
				<ServiceFeatureCode>S123</ServiceFeatureCode>
				<Name xml:lang="EN">Shelter</Name>
				<Icon>http://www.mybus.org/shelter.jpg</Icon>
			</ServiceFeature>
		</Features>
		<Lines>
			<LineRef>A23</LineRef>
		</Lines>
		<Location>
			<Longitude>0.1</Longitude>
			<Latitude>53</Latitude>
		</Location>
	</AnnotatedStopPointRef>
</StopPointsDelivery>"#;

    const LINES_DELIVERY: &str = r#"<LinesDelivery version="2.0">
	<ResponseTimestamp>2004-12-17T09:30:47-05:00</ResponseTimestamp>
	<Status>true</Status>
	<AnnotatedLineRef>
		<LineRef>Line564</LineRef>
		<LineName xml:lang="EN">564</LineName>
		<Monitored>true</Monitored>
		<Destinations>
			<Destination>
				<DestinationRef>PLace45</DestinationRef>
				<PlaceName xml:lang="EN">Paradise</PlaceName>
			</Destination>
		</Destinations>
		<Directions>
			<Direction>
				<DirectionRef>DIR01</DirectionRef>
				<DirectionName xml:lang="EN">Outbound</DirectionName>
				<JourneyPatterns>
					<JourneyPattern>
						<StopsInPattern>
							<StopPointInPattern>
								<StopPointRef>ST001</StopPointRef>
								<StopName xml:lang="EN">Alpha Road</StopName>
								<Lines>
									<LineRef>Line564</LineRef>
									<LineRef>22</LineRef>
								</Lines>
								<Location>
									<Longitude>0.11</Longitude>
									<Latitude>53.01</Latitude>
								</Location>
								<Order>1</Order>
							</StopPointInPattern>
							<StopPointInPattern>
								<StopPointRef>ST003</StopPointRef>
								<StopName xml:lang="EN">Charley Square</StopName>
								<StopAreaRef>SA2001</StopAreaRef>
								<Lines>
									<LineDirection>
										<LineRef>Line564</LineRef>
										<DirectionRef>NORTH</DirectionRef>
									</LineDirection>
									<LineRef>44</LineRef>
								</Lines>
								<Location>
									<Longitude>0.13</Longitude>
									<Latitude>53.03</Latitude>
								</Location>
								<Order>3</Order>
							</StopPointInPattern>
						</StopsInPattern>
					</JourneyPattern>
				</JourneyPatterns>
			</Direction>
			<Direction>
				<DirectionRef>DIR02</DirectionRef>
				<DirectionName xml:lang="EN">Inbound</DirectionName>
			</Direction>
		</Directions>
	</AnnotatedLineRef>
</LinesDelivery>"#;

    #[test]
    fn a_stop_is_delivered_with_its_features_and_lines() {
        let delivery: StopPointsDelivery = quick_xml::de::from_str(STOP_POINTS_DELIVERY).unwrap();

        assert_eq!(delivery.status, Some(true));
        let stop = &delivery.annotated_stop_point_ref[0];
        assert_eq!(stop.stop_point_ref, StopPointRef::new("HLTS0001"));
        assert_eq!(stop.stop_name[0].value, "Stop 101");

        let features = stop.features.as_ref().unwrap();
        match &features.items[0] {
            StopPointFeature::ServiceFeature(feature) => {
                assert_eq!(feature.service_feature_code, "S123");
            }
            other => panic!("expected a spelled-out feature, got {other:?}"),
        }
        assert_eq!(
            stop.lines.as_ref().unwrap().items,
            [StopPointLine::LineRef(LineRef::new("A23"))]
        );
    }

    #[test]
    fn a_line_is_delivered_with_its_directions_and_stopping_pattern() {
        let delivery: LinesDelivery = quick_xml::de::from_str(LINES_DELIVERY).unwrap();
        let line = &delivery.annotated_line_ref[0];

        let destinations = line.destinations.as_ref().unwrap();
        assert_eq!(
            destinations.destination[0].destination_ref,
            DestinationRef::new("PLace45")
        );

        let directions = &line.directions.as_ref().unwrap().direction;
        assert_eq!(directions.len(), 2);
        assert!(directions[1].journey_patterns.is_none());

        let stops = directions[0]
            .journey_patterns
            .as_ref()
            .unwrap()
            .journey_pattern[0]
            .stops_in_pattern
            .as_ref()
            .unwrap();
        assert_eq!(stops.stop_point_in_pattern[0].order, 1);
        assert_eq!(stops.stop_point_in_pattern[1].order, 3);
        assert_eq!(
            stops.stop_point_in_pattern[1].stop_area_ref,
            Some(StopAreaRef::new("SA2001"))
        );
        assert_eq!(
            stops.stop_point_in_pattern[1]
                .lines
                .as_ref()
                .unwrap()
                .items[0],
            StopPointLine::LineDirection(LineDirection::with_direction("Line564", "NORTH"))
        );
    }

    #[test]
    fn deliveries_survive_a_round_trip() {
        let delivery: LinesDelivery = quick_xml::de::from_str(LINES_DELIVERY).unwrap();
        let written = quick_xml::se::to_string_with_root("LinesDelivery", &delivery).unwrap();
        let reread: LinesDelivery = quick_xml::de::from_str(&written).unwrap();
        assert_eq!(delivery, reread);
    }

    #[test]
    fn a_stop_points_request_reports_which_area_it_narrows_to() {
        let timestamp = DateTime::parse_from_rfc3339("2004-12-17T09:30:47-05:00").unwrap();
        let mut request = StopPointsRequest::new(timestamp, "NADER");
        assert_eq!(request.scope(), None);

        let area = BoundingBox::new(Location::wgs84(0.1, 53.1), Location::wgs84(0.2, 53.2));
        request.bounding_box = Some(area.clone());
        assert_eq!(request.scope(), Some(StopPointsScope::BoundingBox(&area)));

        let xml = quick_xml::se::to_string_with_root("StopPointsRequest", &request).unwrap();
        assert!(xml.contains("<RequestorRef>NADER</RequestorRef>"));
        assert!(xml.contains("<UpperLeft>"));
        assert!(!xml.contains("StopPointsDetailLevel"));
    }

    #[test]
    fn a_service_features_delivery_carries_the_whole_code_list() {
        let timestamp = DateTime::parse_from_rfc3339("2004-12-17T09:30:47-05:00").unwrap();
        let delivery = ServiceFeaturesDelivery::new(
            timestamp,
            vec![
                ServiceFeature {
                    service_feature_code: "CAT01".to_owned(),
                    name: vec![NaturalLanguageString::new("Express")],
                    icon: "express.gif".to_owned(),
                },
                ServiceFeature {
                    service_feature_code: "CAT021".to_owned(),
                    name: vec![NaturalLanguageString::new("School")],
                    icon: "school.gif".to_owned(),
                },
            ],
        );

        let xml =
            quick_xml::se::to_string_with_root("ServiceFeaturesDelivery", &delivery).unwrap();
        assert!(xml.contains("<ServiceFeatureCode>CAT01</ServiceFeatureCode>"));
        assert!(xml.contains("<ServiceFeatureCode>CAT021</ServiceFeatureCode>"));
        // Alone among the discovery deliveries, this one has no Extensions element.
        assert!(!xml.contains("Extensions"));

        let reread: ServiceFeaturesDelivery = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(reread, delivery);
    }
}
