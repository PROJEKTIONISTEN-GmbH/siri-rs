//! Reading an element the schema gives a value to when it is written empty.
//!
//! XML Schema lets an element declaration carry a `default` or a `fixed` value. An
//! element so declared that is written with no content — `<Monitored/>`,
//! `<Monitored></Monitored>` — is valid and means the declared value; one written
//! with content, even whitespace alone, means what it says. The serde layer reports
//! both empty spellings as an empty string, which is no boolean, so a document
//! carrying one used to fail on it.
//!
//! [`Defaulted`] is the reading that follows the schema's rule: the declared value
//! when the element is empty, and otherwise what is written, read the way the plain
//! type reads it. It is implemented for the kinds of value the schema declares
//! defaults for — booleans, integers, decimals, durations, language tags and the
//! enumerations — and for an optional or a repeated element of any of them, so that
//! a field of any of those shapes reads through the same function. The functions,
//! one per declared value, are generated from the schemas into
//! [`schema_default`](super::schema_default), and a field names the one for the
//! declaration it transcribes.
//!
//! The value is written back in full, `<Monitored>true</Monitored>`, which the
//! schema holds equal to the empty element. Whether the document spelled the value
//! out is not kept; the one element read the other way is `Allow`, through
//! [`DefaultedBoolean`](crate::types::DefaultedBoolean).
//!
//! Where the same element is declared with a default in one place and without one
//! in another — `Severity` has one in a situation and none in a consequence — every
//! field of that name reads the default. An empty element where no default is
//! declared is not valid, so no valid document reads differently for it; an invalid
//! one reads as the neighbouring declaration says rather than failing.

use std::fmt;
use std::marker::PhantomData;

use serde::de::{self, DeserializeSeed, Deserializer, SeqAccess, Unexpected, Visitor};

use crate::types::Duration;

/// A value the schema declares a default for, read so that an empty element yields
/// the declared value.
pub(crate) trait Defaulted<'de>: Sized {
    /// Reads the value, taking `default` — the declared value in its lexical form —
    /// where the element is empty.
    fn read<D: Deserializer<'de>>(deserializer: D, default: &'static str) -> Result<Self, D::Error>;
}

impl<'de> Defaulted<'de> for bool {
    fn read<D: Deserializer<'de>>(deserializer: D, default: &'static str) -> Result<Self, D::Error> {
        deserializer.deserialize_bool(Boolean { default })
    }
}

impl<'de> Defaulted<'de> for u64 {
    fn read<D: Deserializer<'de>>(deserializer: D, default: &'static str) -> Result<Self, D::Error> {
        deserializer.deserialize_u64(Integer { default })
    }
}

impl<'de> Defaulted<'de> for f64 {
    fn read<D: Deserializer<'de>>(deserializer: D, default: &'static str) -> Result<Self, D::Error> {
        deserializer.deserialize_f64(Decimal { default })
    }
}

impl<'de> Defaulted<'de> for String {
    fn read<D: Deserializer<'de>>(deserializer: D, default: &'static str) -> Result<Self, D::Error> {
        deserializer.deserialize_string(Text { default })
    }
}

impl<'de> Defaulted<'de> for Duration {
    fn read<D: Deserializer<'de>>(deserializer: D, default: &'static str) -> Result<Self, D::Error> {
        String::read(deserializer, default).map(Duration::as_written)
    }
}

impl<'de, T: Defaulted<'de>> Defaulted<'de> for Option<T> {
    fn read<D: Deserializer<'de>>(deserializer: D, default: &'static str) -> Result<Self, D::Error> {
        T::read(deserializer, default).map(Some)
    }
}

impl<'de, T: Defaulted<'de>> Defaulted<'de> for Vec<T> {
    fn read<D: Deserializer<'de>>(deserializer: D, default: &'static str) -> Result<Self, D::Error> {
        deserializer.deserialize_seq(Repeated {
            default,
            value: PhantomData,
        })
    }
}

/// Reads an enumerated token: the declared one where the element is empty, and
/// otherwise the one written, each turned into its value by `from_token`.
///
/// The enumerations are declared by a macro, which routes their reading here so
/// that the token is not copied on the way.
pub(crate) fn token<'de, D, T>(
    deserializer: D,
    default: &'static str,
    from_token: fn(&str) -> T,
) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(Token {
        default,
        from_token,
    })
}

/// The value of an `xsd:boolean` literal: `true` and `1`, or `false` and `0`.
fn xsd_boolean<E: de::Error>(literal: &str) -> Result<bool, E> {
    match literal {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        other => Err(E::custom(format!("{other:?} is not an xsd:boolean"))),
    }
}

/// Reads a boolean the reader could parse, or takes the declared value for an
/// empty element. The reader hands over text it could not parse as a string,
/// which is refused as the plain reading refuses it.
struct Boolean {
    default: &'static str,
}

impl<'de> Visitor<'de> for Boolean {
    type Value = bool;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a boolean")
    }

    fn visit_bool<E: de::Error>(self, value: bool) -> Result<bool, E> {
        Ok(value)
    }

    fn visit_str<E: de::Error>(self, text: &str) -> Result<bool, E> {
        if text.is_empty() {
            xsd_boolean(self.default)
        } else {
            Err(E::invalid_type(Unexpected::Str(text), &self))
        }
    }
}

struct Integer {
    default: &'static str,
}

impl<'de> Visitor<'de> for Integer {
    type Value = u64;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a non-negative integer")
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<u64, E> {
        Ok(value)
    }

    fn visit_str<E: de::Error>(self, text: &str) -> Result<u64, E> {
        if text.is_empty() {
            self.default.parse().map_err(E::custom)
        } else {
            Err(E::invalid_type(Unexpected::Str(text), &self))
        }
    }
}

struct Decimal {
    default: &'static str,
}

impl<'de> Visitor<'de> for Decimal {
    type Value = f64;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a decimal number")
    }

    fn visit_f64<E: de::Error>(self, value: f64) -> Result<f64, E> {
        Ok(value)
    }

    fn visit_str<E: de::Error>(self, text: &str) -> Result<f64, E> {
        if text.is_empty() {
            self.default.parse().map_err(E::custom)
        } else {
            Err(E::invalid_type(Unexpected::Str(text), &self))
        }
    }
}

struct Text {
    default: &'static str,
}

impl<'de> Visitor<'de> for Text {
    type Value = String;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("text")
    }

    fn visit_str<E: de::Error>(self, text: &str) -> Result<String, E> {
        Ok(if text.is_empty() { self.default } else { text }.to_owned())
    }

    fn visit_string<E: de::Error>(self, text: String) -> Result<String, E> {
        Ok(if text.is_empty() {
            self.default.to_owned()
        } else {
            text
        })
    }
}

struct Token<T> {
    default: &'static str,
    from_token: fn(&str) -> T,
}

impl<'de, T> Visitor<'de> for Token<T> {
    type Value = T;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("an enumerated token")
    }

    fn visit_str<E: de::Error>(self, text: &str) -> Result<T, E> {
        Ok((self.from_token)(if text.is_empty() {
            self.default
        } else {
            text
        }))
    }
}

/// Reads each occurrence of a repeated element as a [`Defaulted`] value.
struct Repeated<T> {
    default: &'static str,
    value: PhantomData<T>,
}

impl<'de, T: Defaulted<'de>> Visitor<'de> for Repeated<T> {
    type Value = Vec<T>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a repeated element")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut elements: A) -> Result<Vec<T>, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = elements.next_element_seed(Occurrence {
            default: self.default,
            value: PhantomData,
        })? {
            values.push(value);
        }
        Ok(values)
    }
}

/// One occurrence of a repeated element.
struct Occurrence<T> {
    default: &'static str,
    value: PhantomData<T>,
}

impl<'de, T: Defaulted<'de>> DeserializeSeed<'de> for Occurrence<T> {
    type Value = T;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<T, D::Error> {
        T::read(deserializer, self.default)
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;
    use crate::enumerations::Severity;

    fn read_true<'de, D: Deserializer<'de>, T: Defaulted<'de>>(d: D) -> Result<T, D::Error> {
        T::read(d, "true")
    }

    fn read_thirty<'de, D: Deserializer<'de>, T: Defaulted<'de>>(d: D) -> Result<T, D::Error> {
        T::read(d, "30")
    }

    fn read_normal<'de, D: Deserializer<'de>, T: Defaulted<'de>>(d: D) -> Result<T, D::Error> {
        T::read(d, "normal")
    }

    fn read_en<'de, D: Deserializer<'de>, T: Defaulted<'de>>(d: D) -> Result<T, D::Error> {
        T::read(d, "en")
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Document {
        #[serde(rename = "Flag", default, deserialize_with = "read_true")]
        flag: Option<bool>,
        #[serde(rename = "Count", default, deserialize_with = "read_thirty")]
        count: Option<u64>,
        #[serde(rename = "Severity", default, deserialize_with = "read_normal")]
        severity: Option<Severity>,
        #[serde(rename = "Language", default, deserialize_with = "read_en")]
        language: Vec<String>,
    }

    fn read(xml: &str) -> Result<Document, quick_xml::DeError> {
        quick_xml::de::from_str(xml)
    }

    #[test]
    fn an_empty_element_reads_as_the_declared_value_in_either_spelling() {
        let document = read("<D><Flag/><Count></Count><Severity/><Language/></D>").unwrap();
        assert_eq!(document.flag, Some(true));
        assert_eq!(document.count, Some(30));
        assert_eq!(document.severity, Some(Severity::Normal));
        assert_eq!(document.language, ["en"]);
    }

    #[test]
    fn a_written_value_reads_as_written() {
        let document =
            read("<D><Flag>0</Flag><Count>7</Count><Severity>severe</Severity><Language>de</Language><Language/></D>")
                .unwrap();
        assert_eq!(document.flag, Some(false));
        assert_eq!(document.count, Some(7));
        assert_eq!(document.severity, Some(Severity::Severe));
        assert_eq!(document.language, ["de", "en"]);
    }

    #[test]
    fn an_absent_element_is_still_absent() {
        assert_eq!(
            read("<D/>").unwrap(),
            Document {
                flag: None,
                count: None,
                severity: None,
                language: Vec::new(),
            }
        );
    }

    #[test]
    fn text_that_is_no_value_is_refused_as_before() {
        let error = read("<D><Flag>yes</Flag></D>").unwrap_err().to_string();
        assert!(error.contains(r#"invalid type: string "yes", expected a boolean"#), "{error}");
        let error = read("<D><Count>-1</Count></D>").unwrap_err().to_string();
        assert!(error.contains(r#"invalid type: string "-1""#), "{error}");
    }

    #[test]
    fn a_declared_boolean_is_read_in_the_lexical_forms_the_schema_allows() {
        assert_eq!(xsd_boolean::<serde::de::value::Error>("1"), Ok(true));
        assert_eq!(xsd_boolean::<serde::de::value::Error>("0"), Ok(false));
        assert!(xsd_boolean::<serde::de::value::Error>("yes").is_err());
    }
}
