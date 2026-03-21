use crate::altstore::AltStoreColor;
use crate::serde_support::chrono_iso8601;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewsItem {
    pub identifier: String,

    #[serde(with = "chrono_iso8601")]
    pub date: DateTime<Utc>,

    pub title: String,

    pub caption: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tint_color: Option<AltStoreColor>,

    #[serde(rename = "imageURL")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<Url>,

    #[serde(rename = "url")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_url: Option<Url>,

    #[serde(rename = "appID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,

    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub notify: bool,
}
