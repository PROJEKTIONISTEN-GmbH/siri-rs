//! Delivering what a control room has decided.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::framework::{ErrorCondition, ServiceRequestError};
use crate::model::{
    ControlActionRef, FacilityRef, FramedVehicleJourneyRef, Location, SituationRef, TypeOfValue,
    VehicleRef,
};
use crate::types::{
    Duration, EndpointAddress, Extensions, ItemIdentifier, ItemRef, MessageQualifier, MessageRef,
    NaturalLanguageString, ParticipantRef, SubscriptionFilterRef, SubscriptionRef,
};

use super::action::{ifopt_namespace, ControlAction, JourneyScope};

/// What a control room has decided, and what came of it.
///
/// A delivery either answers a
/// [`ControlActionRequest`](crate::ca::ControlActionRequest) — in which case it
/// quotes the request's identifier — or satisfies a subscription, in which case it
/// quotes the subscription's. Unlike the other services it carries five kinds of
/// payload rather than one: the actions themselves, the sets they are grouped
/// into, the ones being revoked, the messages exchanged with drivers, and the
/// vehicles trackside equipment has detected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlActionDelivery {
    /// Version of SIRI-CA the delivery conforms to.
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
    /// The actions themselves.
    #[serde(rename = "controlActions", default, skip_serializing_if = "Option::is_none")]
    pub control_actions: Option<ControlActions>,
    /// Actions grouped into sets, and the sets themselves.
    #[serde(
        rename = "groupsOfControlActions",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub groups_of_control_actions: Option<GroupsOfControlActions>,
    /// Actions the control room is taking back.
    #[serde(
        rename = "revokedControlActions",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub revoked_control_actions: Option<RevokedControlActions>,
    /// Messages exchanged with drivers.
    #[serde(rename = "driverMessages", default, skip_serializing_if = "Option::is_none")]
    pub driver_messages: Option<DriverMessages>,
    /// Vehicles trackside equipment has detected.
    #[serde(rename = "vehicleDetectings", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_detectings: Option<VehicleDetectings>,
    /// Notes about the delivery as a whole, one per language.
    #[serde(rename = "Note", default, skip_serializing_if = "Vec::is_empty")]
    pub note: Vec<NaturalLanguageString>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl ControlActionDelivery {
    /// An empty delivery, to be filled in with the payloads it reports.
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
            control_actions: None,
            groups_of_control_actions: None,
            revoked_control_actions: None,
            driver_messages: None,
            vehicle_detectings: None,
            note: Vec::new(),
            extensions: None,
        }
    }

    /// A delivery carrying the given actions.
    pub fn of_actions(
        response_timestamp: DateTime<FixedOffset>,
        control_action: Vec<ControlAction>,
    ) -> Self {
        Self {
            control_actions: Some(ControlActions { control_action }),
            ..Self::new(response_timestamp)
        }
    }

    /// The actions this delivery carries.
    pub fn actions(&self) -> &[ControlAction] {
        self.control_actions
            .as_ref()
            .map(|actions| actions.control_action.as_slice())
            .unwrap_or_default()
    }
}

/// The actions a delivery carries.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ControlActions {
    /// One entry per action.
    #[serde(rename = "ControlAction")]
    pub control_action: Vec<ControlAction>,
}

/// The sets of related actions a delivery carries.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GroupsOfControlActions {
    /// One entry per set.
    #[serde(rename = "GroupOfControlActions")]
    pub group_of_control_actions: Vec<GroupOfControlActions>,
}

/// A set of actions taken together, and what is being added to or removed from it.
///
/// The set is anchored either on the master case it belongs to or on a situation;
/// [`GroupOfControlActions::for_master_case`] and
/// [`GroupOfControlActions::for_situation`] build the two.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupOfControlActions {
    /// The producer's identifier for the set.
    #[serde(rename = "GroupOfControlActionsCode")]
    pub group_of_control_actions_code: String,
    /// What the set is for.
    #[serde(rename = "PurposeOfGrouping", default, skip_serializing_if = "Option::is_none")]
    pub purpose_of_grouping: Option<NaturalLanguageString>,
    /// What the set is, in words, one text per language.
    #[serde(rename = "Description", default, skip_serializing_if = "Vec::is_empty")]
    pub description: Vec<NaturalLanguageString>,
    /// The master case the set belongs to.
    #[serde(rename = "MasterControlCaseRef", default, skip_serializing_if = "Option::is_none")]
    pub master_control_case_ref: Option<ControlActionRef>,
    /// The situation it belongs to, when it is anchored on one instead.
    #[serde(rename = "SituationRef", default, skip_serializing_if = "Option::is_none")]
    pub situation_ref: Option<SituationRef>,
    /// Actions being added to the set.
    #[serde(rename = "AddedControlActions", default, skip_serializing_if = "Option::is_none")]
    pub added_control_actions: Option<AddedControlActions>,
    /// Actions being taken out of it.
    #[serde(rename = "RemovedControlActions", default, skip_serializing_if = "Option::is_none")]
    pub removed_control_actions: Option<RemovedControlActions>,
}

impl GroupOfControlActions {
    /// A set anchored on a master case.
    pub fn for_master_case(
        group_of_control_actions_code: impl Into<String>,
        master_control_case_ref: impl Into<ControlActionRef>,
    ) -> Self {
        Self {
            master_control_case_ref: Some(master_control_case_ref.into()),
            ..Self::of(group_of_control_actions_code)
        }
    }

    /// A set anchored on a situation.
    pub fn for_situation(
        group_of_control_actions_code: impl Into<String>,
        situation_ref: impl Into<SituationRef>,
    ) -> Self {
        Self {
            situation_ref: Some(situation_ref.into()),
            ..Self::of(group_of_control_actions_code)
        }
    }

    /// The set with nothing anchoring it yet, which the schema does not allow.
    fn of(group_of_control_actions_code: impl Into<String>) -> Self {
        Self {
            group_of_control_actions_code: group_of_control_actions_code.into(),
            purpose_of_grouping: None,
            description: Vec::new(),
            master_control_case_ref: None,
            situation_ref: None,
            added_control_actions: None,
            removed_control_actions: None,
        }
    }
}

/// Actions being added to a set.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AddedControlActions {
    /// One reference per action.
    #[serde(rename = "AddedControlActionRef")]
    pub added_control_action_ref: Vec<ControlActionRef>,
}

/// Actions being taken out of a set.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RemovedControlActions {
    /// One reference per action.
    #[serde(rename = "RemovedControlActionRef")]
    pub removed_control_action_ref: Vec<ControlActionRef>,
}

/// The actions a delivery is taking back.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RevokedControlActions {
    /// One entry per revocation.
    #[serde(rename = "RevokedControlAction")]
    pub revoked_control_action: Vec<RevokedControlAction>,
}

/// One action taken back.
///
/// Either the whole action is revoked, named by its code, or only one update to it
/// is, named by the identifier that update was delivered under.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RevokedControlAction {
    /// The action being taken back.
    #[serde(rename = "ControlActionRef", default, skip_serializing_if = "Option::is_none")]
    pub control_action_ref: Option<ControlActionRef>,
    /// The one update being taken back, when the action itself stands.
    #[serde(rename = "ItemIdentifierRef", default, skip_serializing_if = "Option::is_none")]
    pub item_identifier_ref: Option<ItemRef>,
    /// When the revocation takes, or took, effect.
    #[serde(rename = "RevokedFromDateTime", default, skip_serializing_if = "Option::is_none")]
    pub revoked_from_date_time: Option<DateTime<FixedOffset>>,
    /// Who revoked it.
    #[serde(rename = "SourceNote", default, skip_serializing_if = "Option::is_none")]
    pub source_note: Option<NaturalLanguageString>,
}

impl RevokedControlAction {
    /// The revocation of a whole action.
    pub fn of_action(control_action_ref: impl Into<ControlActionRef>) -> Self {
        Self {
            control_action_ref: Some(control_action_ref.into()),
            item_identifier_ref: None,
            revoked_from_date_time: None,
            source_note: None,
        }
    }

    /// The revocation of one update to an action.
    pub fn of_update(item_identifier_ref: impl Into<ItemRef>) -> Self {
        Self {
            control_action_ref: None,
            item_identifier_ref: Some(item_identifier_ref.into()),
            revoked_from_date_time: None,
            source_note: None,
        }
    }
}

/// The messages exchanged with drivers a delivery carries.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DriverMessages {
    /// One entry per message.
    #[serde(rename = "DriverMessage")]
    pub driver_message: Vec<DriverMessage>,
}

/// One message exchanged with a driver.
///
/// The message is either written out, once per language, or given as a code the two
/// sides agreed on beforehand. `OriginatedByDriver` says which way it went.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DriverMessage {
    /// Reference to the format the content is in; free text when absent.
    #[serde(rename = "@formatRef", default, skip_serializing_if = "Option::is_none")]
    pub format_ref: Option<String>,
    /// Binding of the `ifopt` prefix the driver's journey may use.
    #[serde(rename = "@xmlns:ifopt", default = "ifopt_namespace")]
    pub ifopt_namespace: String,
    /// When the producer last knew this to be true.
    #[serde(rename = "RecordedAtTime")]
    pub recorded_at_time: DateTime<FixedOffset>,
    /// The producer's identifier for this record, so that it can be superseded.
    #[serde(rename = "ItemIdentifier", default, skip_serializing_if = "Option::is_none")]
    pub item_identifier: Option<ItemIdentifier>,
    /// The sender's identifier for the message.
    #[serde(rename = "MessageIdentifier")]
    pub message_identifier: MessageQualifier,
    /// Which update to the message this is.
    #[serde(rename = "MessageVersion", default, skip_serializing_if = "Option::is_none")]
    pub message_version: Option<u64>,
    /// How long the message holds; open-ended when absent.
    #[serde(rename = "ValidUntilTime", default, skip_serializing_if = "Option::is_none")]
    pub valid_until_time: Option<DateTime<FixedOffset>>,
    /// The situation the message is about.
    #[serde(rename = "SituationRef", default, skip_serializing_if = "Option::is_none")]
    pub situation_ref: Option<SituationRef>,
    /// The title of the message.
    #[serde(rename = "MessageHeading", default, skip_serializing_if = "Option::is_none")]
    pub message_heading: Option<NaturalLanguageString>,
    /// The message itself, one text per language.
    #[serde(rename = "MessageContents", default, skip_serializing_if = "Option::is_none")]
    pub message_contents: Option<MessageContents>,
    /// The agreed code standing for the message, when it is not written out.
    #[serde(rename = "MessageCode", default, skip_serializing_if = "Option::is_none")]
    pub message_code: Option<String>,
    /// Whether the driver sent the message rather than received it.
    #[serde(rename = "OriginatedByDriver", default, skip_serializing_if = "Option::is_none")]
    pub originated_by_driver: Option<bool>,
    /// Whether the message asks a parking point for a vehicle held in reserve.
    #[serde(rename = "CallForMeans", default, skip_serializing_if = "Option::is_none")]
    pub call_for_means: Option<bool>,
    /// Whether it asks a garage to repair a vehicle.
    #[serde(rename = "CallForRepairs", default, skip_serializing_if = "Option::is_none")]
    pub call_for_repairs: Option<bool>,
    /// Who the message is to, or from.
    #[serde(rename = "DriverScope")]
    pub driver_scope: DriverScope,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl DriverMessage {
    /// A message to or from the given driver, with no content yet.
    pub fn new(
        recorded_at_time: DateTime<FixedOffset>,
        message_identifier: impl Into<MessageQualifier>,
        driver_scope: DriverScope,
    ) -> Self {
        Self {
            format_ref: None,
            ifopt_namespace: ifopt_namespace(),
            recorded_at_time,
            item_identifier: None,
            message_identifier: message_identifier.into(),
            message_version: None,
            valid_until_time: None,
            situation_ref: None,
            message_heading: None,
            message_contents: None,
            message_code: None,
            originated_by_driver: None,
            call_for_means: None,
            call_for_repairs: None,
            driver_scope,
            extensions: None,
        }
    }
}

/// The text of a driver message, one entry per language.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MessageContents {
    /// The message.
    #[serde(rename = "MessageContent")]
    pub message_content: Vec<NaturalLanguageString>,
}

/// Who a driver message is to, or from.
///
/// The driver is named directly, by the duty being worked, by the vehicle being
/// driven, or by the journey being run.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DriverScope {
    /// The employee.
    #[serde(rename = "EmployeeRef", default, skip_serializing_if = "Option::is_none")]
    pub employee_ref: Option<String>,
    /// The day's work being carried out.
    #[serde(rename = "DutyRef", default, skip_serializing_if = "Option::is_none")]
    pub duty_ref: Option<String>,
    /// The vehicle being driven.
    #[serde(rename = "VehicleRef", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_ref: Option<VehicleRef>,
    /// The journey being run.
    #[serde(rename = "VehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_journey_ref: Option<crate::model::VehicleJourneyRef>,
    /// Every journey running in this direction of its line.
    #[serde(rename = "DirectionOfLineRef", default, skip_serializing_if = "Option::is_none")]
    pub direction_of_line_ref: Option<crate::model::DirectionRef>,
    /// Every journey on this line.
    #[serde(rename = "LineRef", default, skip_serializing_if = "Option::is_none")]
    pub line_ref: Option<crate::model::LineRef>,
    /// Every journey belonging to this authority.
    #[serde(rename = "TransportAuthorityRef", default, skip_serializing_if = "Option::is_none")]
    pub transport_authority_ref: Option<crate::model::AuthorityRef>,
    /// When the journey scope above holds.
    #[serde(rename = "TimeScope", default, skip_serializing_if = "Option::is_none")]
    pub time_scope: Option<crate::model::ValidityCondition>,
    /// Whose journeys, when the scope covers more than one operator's.
    #[serde(rename = "OperatorRef", default, skip_serializing_if = "Option::is_none")]
    pub operator_ref: Option<crate::model::OperatorRef>,
    /// One journey on one operational day, named in full.
    #[serde(rename = "DatedVehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub dated_vehicle_journey_ref: Option<FramedVehicleJourneyRef>,
}

impl DriverScope {
    /// The driver, by employee reference.
    pub fn for_employee(employee_ref: impl Into<String>) -> Self {
        Self {
            employee_ref: Some(employee_ref.into()),
            ..Self::default()
        }
    }

    /// Whoever is driving the given vehicle.
    pub fn for_vehicle(vehicle_ref: impl Into<VehicleRef>) -> Self {
        Self {
            vehicle_ref: Some(vehicle_ref.into()),
            ..Self::default()
        }
    }

    /// Whoever is running the given journeys.
    pub fn for_journeys(journey_scope: JourneyScope) -> Self {
        Self {
            vehicle_journey_ref: journey_scope.vehicle_journey_ref,
            direction_of_line_ref: journey_scope.direction_of_line_ref,
            line_ref: journey_scope.line_ref,
            transport_authority_ref: journey_scope.transport_authority_ref,
            time_scope: journey_scope.time_scope,
            operator_ref: journey_scope.operator_ref,
            dated_vehicle_journey_ref: journey_scope.dated_vehicle_journey_ref,
            ..Self::default()
        }
    }
}

/// The vehicle detections a delivery carries.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VehicleDetectings {
    /// One entry per detection.
    #[serde(rename = "VehicleDetecting")]
    pub vehicle_detecting: Vec<VehicleDetecting>,
}

/// A vehicle seen going past a fixed piece of equipment.
///
/// This is not vehicle tracking — that is
/// [`VehicleMonitoring`](crate::vm) — but a log entry from a device beside, under
/// or on the way the vehicle takes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleDetecting {
    /// Binding of the `ifopt` prefix the detected journey may use.
    #[serde(rename = "@xmlns:ifopt", default = "ifopt_namespace")]
    pub ifopt_namespace: String,
    /// The producer's identifier for this log entry.
    #[serde(rename = "Id")]
    pub id: ItemIdentifier,
    /// When the entry was first logged.
    #[serde(rename = "LoggedAtTime", default, skip_serializing_if = "Option::is_none")]
    pub logged_at_time: Option<DateTime<FixedOffset>>,
    /// What the entry is, in words.
    #[serde(rename = "Description", default, skip_serializing_if = "Option::is_none")]
    pub description: Option<NaturalLanguageString>,
    /// The entry's name.
    #[serde(rename = "Name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<NaturalLanguageString>,
    /// Where the detecting device is — not where the vehicle is.
    #[serde(rename = "DetectingLocation")]
    pub detecting_location: Location,
    /// How fast the vehicle was going, in kilometres per hour.
    #[serde(rename = "DetectedSpeed", default, skip_serializing_if = "Option::is_none")]
    pub detected_speed: Option<u64>,
    /// The equipment that made the detection.
    #[serde(rename = "DetectingEquipmentRef", default, skip_serializing_if = "Option::is_none")]
    pub detecting_equipment_ref: Option<FacilityRef>,
    /// What kind of detection it was, in the producer's own vocabulary.
    #[serde(rename = "TypeOfDetecting", default, skip_serializing_if = "Option::is_none")]
    pub type_of_detecting: Option<TypeOfValue>,
    /// The value that goes with that kind, e.g. a weight or a length.
    #[serde(rename = "MeasuredValue", default, skip_serializing_if = "Option::is_none")]
    pub measured_value: Option<f64>,
    /// The vehicle detected.
    #[serde(rename = "VehicleRef", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_ref: Option<VehicleRef>,
    /// The journey it was running, by the timetable identifier alone.
    #[serde(rename = "VehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub vehicle_journey_ref: Option<crate::model::VehicleJourneyRef>,
    /// The direction of the line it was running.
    #[serde(rename = "DirectionOfLineRef", default, skip_serializing_if = "Option::is_none")]
    pub direction_of_line_ref: Option<crate::model::DirectionRef>,
    /// The line it was running.
    #[serde(rename = "LineRef", default, skip_serializing_if = "Option::is_none")]
    pub line_ref: Option<crate::model::LineRef>,
    /// The authority the journey belongs to.
    #[serde(rename = "TransportAuthorityRef", default, skip_serializing_if = "Option::is_none")]
    pub transport_authority_ref: Option<crate::model::AuthorityRef>,
    /// When the journey scope above holds.
    #[serde(rename = "TimeScope", default, skip_serializing_if = "Option::is_none")]
    pub time_scope: Option<crate::model::ValidityCondition>,
    /// Whose journey it was.
    #[serde(rename = "OperatorRef", default, skip_serializing_if = "Option::is_none")]
    pub operator_ref: Option<crate::model::OperatorRef>,
    /// The journey it was running, named in full.
    #[serde(rename = "DatedVehicleJourneyRef", default, skip_serializing_if = "Option::is_none")]
    pub dated_vehicle_journey_ref: Option<FramedVehicleJourneyRef>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

impl VehicleDetecting {
    /// A detection by equipment at the given position.
    pub fn new(id: impl Into<ItemIdentifier>, detecting_location: Location) -> Self {
        Self {
            ifopt_namespace: ifopt_namespace(),
            id: id.into(),
            logged_at_time: None,
            description: None,
            name: None,
            detecting_location,
            detected_speed: None,
            detecting_equipment_ref: None,
            type_of_detecting: None,
            measured_value: None,
            vehicle_ref: None,
            vehicle_journey_ref: None,
            direction_of_line_ref: None,
            line_ref: None,
            transport_authority_ref: None,
            time_scope: None,
            operator_ref: None,
            dated_vehicle_journey_ref: None,
            extensions: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timestamp() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2026-03-04T08:15:00+01:00").expect("valid timestamp")
    }

    #[test]
    fn a_delivery_writes_its_payloads_in_schema_order_and_reads_back() {
        let delivery = ControlActionDelivery {
            revoked_control_actions: Some(RevokedControlActions {
                revoked_control_action: vec![RevokedControlAction::of_action("CA-4711")],
            }),
            driver_messages: Some(DriverMessages {
                driver_message: vec![DriverMessage::new(
                    timestamp(),
                    "DM-1",
                    DriverScope::for_vehicle("VEH-4711"),
                )],
            }),
            ..ControlActionDelivery::of_actions(
                timestamp(),
                vec![ControlAction::new(timestamp(), "CA-4711")],
            )
        };

        let xml = quick_xml::se::to_string_with_root("ControlActionDelivery", &delivery)
            .expect("delivery serialises");
        let actions = xml.find("<controlActions>").expect("the actions are written");
        let revoked = xml
            .find("<revokedControlActions>")
            .expect("the revocations are written");
        let messages = xml
            .find("<driverMessages>")
            .expect("the driver messages are written");
        assert!(actions < revoked && revoked < messages, "{xml}");

        let read: ControlActionDelivery =
            quick_xml::de::from_str(&xml).expect("delivery round-trips");
        assert_eq!(read, delivery);
        assert_eq!(read.actions().len(), 1);
    }

    #[test]
    fn a_revocation_names_either_the_action_or_one_update_to_it() {
        let whole = RevokedControlAction::of_action("CA-4711");
        assert!(whole.control_action_ref.is_some() && whole.item_identifier_ref.is_none());

        let update = RevokedControlAction::of_update("item-17");
        assert!(update.control_action_ref.is_none() && update.item_identifier_ref.is_some());
    }
}
