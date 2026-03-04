use super::{AltStoreColor, OptionalPermissions};
use crate::serde_support::chrono_iso8601;
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
    #[serde(skip_serializing_if = "Option::is_none")]
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
