//! Control Actions, checked against the schemas rather than against examples.
//!
//! The other services are proved against the example documents published with the
//! standard: each is read, written back and compared. Control Actions ships no
//! example messages at all, so there is nothing to compare against and this checks
//! what can still be checked — that a message the crate builds is one the official
//! schemas accept, and that reading it back yields what was written.
//!
//! Each control action the schema allows is exercised once, because the schema's
//! choice between them is the part of this service most easily got wrong.

mod support;

use chrono::{DateTime, Duration, FixedOffset};

use siri_rs::ca::{
    CallCancellationAction, CancelledConnection, ChangeOfJourneyTiming, ChangeOfStopPointStatus,
    ChangedPoint, ControlAction, ControlActionCapabilitiesResponse, ControlActionDelivery,
    ControlActionKind, ControlActionReason, ControlActionRequest,
    ControlActionServiceCapabilities, ControlActionSubscriptionRequest, ControlActions,
    DatedCallRef, DriverMessage, DriverMessages, DriverScope, ExtraConnection,
    FlexibleJourneyActivation, GroupOfControlActions, GroupsOfControlActions, JourneyCreation,
    JourneyEnd, JourneyPatternModification, JourneyScope, JourneyStart, MessageContents,
    MiddleCall, ModifiedConnection, PointInJourneyPatternRef, RelativeTime, RevokedControlAction,
    RevokedControlActions, StopRefs, StopPointStatusTimeScope, TargetPoint, TimeScope,
    VehicleDetecting, VehicleDetectings, VehicleWorkAssignment,
};
use siri_rs::enumerations::{
    ChangeModel, ChangeOfJourneyTimingType, ControlActionReasonCategory, StopPlaceStatus,
    TypeOfActivatedJourney,
};
use siri_rs::framework::{ServiceDelivery, ServiceRequest, SubscriptionRequest};
use siri_rs::model::{FramedVehicleJourneyRef, ValidityCondition};
use siri_rs::types::{Duration as SiriDuration, NaturalLanguageString};
use siri_rs::{Siri, SiriRoot};
use support::{validate, validator_available, VALIDATOR_MISSING};

/// The version the messages built here declare.
const VERSION: &str = "2.1";

#[test]
fn a_request_for_control_actions_is_valid_and_reads_back() {
    let request = ControlActionRequest {
        preview_interval: Some(SiriDuration::parse("PT2H").expect("valid duration")),
        start_time: Some(now()),
        operator_ref: Some("USTRA".into()),
        operational_unit_ref: vec!["control-room-1".into()],
        line_ref: vec!["10".into(), "17".into()],
        include_driver_messages: Some(true),
        include_vehicle_detectings: Some(false),
        language: vec!["de".to_owned()],
        include_situations: Some(true),
        maximum_number_of_control_actions: Some(50),
        ..ControlActionRequest::new(now())
    };

    let message = Siri::new(
        VERSION,
        ServiceRequest::new(now(), "CONTROL-ROOM", vec![request.clone().into()]),
    );
    let read = round_trip(&message);
    assert_eq!(read, message);
}

#[test]
fn a_subscription_to_control_actions_is_valid_and_reads_back() {
    let subscription = ControlActionSubscriptionRequest {
        incremental_updates: Some(true),
        ..ControlActionSubscriptionRequest::new(
            "control-actions",
            now() + Duration::hours(8),
            ControlActionRequest::new(now()),
        )
    };

    let message = Siri::new(
        VERSION,
        SubscriptionRequest::new(now(), "CONTROL-ROOM", vec![subscription.into()]),
    );
    assert_eq!(round_trip(&message), message);
}

/// Every alternative of the schema's choice of action, one control action each.
#[test]
fn every_kind_of_control_action_is_valid_and_reads_back() {
    for (name, action) in every_action() {
        let delivery = ControlActionDelivery {
            control_actions: Some(ControlActions {
                control_action: vec![action.clone()],
            }),
            ..ControlActionDelivery::new(now())
        };
        let message = Siri::new(
            VERSION,
            ServiceDelivery::new(now(), "MY-AGENCY", vec![delivery.into()]),
        );

        assert_valid(&message, name);
        assert_eq!(read_back(&message), message, "{name} does not read back");
        assert_eq!(
            action.kind().map(kind_name),
            Some(name),
            "the action reports the alternative it carries"
        );
    }
}

#[test]
fn a_delivery_carries_groups_revocations_messages_and_detections() {
    let delivery = ControlActionDelivery {
        groups_of_control_actions: Some(GroupsOfControlActions {
            group_of_control_actions: vec![GroupOfControlActions {
                purpose_of_grouping: Some(NaturalLanguageString::new("The bridge is shut")),
                added_control_action_ref: vec!["CA-4711".into()],
                removed_control_action_ref: vec!["CA-4710".into()],
                ..GroupOfControlActions::for_master_case("bridge-works", "MC-2026-0041")
            }],
        }),
        revoked_control_actions: Some(RevokedControlActions {
            revoked_control_action: vec![RevokedControlAction {
                revoked_from_date_time: Some(now() + Duration::minutes(5)),
                source_note: Some(NaturalLanguageString::new("Withdrawn by the duty officer")),
                ..RevokedControlAction::of_action("CA-4711")
            }],
        }),
        driver_messages: Some(DriverMessages {
            driver_message: vec![DriverMessage {
                message_heading: Some(NaturalLanguageString::new("Turn back at Kröpcke")),
                message_contents: Some(MessageContents {
                    message_content: vec![NaturalLanguageString::new(
                        "Please turn back at Kröpcke and resume the timetable outbound",
                    )],
                }),
                ..DriverMessage::new(now(), "DM-1", DriverScope::for_employee("driver-42"))
            }],
        }),
        vehicle_detectings: Some(VehicleDetectings {
            vehicle_detecting: vec![VehicleDetecting {
                detected_speed: Some(35),
                vehicle_ref: Some("VEH-4711".into()),
                ..VehicleDetecting::new("LOG-1", siri_rs::model::Location::wgs84(9.7411, 52.3759))
            }],
        }),
        note: vec![NaturalLanguageString::new("Four items, one of each kind")],
        ..ControlActionDelivery::new(now())
    };

    let message = Siri::new(
        VERSION,
        ServiceDelivery::new(now(), "MY-AGENCY", vec![delivery.into()]),
    );
    assert_valid(&message, "a delivery of every payload kind");
    assert_eq!(read_back(&message), message);
}

#[test]
fn the_capabilities_of_a_control_action_service_are_valid_and_read_back() {
    let response = ControlActionCapabilitiesResponse::new(
        now(),
        ControlActionServiceCapabilities::default(),
    );

    let xml = siri_rs::to_string(&response).expect("the response is writable");
    if let Err(complaint) = validate(&xml) {
        panic!("the capabilities response is not valid SIRI:\n{xml}\n{complaint}");
    }
    let read: ControlActionCapabilitiesResponse =
        siri_rs::from_str(&xml).expect("the response reads back");
    assert_eq!(read, response);
    assert_eq!(
        ControlActionCapabilitiesResponse::ELEMENT_NAME,
        "ControlActionCapabilitiesResponse"
    );
}

/// One control action per alternative of the schema's choice, with the name the
/// accessor is expected to report.
fn every_action() -> Vec<(&'static str, ControlAction)> {
    let action = || {
        ControlAction {
            reason: Some(ControlActionReason::new(
                ControlActionReasonCategory::VehicleBreakdown,
            )),
            source_note: Some(NaturalLanguageString::new("Reported by the duty officer")),
            ..ControlAction::new(now(), "CA-4711")
        }
    };
    let journey = || FramedVehicleJourneyRef {
        data_frame_ref: "2026-03-04".into(),
        dated_vehicle_journey_ref: "10-0815".into(),
    };
    let point = || PointInJourneyPatternRef::at_stop("de:03241:101");
    let call = || DatedCallRef::at_stop(journey(), "de:03241:101");

    vec![
        (
            "JourneyCreation",
            ControlAction {
                journey_creation: Some(JourneyCreation {
                    end: Some(JourneyEnd::new(
                        PointInJourneyPatternRef::at_stop("de:03241:110"),
                        now() + Duration::minutes(40),
                    )),
                    middle_call: vec![MiddleCall {
                        aimed_departure_time: Some(now() + Duration::minutes(20)),
                        ..MiddleCall::at(PointInJourneyPatternRef::at_stop("de:03241:105"))
                    }],
                    ..JourneyCreation::on_pattern(
                        JourneyStart::new(point(), now() + Duration::minutes(10)),
                        "JP-10-OUT",
                        "10-0915",
                    )
                }),
                ..action()
            },
        ),
        (
            "JourneyCancellation",
            ControlAction {
                journey_cancellation: Some(JourneyScope::for_journey(journey())),
                ..action()
            },
        ),
        (
            "PartialJourneyCancellation",
            ControlAction {
                partial_journey_cancellation: Some(CallCancellationAction {
                    concerns_arrivals: Some(true),
                    concerns_departures: Some(false),
                    ..CallCancellationAction::after(JourneyScope::for_journey(journey()), point())
                }),
                ..action()
            },
        ),
        (
            "FlexibleJourneyActivation",
            ControlAction {
                flexible_journey_activation: Some(FlexibleJourneyActivation {
                    type_of_activated_journey: Some(TypeOfActivatedJourney::VirtualLineJourney),
                    ..FlexibleJourneyActivation::new(journey())
                }),
                ..action()
            },
        ),
        (
            "ChangeOfJourneyPattern",
            ControlAction {
                change_of_journey_pattern: Some(JourneyPatternModification::new(
                    JourneyScope::for_journey(journey()),
                    vec![ChangedPoint {
                        target_departure_point: Some(TargetPoint::at_stop("de:03241:102")),
                        ..ChangedPoint::new(point())
                    }],
                )),
                ..action()
            },
        ),
        (
            "ChangeOfJourneyTiming",
            ControlAction {
                change_of_journey_timing: Some(ChangeOfJourneyTiming::by(
                    journey(),
                    ChangeOfJourneyTimingType::Respacing,
                    RelativeTime {
                        from_point_in_journey_pattern_ref: Some(point()),
                        change_model: Some(ChangeModel::Linear),
                        ..RelativeTime::of(SiriDuration::parse("PT3M").expect("valid duration"))
                    },
                )),
                ..action()
            },
        ),
        (
            "ChangeOfStopPointStatus",
            ControlAction {
                change_of_stop_point_status: Some(ChangeOfStopPointStatus::new(
                    StopRefs::at_stops(vec!["de:03241:101".into()]),
                    StopPointStatusTimeScope::between(now(), now() + Duration::hours(3)),
                    StopPlaceStatus::Closed,
                )),
                ..action()
            },
        ),
        (
            "InterchangeCreation",
            ControlAction {
                interchange_creation: Some(ExtraConnection {
                    min_change_duration: Some(
                        SiriDuration::parse("PT2M").expect("valid duration"),
                    ),
                    stay_seated: Some(false),
                    ..ExtraConnection::new(
                        call(),
                        DatedCallRef::at_stop(journey(), "de:03241:102"),
                        SiriDuration::parse("PT5M").expect("valid duration"),
                    )
                }),
                ..action()
            },
        ),
        (
            "InterchangeCancellation",
            ControlAction {
                interchange_cancellation: Some(CancelledConnection::new(
                    call(),
                    DatedCallRef::at_stop(journey(), "de:03241:102"),
                )),
                ..action()
            },
        ),
        (
            "InterchangeModification",
            ControlAction {
                interchange_modification: Some(ModifiedConnection {
                    wait_for_feeder_until_date_time: Some(now() + Duration::minutes(6)),
                    ..ModifiedConnection::new(
                        call(),
                        DatedCallRef::at_stop(journey(), "de:03241:102"),
                    )
                }),
                ..action()
            },
        ),
        (
            "VehicleWorkAssignment",
            ControlAction {
                vehicle_work_assignment: Some(VehicleWorkAssignment {
                    driver_name: Some("A. Fahrer".to_owned()),
                    ..VehicleWorkAssignment::on_journey("VEH-4711", journey())
                }),
                ..action()
            },
        ),
        (
            "JourneyCancellation",
            ControlAction {
                // The other half of the journey-scope choice: a line and a period
                // rather than one dated journey.
                journey_cancellation: Some(JourneyScope::on_line(
                    "10",
                    TimeScope::from(ValidityCondition {
                        from_date_time: Some(now()),
                        to_date_time: Some(now() + Duration::hours(2)),
                        ..ValidityCondition::default()
                    }),
                )),
                ..action()
            },
        ),
    ]
}

/// The element name the alternative an action carries is written as.
fn kind_name(kind: ControlActionKind<'_>) -> &'static str {
    match kind {
        ControlActionKind::JourneyCreation(_) => "JourneyCreation",
        ControlActionKind::JourneyCancellation(_) => "JourneyCancellation",
        ControlActionKind::PartialJourneyCancellation(_) => "PartialJourneyCancellation",
        ControlActionKind::FlexibleJourneyActivation(_) => "FlexibleJourneyActivation",
        ControlActionKind::ChangeOfJourneyPattern(_) => "ChangeOfJourneyPattern",
        ControlActionKind::ChangeOfJourneyTiming(_) => "ChangeOfJourneyTiming",
        ControlActionKind::ChangeOfStopPointStatus(_) => "ChangeOfStopPointStatus",
        ControlActionKind::InterchangeCreation(_) => "InterchangeCreation",
        ControlActionKind::InterchangeCancellation(_) => "InterchangeCancellation",
        ControlActionKind::InterchangeModification(_) => "InterchangeModification",
        ControlActionKind::VehicleWorkAssignment(_) => "VehicleWorkAssignment",
    }
}

/// Validates a message and reads it back into the types it came from.
fn round_trip(message: &Siri) -> Siri {
    assert_valid(message, "the message");
    read_back(message)
}

fn assert_valid(message: &Siri, what: &str) {
    assert!(validator_available(), "{VALIDATOR_MISSING}");
    let xml = siri_rs::to_string(message).expect("the message is writable");
    if let Err(complaint) = validate(&xml) {
        panic!("{what} is not valid SIRI:\n{xml}\n{complaint}");
    }
}

fn read_back(message: &Siri) -> Siri {
    let xml = siri_rs::to_string(message).expect("the message is writable");
    siri_rs::from_str(&xml).expect("the message reads back")
}

fn now() -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339("2026-03-04T08:15:00+01:00").expect("valid instant")
}
