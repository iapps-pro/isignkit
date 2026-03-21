mod application;
mod news;

pub use self::{application::*, news::NewsItem};
use crate::altstore::AltStoreColor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<Url>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<Url>,

    #[serde(rename = "headerURL")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_image_url: Option<Url>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub patreon_url: Option<Url>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tint_color: Option<AltStoreColor>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_info: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub apps: Option<Vec<Application>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub news: Option<Vec<NewsItem>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub featured_apps: Option<Vec<String>>,

    #[serde(rename = "identifier")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Result;

    fn read_source(name: &str) -> Result<Repository> {
        let path = format!("{}/test/sidestore/{}", env!("CARGO_MANIFEST_DIR"), name);
        let json = std::fs::read_to_string(path).expect("Can't read file");
        serde_json::from_str(&json)
    }

    #[test]
    fn parse() {
        let repo = read_source("apps.json").unwrap();
        assert!(repo.featured_apps.is_none());
        assert_eq!(repo.apps.unwrap().len(), 23);
        assert_eq!(repo.news.unwrap().len(), 15);
        assert_eq!(repo.version.unwrap(), 1);

        let repo = read_source("burritosource.json").unwrap();
        assert_eq!(repo.apps.unwrap().len(), 7);
        assert_eq!(repo.news.unwrap().len(), 3);
    }
}
