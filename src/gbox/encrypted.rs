use super::{
    CryptKeys, SchemaVersion,
    gbox_link::{GboxLink, OptionalGboxLink},
    unencrypted::{self, Item as UItem, RepoInfo},
};
use anyhow::{Result, anyhow};
use base64::{Engine, prelude::BASE64_STANDARD};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Repository {
    #[serde(rename = "version")]
    pub schema_version: SchemaVersion,

    #[serde(flatten)]
    pub info: RepoInfo,
    #[serde(rename = "appCategories")]
    pub categories: Vec<String>,
    #[serde(rename = "appRepositories")]
    pub applications: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ItemBase {
    #[serde(rename = "AO4")]
    pub name: String,
    #[serde(rename = "AO8")]
    pub description: String,
    #[serde(rename = "AO5", skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(rename = "AO6")]
    pub image: OptionalGboxLink,
    #[serde(rename = "AO3")]
    pub update_time: String,
    #[serde(rename = "A11", skip_serializing_if = "std::ops::Not::not", default)]
    pub password_locked: bool,
    #[serde(rename = "A12", skip_serializing_if = "std::ops::Not::not", default)]
    pub hide: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub detailed_info: Option<String>,
    #[serde(rename = "forcePPQBypass", default)]
    pub force_ppq_bypass: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "AO1", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Item {
    SelfSign(ItemApplication),
    #[serde(rename = "ENT_SIGN")]
    EnterpriseSign(ItemApplication),
    Link(ItemLink),
    Shareing(ItemShareing),
    File(ItemFile),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ItemApplication {
    #[serde(flatten)]
    pub base: ItemBase,

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
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ItemLink {
    #[serde(flatten)]
    pub base: ItemBase,
    #[serde(rename = "A10", skip_serializing_if = "Option::is_none")]
    pub link: Option<GboxLink>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ItemShareing {
    #[serde(flatten)]
    pub base: ItemBase,

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
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ItemFile {
    #[serde(flatten)]
    pub base: ItemBase,
    #[serde(rename = "AO7")]
    pub link: GboxLink,
}

impl Repository {
    pub fn decrypt(self, keys: &CryptKeys) -> Result<unencrypted::Repository> {
        let items = BASE64_STANDARD.decode(self.applications.as_bytes())?;
        let items =
            rncryptor::v3::decrypt(&keys.primary, &items).map_err(|error| anyhow!("{error:?}"))?;
        let items: Vec<Item> = serde_json::from_slice(items.as_slice())?;
        let items = items.into_iter().map(Into::into).collect();

        Ok(unencrypted::Repository {
            schema_version: self.schema_version,
            info: self.info,
            categories: self.categories,
            applications: items,
        })
    }
}

impl From<UItem> for Item {
    fn from(item: UItem) -> Self {
        match item {
            UItem::SelfSign(item) => Self::SelfSign(item.into()),
            UItem::EnterpriseSign(item) => Self::EnterpriseSign(item.into()),
            UItem::Link(item) => Self::Link(item.into()),
            UItem::Shareing(item) => Self::Shareing(item.into()),
            UItem::File(item) => Self::File(item.into()),
        }
    }
}

impl From<unencrypted::ItemApplication> for ItemApplication {
    fn from(app: unencrypted::ItemApplication) -> Self {
        Self {
            base: app.base.into(),
            link: app.link,
            ext_info_link: app.ext_info_link,
        }
    }
}

impl From<unencrypted::ItemLink> for ItemLink {
    fn from(link: unencrypted::ItemLink) -> Self {
        Self {
            base: link.base.into(),
            link: link.link,
        }
    }
}

impl From<unencrypted::ItemShareing> for ItemShareing {
    fn from(item: unencrypted::ItemShareing) -> Self {
        Self {
            base: item.base.into(),
            ext_info_link: item.ext_info_link,
        }
    }
}

impl From<unencrypted::ItemFile> for ItemFile {
    fn from(item: unencrypted::ItemFile) -> Self {
        Self {
            base: item.base.into(),
            link: item.link,
        }
    }
}

impl From<unencrypted::ItemBase> for ItemBase {
    fn from(base: unencrypted::ItemBase) -> Self {
        Self {
            name: base.name,
            description: base.description,
            version: base.version,
            image: base.image,
            update_time: base.update_time,
            password_locked: base.password_locked,
            hide: base.hide,
            detailed_info: base.detailed_info,
            force_ppq_bypass: base.force_ppq_bypass,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gbox::CryptKeysStorage;

    #[test]
    fn decrypt() {
        let json = r#"
 {
  "appCategories" : [
    "工具",
    "模拟器"
  ],
  "sourceUpdateTime" : "2025-09-17T21:26+0300",
  "sourceAuthor" : "GBox Official",
  "sourceLinkUrl" : "http:\/\/gbox.run",
  "sourceDescription" : "GBox官方软件源，源内App均来之互联网，仅用于学习交流。如有冒犯，请联系删除，邮箱：gboxrun@gmail.com",
  "sourceExportEnable" : true,
  "version" : "1.0",
  "sourceName" : "GBox官方软件源",
  "sourceLinkTitle" : "主页",
  "sourceImage" : "http:\/\/gbox.run\/Public\/images\/source.png",
  "appRepositories" : "AwFTq7MS2ugoxvONWXEx7YsBC02jj3liIYAnUneSHqLEWa36snlQ2pe7V0OZNIsCB+h+Jz6gozBtaNfiwO7ytRp2oDvMSIt6liSDBBYPRIhOTxrXyyX7lC04Pel5b0Ku5ai9KJ4A34eggUL\/tl+Q1ORFMXrOL2s9gA80INpUyRCuQzWH9bkII0e+hw6Nv18rhbnaO1vV2aBeOxoqpdZjm43mCYisVfyfcPPRKSsoOXpbYJibvZava4ty9p\/55gusjUXa6tbtpQhRRbqmxjldvVopTSn3feegzSlVBkUirmJmYpi0b25qSB1mtqEn+iuUDuvUPG7TG8z5dErQEzyj8vSnw3JjdM6XenuxDsmkecSZWuL+fXjxqBCAugQ9rzHH8lrGBq7IUFNCg8+IufVvNCGy78WOcdoJ8NWx0OAa5HdD\/zKFK6m5MXEOAc7SiCW2LG4TKomXDBc9VIJP3jtVtC1vJLbUufG83m9t8svWDJhRh9XfP2YfC9xmA6+NQj+aqk1iaq5qU1TvBpwWrpDRvaC6ou3JQ8q8YFYnpLEt22kUpLDokWo+dURoPGjXZHq8WEifwlp8V+YTkevq4FnzAtkK"
}
        "#;

        let repo: Repository = serde_json::from_str(&json).unwrap();

        let keys_storage = CryptKeysStorage::default();
        let keys = keys_storage.newest_keys();
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
