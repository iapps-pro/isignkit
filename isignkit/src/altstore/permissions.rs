use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModernPermissions {
    /// A list of all entitlements used by the app and its app extensions.
    pub entitlements: Vec<String>,

    /// A dictionary with all the `UsageDescription` keys in the app's Info.plist
    /// along with their descriptions.
    pub privacy: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LegacyPermissions {
    pub r#type: String,

    pub usage_description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Permissions {
    Modern(ModernPermissions),
    Legacy(LegacyPermissions),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct OptionalPermissions(
    #[serde(
        deserialize_with = "deserialize_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub Option<Permissions>,
);

impl OptionalPermissions {
    #[must_use]
    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

impl From<Permissions> for OptionalPermissions {
    fn from(permissions: Permissions) -> Self {
        OptionalPermissions(Some(permissions))
    }
}

impl From<ModernPermissions> for OptionalPermissions {
    fn from(permissions: ModernPermissions) -> Self {
        OptionalPermissions(Some(Permissions::Modern(permissions)))
    }
}

impl From<LegacyPermissions> for OptionalPermissions {
    fn from(permissions: LegacyPermissions) -> Self {
        OptionalPermissions(Some(Permissions::Legacy(permissions)))
    }
}

#[allow(clippy::unnecessary_wraps, reason = "Serde API")]
fn deserialize_option<'de, D>(deserializer: D) -> Result<Option<Permissions>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Permissions::deserialize(deserializer).ok())
}
