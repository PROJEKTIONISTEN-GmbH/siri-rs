//! Delivering the state of passenger facilities.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::framework::{ErrorCondition, ServiceRequestError};
use crate::model::FacilityCondition;
use crate::types::{
    Duration, EndpointAddress, Extensions, MessageRef, ParticipantRef, SubscriptionFilterRef,
    SubscriptionRef,
};

/// The state of the facilities a producer watches.
///
/// A delivery either answers a
/// [`FacilityMonitoringRequest`](crate::fm::FacilityMonitoringRequest) — in which
/// case it quotes the request's identifier — or satisfies a subscription, in which
/// case it quotes the subscription's.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FacilityMonitoringDelivery {
    /// Version of SIRI-FM the delivery conforms to.
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
    /// The language texts are in unless a condition says otherwise.
    #[serde(rename = "DefaultLanguage", default, skip_serializing_if = "Option::is_none")]
    pub default_language: Option<String>,
    /// The facilities and what state they are in.
    #[serde(rename = "FacilityCondition", default, skip_serializing_if = "Vec::is_empty")]
    pub facility_condition: Vec<FacilityCondition>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl FacilityMonitoringDelivery {
    /// A delivery carrying the given facility conditions.
    pub fn new(
        response_timestamp: DateTime<FixedOffset>,
        facility_condition: Vec<FacilityCondition>,
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
            facility_condition,
            extensions: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enumerations::FacilityStatus as FacilityStatusValue;
    use crate::model::{Facility, FacilityStatus};

    #[test]
    fn a_delivery_carries_the_condition_of_each_facility() {
        let timestamp =
            DateTime::parse_from_rfc3339("2004-12-17T09:30:47-05:00").expect("valid timestamp");
        let delivery = FacilityMonitoringDelivery::new(
            timestamp,
            vec![FacilityCondition::for_facility(
                Facility::with_code("134567-L4"),
                FacilityStatus::new(FacilityStatusValue::NotAvailable),
            )],
        );

        let xml = quick_xml::se::to_string_with_root("FacilityMonitoringDelivery", &delivery)
            .expect("delivery serialises");
        assert!(xml.contains("<FacilityCode>134567-L4</FacilityCode>"), "{xml}");
        assert!(xml.contains("<Status>notAvailable</Status>"), "{xml}");

        let read: FacilityMonitoringDelivery =
            quick_xml::de::from_str(&xml).expect("delivery round-trips");
        assert_eq!(read, delivery);
    }
}
