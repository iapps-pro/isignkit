#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Formatter;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct UnsignedNumber(pub u64);

impl<'de> Deserialize<'de> for UnsignedNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NumberVisitor;

        impl Visitor<'_> for NumberVisitor {
            type Value = UnsignedNumber;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("numeric string or integer")
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(UnsignedNumber(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                if let Ok(n) = v.parse() {
                    Ok(UnsignedNumber(n))
                } else {
                    Err(E::custom(format!("{v} is not a numeric string")))
                }
            }
        }

        deserializer.deserialize_any(NumberVisitor)
    }
}

impl Serialize for UnsignedNumber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.0)
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for UnsignedNumber {
    fn inline_schema() -> bool {
        false
    }

    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("UnsignedNumber")
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        #[derive(JsonSchema)]
        #[allow(unused)]
        #[schemars(untagged)]
        enum UnsignedNumberSchema {
            Number(u64),
            String(#[schemars(pattern(r"^\d+$"))] String),
        }

        UnsignedNumberSchema::json_schema(generator)
    }
}
