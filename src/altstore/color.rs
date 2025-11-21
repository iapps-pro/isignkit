use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug)]
#[repr(transparent)]
pub struct AltStoreColor(String);

impl FromStr for AltStoreColor {
    type Err = anyhow::Error;

    fn from_str(hex_string: &str) -> Result<Self, Self::Err> {
        let hex_string = hex_string.strip_prefix('#').unwrap_or(hex_string);
        if hex_string.len() != 6 {
            return Err(anyhow::anyhow!("{hex_string} is not a hexadecimal color"));
        }

        let valid = hex_string.as_bytes().iter().all(u8::is_ascii_hexdigit);
        if !valid {
            return Err(anyhow::anyhow!("{hex_string} is not a hexadecimal color"));
        }

        Ok(Self(hex_string.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::AltStoreColor;

    #[test]
    fn test_color() {
        assert!("F74703".parse::<AltStoreColor>().is_ok());
        assert!("#F74703".parse::<AltStoreColor>().is_ok());

        assert!("F7470".parse::<AltStoreColor>().is_err());
        assert!("H74703".parse::<AltStoreColor>().is_err());
        assert!("#H74703".parse::<AltStoreColor>().is_err());
    }
}
