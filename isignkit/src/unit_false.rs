// https://github.com/teloxide/teloxide/blob/c14439ecdef18c41101ba2ca785edf4164a36a28/crates/teloxide-core/src/types/unit_false.rs

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor};

/// A type that is always false.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Default)]
pub struct False;

impl TryFrom<bool> for False {
    type Error = ();

    fn try_from(value: bool) -> Result<Self, Self::Error> {
        if value { Err(()) } else { Ok(False) }
    }
}

impl<'de> Deserialize<'de> for False {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bool(FalseVisitor)
    }
}

struct FalseVisitor;

impl Visitor<'_> for FalseVisitor {
    type Value = False;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "bool, equal to `false`")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if value {
            Err(E::custom("expected `false`, found `true`"))
        } else {
            Ok(False)
        }
    }
}

impl Serialize for False {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bool(false)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_str, to_string};

    use super::False;

    #[test]
    fn unit_false_de() {
        let json = "false";
        let expected = False;
        let actual = from_str(json).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn unit_false_se() {
        let actual = to_string(&False).unwrap();
        let expected = "false";
        assert_eq!(expected, actual);
    }
}
