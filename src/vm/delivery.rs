//! Delivering vehicle positions.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::VehicleModesOfTransport;
use crate::framework::{ErrorCondition, ServiceRequestError};
use crate::model::{
    Branding, BrandingRef, DirectionRef, FramedVehicleJourneyRef, GroupOfLinesRef,
    JourneyPatternRef, LineRef, MonitoredVehicleJourney, ProgressBetweenStops, RouteRef,
};
use crate::types::{
    Duration, EndpointAddress, Extensions, ItemIdentifier, ItemRef, MessageRef,
    NaturalLanguageString, ParticipantRef, SubscriptionFilterRef, SubscriptionRef,
};

use super::VehicleMonitoringRef;

/// The vehicles a producer is tracking.
///
/// A delivery either answers a
/// [`VehicleMonitoringRequest`](crate::vm::VehicleMonitoringRequest) — in which case
/// it quotes the request's identifier — or satisfies a subscription, in which case
/// it quotes the subscription's.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleMonitoringDelivery {
    /// Version of SIRI-VM the delivery conforms to.
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
    /// The language texts are in unless an activity says otherwise.
    #[serde(rename = "DefaultLanguage", default, skip_serializing_if = "Option::is_none")]
    pub default_language: Option<String>,
    /// The vehicles themselves.
    #[serde(rename = "VehicleActivity", default, skip_serializing_if = "Vec::is_empty")]
    pub vehicle_activity: Vec<VehicleActivity>,
    /// Vehicles the producer previously reported and is now withdrawing.
    #[serde(rename = "VehicleActivityCancellation", default, skip_serializing_if = "Vec::is_empty")]
    pub vehicle_activity_cancellation: Vec<VehicleActivityCancellation>,
    /// Notes about the delivery as a whole, one per language.
    #[serde(rename = "VehicleActivityNote", default, skip_serializing_if = "Vec::is_empty")]
    pub vehicle_activity_note: Vec<NaturalLanguageString>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl VehicleMonitoringDelivery {
    /// A delivery carrying the given vehicles.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        vehicle_activity: Vec<VehicleActivity>,
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
            vehicle_activity,
            vehicle_activity_cancellation: Vec::new(),
            vehicle_activity_note: Vec::new(),
            extensions: None,
        }
    }
}

/// One vehicle, where it is and what journey it is running.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleActivity {
    /// When the producer last knew this to be true.
    #[serde(rename = "RecordedAtTime")]
    pub recorded_at_time: DateTime<FixedOffset>,
    /// The producer's identifier for this record, so that it can be superseded.
    #[serde(rename = "ItemIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub item_identifier: Option<ItemIdentifier>,
    /// How long the record may be relied on.
    #[serde(rename = "ValidUntilTime")]
    pub valid_until_time: DateTime<FixedOffset>,
    /// The producer's vehicle-monitoring service this record belongs to.
    #[serde(rename = "VehicleMonitoringRef", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_monitoring_ref: Option<VehicleMonitoringRef>,
    /// Names of that service, one per language.
    #[serde(rename = "MonitoringName", default, skip_serializing_if = "Vec::is_empty")]
    pub monitoring_name: Vec<NaturalLanguageString>,
    /// How far the vehicle has come since the stop behind it.
    #[serde(rename = "ProgressBetweenStops", default, skip_serializing_if = "Option::is_none")]
    pub progress_between_stops: Option<ProgressBetweenStops>,
    /// The journey the vehicle is running.
    #[serde(rename = "MonitoredVehicleJourney")]
    pub monitored_vehicle_journey: MonitoredVehicleJourney,
    /// Notes about this vehicle, one per language.
    #[serde(rename = "VehicleActivityNote", default, skip_serializing_if = "Vec::is_empty")]
    pub vehicle_activity_note: Vec<NaturalLanguageString>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl VehicleActivity {
    /// A record of the given journey, good until `valid_until_time`.
    pub fn new(
        recorded_at_time: DateTime<FixedOffset>,
        valid_until_time: DateTime<FixedOffset>,
        monitored_vehicle_journey: MonitoredVehicleJourney,
    ) -> Self {
        Self {
            recorded_at_time,
            item_identifier: None,
            valid_until_time,
            vehicle_monitoring_ref: None,
            monitoring_name: Vec::new(),
            progress_between_stops: None,
            monitored_vehicle_journey,
            vehicle_activity_note: Vec::new(),
            extensions: None,
        }
    }
}

/// A vehicle record the producer is withdrawing.
///
/// A tracking system that has lost a vehicle, or has decided a record was wrong,
/// says so rather than letting the record time out.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleActivityCancellation {
    /// When the withdrawal was decided.
    #[serde(rename = "RecordedAtTime")]
    pub recorded_at_time: DateTime<FixedOffset>,
    /// The record being withdrawn.
    #[serde(rename = "ItemRef", default, skip_serializing_if = "Option::is_none")]
    pub item_ref: Option<ItemRef>,
    /// The producer's vehicle-monitoring service the record belonged to.
    #[serde(rename = "VehicleMonitoringRef", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_monitoring_ref: Option<VehicleMonitoringRef>,
    /// The journey the withdrawn record was about.
    #[serde(rename = "VehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_journey_ref: Option<FramedVehicleJourneyRef>,
    /// The line that journey runs on.
    #[serde(rename = "LineRef", default, skip_serializing_if = "Option::is_none")]
    pub line_ref: Option<LineRef>,
    /// The direction it runs in.
    #[serde(rename = "DirectionRef", default, skip_serializing_if = "Option::is_none")]
    pub direction_ref: Option<DirectionRef>,
    /// The journey pattern it follows.
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
    /// Why the record is being withdrawn, one text per language.
    #[serde(rename = "Reason", default, skip_serializing_if = "Vec::is_empty")]
    pub reason: Vec<NaturalLanguageString>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl VehicleActivityCancellation {
    /// A withdrawal of what was reported about the given journey.
    pub fn new(
        recorded_at_time: DateTime<FixedOffset>,
        vehicle_journey_ref: FramedVehicleJourneyRef,
    ) -> Self {
        Self {
            recorded_at_time,
            item_ref: None,
            vehicle_monitoring_ref: None,
            vehicle_journey_ref: Some(vehicle_journey_ref),
            line_ref: None,
            direction_ref: None,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DataFrameRef, DatedVehicleJourneyRef, Location};

    fn timestamp() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2004-12-17T09:30:47-05:00").expect("valid timestamp")
    }

    fn activity() -> VehicleActivity {
        VehicleActivity {
            vehicle_monitoring_ref: Some("ACT019456".into()),
            ..VehicleActivity::new(
                timestamp(),
                timestamp(),
                MonitoredVehicleJourney {
                    vehicle_location: Some(Location::wgs84(0.1, 53.55)),
                    bearing: Some(123.0),
                    vehicle_ref: Some("VEH987654".into()),
                    ..MonitoredVehicleJourney::on_line("Line123")
                },
            )
        }
    }

    #[test]
    fn an_activity_writes_its_identity_before_the_journey_and_reads_back() {
        let delivery = VehicleMonitoringDelivery::new(timestamp(), vec![activity()]);

        let xml = quick_xml::se::to_string_with_root("VehicleMonitoringDelivery", &delivery)
            .expect("delivery serialises");
        let valid_until = xml.find("<ValidUntilTime>").expect("the horizon is written");
        let monitoring = xml
            .find("<VehicleMonitoringRef>")
            .expect("the monitoring service is written");
        let journey = xml
            .find("<MonitoredVehicleJourney>")
            .expect("the journey is written");
        assert!(valid_until < monitoring && monitoring < journey, "{xml}");

        let read: VehicleMonitoringDelivery =
            quick_xml::de::from_str(&xml).expect("delivery round-trips");
        assert_eq!(read, delivery);
    }

    #[test]
    fn a_cancellation_is_written_after_the_activities_it_withdraws() {
        let mut delivery = VehicleMonitoringDelivery::new(timestamp(), vec![activity()]);
        delivery
            .vehicle_activity_cancellation
            .push(VehicleActivityCancellation::new(
                timestamp(),
                FramedVehicleJourneyRef {
                    data_frame_ref: DataFrameRef::new("2001-12-17"),
                    dated_vehicle_journey_ref: DatedVehicleJourneyRef::new("09867"),
                },
            ));

        let xml = quick_xml::se::to_string_with_root("VehicleMonitoringDelivery", &delivery)
            .expect("delivery serialises");
        let activity = xml.find("<VehicleActivity>").expect("the activity is written");
        let cancellation = xml
            .find("<VehicleActivityCancellation>")
            .expect("the cancellation is written");
        assert!(activity < cancellation, "{xml}");

        let read: VehicleMonitoringDelivery =
            quick_xml::de::from_str(&xml).expect("delivery round-trips");
        assert_eq!(read, delivery);
    }
}
