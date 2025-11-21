pub mod config;
mod crypt_keys;
pub mod encrypted;
mod gbox_link;
mod installable_app;
pub mod links_map;
mod schema;
mod unencrypted;
#[cfg(feature = "schema")]
mod validation;

pub use self::{
    config::EncryptedConfig,
    crypt_keys::{CryptKeys, CryptKeysStorage},
    gbox_link::{GboxLink, OptionalGboxLink},
    installable_app::InstallableApp,
    links_map::{EncryptedLinksMap, LinksMap},
    schema::SchemaVersion,
    unencrypted::*,
};
#[cfg(feature = "schema")]
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
#[serde(untagged)]
#[cfg_attr(feature = "schema", derive(JsonSchema), schemars(deny_unknown_fields))]
pub enum GboxRepo {
    Plain(Repository),
    Encrypted(encrypted::Repository),
}

#[cfg(feature = "schema")]
impl GboxRepo {
    #[must_use]
    pub fn schema() -> impl Serialize {
        schema_for!(Self)
    }

    pub fn validate_and_parse(object: impl AsRef<str>) -> anyhow::Result<Self> {
        let object = serde_json::from_str::<serde_json::Value>(object.as_ref())?;
        let schema = serde_json::to_value(Self::schema())?;
        validation::validate_object(&object, schema)?;

        let repo = serde_json::from_value(object)?;
        Ok(repo)
    }
}
