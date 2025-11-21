#[cfg(feature = "schema")]
use super::validation::{schema_datetime_format, validate_object};
use super::{
    CryptKeys, SchemaVersion,
    gbox_link::{GboxLink, OptionalGboxLink},
    unencrypted::{self, Item as UItem, RepoInfo},
};
use crate::unit_number::UnsignedNumber;
use anyhow::{Result, anyhow};
use base64::{Engine, prelude::BASE64_STANDARD};
#[cfg(feature = "schema")]
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "schema", derive(JsonSchema), schemars(deny_unknown_fields))]
pub struct Repository {
    #[serde(rename = "version")]
    pub schema_version: SchemaVersion,

    #[serde(flatten)]
    pub info: RepoInfo,
    #[serde(rename = "appCategories", default)]
    pub categories: Vec<String>,
    #[serde(rename = "appRepositories")]
    pub applications: String,
}

macro_rules! with_item_base {
    ($name:ident, { $($field:tt)* }) => {
        #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
        #[serde(rename_all = "camelCase")]
        #[cfg_attr(feature = "schema", derive(JsonSchema), schemars(deny_unknown_fields))]
        pub struct $name {
            #[serde(rename = "AO4")]
            pub name: String,

            #[serde(rename = "AO8")]
            pub description: String,

            #[serde(rename = "AO5", skip_serializing_if = "Option::is_none")]
            pub version: Option<String>,

            #[serde(rename = "AO6")]
            pub image: OptionalGboxLink,

            #[serde(rename = "AO3")]
            #[cfg_attr(feature = "schema", schemars(transform = schema_datetime_format))]
            pub update_time: String,

            /// Item will be locked with unlock code when flag is set
            #[serde(rename = "A11", skip_serializing_if = "std::ops::Not::not", default)]
            pub password_locked: bool,

            /// If flag is set, item will be displayed only after unlock
            #[serde(rename = "A12", skip_serializing_if = "std::ops::Not::not", default)]
            pub hide_until_unlocked: bool,

            /// Custom `GBoxPlus` field. Shows extra line below version line if set
            #[serde(skip_serializing_if = "Option::is_none")]
            pub detailed_info: Option<String>,

            /// Custom `GBoxPlus` field. Forces PPQ bypass when signing if set
            #[serde(rename = "forcePPQBypass", default)]
            pub force_ppq_bypass: bool,

            #[serde(rename = "AO2")]
            pub category_index: Option<UnsignedNumber>,

            $($field)*
        }
    };
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
#[serde(tag = "AO1", rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "schema", derive(JsonSchema), schemars(deny_unknown_fields))]
pub enum Item {
    /// Application which mustbe signed with custom certificate
    SelfSign(ItemApplication),

    #[serde(rename = "ENT_SIGN")]
    /// Application which must be signed with GBox-sourced enterprise certificate
    EnterpriseSign(ItemApplication),

    /// Link type, directly open the url for advertising or promotional purposes
    Link(ItemLink),

    /// Application directly obtained from the App Store.
    ///
    /// It is encrypted and already signed with App Store certificate,
    /// so `GBox` will install it without resigning
    #[serde(rename = "SHAREING")]
    AppWithoutSign(ItemApplication),

    File(ItemFile),
}

with_item_base!(ItemApplication, {
    #[serde(rename = "AO7")]
    pub link: Option<GboxLink>,

    /// Link to the external application plist file containing name, version, image and file link
    ///
    ///
    /// `GBox` uses it this way
    /// ```objc
    ///  v8 = objc_msgSend(v7, "objectForKeyChain:", CFSTR("items->0->assets->0->url"));
    ///  v9 = objc_msgSend(v7, "objectForKeyChain:", CFSTR("items->0->assets->1->url"));
    ///  v10 = objc_msgSend(v7, "objectForKeyChain:", CFSTR("items->0->metadata->title"));
    ///  v11 = objc_msgSend(v7, "objectForKeyChain:", CFSTR("items->0->metadata->bundle-version"));
    ///  v12 = -[GBApp init](objc_alloc(&OBJC_CLASS___GBApp), "init");
    ///  -[GBApp setAppPackage:](v12, "setAppPackage:", v8);
    ///  -[GBApp setAppName:](v12, "setAppName:", v10);
    ///  -[GBApp setAppVersion:](v12, "setAppVersion:", v11);
    ///  -[GBApp setAppImage:](v12, "setAppImage:", v9);
    /// ```
    #[serde(rename = "AO9", skip_serializing_if = "Option::is_none")]
    pub ext_info_link: Option<GboxLink>,
});

with_item_base!(ItemLink, {
    #[serde(rename = "A10", skip_serializing_if = "Option::is_none")]
    pub link: Option<GboxLink>,
});

with_item_base!(ItemFile, {
    #[serde(rename = "AO7")]
    pub link: GboxLink,
});

impl Repository {
    pub fn decrypt(self, keys: &CryptKeys) -> Result<unencrypted::Repository> {
        let items: Vec<Item> = serde_json::from_value(self.decrypt_payload(keys)?)?;
        let items = items.into_iter().map(Into::into).collect();

        Ok(unencrypted::Repository {
            schema_version: self.schema_version,
            info: self.info,
            categories: self.categories,
            applications: items,
        })
    }

    #[cfg(feature = "schema")]
    pub fn validate_items(&self, keys: &CryptKeys) -> Result<()> {
        let items = self.decrypt_payload(keys)?;
        let schema = serde_json::to_value(schema_for!(Vec<Item>))?;
        validate_object(&items, schema)?;

        Ok(())
    }

    pub fn decrypt_payload(&self, keys: &CryptKeys) -> Result<serde_json::Value> {
        let items = BASE64_STANDARD.decode(self.applications.as_bytes())?;

        let items = rncryptor::v3::decrypt(&keys.primary, &items)
            .map_err(|error| anyhow!("Decryption failed: {error:?}"))?;

        let items = serde_json::from_slice(items.as_slice())?;

        Ok(items)
    }
}

impl From<UItem> for Item {
    fn from(item: UItem) -> Self {
        match item {
            UItem::SelfSign(item) => Self::SelfSign(item.into()),
            UItem::EnterpriseSign(item) => Self::EnterpriseSign(item.into()),
            UItem::Link(item) => Self::Link(item.into()),
            UItem::AppWithoutSign(item) => Self::AppWithoutSign(item.into()),
            UItem::File(item) => Self::File(item.into()),
        }
    }
}

impl From<unencrypted::ItemApplication> for ItemApplication {
    fn from(item: unencrypted::ItemApplication) -> Self {
        Self {
            name: item.name,
            description: item.description,
            version: item.version,
            image: item.image,
            update_time: item.update_time,
            password_locked: item.password_locked,
            hide_until_unlocked: item.hide_until_unlocked,
            detailed_info: item.detailed_info,
            force_ppq_bypass: item.force_ppq_bypass,
            category_index: item.category_index,
            link: item.link,
            ext_info_link: item.ext_info_link,
        }
    }
}

impl From<unencrypted::ItemLink> for ItemLink {
    fn from(item: unencrypted::ItemLink) -> Self {
        Self {
            name: item.name,
            description: item.description,
            version: item.version,
            image: item.image,
            update_time: item.update_time,
            password_locked: item.password_locked,
            hide_until_unlocked: item.hide_until_unlocked,
            detailed_info: item.detailed_info,
            force_ppq_bypass: item.force_ppq_bypass,
            category_index: item.category_index,
            link: item.link,
        }
    }
}

impl From<unencrypted::ItemFile> for ItemFile {
    fn from(item: unencrypted::ItemFile) -> Self {
        Self {
            name: item.name,
            description: item.description,
            version: item.version,
            image: item.image,
            update_time: item.update_time,
            password_locked: item.password_locked,
            hide_until_unlocked: item.hide_until_unlocked,
            detailed_info: item.detailed_info,
            force_ppq_bypass: item.force_ppq_bypass,
            category_index: item.category_index,
            link: item.link,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gbox::CryptKeysStorage;

    #[test]
    fn decrypt() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/test/gbox/repository.json");
        let json = std::fs::read_to_string(path).expect("Can't read file");

        let repo: Repository = serde_json::from_str(&json).unwrap();

        let keys_storage = CryptKeysStorage::default();
        let keys = keys_storage.latest_keys();
        let repo = repo.decrypt(keys).unwrap();

        let UItem::SelfSign(ref app) = repo.applications[0] else {
            panic!("Not an app");
        };

        assert_eq!(
            app.ext_info_link.as_deref(),
            Some("https://gbox.lanzouw.com/iXSbho00o3g")
        );

        let repo = repo.encrypt(&keys);
        assert!(repo.is_ok());
    }
}
