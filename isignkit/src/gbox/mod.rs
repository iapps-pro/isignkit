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
use super::{altstore, sidestore};
use crate::error::ConversionError;
use chrono::Utc;
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

    pub fn validate_and_parse(object: impl AsRef<str>) -> Result<Self, crate::error::GboxError> {
        let object = serde_json::from_str::<serde_json::Value>(object.as_ref())?;
        let schema = serde_json::to_value(Self::schema())?;
        validation::validate_object(&object, schema)?;

        let repo = serde_json::from_value(object)?;
        Ok(repo)
    }

    #[must_use]
    pub fn is_encrypted(&self) -> bool {
        matches!(self, Self::Encrypted(_))
    }

    #[must_use]
    pub fn general_info(&self) -> &RepoInfo {
        match self {
            Self::Plain(repo) => &repo.info,
            Self::Encrypted(repo) => &repo.info,
        }
    }
}

impl From<altstore::Repository> for GboxRepo {
    fn from(repo: altstore::Repository) -> Self {
        Self::Plain(Repository::from(repo))
    }
}

impl From<altstore::Repository> for Repository {
    fn from(repo: altstore::Repository) -> Self {
        let apps = repo.apps.unwrap_or_default();

        let categories = apps
            .iter()
            .filter_map(|app| app.category.clone())
            .map(|cat| cat.to_string())
            .collect();

        let updated_at = apps
            .iter()
            .map(|app| app.version_asset.date_for_newest())
            .max()
            .flatten()
            .unwrap_or(Utc::now());

        Repository {
            schema_version: SchemaVersion::default(),
            info: RepoInfo {
                name: repo.name,
                author: repo.fedi_username.unwrap_or_default(),
                icon_image_url: repo
                    .icon_url
                    .map_or_else(String::new, |url| url.to_string()),
                description: repo.description.unwrap_or_default(),
                link_title: String::new(),
                link_url: repo
                    .website
                    .unwrap_or("https://example.com".parse().unwrap()),
                update_time: updated_at.to_rfc3339(),
                items_unlock_hash: None,
                source_processor: None,
                export_enable: None,
            },
            categories,
            applications: apps
                .into_iter()
                .flat_map(|app| Ok::<_, ConversionError>(Item::SelfSign(app.try_into()?)))
                .collect(),
        }
    }
}

impl From<sidestore::Repository> for GboxRepo {
    fn from(repo: sidestore::Repository) -> Self {
        Self::Plain(Repository::from(repo))
    }
}

impl From<sidestore::Repository> for Repository {
    fn from(repo: sidestore::Repository) -> Self {
        let apps = repo.apps.unwrap_or_default();

        let categories = apps
            .iter()
            .filter_map(|app| app.category.clone())
            .map(|cat| cat.to_string())
            .collect();

        let updated_at = apps
            .iter()
            .map(|app| app.download_asset.date_for_newest())
            .max()
            .flatten()
            .unwrap_or(Utc::now());

        Repository {
            schema_version: SchemaVersion::default(),
            info: RepoInfo {
                name: repo.name,
                author: String::new(),
                icon_image_url: repo
                    .icon_url
                    .map_or_else(String::new, |url| url.to_string()),
                description: repo.description.unwrap_or_default(),
                link_title: String::new(),
                link_url: repo
                    .website
                    .unwrap_or("https://example.com".parse().unwrap()),
                update_time: updated_at.to_rfc3339(),
                items_unlock_hash: None,
                source_processor: None,
                export_enable: None,
            },
            categories,
            applications: apps
                .into_iter()
                .flat_map(|app| Ok::<_, ConversionError>(Item::SelfSign(app.try_into()?)))
                .collect(),
        }
    }
}
