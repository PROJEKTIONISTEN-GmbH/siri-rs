//! Internal macros shared by the type modules.

/// Declares a SIRI enumeration: a Rust enum whose variants are the tokens of an
/// XML Schema `simpleType` restriction, in schema order.
///
/// Beyond the enum itself this generates `XSD_TYPE` and `ALL`, which the
/// conformance tests use to check the transcription against the schema token by
/// token.
macro_rules! siri_enum {
    (
        $(#[$meta:meta])*
        $name:ident as $xsd:literal {
            $($(#[$variant_meta:meta])* $variant:ident = $token:literal),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
        pub enum $name {
            $(
                $(#[$variant_meta])*
                #[serde(rename = $token)]
                $variant,
            )*
        }

        impl $name {
            /// Name of the XML Schema type this enumeration transcribes.
            pub const XSD_TYPE: &'static str = $xsd;

            /// Every value the schema defines, in schema order.
            pub const ALL: &'static [Self] = &[$(Self::$variant,)*];

            /// The token this value is written as on the wire.
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $token,)*
                }
            }
        }

        impl ::core::fmt::Display for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl ::core::str::FromStr for $name {
            type Err = $crate::Error;

            fn from_str(s: &str) -> ::core::result::Result<Self, Self::Err> {
                match s {
                    $($token => Ok(Self::$variant),)*
                    other => Err($crate::Error::InvalidValue {
                        datatype: $xsd,
                        value: other.to_owned(),
                    }),
                }
            }
        }
    };
}

/// Declares a newtype over `String` that serialises as bare element content.
///
/// SIRI derives its identifier types from `xsd:NMTOKEN` or `xsd:normalizedString`.
/// Giving each its own type keeps a `LineRef` from being passed where a
/// `StopPointRef` is meant, at no cost on the wire.
macro_rules! siri_ref {
    ($($(#[$meta:meta])* $name:ident;)*) => {
        $(
            $(#[$meta])*
            #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
            #[serde(transparent)]
            pub struct $name(String);

            impl $name {
                /// Wraps a value of this reference type.
                pub fn new(value: impl Into<String>) -> Self {
                    Self(value.into())
                }

                /// The underlying text.
                pub fn as_str(&self) -> &str {
                    &self.0
                }

                /// Consumes the reference and returns the underlying text.
                pub fn into_inner(self) -> String {
                    self.0
                }
            }

            impl ::core::fmt::Display for $name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.write_str(&self.0)
                }
            }

            impl From<&str> for $name {
                fn from(value: &str) -> Self {
                    Self::new(value)
                }
            }

            impl From<String> for $name {
                fn from(value: String) -> Self {
                    Self(value)
                }
            }

            impl AsRef<str> for $name {
                fn as_ref(&self) -> &str {
                    &self.0
                }
            }
        )*
    };
}
