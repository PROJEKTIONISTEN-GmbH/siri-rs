//! Reading an `xsd:dateTime` into a [`DateTime<FixedOffset>`], and saying why one
//! does not read.
//!
//! A timestamp is read as chrono reads it, which is what every field holding one did
//! before it read through here. Two lexical forms XML Schema 1.0 allows are not read,
//! deliberately: a timestamp without a time-zone offset names no instant, and this
//! crate does not guess the zone its producer meant; and `24:00:00`, the end of a day,
//! is an hour chrono does not have. Both used to fail with chrono's own words —
//! "premature end of input", "input is out of range" — which say nothing of either;
//! the reasons here do.

use std::fmt;

use chrono::{DateTime, FixedOffset};
use serde::de::{self, Deserializer, Visitor};

/// Reads a mandatory `xsd:dateTime` element.
pub(crate) fn required<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<DateTime<FixedOffset>, D::Error> {
    deserializer.deserialize_str(Timestamp)
}

/// Reads an optional `xsd:dateTime` element that is present.
pub(crate) fn optional<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<DateTime<FixedOffset>>, D::Error> {
    required(deserializer).map(Some)
}

/// The visitor [`required`] reads through.
struct Timestamp;

impl Visitor<'_> for Timestamp {
    type Value = DateTime<FixedOffset>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("an xsd:dateTime with a time-zone offset")
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        value.parse().map_err(|refused: chrono::ParseError| match unread_form(value) {
            Some(reason) => E::custom(format_args!("{value:?} {reason}")),
            None => E::custom(refused),
        })
    }
}

/// Why a lexical form XML Schema allows is not read, when that is why `value` did
/// not read.
fn unread_form(value: &str) -> Option<&'static str> {
    let (date, time) = value.split_once('T')?;
    if !is_date(date) {
        return None;
    }
    let (clock, zone) = split_zone(time);
    if !is_clock(clock) {
        return None;
    }
    if is_end_of_day(clock) {
        Some("ends a day at 24:00:00, which is not read: the same instant is 00:00:00 of the next day")
    } else if zone.is_empty() {
        Some("has no time-zone offset, and a time-zone offset is what makes it an instant")
    } else {
        None
    }
}

/// Whether `date` is `YYYY-MM-DD`, the year four digits or more and optionally
/// negative.
fn is_date(date: &str) -> bool {
    let date = date.strip_prefix('-').unwrap_or(date);
    let mut parts = date.rsplitn(3, '-');
    let (Some(day), Some(month), Some(year)) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    year.len() >= 4 && month.len() == 2 && day.len() == 2 && all_digits(year) && all_digits(month) && all_digits(day)
}

/// The clock time and the zone after it — `Z`, `+hh:mm`, `-hh:mm` or nothing.
fn split_zone(time: &str) -> (&str, &str) {
    match time.find(['Z', '+', '-']) {
        Some(at) => time.split_at(at),
        None => (time, ""),
    }
}

/// Whether `clock` is `hh:mm:ss` with an optional fraction of a second.
fn is_clock(clock: &str) -> bool {
    let (whole, fraction) = clock.split_once('.').unwrap_or((clock, "0"));
    let bytes = whole.as_bytes();
    bytes.len() == 8
        && bytes[2] == b':'
        && bytes[5] == b':'
        && [&whole[0..2], &whole[3..5], &whole[6..8]].iter().all(|part| all_digits(part))
        && !fraction.is_empty()
        && all_digits(fraction)
}

/// Whether `clock` is the end of a day, `24:00:00` with any fraction of zeros.
fn is_end_of_day(clock: &str) -> bool {
    let (whole, fraction) = clock.split_once('.').unwrap_or((clock, ""));
    whole == "24:00:00" && fraction.bytes().all(|digit| digit == b'0')
}

fn all_digits(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_forms_not_read_are_named_and_no_others() {
        for value in ["2004-12-17T09:30:47", "2004-12-17T09:30:47.250", "-0044-03-15T12:00:00"] {
            assert!(unread_form(value).is_some_and(|reason| reason.contains("time-zone offset")), "{value}");
        }
        for value in ["2004-12-17T24:00:00", "2004-12-17T24:00:00.000Z", "2004-12-17T24:00:00+01:00"] {
            assert!(unread_form(value).is_some_and(|reason| reason.contains("24:00:00")), "{value}");
        }
        // Not a form XML Schema allows, or one with a zone: chrono's words say it.
        for value in ["", "2004-12-17", "2004-12-17T09:30", "17.12.2004 09:30", "2004-12-17T09:30:47+01"] {
            assert_eq!(unread_form(value), None, "{value}");
        }
    }

    #[test]
    fn a_timestamp_with_an_offset_reads_as_chrono_reads_it() {
        let read: DateTime<FixedOffset> = Timestamp.visit_str::<de::value::Error>("2004-12-17T09:30:47.5-05:00").unwrap();
        assert_eq!(read, DateTime::parse_from_rfc3339("2004-12-17T09:30:47.5-05:00").unwrap());
    }
}
