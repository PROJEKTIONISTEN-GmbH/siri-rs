//! Delivering how a connection is going, from each side of it.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::VehicleModesOfTransport;
use crate::framework::{ErrorCondition, ServiceRequestError};
use crate::model::{
    Branding, BrandingRef, ClearDownRef, ConnectionLinkRef, DirectionRef, FramedVehicleJourneyRef,
    GroupOfLinesRef, InterchangeJourney, InterchangeRef, JourneyPatternRef, LineRef, Location,
    RouteRef, StopPointRef,
};
use crate::types::{
    Duration, EndpointAddress, Extensions, ItemIdentifier, MessageRef, NaturalLanguageString,
    ParticipantRef, SubscriptionFilterRef, SubscriptionRef,
};

/// Declares the fields every connection-monitoring delivery shares.
///
/// The two deliveries differ only in their payload, so the header the schema gives
/// both of them is written once here rather than twice by hand.
macro_rules! connection_monitoring_delivery {
    (
        $(#[$meta:meta])*
        $name:ident { $($(#[$field_meta:meta])* $field:ident : $ty:ty),* $(,)? }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct $name {
            /// Version of SIRI-CM the delivery conforms to.
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
            /// The language texts are in unless an item says otherwise.
            #[serde(rename = "DefaultLanguage", default, skip_serializing_if = "Option::is_none")]
            pub default_language: Option<String>,
            $($(#[$field_meta])* pub $field: $ty,)*
            /// Implementation-defined content.
            #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
            pub extensions: Option<Extensions>,
        }

        impl $name {
            /// An empty delivery, to be filled in with the items it reports.
            pub fn new(response_timestamp: DateTime<FixedOffset>) -> Self {
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
                    $($field: Default::default(),)*
                    extensions: None,
                }
            }
        }
    };
}

connection_monitoring_delivery! {
    /// What the feeder side of an interchange is reporting.
    ConnectionMonitoringFeederDelivery {
        /// The feeder arrivals.
        #[serde(rename = "MonitoredFeederArrival", default, skip_serializing_if = "Vec::is_empty")]
        monitored_feeder_arrival: Vec<MonitoredFeederArrival>,
        /// Feeder arrivals the producer is withdrawing.
        #[serde(
            rename = "MonitoredFeederArrivalCancellation",
            default,
            skip_serializing_if = "Vec::is_empty"
        )]
        monitored_feeder_arrival_cancellation: Vec<MonitoredFeederArrivalCancellation>,
    }
}

connection_monitoring_delivery! {
    /// What the distributor side of an interchange has decided.
    ConnectionMonitoringDistributorDelivery {
        /// Departures being held for a late feeder.
        #[serde(rename = "WaitProlongedDeparture", default, skip_serializing_if = "Vec::is_empty")]
        wait_prolonged_departure: Vec<WaitProlongedDeparture>,
        /// Departures that have moved to another stopping position.
        #[serde(
            rename = "StoppingPositionChangedDeparture",
            default,
            skip_serializing_if = "Vec::is_empty"
        )]
        stopping_position_changed_departure: Vec<StoppingPositionChangedDeparture>,
        /// Departures that will not run.
        #[serde(
            rename = "DistributorDepartureCancellation",
            default,
            skip_serializing_if = "Vec::is_empty"
        )]
        distributor_departure_cancellation: Vec<DistributorDepartureCancellation>,
    }
}

/// A feeder service arriving at an interchange, as the feeder side now sees it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonitoredFeederArrival {
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
    /// The identifier the on-street clear-down system knows this arrival by.
    #[serde(rename = "ClearDownRef", default, skip_serializing_if = "Option::is_none")]
    pub clear_down_ref: Option<ClearDownRef>,
    /// The feeder journey passengers arrive on.
    #[serde(rename = "FeederJourney")]
    pub feeder_journey: InterchangeJourney,
    /// Whether the feeder is standing at the stop.
    #[serde(rename = "VehicleAtStop", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_at_stop: Option<bool>,
    /// How many passengers want to change here.
    #[serde(rename = "NumberOfTransferPassengers", default, skip_serializing_if = "Option::is_none")]
    pub number_of_transfer_passengers: Option<u64>,
    /// When the feeder was planned to arrive.
    #[serde(rename = "AimedArrivalTime", default, skip_serializing_if = "Option::is_none")]
    pub aimed_arrival_time: Option<DateTime<FixedOffset>>,
    /// When it is now expected to arrive.
    #[serde(rename = "ExpectedArrivalTime", default, skip_serializing_if = "Option::is_none")]
    pub expected_arrival_time: Option<DateTime<FixedOffset>>,
    /// The platform it arrives at.
    #[serde(rename = "ArrivalPlatformName", default, skip_serializing_if = "Option::is_none")]
    pub arrival_platform_name: Option<NaturalLanguageString>,
    /// By when the distributor has to decide whether to wait.
    #[serde(rename = "SuggestedWaitDecisionTime", default, skip_serializing_if = "Option::is_none")]
    pub suggested_wait_decision_time: Option<DateTime<FixedOffset>>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl MonitoredFeederArrival {
    /// An arrival of the given feeder over the given connection link.
    pub fn new(
        recorded_at_time: DateTime<FixedOffset>,
        connection_link_ref: impl Into<ConnectionLinkRef>,
        feeder_journey: InterchangeJourney,
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
            clear_down_ref: None,
            feeder_journey,
            vehicle_at_stop: None,
            number_of_transfer_passengers: None,
            aimed_arrival_time: None,
            expected_arrival_time: None,
            arrival_platform_name: None,
            suggested_wait_decision_time: None,
            extensions: None,
        }
    }
}

/// A feeder arrival the producer is withdrawing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonitoredFeederArrivalCancellation {
    /// When the withdrawal was decided.
    #[serde(rename = "RecordedAtTime")]
    pub recorded_at_time: DateTime<FixedOffset>,
    /// The producer's identifier for this withdrawal.
    #[serde(rename = "ItemIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub item_identifier: Option<ItemIdentifier>,
    /// How long the withdrawal may be relied on.
    #[serde(rename = "ValidUntilTime", default, skip_serializing_if = "Option::is_none")]
    pub valid_until_time: Option<DateTime<FixedOffset>>,
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

impl MonitoredFeederArrivalCancellation {
    /// A withdrawal of the arrival of the given feeder journey.
    pub fn new(
        recorded_at_time: DateTime<FixedOffset>,
        connection_link_ref: impl Into<ConnectionLinkRef>,
        line_ref: impl Into<LineRef>,
        direction_ref: impl Into<DirectionRef>,
        vehicle_journey_ref: FramedVehicleJourneyRef,
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

/// Declares one of the three things a distributor reports about a departure.
///
/// All three name the same departure the same way — the schema gives them a common
/// base type — and differ only in what they then say about it.
macro_rules! distributor_item {
    (
        $(#[$meta:meta])*
        $name:ident { $($(#[$field_meta:meta])* $field:ident : $ty:ty),* $(,)? }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct $name {
            /// When the producer last knew this to be true.
            #[serde(rename = "RecordedAtTime")]
            pub recorded_at_time: DateTime<FixedOffset>,
            /// The planned interchange this departure belongs to.
            #[serde(rename = "InterchangeRef", default, skip_serializing_if = "Option::is_none")]
            pub interchange_ref: Option<InterchangeRef>,
            /// The connection link passengers change over.
            #[serde(rename = "ConnectionLinkRef")]
            pub connection_link_ref: ConnectionLinkRef,
            /// The stop the distributor leaves from.
            #[serde(rename = "StopPointRef", default, skip_serializing_if = "Option::is_none")]
            pub stop_point_ref: Option<StopPointRef>,
            /// Which visit to that stop this is, when the distributor calls more than once.
            #[serde(rename = "DistributorVisitNumber", default, skip_serializing_if = "Option::is_none")]
            pub distributor_visit_number: Option<u64>,
            /// Where the stop comes in the distributor's journey, counting from one.
            #[serde(rename = "DistributorOrder", default, skip_serializing_if = "Option::is_none")]
            pub distributor_order: Option<u64>,
            /// The journey passengers leave on.
            #[serde(rename = "DistributorJourney")]
            pub distributor_journey: InterchangeJourney,
            /// The feeder journeys this departure is being held for.
            #[serde(rename = "FeederVehicleJourneyRef", default, skip_serializing_if = "Vec::is_empty")]
            pub feeder_vehicle_journey_ref: Vec<FramedVehicleJourneyRef>,
            $($(#[$field_meta])* pub $field: $ty,)*
            /// Implementation-defined content.
            #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
            pub extensions: Option<Extensions>,
        }

        impl $name {
            /// A report about the given departure over the given connection link.
            pub fn new(
                recorded_at_time: DateTime<FixedOffset>,
                connection_link_ref: impl Into<ConnectionLinkRef>,
                distributor_journey: InterchangeJourney,
            ) -> Self {
                Self {
                    recorded_at_time,
                    interchange_ref: None,
                    connection_link_ref: connection_link_ref.into(),
                    stop_point_ref: None,
                    distributor_visit_number: None,
                    distributor_order: None,
                    distributor_journey,
                    feeder_vehicle_journey_ref: Vec::new(),
                    $($field: Default::default(),)*
                    extensions: None,
                }
            }
        }
    };
}

distributor_item! {
    /// A departure being held so that passengers off a late feeder can catch it.
    WaitProlongedDeparture {
        /// When the departure is now expected to leave.
        #[serde(rename = "ExpectedDepartureTime", default, skip_serializing_if = "Option::is_none")]
        expected_departure_time: Option<DateTime<FixedOffset>>,
    }
}

distributor_item! {
    /// A departure that has moved to another stopping position.
    StoppingPositionChangedDeparture {
        /// What to tell passengers about the change, one text per language.
        #[serde(rename = "ChangeNote", default, skip_serializing_if = "Vec::is_empty")]
        change_note: Vec<NaturalLanguageString>,
        /// Where the departure now leaves from.
        #[serde(rename = "NewLocation", default, skip_serializing_if = "Option::is_none")]
        new_location: Option<Location>,
    }
}

distributor_item! {
    /// A departure that will not run, so the connection cannot be made.
    DistributorDepartureCancellation {
        /// Why it will not run, one text per language.
        #[serde(rename = "Reason", default, skip_serializing_if = "Vec::is_empty")]
        reason: Vec<NaturalLanguageString>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timestamp() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2001-12-17T09:30:47-05:00").expect("valid timestamp")
    }

    #[test]
    fn a_feeder_delivery_writes_its_arrivals_before_its_cancellations() {
        let mut delivery = ConnectionMonitoringFeederDelivery::new(timestamp());
        delivery.monitored_feeder_arrival.push(MonitoredFeederArrival {
            number_of_transfer_passengers: Some(12),
            ..MonitoredFeederArrival::new(
                timestamp(),
                "98789",
                InterchangeJourney::new("123", "OUT"),
            )
        });
        delivery
            .monitored_feeder_arrival_cancellation
            .push(MonitoredFeederArrivalCancellation::new(
                timestamp(),
                "987259",
                "123",
                "OUT",
                FramedVehicleJourneyRef {
                    data_frame_ref: "1967-08-13".into(),
                    dated_vehicle_journey_ref: "09867".into(),
                },
            ));

        let xml =
            quick_xml::se::to_string_with_root("ConnectionMonitoringFeederDelivery", &delivery)
                .expect("delivery serialises");
        let arrival = xml.find("<MonitoredFeederArrival>").expect("the arrival is written");
        let cancellation = xml
            .find("<MonitoredFeederArrivalCancellation>")
            .expect("the cancellation is written");
        assert!(arrival < cancellation, "{xml}");

        let read: ConnectionMonitoringFeederDelivery =
            quick_xml::de::from_str(&xml).expect("delivery round-trips");
        assert_eq!(read, delivery);
    }

    #[test]
    fn the_three_distributor_items_share_the_way_they_name_a_departure() {
        let mut delivery = ConnectionMonitoringDistributorDelivery::new(timestamp());
        delivery.wait_prolonged_departure.push(WaitProlongedDeparture {
            expected_departure_time: Some(timestamp()),
            ..WaitProlongedDeparture::new(
                timestamp(),
                "HLKT00023",
                InterchangeJourney::new("ABC", "Out"),
            )
        });
        delivery
            .stopping_position_changed_departure
            .push(StoppingPositionChangedDeparture {
                change_note: vec![NaturalLanguageString::new("Now leaves from platform 6")],
                ..StoppingPositionChangedDeparture::new(
                    timestamp(),
                    "HLKT00022",
                    InterchangeJourney::new("A11C", "OUT"),
                )
            });
        delivery
            .distributor_departure_cancellation
            .push(DistributorDepartureCancellation {
                reason: vec![NaturalLanguageString::new("Short staff")],
                ..DistributorDepartureCancellation::new(
                    timestamp(),
                    "987259",
                    InterchangeJourney::new("123", "OUT"),
                )
            });

        let xml = quick_xml::se::to_string_with_root(
            "ConnectionMonitoringDistributorDelivery",
            &delivery,
        )
        .expect("delivery serialises");
        assert_eq!(xml.matches("<ConnectionLinkRef>").count(), 3, "{xml}");
        assert_eq!(xml.matches("<DistributorJourney>").count(), 3, "{xml}");

        let read: ConnectionMonitoringDistributorDelivery =
            quick_xml::de::from_str(&xml).expect("delivery round-trips");
        assert_eq!(read, delivery);
    }
}
