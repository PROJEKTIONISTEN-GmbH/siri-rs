//! Delivering the connections planned over a connection link.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::VehicleModesOfTransport;
use crate::framework::{ErrorCondition, ServiceRequestError};
use crate::model::{
    Branding, BrandingRef, ConnectionLinkRef, DirectionRef, FramedVehicleJourneyRef,
    GroupOfLinesRef, InterchangeJourney, InterchangeRef, JourneyPatternRef, LineRef, RouteRef,
    StopPointRef,
};
use crate::types::{
    Duration, EndpointAddress, Extensions, ItemIdentifier, ItemRef, MessageRef,
    NaturalLanguageString, ParticipantRef, SubscriptionFilterRef, SubscriptionRef,
};

/// The connections planned over one connection link.
///
/// A delivery either answers a
/// [`ConnectionTimetableRequest`](crate::ct::ConnectionTimetableRequest) — in which
/// case it quotes the request's identifier — or satisfies a subscription, in which
/// case it quotes the subscription's.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionTimetableDelivery {
    /// Version of SIRI-CT the delivery conforms to.
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
    /// The language texts are in unless an arrival says otherwise.
    #[serde(rename = "DefaultLanguage", default, skip_serializing_if = "Option::is_none")]
    pub default_language: Option<String>,
    /// The planned feeder arrivals.
    #[serde(rename = "TimetabledFeederArrival", default, skip_serializing_if = "Vec::is_empty")]
    pub timetabled_feeder_arrival: Vec<TimetabledFeederArrival>,
    /// Feeder arrivals the producer previously published and is now withdrawing.
    #[serde(
        rename = "TimetabledFeederArrivalCancellation",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub timetabled_feeder_arrival_cancellation: Vec<TimetabledFeederArrivalCancellation>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl ConnectionTimetableDelivery {
    /// A delivery carrying the given planned feeder arrivals.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        timetabled_feeder_arrival: Vec<TimetabledFeederArrival>,
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
            timetabled_feeder_arrival,
            timetabled_feeder_arrival_cancellation: Vec::new(),
            extensions: None,
        }
    }
}

/// One feeder service planned to arrive in time for a connection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimetabledFeederArrival {
    /// When the producer last knew this to be true.
    #[serde(rename = "RecordedAtTime")]
    pub recorded_at_time: DateTime<FixedOffset>,
    /// The producer's identifier for this record, so that it can be superseded.
    #[serde(rename = "ItemIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub item_identifier: Option<ItemIdentifier>,
    /// How long the record may be relied on.
    #[serde(rename = "ValidUntilTime", default, skip_serializing_if = "Option::is_none")]
    pub valid_until_time: Option<DateTime<FixedOffset>>,
    /// The planned interchange this arrival belongs to.
    #[serde(rename = "InterchangeRef", default, skip_serializing_if = "Option::is_none")]
    pub interchange_ref: Option<InterchangeRef>,
    /// The connection link passengers change over.
    #[serde(rename = "ConnectionLinkRef")]
    pub connection_link_ref: ConnectionLinkRef,
    /// The stop the feeder arrives at.
    #[serde(rename = "StopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_point_ref: Option<StopPointRef>,
    /// Which visit to that stop this is, when the feeder calls more than once.
    #[serde(rename = "VisitNumber", default, skip_serializing_if = "Option::is_none")]
    pub visit_number: Option<u64>,
    /// Where the stop comes in the feeder's journey, counting from one.
    #[serde(rename = "Order", default, skip_serializing_if = "Option::is_none")]
    pub order: Option<u64>,
    /// Names of the stop, one per language.
    #[serde(rename = "StopPointName", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_point_name: Vec<NaturalLanguageString>,
    /// The feeder journey passengers arrive on.
    #[serde(rename = "FeederJourney")]
    pub feeder_journey: InterchangeJourney,
    /// When it is planned to arrive.
    #[serde(rename = "AimedArrivalTime")]
    pub aimed_arrival_time: DateTime<FixedOffset>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl TimetabledFeederArrival {
    /// A feeder arrival planned over the given connection link.
    pub fn new(
        recorded_at_time: DateTime<FixedOffset>,
        connection_link_ref: impl Into<ConnectionLinkRef>,
        feeder_journey: InterchangeJourney,
        aimed_arrival_time: DateTime<FixedOffset>,
    ) -> Self {
        Self {
            recorded_at_time,
            item_identifier: None,
            valid_until_time: None,
            interchange_ref: None,
            connection_link_ref: connection_link_ref.into(),
            stop_point_ref: None,
            visit_number: None,
            order: None,
            stop_point_name: Vec::new(),
            feeder_journey,
            aimed_arrival_time,
            extensions: None,
        }
    }
}

/// A planned feeder arrival the producer is withdrawing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimetabledFeederArrivalCancellation {
    /// When the withdrawal was decided.
    #[serde(rename = "RecordedAtTime")]
    pub recorded_at_time: DateTime<FixedOffset>,
    /// The record being withdrawn.
    #[serde(rename = "ItemRef", default, skip_serializing_if = "Option::is_none")]
    pub item_ref: Option<ItemRef>,
    /// The planned interchange the arrival belonged to.
    #[serde(rename = "InterchangeRef", default, skip_serializing_if = "Option::is_none")]
    pub interchange_ref: Option<InterchangeRef>,
    /// The connection link passengers were to change over.
    #[serde(rename = "ConnectionLinkRef")]
    pub connection_link_ref: ConnectionLinkRef,
    /// The stop the feeder was to arrive at.
    #[serde(rename = "StopPointRef", default, skip_serializing_if = "Option::is_none")]
    pub stop_point_ref: Option<StopPointRef>,
    /// Which visit to that stop it was.
    #[serde(rename = "VisitNumber", default, skip_serializing_if = "Option::is_none")]
    pub visit_number: Option<u64>,
    /// Where the stop came in the feeder's journey, counting from one.
    #[serde(rename = "Order", default, skip_serializing_if = "Option::is_none")]
    pub order: Option<u64>,
    /// Names of the stop, one per language.
    #[serde(rename = "StopPointName", default, skip_serializing_if = "Vec::is_empty")]
    pub stop_point_name: Vec<NaturalLanguageString>,
    /// The line the feeder ran on.
    #[serde(rename = "LineRef")]
    pub line_ref: LineRef,
    /// The direction it ran in.
    #[serde(rename = "DirectionRef")]
    pub direction_ref: DirectionRef,
    /// The feeder journey itself.
    #[serde(rename = "VehicleJourneyRef")]
    pub vehicle_journey_ref: FramedVehicleJourneyRef,
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
    /// Why the arrival is being withdrawn, one text per language.
    #[serde(rename = "Reason", default, skip_serializing_if = "Vec::is_empty")]
    pub reason: Vec<NaturalLanguageString>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl TimetabledFeederArrivalCancellation {
    /// A withdrawal of the planned arrival of the given journey.
    pub fn new(
        recorded_at_time: DateTime<FixedOffset>,
        connection_link_ref: impl Into<ConnectionLinkRef>,
        line_ref: impl Into<LineRef>,
        direction_ref: impl Into<DirectionRef>,
        vehicle_journey_ref: FramedVehicleJourneyRef,
    ) -> Self {
        Self {
            recorded_at_time,
            item_ref: None,
            interchange_ref: None,
            connection_link_ref: connection_link_ref.into(),
            stop_point_ref: None,
            visit_number: None,
            order: None,
            stop_point_name: Vec::new(),
            line_ref: line_ref.into(),
            direction_ref: direction_ref.into(),
            vehicle_journey_ref,
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
    use crate::model::{DataFrameRef, DatedVehicleJourneyRef};

    fn timestamp() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2001-12-17T09:30:47-05:00").expect("valid timestamp")
    }

    fn journey_ref() -> FramedVehicleJourneyRef {
        FramedVehicleJourneyRef {
            data_frame_ref: DataFrameRef::new("1967-08-13"),
            dated_vehicle_journey_ref: DatedVehicleJourneyRef::new("09876"),
        }
    }

    #[test]
    fn an_arrival_writes_its_link_before_its_journey_and_reads_back() {
        let mut delivery = ConnectionTimetableDelivery::new(
            timestamp(),
            vec![TimetabledFeederArrival {
                visit_number: Some(2),
                ..TimetabledFeederArrival::new(
                    timestamp(),
                    "98789",
                    InterchangeJourney::new("123", "OUT"),
                    timestamp(),
                )
            }],
        );
        delivery
            .timetabled_feeder_arrival_cancellation
            .push(TimetabledFeederArrivalCancellation::new(
                timestamp(),
                "98789",
                "123",
                "OUT",
                journey_ref(),
            ));

        let xml = quick_xml::se::to_string_with_root("ConnectionTimetableDelivery", &delivery)
            .expect("delivery serialises");
        let link = xml.find("<ConnectionLinkRef>").expect("the link is written");
        let journey = xml.find("<FeederJourney>").expect("the feeder is written");
        let arrival = xml.find("<AimedArrivalTime>").expect("the arrival is written");
        assert!(link < journey && journey < arrival, "{xml}");

        let read: ConnectionTimetableDelivery =
            quick_xml::de::from_str(&xml).expect("delivery round-trips");
        assert_eq!(read, delivery);
    }
}
