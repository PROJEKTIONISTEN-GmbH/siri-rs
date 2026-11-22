//! The version frame of an estimated timetable that holds no journey.
//!
//! The schema demands a journey in every `EstimatedJourneyVersionFrame` and a
//! frame in every `EstimatedTimetableDelivery`, so it has no form for an
//! estimated timetable of no journeys — and a producer asked for one it has none
//! for still owes an answer. What it writes, this crate's own producer included,
//! is a frame without a journey. The strict reader refuses that, as the schema
//! does; the lenient reader reads it as a frame that holds none, keeps the
//! delivery around it, and reports the frame as a finding that left nothing out.
//!
//! The types read frames as the schema demands. Reading leniently, the reader
//! looks over a delivery for the frames that hold no journey before it reads it,
//! admits such frames only while it reads a delivery in which it found one, and
//! reports every one it found once the delivery reads. A frame is admitted only
//! on the thread the lenient reader reads on and only for that reading, so every
//! frame read as holding none is one it reports, and the strict reader never
//! admits one.

use std::cell::Cell;

use chrono::{DateTime, FixedOffset};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer};

use super::document;
use super::repair::Scope;
use super::{Discarded, Finding, Findings};
use crate::et::EstimatedVersionFrame;
use crate::model::{EstimatedServiceJourneyInterchange, EstimatedVehicleJourney, VersionRef};

/// The delivery whose frames demand a journey.
const DELIVERY: &str = "EstimatedTimetableDelivery";
/// The frame that demands one.
const FRAME: &str = "EstimatedJourneyVersionFrame";
/// The journey it demands.
const JOURNEY: &str = "EstimatedVehicleJourney";

thread_local! {
    /// Whether the reading under way admits frames without a journey.
    static ADMITTING: Cell<bool> = const { Cell::new(false) };
}

/// Runs `read`, admitting frames without a journey on this thread if `admit`
/// says so, then restores what was admitted before — also when `read` unwinds.
///
/// A caller admits only while it reads text in which it has found every such
/// frame, and reports them.
pub(super) fn admitting<T>(admit: bool, read: impl FnOnce() -> T) -> T {
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            ADMITTING.with(|admitting| admitting.set(self.0));
        }
    }
    let _restore = Restore(ADMITTING.with(|admitting| admitting.replace(admit)));
    read()
}

/// Reads the version frames of an estimated timetable delivery: as the types do,
/// unless the lenient reader admits frames without a journey.
pub(crate) fn read<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<EstimatedVersionFrame>, D::Error> {
    if !ADMITTING.with(Cell::get) {
        return Vec::<EstimatedVersionFrame>::deserialize(deserializer);
    }
    let frames = Vec::<FrameAsWritten>::deserialize(deserializer)?;
    Ok(frames.into_iter().map(EstimatedVersionFrame::from).collect())
}

/// An [`EstimatedVersionFrame`] as the text has it, journeys or none.
#[derive(Deserialize)]
struct FrameAsWritten {
    #[serde(rename = "RecordedAtTime", deserialize_with = "crate::xml::date_time::required")]
    recorded_at_time: DateTime<FixedOffset>,
    #[serde(rename = "VersionRef", default)]
    version_ref: Option<VersionRef>,
    #[serde(rename = "EstimatedVehicleJourney", default)]
    estimated_vehicle_journey: Vec<EstimatedVehicleJourney>,
    #[serde(rename = "EstimatedServiceJourneyInterchange", default)]
    estimated_service_journey_interchange: Vec<EstimatedServiceJourneyInterchange>,
}

impl From<FrameAsWritten> for EstimatedVersionFrame {
    fn from(frame: FrameAsWritten) -> Self {
        Self {
            recorded_at_time: frame.recorded_at_time,
            version_ref: frame.version_ref,
            estimated_vehicle_journey: frame.estimated_vehicle_journey,
            estimated_service_journey_interchange: frame.estimated_service_journey_interchange,
        }
    }
}

/// The places of the frames without a journey in `xml`, the fragment `scope`
/// names; none unless it is an estimated timetable delivery.
pub(super) fn without_journeys(xml: &str, scope: &Scope<'_>) -> Vec<usize> {
    if scope.name != DELIVERY {
        return Vec::new();
    }
    document::children(xml, 0..xml.len())
        .into_iter()
        .filter(|child| child.name == FRAME && !child.left_out)
        .filter(|frame| {
            !document::children(xml, frame.range.clone())
                .iter()
                .any(|child| child.name == JOURNEY && !child.left_out)
        })
        .map(|frame| frame.place)
        .collect()
}

/// Reports the frames at `places` of the delivery `scope` names as read holding
/// no journey, each after the findings for what was left out of it and of the
/// frames before it, so that the findings keep to the order of the document.
pub(super) fn report(places: &[usize], scope: &Scope<'_>, findings: &mut Findings) {
    let frames = format!("{}.{FRAME}[", scope.path);
    let reason = quick_xml::DeError::missing_field(JOURNEY).to_string();
    for &place in places {
        let path = format!("{frames}{place}]");
        let at = findings
            .iter()
            .position(|finding| frame_of(&finding.path, &frames).is_some_and(|frame| frame > place))
            .unwrap_or(findings.len());
        findings.insert(
            at,
            Finding {
                path: path.clone(),
                reason: reason.clone(),
                discarded: Discarded::Nothing { path },
            },
        );
    }
}

/// The place of the frame a path leads through, given the path's beginning up to
/// the frame's index.
fn frame_of(path: &str, frames: &str) -> Option<usize> {
    path.strip_prefix(frames)?.split_once(']')?.0.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME_WITHOUT_JOURNEY: &str = "<EstimatedTimetableDelivery><ResponseTimestamp>2001-12-17T09:30:47-05:00</ResponseTimestamp><EstimatedJourneyVersionFrame><RecordedAtTime>2001-12-17T09:30:47-05:00</RecordedAtTime></EstimatedJourneyVersionFrame></EstimatedTimetableDelivery>";

    fn read_delivery(xml: &str) -> Result<crate::et::EstimatedTimetableDelivery, quick_xml::DeError> {
        quick_xml::de::from_str(xml)
    }

    #[test]
    fn a_frame_without_a_journey_reads_only_while_admitted() {
        let refused = read_delivery(FRAME_WITHOUT_JOURNEY).expect_err("nothing is admitted");
        assert_eq!(refused.to_string(), "missing field `EstimatedVehicleJourney`");

        let read = admitting(true, || read_delivery(FRAME_WITHOUT_JOURNEY)).expect("the frame is admitted");
        assert_eq!(read.estimated_journey_version_frame.len(), 1);
        assert!(read.estimated_journey_version_frame[0].estimated_vehicle_journey.is_empty());

        let refused = admitting(false, || read_delivery(FRAME_WITHOUT_JOURNEY)).expect_err("nothing is admitted");
        assert_eq!(refused.to_string(), "missing field `EstimatedVehicleJourney`");
    }

    #[test]
    fn what_is_admitted_ends_with_the_reading_also_when_it_unwinds() {
        admitting(true, || ());
        assert!(read_delivery(FRAME_WITHOUT_JOURNEY).is_err());
        let unwound = std::panic::catch_unwind(|| admitting(true, || panic!("the reading fails")));
        assert!(unwound.is_err());
        assert!(read_delivery(FRAME_WITHOUT_JOURNEY).is_err());
        admitting(true, || {
            admitting(false, || assert!(read_delivery(FRAME_WITHOUT_JOURNEY).is_err()));
            assert!(read_delivery(FRAME_WITHOUT_JOURNEY).is_ok());
        });
    }

    #[test]
    fn a_frame_is_without_a_journey_when_none_is_left_in_it() {
        let scope = |name: &str| Scope {
            name: name.to_string(),
            path: String::new(),
            identifiers: &[],
            records: &[],
        };
        let delivery = "<EstimatedTimetableDelivery>\
            <EstimatedJourneyVersionFrame><RecordedAtTime/><EstimatedVehicleJourney/></EstimatedJourneyVersionFrame>\
            <EstimatedJourneyVersionFrame><RecordedAtTime/><!--left out: EstimatedVehicleJourney--></EstimatedJourneyVersionFrame>\
            <!--left out: EstimatedJourneyVersionFrame-->\
            <EstimatedJourneyVersionFrame><RecordedAtTime/><EstimatedServiceJourneyInterchange/></EstimatedJourneyVersionFrame>\
            <EstimatedJourneyVersionFrame/>\
            </EstimatedTimetableDelivery>";
        assert_eq!(without_journeys(delivery, &scope(DELIVERY)), vec![1, 3, 4]);
        assert!(without_journeys(delivery, &scope("StopMonitoringDelivery")).is_empty());
    }

    #[test]
    fn a_frame_is_reported_after_what_went_from_it_and_before_what_went_from_later_ones() {
        let scope = Scope {
            name: DELIVERY.to_string(),
            path: "ServiceDelivery.EstimatedTimetableDelivery[0]".to_string(),
            identifiers: &[],
            records: &[],
        };
        let mut findings = Findings::default();
        for path in [
            "ServiceDelivery.EstimatedTimetableDelivery[0].EstimatedJourneyVersionFrame[1].EstimatedVehicleJourney[0]",
            "ServiceDelivery.EstimatedTimetableDelivery[0].EstimatedJourneyVersionFrame[12].EstimatedVehicleJourney[0]",
        ] {
            findings.push(Finding {
                path: path.to_string(),
                reason: "missing field `LineRef`".to_string(),
                discarded: Discarded::Element { path: path.to_string() },
            });
        }
        report(&[0, 1, 2], &scope, &mut findings);
        let paths: Vec<&str> = findings.iter().map(|finding| finding.path.as_str()).collect();
        assert_eq!(
            paths,
            [
                "ServiceDelivery.EstimatedTimetableDelivery[0].EstimatedJourneyVersionFrame[0]",
                "ServiceDelivery.EstimatedTimetableDelivery[0].EstimatedJourneyVersionFrame[1].EstimatedVehicleJourney[0]",
                "ServiceDelivery.EstimatedTimetableDelivery[0].EstimatedJourneyVersionFrame[1]",
                "ServiceDelivery.EstimatedTimetableDelivery[0].EstimatedJourneyVersionFrame[2]",
                "ServiceDelivery.EstimatedTimetableDelivery[0].EstimatedJourneyVersionFrame[12].EstimatedVehicleJourney[0]",
            ]
        );
        let read_as_empty: Vec<&str> = findings.read_as_empty().map(|finding| finding.discarded.path()).collect();
        assert_eq!(read_as_empty, [paths[0], paths[2], paths[3]]);
    }
}
