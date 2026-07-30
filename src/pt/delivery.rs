//! Delivering a planned timetable.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::{FirstOrLastJourney, VehicleModesOfTransport};
use crate::framework::{ErrorCondition, ServiceRequestError};
use crate::model::{
    Branding, BrandingRef, DatedVehicleJourney, DirectionRef, GroupOfLinesRef, JourneyPatternRef,
    LineRef, OperatorRef, ProductCategoryRef, RemovedDatedVehicleJourney,
    RemovedServiceJourneyInterchange, RouteRef, ServiceFeatureRef, ServiceJourneyInterchange,
    VehicleFeatureRef, VersionRef,
};
use crate::pt::request::TimetableValidityPeriod;
use crate::types::{
    Duration, EndpointAddress, Extensions, MessageRef, NaturalLanguagePlaceName,
    NaturalLanguageString, ParticipantRef, SubscriptionFilterRef, SubscriptionRef,
};

/// The planned timetable a producer is publishing.
///
/// A delivery either answers a
/// [`ProductionTimetableRequest`](crate::pt::ProductionTimetableRequest) — in which
/// case it quotes the request's identifier — or satisfies a subscription, in which
/// case it quotes the subscription's.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductionTimetableDelivery {
    /// Version of SIRI-PT the delivery conforms to.
    #[serde(rename = "@version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// When the delivery was made.
    #[serde(rename = "ResponseTimestamp")]
    pub response_timestamp: DateTime<FixedOffset>,
    /// The request this delivery answers.
    #[serde(rename = "RequestMessageRef", default, skip_serializing_if = "Option::is_none")]
    pub request_message_ref: Option<MessageRef>,
    /// Who holds the subscription this delivery satisfies.
    #[serde(rename = "SubscriberRef", default, skip_serializing_if = "Option::is_none")]
    pub subscriber_ref: Option<ParticipantRef>,
    /// The shared filter the subscription uses.
    #[serde(rename = "SubscriptionFilterRef", default, skip_serializing_if = "Option::is_none")]
    pub subscription_filter_ref: Option<SubscriptionFilterRef>,
    /// The subscription this delivery satisfies.
    #[serde(rename = "SubscriptionRef", default, skip_serializing_if = "Option::is_none")]
    pub subscription_ref: Option<SubscriptionRef>,
    /// Address of the participant the data is delivered on behalf of.
    #[serde(rename = "DelegatorAddress", default, skip_serializing_if = "Option::is_none")]
    pub delegator_address: Option<EndpointAddress>,
    /// Participant the data is delivered on behalf of.
    #[serde(rename = "DelegatorRef", default, skip_serializing_if = "Option::is_none")]
    pub delegator_ref: Option<ParticipantRef>,
    /// Whether the request or subscription was processed successfully.
    #[serde(rename = "Status", default, skip_serializing_if = "Option::is_none")]
    pub status: Option<bool>,
    /// Why it could not be processed.
    #[serde(rename = "ErrorCondition", default, skip_serializing_if = "Option::is_none")]
    pub error_condition: Option<ErrorCondition<ServiceRequestError>>,
    /// How long this delivery holds.
    #[serde(rename = "ValidUntil", default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<FixedOffset>>,
    /// The shortest interval at which the producer will send updates.
    #[serde(rename = "ShortestPossibleCycle", default, skip_serializing_if = "Option::is_none")]
    pub shortest_possible_cycle: Option<Duration>,
    /// The language texts are in unless a journey says otherwise.
    #[serde(rename = "DefaultLanguage", default, skip_serializing_if = "Option::is_none")]
    pub default_language: Option<String>,
    /// The timetable itself, one frame per line and timetable version.
    #[serde(rename = "DatedTimetableVersionFrame", default, skip_serializing_if = "Vec::is_empty")]
    pub dated_timetable_version_frame: Vec<DatedTimetableVersionFrame>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl ProductionTimetableDelivery {
    /// A delivery carrying the given timetable frames.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        dated_timetable_version_frame: Vec<DatedTimetableVersionFrame>,
    ) -> Self {
        Self {
            version: None,
            response_timestamp,
            request_message_ref: None,
            subscriber_ref: None,
            subscription_filter_ref: None,
            subscription_ref: None,
            delegator_address: None,
            delegator_ref: None,
            status: None,
            error_condition: None,
            valid_until: None,
            shortest_possible_cycle: None,
            default_language: None,
            dated_timetable_version_frame,
            extensions: None,
        }
    }

    /// The planned runs this delivery carries, across all its frames.
    pub fn journeys(&self) -> impl Iterator<Item = &DatedVehicleJourney> {
        self.dated_timetable_version_frame
            .iter()
            .flat_map(|frame| frame.dated_vehicle_journey.iter())
    }
}

/// The planned runs of one line, drawn from one edition of the timetable.
///
/// What the runs have in common — line, direction, operator, product category — is
/// stated once on the frame rather than repeated on every run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatedTimetableVersionFrame {
    /// When the producer last knew the frame's contents to be true.
    #[serde(rename = "RecordedAtTime")]
    pub recorded_at_time: DateTime<FixedOffset>,
    /// The edition of the timetable the runs were drawn from.
    #[serde(rename = "VersionRef", default, skip_serializing_if = "Option::is_none")]
    pub version_ref: Option<VersionRef>,
    /// The period the frame covers.
    #[serde(rename = "ValidityPeriod", default, skip_serializing_if = "Option::is_none")]
    pub validity_period: Option<TimetableValidityPeriod>,
    /// The shortest interval at which the producer will send updates.
    #[serde(rename = "ShortestPossibleCycle", default, skip_serializing_if = "Option::is_none")]
    pub shortest_possible_cycle: Option<Duration>,
    /// The line the runs are on.
    #[serde(rename = "LineRef")]
    pub line_ref: LineRef,
    /// The direction they run in.
    #[serde(rename = "DirectionRef")]
    pub direction_ref: DirectionRef,
    /// The journey pattern they follow.
    #[serde(rename = "JourneyPatternRef", default, skip_serializing_if = "Option::is_none")]
    pub journey_pattern_ref: Option<JourneyPatternRef>,
    /// The journey pattern's name.
    #[serde(rename = "JourneyPatternName", default, skip_serializing_if = "Option::is_none")]
    pub journey_pattern_name: Option<NaturalLanguageString>,
    /// The modes of transport they use.
    #[serde(
        rename = "VehicleMode",
        default,
        deserialize_with = "crate::xml::token_list::deserialize",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub vehicle_mode: Vec<VehicleModesOfTransport>,
    /// The route they follow.
    #[serde(rename = "RouteRef", default, skip_serializing_if = "Option::is_none")]
    pub route_ref: Option<RouteRef>,
    /// The line's name as shown to passengers, one per language.
    #[serde(rename = "PublishedLineName", default, skip_serializing_if = "Vec::is_empty")]
    pub published_line_name: Vec<NaturalLanguageString>,
    /// The group of lines the line is marketed within.
    #[serde(rename = "GroupOfLinesRef", default, skip_serializing_if = "Option::is_none")]
    pub group_of_lines_ref: Option<GroupOfLinesRef>,
    /// The direction's name as shown to passengers, one per language.
    #[serde(rename = "DirectionName", default, skip_serializing_if = "Vec::is_empty")]
    pub direction_name: Vec<NaturalLanguageString>,
    /// The line as another operator's network identifies it.
    #[serde(rename = "ExternalLineRef", default, skip_serializing_if = "Option::is_none")]
    pub external_line_ref: Option<LineRef>,
    /// The brand the service is presented under, stated elsewhere.
    #[serde(rename = "BrandingRef", default, skip_serializing_if = "Option::is_none")]
    pub branding_ref: Option<BrandingRef>,
    /// The brand the service is presented under, stated here.
    #[serde(rename = "Branding", default, skip_serializing_if = "Option::is_none")]
    pub branding: Option<Branding>,
    /// The operator running the services.
    #[serde(rename = "OperatorRef", default, skip_serializing_if = "Option::is_none")]
    pub operator_ref: Option<OperatorRef>,
    /// The commercial category they belong to.
    #[serde(rename = "ProductCategoryRef", default, skip_serializing_if = "Option::is_none")]
    pub product_category_ref: Option<ProductCategoryRef>,
    /// Properties of the services, e.g. that cycles may be carried.
    #[serde(rename = "ServiceFeatureRef", default, skip_serializing_if = "Vec::is_empty")]
    pub service_feature_ref: Vec<ServiceFeatureRef>,
    /// Properties of the vehicles, e.g. that they have a low floor.
    #[serde(rename = "VehicleFeatureRef", default, skip_serializing_if = "Vec::is_empty")]
    pub vehicle_feature_ref: Vec<VehicleFeatureRef>,
    /// What is shown as the origin, one per language.
    #[serde(rename = "OriginDisplay", default, skip_serializing_if = "Vec::is_empty")]
    pub origin_display: Vec<NaturalLanguageString>,
    /// What is shown as the destination, one per language.
    #[serde(rename = "DestinationDisplay", default, skip_serializing_if = "Vec::is_empty")]
    pub destination_display: Vec<NaturalLanguageString>,
    /// Notes about the line, one per language.
    #[serde(rename = "LineNote", default, skip_serializing_if = "Vec::is_empty")]
    pub line_note: Vec<NaturalLanguagePlaceName>,
    /// Whether these are the first or the last runs of the day on the line.
    #[serde(rename = "FirstOrLastJourney", default, skip_serializing_if = "Option::is_none")]
    pub first_or_last_journey: Option<FirstOrLastJourney>,
    /// Whether the services run to a headway rather than to fixed times.
    #[serde(rename = "HeadwayService", default, skip_serializing_if = "Option::is_none")]
    pub headway_service: Option<bool>,
    /// Whether the runs will be tracked in real time.
    #[serde(rename = "Monitored", default, skip_serializing_if = "Option::is_none")]
    pub monitored: Option<bool>,
    /// The runs themselves.
    #[serde(rename = "DatedVehicleJourney", default, skip_serializing_if = "Vec::is_empty")]
    pub dated_vehicle_journey: Vec<DatedVehicleJourney>,
    /// Runs taken out of a timetable already published.
    #[serde(rename = "RemovedDatedVehicleJourney", default, skip_serializing_if = "Vec::is_empty")]
    pub removed_dated_vehicle_journey: Vec<RemovedDatedVehicleJourney>,
    /// The interchanges planned around the runs.
    #[serde(rename = "ServiceJourneyInterchange", default, skip_serializing_if = "Vec::is_empty")]
    pub service_journey_interchange: Vec<ServiceJourneyInterchange>,
    /// Interchanges taken out of a timetable already published.
    #[serde(rename = "RemovedServiceJourneyInterchange", default, skip_serializing_if = "Vec::is_empty")]
    pub removed_service_journey_interchange: Vec<RemovedServiceJourneyInterchange>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl DatedTimetableVersionFrame {
    /// A frame of runs on one line and direction, recorded at the given instant.
    pub fn new(
        recorded_at_time: DateTime<FixedOffset>,
        line_ref: impl Into<LineRef>,
        direction_ref: impl Into<DirectionRef>,
        dated_vehicle_journey: Vec<DatedVehicleJourney>,
    ) -> Self {
        Self {
            recorded_at_time,
            version_ref: None,
            validity_period: None,
            shortest_possible_cycle: None,
            line_ref: line_ref.into(),
            direction_ref: direction_ref.into(),
            journey_pattern_ref: None,
            journey_pattern_name: None,
            vehicle_mode: Vec::new(),
            route_ref: None,
            published_line_name: Vec::new(),
            group_of_lines_ref: None,
            direction_name: Vec::new(),
            external_line_ref: None,
            branding_ref: None,
            branding: None,
            operator_ref: None,
            product_category_ref: None,
            service_feature_ref: Vec::new(),
            vehicle_feature_ref: Vec::new(),
            origin_display: Vec::new(),
            destination_display: Vec::new(),
            line_note: Vec::new(),
            first_or_last_journey: None,
            headway_service: None,
            monitored: None,
            dated_vehicle_journey,
            removed_dated_vehicle_journey: Vec::new(),
            service_journey_interchange: Vec::new(),
            removed_service_journey_interchange: Vec::new(),
            extensions: None,
        }
    }
}
