//! Primitive SIRI datatypes: identifiers, references, texts and durations.
//!
//! SIRI derives most of its identifier types from `xsd:NMTOKEN` or
//! `xsd:normalizedString`. They are modelled here as distinct newtypes so that a
//! `SubscriptionRef` cannot be passed where a `ParticipantRef` is meant, while
//! still serialising as the plain text content the schema prescribes.

use std::fmt;

use chrono::{DateTime, FixedOffset};
use serde::de::{self, DeserializeOwned, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::enumerations::{EndTimePrecision, EndTimeStatus};
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

/// An `xsd:boolean` element the schema gives a default value.
///
/// Such an element may be written empty — `<Allow/>` — to mean "whatever the schema
/// says by default", and that is a different document from one that spells the value
/// out. The lexical form is kept so that both are written back as they were read.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DefaultedBoolean(String);

impl DefaultedBoolean {
    /// A value the document spells out.
    pub fn stated(value: bool) -> Self {
        Self(value.to_string())
    }

    /// An empty element, leaving the value to the schema's default.
    pub fn defaulted() -> Self {
        Self(String::new())
    }

    /// The lexical form as written on the wire; empty when the element was empty.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The value the document states, or `None` when it left the element empty.
    pub fn stated_value(&self) -> Option<bool> {
        match self.0.as_str() {
            "true" | "1" => Some(true),
            "false" | "0" => Some(false),
            _ => None,
        }
    }

    /// The value the document states, falling back to the schema's default.
    pub fn or(&self, default: bool) -> bool {
        self.stated_value().unwrap_or(default)
    }
}

impl From<bool> for DefaultedBoolean {
    fn from(value: bool) -> Self {
        Self::stated(value)
    }
}

impl fmt::Display for DefaultedBoolean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
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

/// A stretch of time with a stated beginning and a stated end.
///
/// Unlike the half-open ranges below this one is closed: a timetable, and the window
/// a stop or connection request asks about, always cover a period that ends.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClosedTimestampRange {
    /// When the period starts.
    #[serde(rename = "StartTime")]
    pub start_time: Timestamp,
    /// When it ends.
    #[serde(rename = "EndTime")]
    pub end_time: Timestamp,
}

impl ClosedTimestampRange {
    /// The period between the two instants.
    pub fn between(start_time: DateTime<FixedOffset>, end_time: DateTime<FixedOffset>) -> Self {
        Self {
            start_time: start_time.into(),
            end_time: end_time.into(),
        }
    }
}

/// A period that starts at a known instant and may not have a stated end.
///
/// This is the form a producer publishes: when the end is unknown, the status says
/// whether the disruption is expected to be a short or a long one, which is what a
/// passenger information system needs in order to word the message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HalfOpenTimestampOutputRange {
    /// The inclusive start of the period.
    #[serde(rename = "StartTime")]
    pub start_time: DateTime<FixedOffset>,
    /// The inclusive end of the period, if it is known.
    #[serde(rename = "EndTime", default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime<FixedOffset>>,
    /// How to read an absent end: as short term, long term, or simply unknown.
    #[serde(rename = "EndTimeStatus", default, skip_serializing_if = "Option::is_none")]
    pub end_time_status: Option<EndTimeStatus>,
}

impl HalfOpenTimestampOutputRange {
    /// A period whose end is not yet known.
    pub fn starting_at(start_time: DateTime<FixedOffset>) -> Self {
        Self {
            start_time,
            end_time: None,
            end_time_status: None,
        }
    }

    /// A period between two instants, both inclusive.
    pub fn between(start_time: DateTime<FixedOffset>, end_time: DateTime<FixedOffset>) -> Self {
        Self {
            start_time,
            end_time: Some(end_time),
            end_time_status: None,
        }
    }
}

/// A period that starts at a known instant and may not have a stated end.
///
/// This is the form a consumer sends when asking for data: the precision says how
/// exactly the end is meant, so that a request ending "today" is not read as ending
/// at midnight exactly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HalfOpenTimestampInputRange {
    /// The inclusive start of the period.
    #[serde(rename = "StartTime")]
    pub start_time: DateTime<FixedOffset>,
    /// The inclusive end of the period, if it is bounded.
    #[serde(rename = "EndTime", default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime<FixedOffset>>,
    /// How exactly the end is to be taken; the default is to the second.
    #[serde(rename = "EndTimePrecision", default, skip_serializing_if = "Option::is_none")]
    pub end_time_precision: Option<EndTimePrecision>,
}

impl HalfOpenTimestampInputRange {
    /// A period whose end is left open.
    pub fn starting_at(start_time: DateTime<FixedOffset>) -> Self {
        Self {
            start_time,
            end_time: None,
            end_time_precision: None,
        }
    }

    /// A period between two instants, both inclusive.
    pub fn between(start_time: DateTime<FixedOffset>, end_time: DateTime<FixedOffset>) -> Self {
        Self {
            start_time,
            end_time: Some(end_time),
            end_time_precision: None,
        }
    }
}

/// A stretch of the day, open-ended when no end time is given.
///
/// The times are `xsd:time` lexical forms, kept verbatim for the same reason
/// [`Duration`] is: the schema admits several spellings of one instant and rewriting
/// one as another would change the document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HalfOpenTimeRange {
    /// When the band starts.
    #[serde(rename = "StartTime")]
    pub start_time: String,
    /// When the band ends.
    #[serde(rename = "EndTime", default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
}

impl HalfOpenTimeRange {
    /// A band that starts at `start_time` and runs to the end of the day.
    pub fn starting_at(start_time: impl Into<String>) -> Self {
        Self {
            start_time: start_time.into(),
            end_time: None,
        }
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

/// An XML subtree the schema leaves to the participants, kept as it was written.
///
/// SIRI leaves several payloads open: the body of a general message may be a plain
/// sentence or a whole document in another vocabulary, an `<Extensions>` element
/// admits anything at all, and a road situation carries its DATEX II record. There
/// is nothing to model, so this keeps the subtree — attributes, character data and
/// children in order, repeated names included — and writes it back unchanged.
///
/// A payload the consumer *does* know the shape of can be read into a type of its
/// own with [`parse`](Self::parse) and built from one with
/// [`from_payload`](Self::from_payload); the crate needs to know nothing about that
/// shape either way.
///
/// # Namespaces
///
/// A subtree may be qualified, and the namespace is part of what it means. A prefix
/// binding is turned into the equivalent default declaration on the way in, so the
/// namespace is kept in [`attributes`](Self::attributes) as `xmlns` and restated
/// when the subtree is written; the prefix itself is not preserved, because the two
/// spellings denote the same element.
///
/// # Attribute prefixes
///
/// The XML reader reports an attribute by its local name, so a prefix on an
/// attribute inside such a subtree does not survive the read — `xsi:type="…"` is
/// indistinguishable from `type="…"` by the time this type sees it, and is written
/// back as the latter. The only prefix XML binds without a declaration is `xml`, and
/// the only attributes it can carry are `xml:lang` and `xml:space`; those two are
/// therefore restored on the way in and kept in
/// [`attributes`](Self::attributes) with their prefix. An attribute named `lang` or
/// `space` in no namespace at all — which the reader cannot tell apart from those
/// two — is written back with the prefix it did not have.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AnyContent {
    /// The element's attributes, without the leading `@` the wire format uses.
    pub attributes: Vec<(String, String)>,
    /// Character data directly inside the element.
    pub text: String,
    /// The child elements, each with its name, in document order.
    pub children: Vec<(String, AnyContent)>,
}

impl AnyContent {
    /// Content that is a single piece of text, as most messages carry.
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            attributes: Vec::new(),
            text: text.into(),
            children: Vec::new(),
        }
    }

    /// The children named `name`, of which there may be none, one or several.
    pub fn children_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a AnyContent> {
        self.children
            .iter()
            .filter_map(move |(child, content)| (child == name).then_some(content))
    }

    /// The value of the attribute named `name`, if the element carries one.
    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    /// Reads this subtree into a type of the consumer's own.
    ///
    /// The profiles that put content here are outside SIRI, so the crate offers the
    /// seam rather than the types: a consumer that knows one declares it the way any
    /// other XML is declared to `serde` — `@name` for an attribute, `$text` for
    /// character data, the element's name for a child — and reads the subtree into
    /// it. What is read is the subtree's content; its own element name is held by
    /// whatever carries it and is not matched against `T`.
    ///
    /// ```
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct Diagnostics {
    ///     #[serde(rename = "@level")]
    ///     level: String,
    ///     #[serde(rename = "Counter")]
    ///     counter: u32,
    /// }
    ///
    /// let xml = r#"<Siri xmlns="http://www.siri.org.uk/siri">
    ///   <CheckStatusRequest>
    ///     <RequestTimestamp>2004-12-17T09:30:47-05:00</RequestTimestamp>
    ///     <RequestorRef>EREWHON</RequestorRef>
    ///     <Extensions>
    ///       <Diagnostics level="verbose"><Counter>17</Counter></Diagnostics>
    ///     </Extensions>
    ///   </CheckStatusRequest>
    /// </Siri>"#;
    ///
    /// let message: siri_rs::Siri = siri_rs::from_str(xml)?;
    /// let extensions = message
    ///     .payload
    ///     .as_check_status_request()
    ///     .and_then(|request| request.extensions.as_ref())
    ///     .unwrap();
    ///
    /// let diagnostics: Diagnostics = extensions
    ///     .children_named("Diagnostics")
    ///     .next()
    ///     .unwrap()
    ///     .parse()?;
    /// assert_eq!(diagnostics.level, "verbose");
    /// assert_eq!(diagnostics.counter, 17);
    /// # Ok::<(), siri_rs::Error>(())
    /// ```
    pub fn parse<T: DeserializeOwned>(&self) -> Result<T> {
        let element = quick_xml::se::to_string_with_root(PAYLOAD_ELEMENT, self)?;
        quick_xml::de::from_str(&element).map_err(Error::from)
    }

    /// Builds a subtree out of a value of the consumer's own, the inverse of
    /// [`parse`](Self::parse).
    ///
    /// A value that serialises to something other than an element — a bare number,
    /// say — is an error rather than a subtree.
    pub fn from_payload<T: Serialize + ?Sized>(payload: &T) -> Result<Self> {
        let element = quick_xml::se::to_string_with_root(PAYLOAD_ELEMENT, payload)?;
        quick_xml::de::from_str(&element).map_err(Error::from)
    }
}

/// The element name [`AnyContent::parse`] and [`AnyContent::from_payload`] write
/// around a subtree while it is XML.
///
/// Both go through the wire format, so that a consumer's type sees exactly what it
/// would have seen reading the payload as a document of its own. The name is the one
/// thing the subtree does not carry — an element's name belongs to whatever holds it
/// — so it is supplied here and discarded on the way back.
const PAYLOAD_ELEMENT: &str = "Payload";

impl Serialize for AnyContent {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let entries = self.attributes.len() + usize::from(!self.text.is_empty()) + self.children.len();
        let mut map = serializer.serialize_map(Some(entries))?;
        // Attributes first: the writer has to emit them before it opens the element.
        for (name, value) in &self.attributes {
            map.serialize_entry(&format!("@{name}"), value)?;
        }
        if !self.text.is_empty() {
            map.serialize_entry("$text", &self.text)?;
        }
        for (name, child) in &self.children {
            map.serialize_entry(name, child)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for AnyContent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        struct AnyContentVisitor;

        impl<'de> Visitor<'de> for AnyContentVisitor {
            type Value = AnyContent;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("an XML element")
            }

            /// An element with children or attributes arrives as a map whose keys are
            /// `@name` for an attribute, `$text` for character data and the element
            /// name for a child.
            fn visit_map<A: MapAccess<'de>>(
                self,
                mut map: A,
            ) -> std::result::Result<AnyContent, A::Error> {
                let mut content = AnyContent::default();
                while let Some(key) = map.next_key::<String>()? {
                    if let Some(name) = key.strip_prefix('@') {
                        let name = match name {
                            "lang" | "space" => format!("xml:{name}"),
                            _ => name.to_owned(),
                        };
                        content.attributes.push((name, map.next_value()?));
                    } else if key == "$text" {
                        content.text = map.next_value()?;
                    } else {
                        content.children.push((key, map.next_value()?));
                    }
                }
                Ok(content)
            }

            fn visit_str<E: de::Error>(self, value: &str) -> std::result::Result<AnyContent, E> {
                Ok(AnyContent::text(value))
            }
        }

        deserializer.deserialize_map(AnyContentVisitor)
    }
}

/// Implementation-defined content carried in a SIRI `<Extensions>` element.
///
/// The schema declares the element as a wildcard — `xsd:any` with
/// `processContents="lax"` — so whatever it holds belongs to a profile outside SIRI:
/// a VDV or DATEX payload, an operator's own settings, anything the two participants
/// agreed on. It is therefore the same thing as any other subtree the schema leaves
/// open, and is carried through by the same [`AnyContent`]: nothing is interpreted,
/// nothing is dropped, and a consumer that knows the payload's shape can read it into
/// a type of its own with [`AnyContent::parse`].
pub type Extensions = AnyContent;

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
    fn unmodelled_content_survives_being_read_and_written_back() {
        let document = concat!(
            r#"<Message><Content><Report version="2.0"><Line><Id>1</Id></Line>"#,
            "<Line><Id>2</Id></Line></Report></Content></Message>"
        );

        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct Message {
            #[serde(rename = "Content")]
            content: AnyContent,
        }

        let message: Message = quick_xml::de::from_str(document).unwrap();
        let report = message.content.children_named("Report").next().unwrap();
        assert_eq!(report.attribute("version"), Some("2.0"));
        assert_eq!(report.children_named("Line").count(), 2);

        assert_eq!(
            quick_xml::se::to_string_with_root("Message", &message).unwrap(),
            document
        );
    }

    #[test]
    fn an_extension_payload_survives_being_read_and_written_back() {
        // `ExtensionsStructure` is an `xsd:any` wildcard, so a payload is whatever the
        // participants agreed on: elements, attributes, repeated names, any depth.
        let element = concat!(
            r#"<Extensions><ProfileVersion>1.4</ProfileVersion>"#,
            r#"<OperatorSettings scope="regional"><Setting name="Language">EN</Setting>"#,
            r#"<Setting name="MaximumAge">15</Setting></OperatorSettings></Extensions>"#
        );

        let extensions: Extensions = quick_xml::de::from_str(element).unwrap();
        let settings = extensions.children_named("OperatorSettings").next().unwrap();
        assert_eq!(settings.attribute("scope"), Some("regional"));
        assert_eq!(settings.children_named("Setting").count(), 2);

        assert_eq!(
            quick_xml::se::to_string_with_root("Extensions", &extensions).unwrap(),
            element
        );
    }

    #[test]
    fn an_extension_that_is_absent_empty_or_only_text_is_written_back_as_it_was() {
        for element in ["<Extensions/>", "<Extensions>opaque</Extensions>"] {
            let extensions: Extensions = quick_xml::de::from_str(element).unwrap();
            assert_eq!(
                quick_xml::se::to_string_with_root("Extensions", &extensions).unwrap(),
                element
            );
        }
    }

    #[test]
    fn unmodelled_content_that_is_only_text_stays_only_text() {
        let content: AnyContent = quick_xml::de::from_str("<Content>Beware the Ides</Content>").unwrap();
        assert_eq!(content, AnyContent::text("Beware the Ides"));
        assert_eq!(
            quick_xml::se::to_string_with_root("Content", &content).unwrap(),
            "<Content>Beware the Ides</Content>"
        );
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
