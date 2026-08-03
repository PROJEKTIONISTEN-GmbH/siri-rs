//! Delivering what is due at a stop.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::{ServiceException as ServiceExceptionStatus, VehicleModesOfTransport};
use crate::framework::{ErrorCondition, ServiceRequestError};
use crate::model::{
    Branding, BrandingRef, ClearDownRef, DirectionRef, FacilityRef, FramedVehicleJourneyRef,
    GroupOfLinesRef, JourneyPatternRef, LineRef, MonitoredVehicleJourney, MonitoringRef, RouteRef,
    SituationNumber, SituationRef, StopPointRef,
};
use crate::types::{
    Duration, EndpointAddress, Extensions, ItemIdentifier, ItemRef, MessageRef,
    NaturalLanguageString, ParticipantRef, SubscriptionFilterRef, SubscriptionRef,
};

/// What is due at one or more monitoring points.
///
/// A delivery either answers a
/// [`StopMonitoringRequest`](crate::sm::StopMonitoringRequest) — in which case it
/// quotes the request's identifier — or satisfies a subscription, in which case it
/// quotes the subscription's.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopMonitoringDelivery {
    /// Version of SIRI-SM the delivery conforms to.
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
    /// The language texts are in unless a visit says otherwise.
    #[serde(rename = "DefaultLanguage", default, skip_serializing_if = "Option::is_none")]
    pub default_language: Option<String>,
    /// The monitoring points this delivery is about.
    #[serde(rename = "MonitoringRef", default, skip_serializing_if = "Vec::is_empty")]
    pub monitoring_ref: Vec<MonitoringRef>,
    /// Names of those points, one per language.
    #[serde(rename = "MonitoringName", default, skip_serializing_if = "Vec::is_empty")]
    pub monitoring_name: Vec<NaturalLanguageString>,
    /// The visits themselves.
    #[serde(rename = "MonitoredStopVisit", default, skip_serializing_if = "Vec::is_empty")]
    pub monitored_stop_visit: Vec<MonitoredStopVisit>,
    /// Visits the producer previously reported and is now withdrawing.
    #[serde(rename = "MonitoredStopVisitCancellation", default, skip_serializing_if = "Vec::is_empty")]
    pub monitored_stop_visit_cancellation: Vec<MonitoredStopVisitCancellation>,
    /// Notices about a line at this stop, whether or not it has a visit due.
    #[serde(rename = "StopLineNotice", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_line_notice: Vec<StopLineNotice>,
    /// Line notices the producer is withdrawing.
    #[serde(rename = "StopLineNoticeCancellation", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_line_notice_cancellation: Vec<StopLineNoticeCancellation>,
    /// Notices about the stop itself.
    #[serde(rename = "StopNotice", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_notice: Vec<StopNotice>,
    /// Stop notices the producer is withdrawing.
    #[serde(rename = "StopNoticeCancellation", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_notice_cancellation: Vec<StopNoticeCancellation>,
    /// Why there is nothing to report, when that is the answer.
    #[serde(rename = "ServiceException", default, skip_serializing_if = "Vec::is_empty")]
    pub service_exception: Vec<ServiceException>,
    /// Notes about the delivery as a whole, one per language.
    #[serde(rename = "Note", default, skip_serializing_if = "Vec::is_empty")]
    pub note: Vec<NaturalLanguageString>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl StopMonitoringDelivery {
    /// A delivery carrying the given visits.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        monitored_stop_visit: Vec<MonitoredStopVisit>,
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
            monitoring_ref: Vec::new(),
            monitoring_name: Vec::new(),
            monitored_stop_visit,
            monitored_stop_visit_cancellation: Vec::new(),
            stop_line_notice: Vec::new(),
            stop_line_notice_cancellation: Vec::new(),
            stop_notice: Vec::new(),
            stop_notice_cancellation: Vec::new(),
            service_exception: Vec::new(),
            note: Vec::new(),
            extensions: None,
        }
    }
}

/// One service due at a monitoring point, with the journey behind it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonitoredStopVisit {
    /// When the producer last knew this to be true.
    #[serde(rename = "RecordedAtTime")]
    pub recorded_at_time: DateTime<FixedOffset>,
    /// The producer's identifier for this record, so that it can be superseded.
    #[serde(rename = "ItemIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub item_identifier: Option<ItemIdentifier>,
    /// How long the record may be relied on.
    #[serde(rename = "ValidUntilTime", default, skip_serializing_if = "Option::is_none")]
    pub valid_until_time: Option<DateTime<FixedOffset>>,
    /// The monitoring point the visit is at.
    #[serde(rename = "MonitoringRef", default, skip_serializing_if = "Option::is_none")]
    pub monitoring_ref: Option<MonitoringRef>,
    /// The identifier the on-street clear-down system knows this arrival by.
    #[serde(rename = "ClearDownRef", default, skip_serializing_if = "Option::is_none")]
    pub clear_down_ref: Option<ClearDownRef>,
    /// The journey making the visit.
    #[serde(rename = "MonitoredVehicleJourney")]
    pub monitored_vehicle_journey: MonitoredVehicleJourney,
    /// Notes about this visit, one per language.
    #[serde(rename = "StopVisitNote", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_visit_note: Vec<NaturalLanguageString>,
    /// The facility passengers wait at for this visit, e.g. a numbered shelter.
    #[serde(rename = "StopFacility", default, skip_serializing_if = "Option::is_none")]
    pub stop_facility: Option<FacilityRef>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl MonitoredStopVisit {
    /// A visit by the given journey, as known at `recorded_at_time`.
    pub fn new(
        recorded_at_time: DateTime<FixedOffset>,
        monitored_vehicle_journey: MonitoredVehicleJourney,
    ) -> Self {
        Self {
            recorded_at_time,
            item_identifier: None,
            valid_until_time: None,
            monitoring_ref: None,
            clear_down_ref: None,
            monitored_vehicle_journey,
            stop_visit_note: Vec::new(),
            stop_facility: None,
            extensions: None,
        }
    }
}

/// A visit the producer is withdrawing.
///
/// The visit is named either by the item identifier the producer gave it or, for a
/// consumer that keeps no such state, by the stop, line and journey it was about.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonitoredStopVisitCancellation {
    /// When the withdrawal was decided.
    #[serde(rename = "RecordedAtTime")]
    pub recorded_at_time: DateTime<FixedOffset>,
    /// The record being withdrawn.
    #[serde(rename = "ItemRef", default, skip_serializing_if = "Option::is_none")]
    pub item_ref: Option<ItemRef>,
    /// The monitoring point the withdrawn visit was at.
    #[serde(rename = "MonitoringRef", default, skip_serializing_if = "Option::is_none")]
    pub monitoring_ref: Option<MonitoringRef>,
    /// Which visit to that point it was, when the journey calls more than once.
    #[serde(rename = "VisitNumber", default, skip_serializing_if = "Option::is_none")]
    pub visit_number: Option<u64>,
    /// The line the withdrawn visit was on.
    #[serde(rename = "LineRef", default, skip_serializing_if = "Option::is_none")]
    pub line_ref: Option<LineRef>,
    /// The direction it ran in.
    #[serde(rename = "DirectionRef", default, skip_serializing_if = "Option::is_none")]
    pub direction_ref: Option<DirectionRef>,
    /// The journey it was about.
    #[serde(rename = "VehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_journey_ref: Option<FramedVehicleJourneyRef>,
    /// The identifier the on-street clear-down system knew the arrival by.
    #[serde(rename = "ClearDownRef", default, skip_serializing_if = "Option::is_none")]
    pub clear_down_ref: Option<ClearDownRef>,
    /// The journey pattern the journey follows.
    #[serde(rename = "JourneyPatternRef", default, skip_serializing_if = "Option::is_none")]
    pub journey_pattern_ref: Option<JourneyPatternRef>,
    /// The journey pattern's name.
    #[serde(rename = "JourneyPatternName", default, skip_serializing_if = "Option::is_none")]
    pub journey_pattern_name: Option<NaturalLanguageString>,
    /// The modes of transport it uses.
    #[serde(
        rename = "VehicleMode",
        default,
        deserialize_with = "crate::xml::token_list::deserialize",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub vehicle_mode: Vec<VehicleModesOfTransport>,
    /// The route it follows.
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
    /// Why the visit is being withdrawn, one text per language.
    #[serde(rename = "Reason", default, skip_serializing_if = "Vec::is_empty")]
    pub reason: Vec<NaturalLanguageString>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl MonitoredStopVisitCancellation {
    /// A withdrawal of the record the producer identified as `item_ref`.
    pub fn new(recorded_at_time: DateTime<FixedOffset>, item_ref: impl Into<ItemRef>) -> Self {
        Self {
            recorded_at_time,
            item_ref: Some(item_ref.into()),
            monitoring_ref: None,
            visit_number: None,
            line_ref: None,
            direction_ref: None,
            vehicle_journey_ref: None,
            clear_down_ref: None,
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
            reason: Vec::new(),
            extensions: None,
        }
    }
}

/// Something to tell passengers about one line at one stop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopLineNotice {
    /// When the notice was raised.
    #[serde(rename = "RecordedAtTime")]
    pub recorded_at_time: DateTime<FixedOffset>,
    /// The producer's identifier for this notice, so that it can be withdrawn.
    #[serde(rename = "ItemIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub item_identifier: Option<ItemIdentifier>,
    /// The monitoring point the notice applies at.
    #[serde(rename = "MonitoringRef")]
    pub monitoring_ref: MonitoringRef,
    /// The line it is about.
    #[serde(rename = "LineRef")]
    pub line_ref: LineRef,
    /// The direction it is about.
    #[serde(rename = "DirectionRef")]
    pub direction_ref: DirectionRef,
    /// The line's name as shown to passengers, one per language.
    #[serde(rename = "PublishedLineName", default, skip_serializing_if = "Vec::is_empty")]
    pub published_line_name: Vec<NaturalLanguageString>,
    /// The notice itself, one text per language.
    #[serde(rename = "LineNote", default, skip_serializing_if = "Vec::is_empty")]
    pub line_note: Vec<NaturalLanguageString>,
    /// The same notice worded for a particular kind of display.
    #[serde(rename = "DeliveryVariant", default, skip_serializing_if = "Vec::is_empty")]
    pub delivery_variant: Vec<DeliveryVariant>,
    /// Situations that explain the notice.
    #[serde(rename = "SituationRef", default, skip_serializing_if = "Vec::is_empty")]
    pub situation_ref: Vec<SituationRef>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl StopLineNotice {
    /// A notice about one line and direction at one monitoring point.
    pub fn new(
        recorded_at_time: DateTime<FixedOffset>,
        monitoring_ref: impl Into<MonitoringRef>,
        line_ref: impl Into<LineRef>,
        direction_ref: impl Into<DirectionRef>,
    ) -> Self {
        Self {
            recorded_at_time,
            item_identifier: None,
            monitoring_ref: monitoring_ref.into(),
            line_ref: line_ref.into(),
            direction_ref: direction_ref.into(),
            published_line_name: Vec::new(),
            line_note: Vec::new(),
            delivery_variant: Vec::new(),
            situation_ref: Vec::new(),
            extensions: None,
        }
    }
}

/// One wording of a notice, for a display that cannot show the full text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliveryVariant {
    /// Which kind of display this wording is for.
    #[serde(rename = "VariantType", default, skip_serializing_if = "Option::is_none")]
    pub variant_type: Option<String>,
    /// The wording.
    #[serde(rename = "Content")]
    pub content: NaturalLanguageString,
}

/// A line notice the producer is withdrawing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopLineNoticeCancellation {
    /// When the withdrawal was decided.
    #[serde(rename = "RecordedAtTime")]
    pub recorded_at_time: DateTime<FixedOffset>,
    /// The notice being withdrawn.
    #[serde(rename = "ItemRef", default, skip_serializing_if = "Option::is_none")]
    pub item_ref: Option<ItemRef>,
    /// The monitoring point the notice applied at.
    #[serde(rename = "MonitoringRef")]
    pub monitoring_ref: MonitoringRef,
    /// The line it was about.
    #[serde(rename = "LineRef")]
    pub line_ref: LineRef,
    /// The direction it was about.
    #[serde(rename = "DirectionRef")]
    pub direction_ref: DirectionRef,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// Something to tell passengers about the stop itself.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopNotice {
    /// When the notice was raised.
    #[serde(rename = "RecordedAtTime")]
    pub recorded_at_time: DateTime<FixedOffset>,
    /// The producer's identifier for this notice, so that it can be withdrawn.
    #[serde(rename = "ItemIdentifier")]
    pub item_identifier: ItemIdentifier,
    /// The monitoring point the notice applies at.
    #[serde(rename = "MonitoringRef")]
    pub monitoring_ref: MonitoringRef,
    /// The stop within that point, where the two differ.
    #[serde(rename = "StopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_point_ref: Option<StopPointRef>,
    /// Situations that explain the notice.
    #[serde(rename = "SituationRef", default, skip_serializing_if = "Vec::is_empty")]
    pub situation_ref: Vec<SituationRef>,
    /// The notice itself, one text per language.
    #[serde(rename = "StopNote", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_note: Vec<NaturalLanguageString>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// A stop notice the producer is withdrawing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopNoticeCancellation {
    /// When the withdrawal was decided.
    #[serde(rename = "RecordedAtTime")]
    pub recorded_at_time: DateTime<FixedOffset>,
    /// The notice being withdrawn.
    #[serde(rename = "ItemRef")]
    pub item_ref: ItemRef,
    /// The monitoring point the notice applied at.
    #[serde(rename = "MonitoringRef")]
    pub monitoring_ref: MonitoringRef,
    /// The stop within that point, where the two differ.
    #[serde(rename = "StopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_point_ref: Option<StopPointRef>,
    /// When the withdrawal takes effect, if not at once.
    #[serde(rename = "AppliesFromTime", default, skip_serializing_if = "Option::is_none")]
    pub applies_from_time: Option<DateTime<FixedOffset>>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// Why a stop has nothing to report.
///
/// An empty departure board is ambiguous — the last bus may have gone, the service
/// may be suspended, or the real-time feed may simply be down. This says which.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceException {
    /// When the producer last knew this to be true.
    #[serde(rename = "RecordedAtTime")]
    pub recorded_at_time: DateTime<FixedOffset>,
    /// The line the exception is about, when it is about one line.
    #[serde(rename = "LineRef", default, skip_serializing_if = "Option::is_none")]
    pub line_ref: Option<LineRef>,
    /// The direction it is about.
    #[serde(rename = "DirectionRef", default, skip_serializing_if = "Option::is_none")]
    pub direction_ref: Option<DirectionRef>,
    /// The stop it is about.
    #[serde(rename = "StopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_point_ref: Option<StopPointRef>,
    /// Which exception this is.
    #[serde(rename = "ServiceStatus", default, skip_serializing_if = "Option::is_none")]
    pub service_status: Option<ServiceExceptionStatus>,
    /// What to tell passengers, one text per language.
    #[serde(rename = "Notice", default, skip_serializing_if = "Vec::is_empty")]
    pub notice: Vec<NaturalLanguageString>,
    /// The situation that explains the exception.
    #[serde(rename = "SituationRef", default, skip_serializing_if = "Option::is_none")]
    pub situation_ref: Option<SituationNumber>,
}

impl ServiceException {
    /// An exception of the given kind, as known at `recorded_at_time`.
    pub fn new(
        recorded_at_time: DateTime<FixedOffset>,
        service_status: ServiceExceptionStatus,
    ) -> Self {
        Self {
            recorded_at_time,
            line_ref: None,
            direction_ref: None,
            stop_point_ref: None,
            service_status: Some(service_status),
            notice: Vec::new(),
            situation_ref: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timestamp() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2004-12-17T09:25:46-05:00").expect("valid timestamp")
    }

    #[test]
    fn a_delivery_writes_its_visits_before_its_notices_and_reads_back() {
        let mut delivery = StopMonitoringDelivery::new(
            timestamp(),
            vec![MonitoredStopVisit {
                monitoring_ref: Some("HLTST011".into()),
                ..MonitoredStopVisit::new(
                    timestamp(),
                    MonitoredVehicleJourney::on_line("Line123"),
                )
            }],
        );
        delivery.monitoring_ref = vec!["HLTST011".into()];
        delivery
            .monitored_stop_visit_cancellation
            .push(MonitoredStopVisitCancellation::new(timestamp(), "SED984"));
        delivery.stop_line_notice.push(StopLineNotice::new(
            timestamp(),
            "HLTST011",
            "123",
            "Out",
        ));

        let xml = quick_xml::se::to_string_with_root("StopMonitoringDelivery", &delivery)
            .expect("delivery serialises");
        let visit = xml.find("<MonitoredStopVisit>").expect("the visit is written");
        let cancellation = xml
            .find("<MonitoredStopVisitCancellation>")
            .expect("the cancellation is written");
        let notice = xml.find("<StopLineNotice>").expect("the notice is written");
        assert!(visit < cancellation && cancellation < notice, "{xml}");

        let read: StopMonitoringDelivery =
            quick_xml::de::from_str(&xml).expect("delivery round-trips");
        assert_eq!(read, delivery);
    }

    #[test]
    fn an_exception_says_why_the_board_is_empty() {
        let exception = ServiceException {
            line_ref: Some("Line123".into()),
            notice: vec![NaturalLanguageString::new("No more buses tonight")],
            ..ServiceException::new(timestamp(), ServiceExceptionStatus::AfterLastJourney)
        };

        let xml = quick_xml::se::to_string_with_root("ServiceException", &exception)
            .expect("exception serialises");
        assert!(xml.contains("<ServiceStatus>afterLastJourney</ServiceStatus>"), "{xml}");
        assert_eq!(
            quick_xml::de::from_str::<ServiceException>(&xml).expect("exception round-trips"),
            exception
        );
    }
}
