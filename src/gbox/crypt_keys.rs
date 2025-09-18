use obfstr::obfstr;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use version_compare::{Cmp, compare_to};

#[derive(Deserialize, Serialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct CryptKeys {
    pub gbox_version: String,
    pub primary: String,
    pub kvp: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CryptKeysStorage(Vec<CryptKeys>);

impl CryptKeysStorage {
    pub fn keys_for_version(&self, version: impl AsRef<str>) -> Option<&CryptKeys> {
        let version = version
            .as_ref()
            .chars()
            .filter(char::is_ascii_digit)
            .collect::<String>();

        self.0
            .iter()
            .rev()
            .find(|keys| compare_to(&version, &keys.gbox_version, Cmp::Ge).unwrap_or_default())
    }

    #[must_use]
    pub fn newest_keys(&self) -> &CryptKeys {
        #[expect(clippy::missing_panics_doc, reason = "infallible")]
        self.0
            .last()
            .expect("Storage must contain at least one keys pair!")
    }
}

impl Default for CryptKeysStorage {
    fn default() -> Self {
        Self(vec![CryptKeys {
            gbox_version: "1".to_string(),
            primary: obfstr!("608ba6563a954bae2c806f98f75d6e0a").to_string(),
            kvp: obfstr!("606ea7867a9588ae2e806f98f75d8c08").to_string(),
        }])
    }
}

impl AsRef<[CryptKeys]> for CryptKeysStorage {
    fn as_ref(&self) -> &[CryptKeys] {
        &self.0
    }
}

impl Deref for CryptKeysStorage {
    type Target = [CryptKeys];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
