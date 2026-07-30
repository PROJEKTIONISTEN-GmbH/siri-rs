//! Delivering a real-time timetable.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::framework::{ErrorCondition, ServiceRequestError};
use crate::model::{EstimatedServiceJourneyInterchange, EstimatedVehicleJourney, VersionRef};
use crate::types::{
    Duration, EndpointAddress, Extensions, MessageRef, ParticipantRef, SubscriptionFilterRef,
    SubscriptionRef,
};

/// The real-time timetable a producer is publishing.
///
/// A delivery either answers an
/// [`EstimatedTimetableRequest`](crate::et::EstimatedTimetableRequest) — in which
/// case it quotes the request's identifier — or satisfies a subscription, in which
/// case it quotes the subscription's.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EstimatedTimetableDelivery {
    /// Version of SIRI-ET the delivery conforms to.
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
    /// The journeys, grouped by the timetable version they were drawn from.
    #[serde(rename = "EstimatedJourneyVersionFrame")]
    pub estimated_journey_version_frame: Vec<EstimatedVersionFrame>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl EstimatedTimetableDelivery {
    /// A delivery carrying the given journeys as one version frame.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        journeys: Vec<EstimatedVehicleJourney>,
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
            estimated_journey_version_frame: vec![EstimatedVersionFrame::new(
                response_timestamp,
                journeys,
            )],
            extensions: None,
        }
    }

    /// The journeys this delivery carries, across all its version frames.
    pub fn journeys(&self) -> impl Iterator<Item = &EstimatedVehicleJourney> {
        self.estimated_journey_version_frame
            .iter()
            .flat_map(|frame| frame.estimated_vehicle_journey.iter())
    }
}

/// The journeys of one timetable version, as of one instant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EstimatedVersionFrame {
    /// When the producer last knew the frame's contents to be true.
    #[serde(rename = "RecordedAtTime")]
    pub recorded_at_time: DateTime<FixedOffset>,
    /// The edition of the timetable the journeys were drawn from.
    #[serde(rename = "VersionRef", default, skip_serializing_if = "Option::is_none")]
    pub version_ref: Option<VersionRef>,
    /// The journeys, at least one.
    #[serde(rename = "EstimatedVehicleJourney")]
    pub estimated_vehicle_journey: Vec<EstimatedVehicleJourney>,
    /// The interchanges planned around them, as now expected to work out.
    #[serde(rename = "EstimatedServiceJourneyInterchange", default, skip_serializing_if = "Vec::is_empty")]
    pub estimated_service_journey_interchange: Vec<EstimatedServiceJourneyInterchange>,
}

impl EstimatedVersionFrame {
    /// A frame holding the given journeys, recorded at the given instant.
    pub fn new(
        recorded_at_time: DateTime<FixedOffset>,
        estimated_vehicle_journey: Vec<EstimatedVehicleJourney>,
    ) -> Self {
        Self {
            recorded_at_time,
            version_ref: None,
            estimated_vehicle_journey,
            estimated_service_journey_interchange: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::EstimatedCall;

    fn timestamp() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2001-12-17T09:30:47-05:00").expect("valid timestamp")
    }

    fn journey(dated_vehicle_journey_ref: &str) -> EstimatedVehicleJourney {
        EstimatedVehicleJourney::dated("LZ123", "INBOUND", dated_vehicle_journey_ref)
            .with_estimated_calls(vec![EstimatedCall::at("00001")])
    }

    #[test]
    fn a_delivery_writes_its_header_before_the_journeys_and_reads_back() {
        let delivery = EstimatedTimetableDelivery {
            subscriber_ref: Some("NADER".into()),
            subscription_ref: Some("0004".into()),
            status: Some(true),
            ..EstimatedTimetableDelivery::new(timestamp(), vec![journey("00008")])
        };

        let xml = quick_xml::se::to_string_with_root("EstimatedTimetableDelivery", &delivery)
            .expect("delivery serialises");
        let subscriber = xml.find("<SubscriberRef>").expect("subscriber is written");
        let status = xml.find("<Status>").expect("status is written");
        let frame = xml
            .find("<EstimatedJourneyVersionFrame>")
            .expect("the frame is written");
        assert!(subscriber < status && status < frame, "{xml}");

        let read: EstimatedTimetableDelivery =
            quick_xml::de::from_str(&xml).expect("delivery round-trips");
        assert_eq!(read, delivery);
    }

    #[test]
    fn the_journeys_of_every_frame_are_reported_together() {
        let mut delivery = EstimatedTimetableDelivery::new(timestamp(), vec![journey("00008")]);
        delivery
            .estimated_journey_version_frame
            .push(EstimatedVersionFrame::new(timestamp(), vec![journey("00009")]));

        assert_eq!(
            delivery
                .journeys()
                .map(|journey| journey.dated_vehicle_journey_ref.as_ref().unwrap().as_str())
                .collect::<Vec<_>>(),
            ["00008", "00009"]
        );
    }
}
