//! Delivering situations.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::{AccessModes, VehicleModesOfTransport};
use crate::framework::{ErrorCondition, ServiceRequestError};
use crate::model::{OperatorRef, Submode};
use crate::sx::action::Actions;
use crate::sx::affects::AffectedOperator;
use crate::sx::situation::{PtSituationElement, RoadSituationElement};
use crate::types::{
    CountryRef, Duration, EndpointAddress, Extensions, MessageRef, NaturalLanguageString,
    ParticipantRef, SubscriptionFilterRef, SubscriptionRef,
};

/// The situations a producer is publishing.
///
/// A delivery either answers a [`SituationExchangeRequest`](crate::sx::SituationExchangeRequest)
/// — in which case it quotes the request's identifier — or satisfies a
/// subscription, in which case it quotes the subscription's.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SituationExchangeDelivery {
    /// Version of SIRI-SX the delivery conforms to.
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
    /// The language texts are in unless a situation says otherwise.
    #[serde(rename = "DefaultLanguage", default, skip_serializing_if = "Option::is_none")]
    pub default_language: Option<String>,
    /// Values that apply to every situation in this delivery unless it overrides them.
    #[serde(rename = "PtSituationContext", default, skip_serializing_if = "Option::is_none")]
    pub pt_situation_context: Option<SituationContext>,
    /// The situations themselves.
    #[serde(rename = "Situations", default, skip_serializing_if = "Option::is_none")]
    pub situations: Option<Situations>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl SituationExchangeDelivery {
    /// A delivery carrying the given public transport situations.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        situations: Vec<PtSituationElement>,
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
            pt_situation_context: None,
            situations: Some(Situations {
                pt_situation_element: situations,
                road_situation_element: Vec::new(),
            }),
            extensions: None,
        }
    }

    /// The public transport situations this delivery carries.
    pub fn pt_situations(&self) -> &[PtSituationElement] {
        self.situations
            .as_ref()
            .map(|situations| situations.pt_situation_element.as_slice())
            .unwrap_or_default()
    }
}

/// The situations a delivery carries.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Situations {
    /// Situations affecting public transport.
    #[serde(rename = "PtSituationElement", default, skip_serializing_if = "Vec::is_empty")]
    pub pt_situation_element: Vec<PtSituationElement>,
    /// Situations affecting the road network.
    #[serde(rename = "RoadSituationElement", default, skip_serializing_if = "Vec::is_empty")]
    pub road_situation_element: Vec<RoadSituationElement>,
}

/// Values shared by every situation in a delivery.
///
/// Stating them once here rather than on each situation keeps a delivery of many
/// situations from repeating the same participant, place and network throughout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SituationContext {
    /// The country the situations are in.
    #[serde(rename = "CountryRef", default, skip_serializing_if = "Option::is_none")]
    pub country_ref: Option<CountryRef>,
    /// The participant whose situation numbers the delivery uses.
    #[serde(rename = "ParticipantRef")]
    pub participant_ref: ParticipantRef,
    /// The topographic place the situations are in.
    #[serde(rename = "TopographicPlaceRef", default, skip_serializing_if = "Option::is_none")]
    pub topographic_place_ref: Option<String>,
    /// Names of that place, one per language.
    #[serde(rename = "TopographicPlaceName", default, skip_serializing_if = "Vec::is_empty")]
    pub topographic_place_name: Vec<NaturalLanguageString>,
    /// The language texts are in unless a situation says otherwise.
    #[serde(rename = "DefaultLanguage", default, skip_serializing_if = "Option::is_none")]
    pub default_language: Option<String>,
    /// The operators and network the situations belong to.
    #[serde(rename = "NetworkContext", default, skip_serializing_if = "Option::is_none")]
    pub network_context: Option<NetworkContext>,
    /// Publishing actions that apply to every situation in the delivery.
    #[serde(rename = "Actions", default, skip_serializing_if = "Option::is_none")]
    pub actions: Option<Actions>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// The operators and network a delivery's situations belong to.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NetworkContext {
    /// The operators running the affected services.
    #[serde(rename = "Operator", default, skip_serializing_if = "Vec::is_empty")]
    pub operator: Vec<AffectedOperator>,
    /// The network the affected services belong to.
    #[serde(rename = "Network", default, skip_serializing_if = "Option::is_none")]
    pub network: Option<Network>,
}

/// A transport network, named and optionally narrowed to one mode.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Network {
    /// The network.
    #[serde(rename = "NetworkRef", default, skip_serializing_if = "Option::is_none")]
    pub network_ref: Option<OperatorRef>,
    /// Names of the network, one per language.
    #[serde(rename = "NetworkName", default, skip_serializing_if = "Vec::is_empty")]
    pub network_name: Vec<NaturalLanguageString>,
    /// The mode of transport the network is being referred to for.
    #[serde(rename = "VehicleMode", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_mode: Option<VehicleModesOfTransport>,
    /// The kind of air service concerned.
    #[serde(rename = "AirSubmode", default, skip_serializing_if = "Option::is_none")]
    pub air_submode: Option<crate::enumerations::AirSubmodesOfTransport>,
    /// The kind of bus service concerned.
    #[serde(rename = "BusSubmode", default, skip_serializing_if = "Option::is_none")]
    pub bus_submode: Option<crate::enumerations::BusSubmodesOfTransport>,
    /// The kind of coach service concerned.
    #[serde(rename = "CoachSubmode", default, skip_serializing_if = "Option::is_none")]
    pub coach_submode: Option<crate::enumerations::CoachSubmodesOfTransport>,
    /// The kind of metro service concerned.
    #[serde(rename = "MetroSubmode", default, skip_serializing_if = "Option::is_none")]
    pub metro_submode: Option<crate::enumerations::MetroSubmodesOfTransport>,
    /// The kind of rail service concerned.
    #[serde(rename = "RailSubmode", default, skip_serializing_if = "Option::is_none")]
    pub rail_submode: Option<crate::enumerations::RailSubmodesOfTransport>,
    /// The kind of tram service concerned.
    #[serde(rename = "TramSubmode", default, skip_serializing_if = "Option::is_none")]
    pub tram_submode: Option<crate::enumerations::TramSubmodesOfTransport>,
    /// The kind of water-borne service concerned.
    #[serde(rename = "WaterSubmode", default, skip_serializing_if = "Option::is_none")]
    pub water_submode: Option<crate::enumerations::WaterSubmodesOfTransport>,
    /// The kind of cable-drawn service concerned.
    #[serde(rename = "TelecabinSubmode", default, skip_serializing_if = "Option::is_none")]
    pub telecabin_submode: Option<crate::enumerations::TelecabinSubmodesOfTransport>,
    /// The way of reaching or leaving a stop concerned.
    #[serde(rename = "AccessMode", default, skip_serializing_if = "Option::is_none")]
    pub access_mode: Option<AccessModes>,
}

impl Network {
    /// The submode this network is narrowed to, if any.
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
