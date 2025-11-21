mod application;
mod color;
mod news;

pub use self::{
    application::{
        Application, Category as AppCategory, Patreon as AppPatreon, Permissions as AppPermissions,
        Screenshot as AppScreenshot, Version as AppVersion,
    },
    color::AltStoreColor,
    news::NewsItem,
};
use serde::{Deserialize, Serialize};
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
    pub apps: Vec<Application>,

    /// A list of the News items in the source. The ordering does not matter because
    /// `AltStore` will display them in reverse chronological order according to their date.
    pub news: Vec<NewsItem>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/test/altstore/apps.json");
        let json = std::fs::read_to_string(path).expect("Can't read file");

        let repo: Repository = serde_json::from_str(&json).unwrap();
        assert_eq!(repo.featured_apps.unwrap().len(), 1);
        assert_eq!(repo.apps.len(), 2);
    }
}
