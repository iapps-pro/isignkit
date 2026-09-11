use super::{AltStoreColor, OptionalPermissions};
use crate::{
    error::ConversionError,
    gbox,
    serde_support::{chrono_iso8601, de_optional},
    sidestore,
};
use chrono::{DateTime, Utc};
pub use codes_iso_4217::CurrencyCode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::{Display, EnumString};
use url::Url;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Application {
    /// The name of your app as it will appear on its store page.
    pub name: String,

    /// App's bundle identifier (`CFBundleIdentifier`).
    /// It is case sensitive and should match exactly what is in the Info.plist.
    pub bundle_identifier: String,

    /// The "Apple ID" of your notarized app.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marketplace_id: Option<String>,

    /// The name of the developer or developers as it will appear on the store page.
    pub developer_name: String,

    /// A short, one-sentence description of the app that will appear in the Browse tab.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,

    /// A full-length description of your app.
    pub localized_description: String,

    /// A link to you app's icon image
    #[serde(rename = "iconURL")]
    pub icon_url: Url,

    /// The color used to theme your app's store page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tint_color: Option<AltStoreColor>,

    /// The store category best representing your app.
    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "de_optional",
        default
    )]
    pub category: Option<Category>,

    #[serde(rename = "screenshots", alias = "screenshotURLs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshot_asset: Option<ScreenshotAsset>,

    #[serde(flatten)]
    pub version_asset: VersionAsset,

    /// An object listing all entitlements and privacy permissions information used by the app.
    #[serde(rename = "appPermissions")]
    #[serde(skip_serializing_if = "OptionalPermissions::is_none")]
    pub permissions: OptionalPermissions,

    /// An object specifying the required pledge/tiers to download the app.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patreon: Option<Patreon>,

    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub beta: bool,
}

#[derive(
    Serialize, Deserialize, Debug, Default, Clone, Eq, PartialEq, Hash, EnumString, Display,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Category {
    Developer,
    Entertainment,
    Games,
    Lifestyle,
    #[default]
    Other,
    PhotoVideo,
    Social,
    Utilities,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    /// App's version number (`CFBundleShortVersionString`).
    /// It is case sensitive and should match exactly what is in the Info.plist.
    pub version: String,

    /// App's build number (`CFBundleVersion`).
    /// It is case sensitive and should match exactly what is in the Info.plist.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_version: Option<String>,

    /// The full version displayed to users on your app's store page and throughout the UI.
    /// This can be anything you want and does not need to match version or buildVersion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marketing_version: Option<String>,

    /// The release date for this version.
    #[serde(with = "chrono_iso8601")]
    pub date: DateTime<Utc>,

    /// A description of what's new in this version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub localized_description: Option<String>,

    /// The URL of the uploaded .ipa file.
    #[serde(rename = "downloadURL")]
    pub download_url: Url,

    pub size: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,

    /// If you are unable to preserve an ADP's directory structure as-is,
    /// this allows you to manually specify the download URL for individual files in an ADP
    #[serde(rename = "assetURLs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets_urls: Option<HashMap<String, Url>>,

    /// The minimum iOS version supported by this release.
    /// `AltStore` will hide any updates that are not supported by the user's device.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minos_version: Option<String>,

    /// The maximum iOS version supported by this release (inclusive).
    /// `AltStore` will hide any updates that are not supported by the user's device.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxos_version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ScreenshotAsset {
    Objects(ScreenshotObjects),
    Urls(Vec<Url>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ScreenshotObjects {
    Universal(Vec<ScreenshotObject>),
    Individual(HashMap<ScreenshotDevice, Vec<ScreenshotObject>>),
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ScreenshotDevice {
    Ipad,
    Iphone,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotObject {
    /// Link to a screenshot of the app.
    #[serde(rename = "imageURL")]
    pub image_url: Url,

    /// The pixel width of the image. If not provided, `AltStore` will assume
    /// a default size of 393 x 852 points (iPhone 15 in portrait mode).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u64>,

    /// The pixel height of the image. If not provided, `AltStore` will assume
    /// a default size of 393 x 852 points (iPhone 15 in portrait mode).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Patreon {
    /// The minimum pledge amount required for download.
    /// This can be used to limit downloads to higher tiers.
    ///
    /// This amount is assumed to be in USD by default.  If using a non-USD currency
    /// for the campaign, you must specify it using the currency key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pledge: Option<u64>,

    /// The ISO currency code of your campaign's currency.
    ///
    /// Required if you provide a pledge amount and the campaign uses a non-USD currency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<CurrencyCode>,

    /// The identifier of a campaign benefit.
    /// You can add [benefits](https://support.patreon.com/hc/en-us/articles/203913559-How-to-set-up-paid-tiers-and-benefits)
    /// to any of your Patreon campaign tiers, then specify it using this key
    /// to allow anyone with that benefit to download your app.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benefit: Option<String>,

    /// A list of tier identifiers designating which tiers are required to download.
    /// A user must be a member of one of these tiers to download your app.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tiers: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum VersionAsset {
    Multiple {
        versions: Vec<Version>,
    },
    Single {
        version: String,

        #[serde(rename = "versionDate", with = "chrono_iso8601")]
        date: DateTime<Utc>,

        #[serde(rename = "versionDescription", skip_serializing_if = "Option::is_none")]
        description: Option<String>,

        #[serde(rename = "downloadURL")]
        download_url: Url,

        size: i32,
    },
}

impl VersionAsset {
    #[must_use]
    pub fn date_for_newest(&self) -> Option<DateTime<Utc>> {
        match self {
            Self::Multiple { versions } => versions.iter().map(|vers| vers.date).max(),
            Self::Single { date, .. } => Some(*date),
        }
    }

    #[must_use]
    pub fn url_for_newest(&self) -> Option<&Url> {
        match self {
            Self::Multiple { versions } => versions
                .iter()
                .max_by_key(|vers| vers.date)
                .map(|vers| &vers.download_url),
            Self::Single { download_url, .. } => Some(download_url),
        }
    }

    #[must_use]
    pub fn newest_version_str(&self) -> Option<&String> {
        match self {
            Self::Multiple { versions } => versions
                .iter()
                .max_by_key(|vers| vers.date)
                .map(|vers| &vers.version),
            Self::Single { version, .. } => Some(version),
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

        let version = VersionAsset::Single {
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
            version_asset: version,
            permissions: OptionalPermissions(None),
            patreon: None,
            beta: false,
        })
    }
}

impl From<sidestore::Version> for Version {
    fn from(sidestore_vers: sidestore::Version) -> Self {
        Self {
            version: sidestore_vers.version,
            build_version: sidestore_vers.build_version,
            marketing_version: None,
            date: sidestore_vers.date,
            localized_description: sidestore_vers.localized_description,
            download_url: sidestore_vers.download_url,
            size: sidestore_vers.size,
            sha256: sidestore_vers.sha256,
            assets_urls: None,
            minos_version: sidestore_vers.minos_version,
            maxos_version: sidestore_vers.maxos_version,
        }
    }
}

impl TryFrom<sidestore::DownloadAsset> for VersionAsset {
    type Error = ConversionError;

    fn try_from(sidestore_asset: sidestore::DownloadAsset) -> Result<Self, Self::Error> {
        match sidestore_asset {
            sidestore::DownloadAsset::Tracks { .. } => Err(ConversionError::MissingVersionAsset),
            sidestore::DownloadAsset::SingleVersion {
                version,
                date,
                description,
                download_url,
                size,
            } => Ok(Self::Single {
                version,
                date,
                description,
                download_url,
                size,
            }),
            sidestore::DownloadAsset::Multiple { versions } => {
                let versions = versions.into_iter().map(Into::into).collect();
                Ok(Self::Multiple { versions })
            }
        }
    }
}

impl TryFrom<sidestore::Application> for Application {
    type Error = ConversionError;

    fn try_from(app: sidestore::Application) -> Result<Self, Self::Error> {
        Ok(Application {
            name: app.name,
            bundle_identifier: app.bundle_identifier,
            marketplace_id: app.marketplace_id,
            developer_name: app.developer_name,
            subtitle: app.subtitle,
            localized_description: app.localized_description,
            icon_url: app.icon_url,
            tint_color: app.tint_color,
            category: app.category,
            screenshot_asset: app.screenshot_asset,
            version_asset: app.download_asset.try_into()?,
            permissions: app.permissions,
            patreon: None,
            beta: app.beta,
        })
    }
}
