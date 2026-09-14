//! Reading a repeated element whose content is an enumerated token.
//!
//! Deserialising `Vec<T>` for an enumerated `T` does not work directly: the
//! deserialiser offers the *element name* as the enum's variant, so a sequence of
//! `<Scope>line</Scope>` is reported as the unknown variant `Scope`. Reading the
//! tokens as strings and parsing each one keeps the wire format unchanged, and
//! keeps a token the schema does not define the way a single element does — as
//! the enumeration's unrecognised variant.
//!
//! Only repeated enumerated elements need this; a single one deserialises as
//! itself.

use serde::{Deserialize, Deserializer};

/// Reads a repeated enumerated element.
pub(crate) fn deserialize<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    Vec::<String>::deserialize(deserializer)?
        .into_iter()
        .map(|token| token.parse().map_err(serde::de::Error::custom))
        .collect()
}

/// Reading and writing one element whose content is a whitespace-separated list of
/// enumerated tokens, such as `<FareClasses>firstClass secondClass</FareClasses>`.
///
/// The schema declares these as `xsd:list` types, so the several values live in one
/// element rather than in one element each — the opposite of what
/// [`deserialize`](super::token_list::deserialize) handles.
pub(crate) mod space_separated {
    use serde::{Deserialize, Deserializer, Serializer};

    /// Writes the tokens into a single element, separated by spaces.
    pub(crate) fn serialize<S, T>(values: &[T], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: std::fmt::Display,
    {
        let joined = values
            .iter()
            .map(T::to_string)
            .collect::<Vec<_>>()
            .join(" ");
        serializer.serialize_str(&joined)
    }

    /// Splits one element's content on whitespace and parses each token.
    pub(crate) fn deserialize<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        String::deserialize(deserializer)?
            .split_whitespace()
            .map(|token| token.parse().map_err(serde::de::Error::custom))
            .collect()
    }
}
