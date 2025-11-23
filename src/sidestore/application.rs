use crate::altstore::{AltStoreColor, OptionalPermissions, Patreon, ScreenshotAsset};
use crate::serde_support::chrono_iso8601;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use url::Url;

// Not mapped to AltStore category, can be arbitrary
pub type Category = String;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Application {
    pub name: String,

    pub bundle_identifier: String,

    pub developer_name: String,

    pub localized_description: String,

    #[serde(rename = "iconURL")]
    pub icon_url: Url,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,

    #[serde(rename = "marketplaceID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marketplace_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tint_color: Option<AltStoreColor>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<Category>,

    #[serde(rename = "screenshots", alias = "screenshotURLs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshot_asset: Option<ScreenshotAsset>,

    #[serde(rename = "appPermissions")]
    #[serde(skip_serializing_if = "OptionalPermissions::is_none")]
    pub permissions: OptionalPermissions,

    #[serde(rename = "platformURLs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platforms_urls: Option<HashSet<PlatformUrl>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub patreon: Option<Patreon>,

    #[serde(flatten)]
    pub download_asset: DownloadAsset,

    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub beta: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub struct PlatformUrl {
    pub platform: Platform,

    #[serde(rename = "downloadURL")]
    pub download_url: Url,
}

impl Hash for PlatformUrl {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.platform.hash(state);
    }
}

impl PartialEq<Self> for PlatformUrl {
    fn eq(&self, other: &Self) -> bool {
        self.platform == other.platform
    }
}

impl Eq for PlatformUrl {}

#[derive(Deserialize, Serialize, Debug, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Ios,
    Tvos,
    Macos,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReleaseTrack {
    pub track: String,
    pub releases: Vec<Version>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub version: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_version: Option<String>,

    #[serde(with = "chrono_iso8601")]
    pub date: DateTime<Utc>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub localized_description: Option<String>,

    #[serde(rename = "downloadURL")]
    pub download_url: Url,

    pub size: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,

    #[serde(rename = "minOSVersion")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minos_version: Option<String>,

    #[serde(rename = "maxOSVersion")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxos_version: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum DownloadAsset {
    Tracks {
        tracks: Vec<ReleaseTrack>,
    },
    Multiple {
        versions: Vec<Version>,
    },
    SingleVersion {
        version: String,

        #[serde(rename = "versionDate", with = "chrono_iso8601")]
        date: DateTime<Utc>,

        #[serde(rename = "versionDescription")]
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,

        #[serde(rename = "downloadURL")]
        download_url: Url,

        size: i32,
    },
}
