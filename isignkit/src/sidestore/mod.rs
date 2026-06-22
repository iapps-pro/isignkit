mod application;
mod news;

pub use self::{application::*, news::NewsItem};
use super::{altstore, gbox};
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

impl From<gbox::GboxRepo> for Repository {
    fn from(gbox: gbox::GboxRepo) -> Self {
        let general = gbox.general_info();
        let mut repo = Self {
            name: general.name.clone(),
            version: None,
            subtitle: None,
            website: Some(general.link_url.clone()),
            description: None,
            icon_url: Url::parse(&general.icon_image_url).ok(),
            header_image_url: None,
            patreon_url: None,
            tint_color: None,
            user_info: None,
            apps: None,
            news: None,
            featured_apps: None,
            group_id: None,
        };

        if let gbox::GboxRepo::Plain(gbox) = gbox {
            let apps = gbox
                .applications
                .into_iter()
                .flat_map(TryInto::try_into)
                .collect();

            repo.apps = Some(apps);
        }

        repo
    }
}

impl From<altstore::Repository> for Repository {
    fn from(altstore: altstore::Repository) -> Self {
        Self {
            name: altstore.name,
            version: None,
            subtitle: altstore.subtitle,
            description: altstore.description,
            icon_url: altstore.icon_url,
            website: altstore.website,
            patreon_url: None,
            tint_color: altstore.tint_color,
            featured_apps: altstore.featured_apps,
            apps: altstore
                .apps
                .map(|apps| apps.into_iter().flat_map(TryInto::try_into).collect()),
            news: altstore
                .news
                .map(|apps| apps.into_iter().flat_map(TryInto::try_into).collect()),
            user_info: None,
            header_image_url: None,
            group_id: None,
        }
    }
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
