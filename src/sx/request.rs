//! Asking for situations, once or by subscription.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::{
    AccessModes, AirSubmodesOfTransport, BusSubmodesOfTransport, CoachSubmodesOfTransport,
    Direction, MetroSubmodesOfTransport, Predictability, RailSubmodesOfTransport, ScopeType,
    Severity, TelecabinSubmodesOfTransport, TramSubmodesOfTransport, VehicleModesOfTransport,
    VerificationStatus, WaterSubmodesOfTransport, WorkflowStatus,
};
use crate::model::{
    ConnectionLinkRef, FacilityRef, FramedVehicleJourneyRef, InterchangeRef, LineRef,
    Location, OperationalUnitRef, OperatorRef, PassengerAccessibilityNeeds, RequestedLines,
    StopPlaceComponentRef, StopPlaceRef, StopPointRef, Submode, VehicleJourneyRef, VehicleRef,
};
use crate::types::{
    CountryRef, Duration, Extensions, HalfOpenTimestampInputRange, MessageQualifier, ParticipantRef,
    SubscriptionQualifier,
};

/// A request for the situations a producer holds.
///
/// Every field beyond the timestamp narrows the answer; a request with no filters
/// asks for everything the producer is willing to publish. Filters of different
/// kinds are combined with "and".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SituationExchangeRequest {
    /// Version of SIRI-SX the request conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// When the request was made.
    #[serde(rename = "RequestTimestamp")]
    pub request_timestamp: DateTime<FixedOffset>,
    /// Identifier the requestor puts on this request.
    #[serde(rename = "MessageIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub message_identifier: Option<MessageQualifier>,
    /// How far ahead of `start_time` to look for situations.
    #[serde(rename = "PreviewInterval", default, skip_serializing_if = "Option::is_none")]
    pub preview_interval: Option<Duration>,
    /// The instant the preview interval starts at; the default is now.
    #[serde(rename = "StartTime", default, skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime<FixedOffset>>,
    /// Only situations whose validity overlaps this period.
    #[serde(rename = "ValidityPeriod", default, skip_serializing_if = "Option::is_none")]
    pub validity_period: Option<HalfOpenTimestampInputRange>,
    /// Only situations the producer is currently publishing.
    #[serde(rename = "IncludeOnlyIfInPublicationWindow", default, skip_serializing_if = "Option::is_none")]
    pub include_only_if_in_publication_window: Option<bool>,
    /// Only situations affecting this mode of transport.
    #[serde(rename = "VehicleMode", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_mode: Option<VehicleModesOfTransport>,
    /// Only situations affecting this kind of air service.
    #[serde(rename = "AirSubmode", default, skip_serializing_if = "Option::is_none")]
    pub air_submode: Option<AirSubmodesOfTransport>,
    /// Only situations affecting this kind of bus service.
    #[serde(rename = "BusSubmode", default, skip_serializing_if = "Option::is_none")]
    pub bus_submode: Option<BusSubmodesOfTransport>,
    /// Only situations affecting this kind of coach service.
    #[serde(rename = "CoachSubmode", default, skip_serializing_if = "Option::is_none")]
    pub coach_submode: Option<CoachSubmodesOfTransport>,
    /// Only situations affecting this kind of metro service.
    #[serde(rename = "MetroSubmode", default, skip_serializing_if = "Option::is_none")]
    pub metro_submode: Option<MetroSubmodesOfTransport>,
    /// Only situations affecting this kind of rail service.
    #[serde(rename = "RailSubmode", default, skip_serializing_if = "Option::is_none")]
    pub rail_submode: Option<RailSubmodesOfTransport>,
    /// Only situations affecting this kind of tram service.
    #[serde(rename = "TramSubmode", default, skip_serializing_if = "Option::is_none")]
    pub tram_submode: Option<TramSubmodesOfTransport>,
    /// Only situations affecting this kind of water-borne service.
    #[serde(rename = "WaterSubmode", default, skip_serializing_if = "Option::is_none")]
    pub water_submode: Option<WaterSubmodesOfTransport>,
    /// Only situations affecting this kind of cable-drawn service.
    #[serde(rename = "TelecabinSubmode", default, skip_serializing_if = "Option::is_none")]
    pub telecabin_submode: Option<TelecabinSubmodesOfTransport>,
    /// Only situations affecting this way of reaching or leaving a stop.
    #[serde(rename = "AccessMode", default, skip_serializing_if = "Option::is_none")]
    pub access_mode: Option<AccessModes>,
    /// Only situations at least this severe.
    #[serde(rename = "Severity", default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<Severity>,
    /// Only situations whose scope is one of these.
    #[serde(
        rename = "Scope",
        default,
        deserialize_with = "crate::xml::token_list::deserialize",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub scope: Vec<ScopeType>,
    /// Only planned situations, only unplanned ones, or both.
    #[serde(rename = "Predictability", default, skip_serializing_if = "Option::is_none")]
    pub predictability: Option<Predictability>,
    /// Only situations tagged with one of these keywords, separated by spaces.
    #[serde(rename = "Keywords", default, skip_serializing_if = "Option::is_none")]
    pub keywords: Option<String>,
    /// Only situations in this state of verification.
    #[serde(rename = "Verification", default, skip_serializing_if = "Option::is_none")]
    pub verification: Option<VerificationStatus>,
    /// Only situations at one of these points in the editorial workflow.
    #[serde(
        rename = "Progress",
        default,
        deserialize_with = "crate::xml::token_list::deserialize",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub progress: Vec<WorkflowStatus>,
    /// Only real situations, or only exercises and tests.
    #[serde(rename = "Reality", default, skip_serializing_if = "Option::is_none")]
    pub reality: Option<crate::enumerations::InformationStatus>,
    /// Only situations affecting this operator.
    #[serde(rename = "OperatorRef", default, skip_serializing_if = "Option::is_none")]
    pub operator_ref: Option<OperatorRef>,
    /// Only situations affecting these parts of the operator's organisation.
    #[serde(rename = "OperationalUnitRef", default, skip_serializing_if = "Vec::is_empty")]
    pub operational_unit_ref: Vec<OperationalUnitRef>,
    /// Only situations affecting this network.
    #[serde(rename = "NetworkRef", default, skip_serializing_if = "Option::is_none")]
    pub network_ref: Option<OperatorRef>,
    /// Only situations affecting these lines.
    #[serde(rename = "LineRef", default, skip_serializing_if = "Vec::is_empty")]
    pub line_ref: Vec<LineRef>,
    /// Only situations affecting these lines in these directions.
    #[serde(rename = "Lines", default, skip_serializing_if = "Option::is_none")]
    pub lines: Option<RequestedLines>,
    /// Only situations affecting these stop points.
    #[serde(rename = "StopPointRef", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_point_ref: Vec<StopPointRef>,
    /// Only situations affecting these connection links.
    #[serde(rename = "ConnectionLinkRef", default, skip_serializing_if = "Vec::is_empty")]
    pub connection_link_ref: Vec<ConnectionLinkRef>,
    /// Only situations affecting these facilities.
    #[serde(rename = "FacilityRef", default, skip_serializing_if = "Vec::is_empty")]
    pub facility_ref: Vec<FacilityRef>,
    /// Only situations affecting this stop place.
    #[serde(rename = "StopPlaceRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_place_ref: Option<StopPlaceRef>,
    /// Only situations affecting this part of a stop place.
    #[serde(rename = "StopPlaceComponentRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_place_component_ref: Option<StopPlaceComponentRef>,
    /// Only situations affecting this journey on this operational day.
    #[serde(rename = "FramedVehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub framed_vehicle_journey_ref: Option<FramedVehicleJourneyRef>,
    /// Only situations affecting this journey.
    #[serde(rename = "VehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_journey_ref: Option<VehicleJourneyRef>,
    /// Only situations affecting this interchange.
    #[serde(rename = "InterchangeRef", default, skip_serializing_if = "Option::is_none")]
    pub interchange_ref: Option<InterchangeRef>,
    /// Only situations affecting this vehicle.
    #[serde(rename = "VehicleRef", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_ref: Option<VehicleRef>,
    /// Only situations in this country.
    #[serde(rename = "CountryRef", default, skip_serializing_if = "Option::is_none")]
    pub country_ref: Option<CountryRef>,
    /// Only situations at this topographic place.
    #[serde(rename = "PlaceRef", default, skip_serializing_if = "Option::is_none")]
    pub place_ref: Option<String>,
    /// Only situations near this point, or within the box its two points span.
    #[serde(rename = "Location", default, skip_serializing_if = "Vec::is_empty")]
    pub location: Vec<Location>,
    /// Only situations on these roads.
    #[serde(rename = "SituationRoadFilter", default, skip_serializing_if = "Option::is_none")]
    pub situation_road_filter: Option<SituationRoadFilter>,
    /// Only situations relevant to passengers with these needs.
    #[serde(rename = "AccessibilityNeedFilter", default, skip_serializing_if = "Vec::is_empty")]
    pub accessibility_need_filter: Vec<PassengerAccessibilityNeeds>,
    /// Languages the requestor would like texts in, most preferred first.
    #[serde(rename = "Language", default, skip_serializing_if = "Vec::is_empty")]
    pub language: Vec<String>,
    /// Whether to include every translation of a text rather than only one.
    #[serde(rename = "IncludeTranslations", default, skip_serializing_if = "Option::is_none")]
    pub include_translations: Option<bool>,
    /// At most this many situations.
    #[serde(rename = "MaximumNumberOfSituationElements", default, skip_serializing_if = "Option::is_none")]
    pub maximum_number_of_situation_elements: Option<u64>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl SituationExchangeRequest {
    /// An unfiltered request for everything the producer publishes.
    pub fn new(request_timestamp: DateTime<FixedOffset>) -> Self {
        Self {
            version: None,
            request_timestamp,
            message_identifier: None,
            preview_interval: None,
            start_time: None,
            validity_period: None,
            include_only_if_in_publication_window: None,
            vehicle_mode: None,
            air_submode: None,
            bus_submode: None,
            coach_submode: None,
            metro_submode: None,
            rail_submode: None,
            tram_submode: None,
            water_submode: None,
            telecabin_submode: None,
            access_mode: None,
            severity: None,
            scope: Vec::new(),
            predictability: None,
            keywords: None,
            verification: None,
            progress: Vec::new(),
            reality: None,
            operator_ref: None,
            operational_unit_ref: Vec::new(),
            network_ref: None,
            line_ref: Vec::new(),
            lines: None,
            stop_point_ref: Vec::new(),
            connection_link_ref: Vec::new(),
            facility_ref: Vec::new(),
            stop_place_ref: None,
            stop_place_component_ref: None,
            framed_vehicle_journey_ref: None,
            vehicle_journey_ref: None,
            interchange_ref: None,
            vehicle_ref: None,
            country_ref: None,
            place_ref: None,
            location: Vec::new(),
            situation_road_filter: None,
            accessibility_need_filter: Vec::new(),
            language: Vec::new(),
            include_translations: None,
            maximum_number_of_situation_elements: None,
            extensions: None,
        }
    }

    /// The submode this request filters on, if any.
    pub fn submode(&self) -> Option<Submode> {
        self.air_submode
            .map(Submode::Air)
            .or(self.bus_submode.map(Submode::Bus))
            .or(self.coach_submode.map(Submode::Coach))
            .or(self.metro_submode.map(Submode::Metro))
            .or(self.rail_submode.map(Submode::Rail))
            .or(self.tram_submode.map(Submode::Tram))
            .or(self.water_submode.map(Submode::Water))
            .or(self.telecabin_submode.map(Submode::Telecabin))
    }
}

/// Roads that a request filters on.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SituationRoadFilter {
    /// The individual roads.
    #[serde(rename = "RoadFilter")]
    pub road_filter: Vec<RoadFilter>,
}

/// One road, or a stretch of one, that a request filters on.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoadFilter {
    /// The road's designation, e.g. `A255`.
    #[serde(rename = "roadNumber", default, skip_serializing_if = "Option::is_none")]
    pub road_number: Option<String>,
    /// The carriageway the filter applies to.
    #[serde(rename = "directionBound", default, skip_serializing_if = "Option::is_none")]
    pub direction_bound: Option<Direction>,
    /// A point along the road the filter is anchored to.
    #[serde(rename = "referencePointIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub reference_point_identifier: Option<String>,
}

/// A subscription to the situations a producer holds.
///
/// The producer answers the matching situations immediately and then keeps sending
/// them as they change until the subscription is terminated or its
/// `initial_termination_time` passes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SituationExchangeSubscriptionRequest {
    /// Who is subscribing, when different from the requestor of the enclosing message.
    #[serde(rename = "SubscriberRef", default, skip_serializing_if = "Option::is_none")]
    pub subscriber_ref: Option<ParticipantRef>,
    /// The subscriber's name for this subscription, quoted in every delivery.
    #[serde(rename = "SubscriptionIdentifier")]
    pub subscription_identifier: SubscriptionQualifier,
    /// When the subscription lapses unless renewed.
    #[serde(rename = "InitialTerminationTime")]
    pub initial_termination_time: DateTime<FixedOffset>,
    /// Whether this replaces an existing subscription with the same identifier.
    #[serde(rename = "SubscriptionRenewal", default, skip_serializing_if = "Option::is_none")]
    pub subscription_renewal: Option<bool>,
    /// What to subscribe to.
    #[serde(rename = "SituationExchangeRequest")]
    pub situation_exchange_request: SituationExchangeRequest,
    /// Whether to send only what has changed rather than the full set each time.
    #[serde(rename = "IncrementalUpdates", default, skip_serializing_if = "Option::is_none")]
    pub incremental_updates: Option<bool>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl SituationExchangeSubscriptionRequest {
    /// A subscription running until `initial_termination_time`.
    pub fn new(
        subscription_identifier: impl Into<SubscriptionQualifier>,
        initial_termination_time: DateTime<FixedOffset>,
        situation_exchange_request: SituationExchangeRequest,
    ) -> Self {
        Self {
            subscriber_ref: None,
            subscription_identifier: subscription_identifier.into(),
            initial_termination_time,
            subscription_renewal: None,
            situation_exchange_request,
            incremental_updates: None,
            extensions: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timestamp() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2004-12-17T09:30:47-05:00").expect("valid timestamp")
    }

    #[test]
    fn an_unfiltered_request_writes_only_its_timestamp() {
        let request = SituationExchangeRequest::new(timestamp());
        let xml = quick_xml::se::to_string_with_root("SituationExchangeRequest", &request)
            .expect("request serialises");
        assert_eq!(
            xml,
            "<SituationExchangeRequest><RequestTimestamp>2004-12-17T09:30:47-05:00\
             </RequestTimestamp></SituationExchangeRequest>"
        );
    }

    #[test]
    fn filters_are_written_in_schema_order() {
        let mut request = SituationExchangeRequest::new(timestamp());
        request.scope = vec![ScopeType::Line];
        request.line_ref = vec![LineRef::new("52")];
        request.severity = Some(Severity::Severe);

        let xml = quick_xml::se::to_string_with_root("SituationExchangeRequest", &request)
            .expect("request serialises");
        let severity = xml.find("<Severity>").expect("severity is written");
        let scope = xml.find("<Scope>").expect("scope is written");
        let line = xml.find("<LineRef>").expect("line is written");
        assert!(severity < scope && scope < line, "{xml}");
    }

    #[test]
    fn a_request_reports_the_submode_it_filters_on() {
        let mut request = SituationExchangeRequest::new(timestamp());
        assert_eq!(request.submode(), None);
        request.rail_submode = Some(RailSubmodesOfTransport::SuburbanRailway);
        assert_eq!(
            request.submode(),
            Some(Submode::Rail(RailSubmodesOfTransport::SuburbanRailway))
        );
    }

    #[test]
    fn a_subscription_carries_the_request_it_subscribes_to() {
        let subscription = SituationExchangeSubscriptionRequest::new(
            "000234",
            timestamp(),
            SituationExchangeRequest::new(timestamp()),
        );
        let xml =
            quick_xml::se::to_string_with_root("SituationExchangeSubscriptionRequest", &subscription)
                .expect("subscription serialises");
        assert!(xml.contains("<SubscriptionIdentifier>000234</SubscriptionIdentifier>"), "{xml}");
        assert!(xml.contains("<SituationExchangeRequest>"), "{xml}");

        let read: SituationExchangeSubscriptionRequest =
            quick_xml::de::from_str(&xml).expect("subscription round-trips");
        assert_eq!(read, subscription);
    }
}
