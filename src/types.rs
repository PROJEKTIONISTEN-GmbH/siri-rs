//! Primitive SIRI datatypes: identifiers, references, texts and durations.
//!
//! SIRI derives most of its identifier types from `xsd:NMTOKEN` or
//! `xsd:normalizedString`. They are modelled here as distinct newtypes so that a
//! `SubscriptionRef` cannot be passed where a `ParticipantRef` is meant, while
//! still serialising as the plain text content the schema prescribes.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

siri_ref! {
    /// Identifies a participant — a system exchanging SIRI messages.
    ///
    /// Used for `RequestorRef`, `ProducerRef`, `ConsumerRef`, `ResponderRef`,
    /// `SubscriberRef` and `DelegatorRef`, all of which are `ParticipantRefStructure`.
    ParticipantRef;
    /// A unique identifier a sender puts on a message so it can be referenced back.
    MessageQualifier;
    /// A reference to a previously sent message's [`MessageQualifier`].
    MessageRef;
    /// Identifies a subscription within the scope of its subscriber.
    SubscriptionQualifier;
    /// A reference to an established subscription.
    SubscriptionRef;
    /// A reference to a subscription filter shared by several subscriptions.
    SubscriptionFilterRef;
    /// A network address a SIRI endpoint can be reached at.
    EndpointAddress;
    /// Identifies a named capability of a service.
    CapabilityRef;
    /// An ISO 3166-1 alpha-2 country code.
    CountryRef;
    /// A producer's identifier for one item within a delivery.
    ///
    /// Quoting it lets a later message supersede or withdraw that item rather than
    /// the whole delivery.
    ItemIdentifier;
    /// A reference to an item a producer identified earlier.
    ItemRef;
}

/// A human-readable text with an optional language tag.
///
/// SIRI carries free text in `NaturalLanguageStringStructure`, which is element
/// content plus an `xml:lang` attribute. Repeating the element with different
/// languages is how SIRI expresses a multilingual value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NaturalLanguageString {
    /// The `xml:lang` tag of this text, e.g. `DE`.
    #[serde(rename = "@xml:lang", alias = "@lang", default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    /// The text itself.
    #[serde(rename = "$text")]
    pub value: String,
}

impl NaturalLanguageString {
    /// A text without a language tag.
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            lang: None,
            value: value.into(),
        }
    }

    /// A text tagged with the language it is written in.
    pub fn with_lang(lang: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            lang: Some(lang.into()),
            value: value.into(),
        }
    }
}

impl fmt::Display for NaturalLanguageString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}

/// A place name, i.e. a [`NaturalLanguageString`] the schema restricts to a
/// character set free of punctuation used as field separators elsewhere.
pub type NaturalLanguagePlaceName = NaturalLanguageString;

/// Free text that a producer may have derived from structured data.
///
/// `overridden` distinguishes text an operator typed by hand (`true`) from text a
/// system generated out of the situation's classifiers (`false`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefaultedText {
    /// The `xml:lang` tag of this text.
    #[serde(rename = "@xml:lang", alias = "@lang", default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    /// Whether the text replaces the value that would otherwise be derived.
    #[serde(rename = "@overridden", default, skip_serializing_if = "Option::is_none")]
    pub overridden: Option<bool>,
    /// The text itself.
    #[serde(rename = "$text")]
    pub value: String,
}

impl DefaultedText {
    /// A text without a language tag that does not override a derived value.
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            lang: None,
            overridden: None,
            value: value.into(),
        }
    }

    /// A text tagged with the language it is written in.
    pub fn with_lang(lang: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            lang: Some(lang.into()),
            overridden: None,
            value: value.into(),
        }
    }
}

impl fmt::Display for DefaultedText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}

/// An `xsd:duration`, e.g. `PT5M` or `P1Y2M3DT10H30M`.
///
/// The lexical form is kept exactly as written, because `xsd:duration` has no
/// canonical form: `PT60M` and `PT1H` denote the same length but are distinct
/// documents, and a library that re-writes one as the other loses information a
/// conformance test would flag. Use [`Duration::to_std`] to get a measurable length.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Duration(String);

impl Duration {
    /// Parses an `xsd:duration` lexical form, keeping it verbatim.
    ///
    /// ```
    /// use siri_rs::Duration;
    /// assert_eq!(Duration::parse("PT5M")?.as_str(), "PT5M");
    /// assert!(Duration::parse("5 minutes").is_err());
    /// # Ok::<(), siri_rs::Error>(())
    /// ```
    pub fn parse(lexical: impl Into<String>) -> Result<Self> {
        let lexical = lexical.into();
        let duration = Self(lexical);
        duration.components()?;
        Ok(duration)
    }

    /// Builds a duration from a number of seconds, as `PT<n>S`.
    pub fn from_secs(seconds: u64) -> Self {
        Self(format!("PT{seconds}S"))
    }

    /// The lexical form as written on the wire.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The duration as a [`std::time::Duration`].
    ///
    /// Years and months have no fixed length, so they are converted with the
    /// nominal lengths used for scheduling intervals: 365 days and 30 days. Returns
    /// `None` for a negative duration, which [`std::time::Duration`] cannot hold.
    pub fn to_std(&self) -> Option<std::time::Duration> {
        let c = self.components().ok()?;
        if c.negative {
            return None;
        }
        let seconds = c.years * 365 * 86_400
            + c.months * 30 * 86_400
            + c.days * 86_400
            + c.hours * 3_600
            + c.minutes * 60;
        Some(std::time::Duration::from_secs_f64(seconds as f64 + c.seconds))
    }

    /// Splits the lexical form into its designators, rejecting malformed input.
    fn components(&self) -> Result<DurationComponents> {
        let invalid = || Error::InvalidValue {
            datatype: "xsd:duration",
            value: self.0.clone(),
        };
        let mut rest = self.0.as_str();
        let negative = match rest.strip_prefix('-') {
            Some(r) => {
                rest = r;
                true
            }
            None => false,
        };
        rest = rest.strip_prefix('P').ok_or_else(invalid)?;

        let (date_part, time_part) = match rest.split_once('T') {
            Some((d, t)) => {
                if t.is_empty() {
                    return Err(invalid());
                }
                (d, Some(t))
            }
            None => (rest, None),
        };

        let mut out = DurationComponents {
            negative,
            ..Default::default()
        };
        let mut any = false;
        let mut number = String::new();
        for ch in date_part.chars() {
            match ch {
                '0'..='9' => number.push(ch),
                'Y' | 'M' | 'D' if !number.is_empty() => {
                    let value: u64 = number.parse().map_err(|_| invalid())?;
                    number.clear();
                    any = true;
                    match ch {
                        'Y' => out.years = value,
                        'M' => out.months = value,
                        _ => out.days = value,
                    }
                }
                _ => return Err(invalid()),
            }
        }
        if !number.is_empty() {
            return Err(invalid());
        }

        if let Some(time_part) = time_part {
            for ch in time_part.chars() {
                match ch {
                    '0'..='9' | '.' => number.push(ch),
                    'H' | 'M' | 'S' if !number.is_empty() => {
                        any = true;
                        match ch {
                            'H' => out.hours = number.parse().map_err(|_| invalid())?,
                            'M' => out.minutes = number.parse().map_err(|_| invalid())?,
                            _ => out.seconds = number.parse().map_err(|_| invalid())?,
                        }
                        number.clear();
                    }
                    _ => return Err(invalid()),
                }
            }
            if !number.is_empty() {
                return Err(invalid());
            }
        }

        if any {
            Ok(out)
        } else {
            Err(invalid())
        }
    }
}

#[derive(Default)]
struct DurationComponents {
    negative: bool,
    years: u64,
    months: u64,
    days: u64,
    hours: u64,
    minutes: u64,
    seconds: f64,
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::str::FromStr for Duration {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

/// An `xsd:dateTime` whose time-zone offset the schema leaves optional.
///
/// SIRI timestamps normally state an offset, and this crate reads those into
/// [`chrono::DateTime<FixedOffset>`]. `xsd:dateTime` allows the offset to be left
/// out, though, and the official Production Timetable examples do so for the period
/// a timetable covers: `2001-12-17T14:20:00` is a wall-clock time in whichever zone
/// the two ends have agreed on. Turning that into an instant would mean inventing
/// the offset, so this type keeps the lexical form and offers the instant only when
/// the document actually gave one.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(String);

impl Timestamp {
    /// Parses an `xsd:dateTime` lexical form, keeping it verbatim.
    ///
    /// ```
    /// use siri_rs::types::Timestamp;
    /// assert!(Timestamp::parse("2001-12-17T14:20:00")?.instant().is_none());
    /// assert!(Timestamp::parse("2001-12-17T14:20:00+01:00")?.instant().is_some());
    /// assert!(Timestamp::parse("yesterday").is_err());
    /// # Ok::<(), siri_rs::Error>(())
    /// ```
    pub fn parse(lexical: impl Into<String>) -> Result<Self> {
        let lexical = lexical.into();
        if chrono::DateTime::parse_from_rfc3339(&lexical).is_ok()
            || chrono::NaiveDateTime::parse_from_str(&lexical, "%Y-%m-%dT%H:%M:%S%.f").is_ok()
        {
            Ok(Self(lexical))
        } else {
            Err(Error::InvalidValue {
                datatype: "xsd:dateTime",
                value: lexical,
            })
        }
    }

    /// The lexical form as written on the wire.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The instant this names, or `None` when no offset was given.
    pub fn instant(&self) -> Option<chrono::DateTime<chrono::FixedOffset>> {
        chrono::DateTime::parse_from_rfc3339(&self.0).ok()
    }
}

impl From<chrono::DateTime<chrono::FixedOffset>> for Timestamp {
    fn from(instant: chrono::DateTime<chrono::FixedOffset>) -> Self {
        Self(instant.to_rfc3339())
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::str::FromStr for Timestamp {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

/// An element with no content, used by SIRI where the presence of the element is
/// itself the value — `<All/>`, `<AllOperators/>`, `<WgsDecimalDegrees/>`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Empty {}

impl Empty {
    /// The single value of this type.
    pub const fn new() -> Self {
        Self {}
    }
}

/// Implementation-defined content carried in a SIRI `<Extensions>` element.
///
/// The schema declares `Extensions` as `xsd:anyType`, so its children are outside
/// SIRI. This release round-trips the element and any character data it holds;
/// structured extension payloads are not modelled.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Extensions {
    /// Character data directly inside the element, if any.
    #[serde(rename = "$text", default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn references_of_different_kinds_do_not_mix() {
        let participant = ParticipantRef::new("NADER");
        assert_eq!(participant.as_str(), "NADER");
        assert_eq!(participant.to_string(), "NADER");
        assert_eq!(SubscriptionRef::from("0003456").into_inner(), "0003456");
    }

    #[test]
    fn a_language_tag_survives_being_read_back() {
        // The deserialiser matches attributes by local name while the serialiser
        // writes the qualified one, so the field needs both spellings.
        let read: NaturalLanguageString =
            quick_xml::de::from_str(r#"<Name xml:lang="EN">Express</Name>"#).unwrap();
        assert_eq!(read, NaturalLanguageString::with_lang("EN", "Express"));
        assert_eq!(
            quick_xml::se::to_string_with_root("Name", &read).unwrap(),
            r#"<Name xml:lang="EN">Express</Name>"#
        );

        let text: DefaultedText =
            quick_xml::de::from_str(r#"<Summary xml:lang="DE" overridden="true">Hallo</Summary>"#)
                .unwrap();
        assert_eq!(text.lang.as_deref(), Some("DE"));
        assert_eq!(text.overridden, Some(true));
    }

    #[test]
    fn a_timestamp_keeps_its_lexical_form_and_reports_whether_it_names_an_instant() {
        let zoned = Timestamp::parse("2001-12-17T14:20:00+01:00").unwrap();
        assert_eq!(zoned.as_str(), "2001-12-17T14:20:00+01:00");
        assert_eq!(
            zoned.instant(),
            Some(
                chrono::DateTime::parse_from_rfc3339("2001-12-17T14:20:00+01:00").unwrap()
            )
        );

        let local = Timestamp::parse("2001-12-17T14:20:00").unwrap();
        assert_eq!(local.as_str(), "2001-12-17T14:20:00");
        assert_eq!(local.instant(), None);

        for lexical in ["", "2001-12-17", "14:20:00", "2001-12-17 14:20:00"] {
            assert!(Timestamp::parse(lexical).is_err(), "{lexical:?} should not parse");
        }
    }

    #[test]
    fn duration_keeps_its_lexical_form() {
        for lexical in ["PT5M", "PT60M", "P1Y2M3DT10H30M", "PT0S", "-P1D", "PT1.5S"] {
            assert_eq!(Duration::parse(lexical).unwrap().as_str(), lexical);
        }
    }

    #[test]
    fn duration_rejects_forms_the_schema_does_not_allow() {
        for lexical in ["", "P", "5M", "PT", "PT5X", "P1YT", "PTM", "1Y"] {
            assert!(
                Duration::parse(lexical).is_err(),
                "{lexical:?} should not parse"
            );
        }
    }

    #[test]
    fn duration_converts_to_a_measurable_length() {
        assert_eq!(
            Duration::parse("PT5M").unwrap().to_std(),
            Some(std::time::Duration::from_secs(300))
        );
        assert_eq!(
            Duration::parse("P1DT2H3M4S").unwrap().to_std(),
            Some(std::time::Duration::from_secs(86_400 + 7_200 + 180 + 4))
        );
        assert_eq!(Duration::from_secs(90).as_str(), "PT90S");
        assert_eq!(Duration::parse("-P1D").unwrap().to_std(), None);
    }
}
