use super::{
    CryptKeys, SchemaVersion,
    encrypted::{self, Item as EItem},
    gbox_link::{GboxLink, OptionalGboxLink},
};
use anyhow::{Result, anyhow};
use base64::{Engine, prelude::BASE64_STANDARD};
use serde::{Deserialize, Serialize};
use std::ops::Not;
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct Repository {
    #[serde(rename = "version")]
    pub schema_version: SchemaVersion,

    #[serde(flatten)]
    pub info: RepoInfo,
    #[serde(rename = "appCategories")]
    pub categories: Vec<String>,
    #[serde(rename = "appRepositories")]
    pub applications: Vec<Item>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepoInfo {
    #[serde(rename = "sourceName")]
    pub name: String,
    #[serde(rename = "sourceAuthor")]
    pub author: String,
    #[serde(rename = "sourceImage")]
    pub icon_image_url: String,
    #[serde(rename = "sourceDescription")]
    pub description: String,
    #[serde(rename = "sourceLinkTitle")]
    pub link_title: String,
    #[serde(rename = "sourceLinkUrl")]
    pub link_url: Url,
    #[serde(rename = "sourceUpdateTime")]
    pub update_time: String,
    #[serde(rename = "sourceUnlockHash", skip_serializing_if = "Option::is_none")]
    pub items_unlock_hash: Option<String>,
    #[serde(rename = "sourceProcessor", skip_serializing_if = "Option::is_none")]
    pub source_processor: Option<serde_json::Value>,
    #[serde(rename = "sourceExportEnable", skip_serializing_if = "Option::is_none")]
    pub export_enable: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ItemBase {
    #[serde(rename = "appName")]
    pub name: String,
    #[serde(rename = "appDescription")]
    pub description: String,
    #[serde(rename = "appVersion", skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(rename = "appImage")]
    pub image: OptionalGboxLink,
    #[serde(rename = "appUpdateTime")]
    pub update_time: String,
    #[serde(rename = "lock", skip_serializing_if = "Not::not", default)]
    pub password_locked: bool,
    #[serde(skip_serializing_if = "Not::not", default)]
    pub hide: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub detailed_info: Option<String>,
    #[serde(rename = "forcePPQBypass", skip_serializing_if = "Not::not", default)]
    pub force_ppq_bypass: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "appType", rename_all = "SCREAMING_SNAKE_CASE")]
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

    #[serde(rename = "appPackage", skip_serializing_if = "Option::is_none")]
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
    #[serde(rename = "appPlist", skip_serializing_if = "Option::is_none")]
    pub ext_info_link: Option<GboxLink>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ItemLink {
    #[serde(flatten)]
    pub base: ItemBase,
    #[serde(rename = "appLink", skip_serializing_if = "Option::is_none")]
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
    #[serde(rename = "appPlist", skip_serializing_if = "Option::is_none")]
    pub ext_info_link: Option<GboxLink>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ItemFile {
    #[serde(flatten)]
    pub base: ItemBase,

    #[serde(rename = "appPackage")]
    pub link: GboxLink,
}

pub struct RepoUnlockInfo {
    pub hash: String,
    pub url: Url,
}

impl Repository {
    pub fn encrypt(self, keys: &CryptKeys) -> Result<encrypted::Repository> {
        let items: Vec<encrypted::Item> = self.applications.into_iter().map(Into::into).collect();

        let items = serde_json::to_string(&items)?;
        let items = rncryptor::v3::encrypt(&keys.primary, items.as_bytes())
            .map_err(|error| anyhow!("{error:?}"))?;
        let items = BASE64_STANDARD.encode(items);

        Ok(encrypted::Repository {
            schema_version: self.schema_version,
            info: self.info,
            categories: self.categories,
            applications: items,
        })
    }
}

impl From<EItem> for Item {
    fn from(item: EItem) -> Self {
        match item {
            EItem::SelfSign(item) => Self::SelfSign(item.into()),
            EItem::EnterpriseSign(item) => Self::EnterpriseSign(item.into()),
            EItem::Link(item) => Self::Link(item.into()),
            EItem::Shareing(item) => Self::Shareing(item.into()),
            EItem::File(item) => Self::File(item.into()),
        }
    }
}

impl From<encrypted::ItemBase> for ItemBase {
    fn from(item: encrypted::ItemBase) -> Self {
        Self {
            name: item.name,
            description: item.description,
            version: item.version,
            image: item.image,
            update_time: item.update_time,
            password_locked: item.password_locked,
            hide: item.hide,
            detailed_info: item.detailed_info,
            force_ppq_bypass: item.force_ppq_bypass,
        }
    }
}

impl From<encrypted::ItemApplication> for ItemApplication {
    fn from(item: encrypted::ItemApplication) -> Self {
        Self {
            base: item.base.into(),
            link: item.link,
            ext_info_link: item.ext_info_link,
        }
    }
}

impl From<encrypted::ItemLink> for ItemLink {
    fn from(item: encrypted::ItemLink) -> Self {
        Self {
            base: item.base.into(),
            link: item.link,
        }
    }
}

impl From<encrypted::ItemShareing> for ItemShareing {
    fn from(item: encrypted::ItemShareing) -> Self {
        Self {
            base: item.base.into(),
            ext_info_link: item.ext_info_link,
        }
    }
}

impl From<encrypted::ItemFile> for ItemFile {
    fn from(item: encrypted::ItemFile) -> Self {
        Self {
            base: item.base.into(),
            link: item.link,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_app() {
        let json = r#"
        {
            "hide": true,
            "appType": "SELF_SIGN",
            "appCateIndex": 0,
            "appUpdateTime": "2021-04-16T21:00:00+0800",
            "appName": "Taurine",
            "appVersion": "1.0.4",
            "appImage": "https://gbox.run/Public/appicons/Taurine.png",
            "appPlist": "https://gbox.lanzouw.com/iBBjto6ugyf",
            "appDescription": "CoolStar最新出品的越狱工具，支持全系列iOS设备14.0～14.3系统越狱"
        }
        "#;

        let item: Item = serde_json::from_str(&json).unwrap();
        let Item::SelfSign(app) = item else {
            panic!("Not an application");
        };

        assert_eq!(app.base.name, "Taurine");
        assert_eq!(app.base.version.as_deref(), Some("1.0.4"));
        assert!(app.base.hide);
        assert_eq!(
            app.ext_info_link.as_deref(),
            Some("https://gbox.lanzouw.com/iBBjto6ugyf")
        );
    }

    #[test]
    fn shareing() {
        let json = r#"
       {
            "hide": true,
            "appType": "FILE",
            "appCateIndex": 0,
            "appUpdateTime": "2023-05-05T18:20:00+0800",
            "appName": "AlertTest.dylib",
            "appVersion": "4.0",
            "appImage": "https://gbox.run/Public/appicons/GBox.png",
            "appPackage": "https://gbox.pub/Public/ios/AlertTest.dylib",
            "appDescription": "GBox TS"
        }
        "#;

        let item: Item = serde_json::from_str(&json).unwrap();
        let Item::File(file) = item else {
            panic!("Not a shareing");
        };

        assert_eq!(file.base.version.as_deref(), Some("4.0"));
        assert_eq!(&*file.link, "https://gbox.pub/Public/ios/AlertTest.dylib");
    }

    #[test]
    fn repository() {
        let json = r#"
       {
    "version": "1.0",
    "sourceName": "GBox官方软件源",
    "sourceAuthor": "GBox Official",
    "sourceExportEnable": false,
    "sourceLinkTitle": "主页",
    "sourceLinkUrl": "http://gbox.run",
    "sourceImage": "http://gbox.run/Public/images/source.png",
    "sourceUpdateTime": "2025-06-23T16:00:00+0800",
    "sourceDescription": "GBox官方软件源，源内App均来之互联网，仅用于学习交流。如有冒犯，请联系删除，邮箱：gboxrun@gmail.com",
    "appCategories": [
        "工具",
        "模拟器"
    ],
    "appRepositories": []
    }
        "#;

        let repo: Repository = serde_json::from_str(&json).unwrap();

        assert_eq!(repo.info.author, "GBox Official");
        assert_eq!(repo.info.export_enable, Some(false));
        assert_eq!(repo.categories.len(), 2);
    }
}
