//! Reading a repeated element whose content is an enumerated token.
//!
//! Deserialising `Vec<T>` for an enumerated `T` does not work directly: the
//! deserialiser offers the *element name* as the enum's variant, so a sequence of
//! `<Scope>line</Scope>` is reported as the unknown variant `Scope`. Reading the
//! tokens as strings and parsing each one keeps the wire format unchanged and
//! still rejects a token the schema does not define.
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
