use super::AltStoreColor;
use crate::serde_support::chrono_iso8601;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NewsItem {
    /// The title of the News item.
    pub title: String,

    /// A unique value to distinguish this News item from others in the source.
    pub identifier: String,

    /// A short, one-sentence description of the News item.
    pub caption: String,

    /// The publishing date for this News item.
    #[serde(with = "chrono_iso8601")]
    pub date: DateTime<Utc>,

    /// The background color for the News item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tint_color: Option<AltStoreColor>,

    /// A link to the image you want featured with your News item.
    #[serde(rename = "imageURL")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<Url>,

    /// When true, `AltStore` will send a push notification about this News item
    /// when it next checks for updates in the background.
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub notify: bool,

    /// A link that `AltStore` should open when the News item is tapped.
    /// Links will be opened in an in-app web browser.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,

    /// The bundle identifier of an associated app. This will make the app's info banner
    /// appear below the News item, which will open the app's Store page when tapped.
    #[serde(rename = "appID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,
}
