mod application;
mod color;
mod news;
mod permissions;

pub use self::{application::*, color::AltStoreColor, news::NewsItem, permissions::*};
use super::{gbox, sidestore};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
    /// The name of the source as it will appear in `AltStore`.
    pub name: String,

    /// A short, one-sentence description of source.
    /// This will appear underneath the source's name on its About page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,

    /// A full-length description of the source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A link to an image that will be used to visually identify the source.
    #[serde(rename = "iconURL")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<Url>,

    /// A link to an image that will be displayed as the header of the source's About page
    #[serde(rename = "headerURL")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_url: Option<Url>,

    /// A link to the primary website for the source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<Url>,

    /// Your preferred username for your source's account on <https://explore.alt.store>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fedi_username: Option<String>,

    /// A link to the Patreon campaign.
    #[serde(rename = "patreonURL")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patreon_url: Option<Url>,

    /// A color that will be used to theme the source's About page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tint_color: Option<AltStoreColor>,

    /// An ordered list of app bundleIdentifier's you want featured on the source's About page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub featured_apps: Option<Vec<String>>,

    /// An ordered list of the apps in your source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apps: Option<Vec<Application>>,

    /// A list of the News items in the source. The ordering does not matter because
    /// `AltStore` will display them in reverse chronological order according to their date.v
    #[serde(skip_serializing_if = "Option::is_none")]
    pub news: Option<Vec<NewsItem>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_info: Option<HashMap<String, String>>,
}

impl From<gbox::GboxRepo> for Repository {
    fn from(gbox: gbox::GboxRepo) -> Self {
        let info = gbox.general_info();

        let mut repo = Self {
            name: info.name.clone(),
            subtitle: None,
            description: None,
            icon_url: info.icon_image_url.parse().ok(),
            header_url: None,
            website: Some(info.link_url.clone()),
            fedi_username: None,
            patreon_url: None,
            tint_color: None,
            featured_apps: None,
            apps: None,
            news: None,
            user_info: None,
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

impl From<sidestore::Repository> for Repository {
    fn from(sidestore: sidestore::Repository) -> Self {
        Self {
            name: sidestore.name,
            subtitle: sidestore.subtitle,
            description: sidestore.description,
            icon_url: sidestore.icon_url,
            header_url: sidestore.header_image_url,
            website: sidestore.website,
            fedi_username: None,
            patreon_url: None,
            tint_color: sidestore.tint_color,
            featured_apps: sidestore.featured_apps,
            apps: sidestore
                .apps
                .map(|apps| apps.into_iter().flat_map(TryInto::try_into).collect()),
            news: sidestore
                .news
                .map(|apps| apps.into_iter().flat_map(TryInto::try_into).collect()),
            user_info: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Result;

    fn read_source(name: &str) -> Result<Repository> {
        let path = format!("{}/test/altstore/{}", env!("CARGO_MANIFEST_DIR"), name);
        let json = std::fs::read_to_string(path).expect("Can't read file");
        serde_json::from_str(&json)
    }

    #[test]
    fn parse() {
        let repo = read_source("apps.json").unwrap();
        assert_eq!(repo.featured_apps.unwrap().len(), 1);
        assert_eq!(repo.apps.unwrap().len(), 2);

        let repo = read_source("dvntm.json").unwrap();
        assert_eq!(repo.featured_apps.unwrap().len(), 3);
        assert_eq!(repo.apps.unwrap().len(), 6);

        let repo = read_source("flycast.json").unwrap();
        assert_eq!(repo.apps.unwrap().len(), 1);
        assert!(repo.news.is_none());
    }
}
