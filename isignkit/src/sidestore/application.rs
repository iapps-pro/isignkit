use crate::altstore::{
    self, AltStoreColor, Category, OptionalPermissions, Patreon, ScreenshotAsset,
};
use crate::{
    error::ConversionError,
    gbox,
    serde_support::{chrono_iso8601, de_optional},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use url::Url;

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

    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "de_optional",
        default
    )]
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

impl DownloadAsset {
    #[must_use]
    pub fn date_for_newest(&self) -> Option<DateTime<Utc>> {
        match self {
            Self::Tracks { .. } => None,
            Self::Multiple { versions } => versions.iter().map(|vers| vers.date).max(),
            Self::SingleVersion { date, .. } => Some(*date),
        }
    }

    #[must_use]
    pub fn url_for_newest(&self) -> Option<&Url> {
        match self {
            Self::Tracks { .. } => None,
            Self::Multiple { versions } => versions
                .iter()
                .max_by_key(|vers| vers.date)
                .map(|vers| &vers.download_url),
            Self::SingleVersion { download_url, .. } => Some(download_url),
        }
    }

    #[must_use]
    pub fn newest_version_str(&self) -> Option<&String> {
        match self {
            Self::Tracks { .. } => None,
            Self::Multiple { versions } => versions
                .iter()
                .max_by_key(|vers| vers.date)
                .map(|vers| &vers.version),
            Self::SingleVersion { version, .. } => Some(version),
        }
    }
}

impl From<altstore::Version> for Version {
    fn from(altstore_version: altstore::Version) -> Self {
        Self {
            version: altstore_version.version,
            build_version: altstore_version.build_version,
            date: altstore_version.date,
            localized_description: altstore_version.localized_description,
            download_url: altstore_version.download_url,
            size: altstore_version.size,
            sha256: altstore_version.sha256,
            minos_version: altstore_version.minos_version,
            maxos_version: altstore_version.maxos_version,
        }
    }
}

impl From<altstore::VersionAsset> for DownloadAsset {
    fn from(altstore_asset: altstore::VersionAsset) -> Self {
        match altstore_asset {
            altstore::VersionAsset::Single {
                version,
                date,
                description,
                download_url,
                size,
            } => DownloadAsset::SingleVersion {
                version,
                date,
                description,
                download_url,
                size,
            },
            altstore::VersionAsset::Multiple { versions } => {
                let versions = versions.into_iter().map(Into::into).collect();
                Self::Multiple { versions }
            }
        }
    }
}

impl From<altstore::Application> for Application {
    fn from(app: altstore::Application) -> Self {
        Self {
            name: app.name,
            bundle_identifier: app.bundle_identifier,
            developer_name: app.developer_name,
            localized_description: app.localized_description,
            icon_url: app.icon_url,
            subtitle: app.subtitle,
            marketplace_id: app.marketplace_id,
            tint_color: app.tint_color,
            category: app.category,
            screenshot_asset: app.screenshot_asset,
            permissions: app.permissions,
            platforms_urls: None,
            patreon: app.patreon,
            download_asset: app.version_asset.into(),
            beta: app.beta,
        }
    }
}

impl TryFrom<gbox::Item> for Application {
    type Error = ConversionError;

    fn try_from(item: gbox::Item) -> Result<Self, Self::Error> {
        let (gbox::Item::SelfSign(app)
        | gbox::Item::EnterpriseSign(app)
        | gbox::Item::AppWithoutSign(app)) = item
        else {
            return Err(ConversionError::UnsupportedItemType);
        };

        Self::try_from(app)
    }
}

impl TryFrom<gbox::ItemApplication> for Application {
    type Error = ConversionError;

    fn try_from(app_item: gbox::ItemApplication) -> Result<Self, Self::Error> {
        let icon_link = app_item.image.0;
        let icon_url = icon_link
            .and_then(gbox::GboxLink::into_url)
            .unwrap_or_else(|| "https://example.com".parse().unwrap());

        let update_dt = DateTime::parse_from_rfc3339(&app_item.update_time)
            .map_or_else(|_| Utc::now(), |dt| dt.to_utc());

        let download_asset = DownloadAsset::SingleVersion {
            version: app_item.version.unwrap_or_default(),
            date: update_dt,
            description: None,
            download_url: app_item
                .link
                .and_then(gbox::GboxLink::into_url)
                .ok_or(ConversionError::InvalidOrMissingUrl)?,
            size: 0,
        };

        Ok(Application {
            name: app_item.name,
            bundle_identifier: "org.example.app".to_string(),
            marketplace_id: None,
            developer_name: String::new(),
            subtitle: None,
            localized_description: app_item.description,
            icon_url,
            tint_color: None,
            category: None,
            screenshot_asset: None,
            download_asset,
            permissions: OptionalPermissions(None),
            platforms_urls: None,
            patreon: None,
            beta: false,
        })
    }
}
