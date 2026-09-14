//! Internal macros shared by the type modules.

/// Declares a SIRI enumeration: a Rust enum whose variants are the tokens of an
/// XML Schema `simpleType` restriction, in schema order, plus one variant that
/// keeps any token the schema release this crate transcribes does not list.
///
/// Beyond the enum itself this generates `XSD_TYPE` and `ALL`, which the
/// conformance tests use to check the transcription against the schema token by
/// token. `ALL` lists the schema's tokens only.
macro_rules! siri_enum {
    (
        $(#[$meta:meta])*
        $name:ident as $xsd:literal {
            $($(#[$variant_meta:meta])* $variant:ident = $token:literal),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum $name {
            $(
                $(#[$variant_meta])*
                $variant,
            )*
            /// A token the schema release this crate transcribes does not list.
            ///
            /// The standard and its national profiles extend these lists with every
            /// revision, and a document from the other side may be written to a
            /// later one. The token is kept as it was read and written back
            /// unchanged, so that one value the crate does not know does not cost
            /// the document that carries it; what to make of it is the
            /// application's decision.
            Unrecognised(String),
        }

        impl $name {
            /// Name of the XML Schema type this enumeration transcribes.
            pub const XSD_TYPE: &'static str = $xsd;

            /// Every value the schema defines, in schema order.
            pub const ALL: &'static [Self] = &[$(Self::$variant,)*];

            /// The value a wire token denotes: the schema's variant when it lists
            /// the token, [`Unrecognised`](Self::Unrecognised) otherwise.
            pub fn from_token(token: &str) -> Self {
                match token {
                    $($token => Self::$variant,)*
                    other => Self::Unrecognised(other.to_owned()),
                }
            }

            /// The token this value is written as on the wire.
            pub fn as_str(&self) -> &str {
                match self {
                    $(Self::$variant => $token,)*
                    Self::Unrecognised(token) => token,
                }
            }
        }

        impl ::core::fmt::Display for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl ::core::str::FromStr for $name {
            type Err = ::core::convert::Infallible;

            /// Every token is a value, so this never fails; see
            /// [`from_token`](Self::from_token).
            fn from_str(s: &str) -> ::core::result::Result<Self, Self::Err> {
                Ok(Self::from_token(s))
            }
        }

        impl ::serde::Serialize for $name {
            fn serialize<S: ::serde::Serializer>(
                &self,
                serializer: S,
            ) -> ::core::result::Result<S::Ok, S::Error> {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D: ::serde::Deserializer<'de>>(
                deserializer: D,
            ) -> ::core::result::Result<Self, D::Error> {
                struct Token;

                impl<'de> ::serde::de::Visitor<'de> for Token {
                    type Value = $name;

                    fn expecting(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                        f.write_str(concat!("a token of ", $xsd))
                    }

                    fn visit_str<E: ::serde::de::Error>(
                        self,
                        token: &str,
                    ) -> ::core::result::Result<$name, E> {
                        Ok($name::from_token(token))
                    }
                }

                deserializer.deserialize_str(Token)
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
