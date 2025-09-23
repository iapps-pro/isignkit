use anyhow::Result;
use serde::{Deserialize, Deserializer, Serialize, de::Error};
use std::fmt::{Display, Formatter};
use std::{ops::Deref, str::FromStr};
use url::Url;

#[derive(Serialize, Debug, Clone, Eq, PartialEq, Hash)]
#[serde(untagged)]
pub enum GboxLink {
    Url(Url),
    Digest(String),
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq, Hash)]
#[serde(transparent)]
pub struct OptionalGboxLink(
    #[serde(
        deserialize_with = "deserialize_option",
        skip_serializing_if = "Option::is_none"
    )]
    Option<GboxLink>,
);

impl GboxLink {
    pub fn new_url(url: impl AsRef<str>) -> Result<Self> {
        Ok(Self::Url(Url::parse(url.as_ref())?))
    }

    pub fn new_digest(id: impl AsRef<str>) -> Self {
        let digest = md5::compute(id.as_ref());
        let digest = hex::encode(&digest[4..12]);

        Self::Digest(digest)
    }

    #[must_use]
    pub fn as_url(&self) -> Option<&Url> {
        match self {
            Self::Url(url) => Some(url),
            Self::Digest(_) => None,
        }
    }

    #[must_use]
    pub fn into_digest(self) -> Self {
        match self {
            Self::Url(url) => Self::new_digest(url.as_str()),
            Self::Digest(digest) => Self::Digest(digest),
        }
    }

    #[must_use]
    pub fn to_digest(&self) -> Self {
        match self {
            Self::Url(url) => Self::new_digest(url.as_str()),
            Self::Digest(digest) => Self::Digest(digest.clone()),
        }
    }

    #[must_use]
    pub fn into_string(self) -> String {
        match self {
            Self::Url(url) => url.into(),
            Self::Digest(digest) => digest,
        }
    }

    #[must_use]
    pub fn is_digest(&self) -> bool {
        matches!(self, Self::Digest(_))
    }

    #[must_use]
    pub fn is_url(&self) -> bool {
        matches!(self, Self::Url(_))
    }
}

impl OptionalGboxLink {
    pub fn new_url(url: impl AsRef<str>) -> Result<Self> {
        Ok(Self(Some(GboxLink::new_url(url)?)))
    }

    pub fn new_id(url: impl AsRef<str>) -> Self {
        Self(Some(GboxLink::new_digest(url)))
    }
}

impl Display for GboxLink {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Url(url) => write!(f, "{}", url.as_str()),
            Self::Digest(digest) => write!(f, "{digest}"),
        }
    }
}

impl AsRef<str> for GboxLink {
    fn as_ref(&self) -> &str {
        match self {
            Self::Url(url) => url.as_str(),
            Self::Digest(id) => id,
        }
    }
}

impl Deref for GboxLink {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Url(url) => url.as_str(),
            Self::Digest(id) => id,
        }
    }
}

impl<'de> Deserialize<'de> for GboxLink {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let identifier = String::deserialize(deserializer)?;

        match Url::from_str(&identifier) {
            Ok(url) => Ok(Self::Url(url)),
            Err(_) if identifier.len() == 16 => Ok(Self::Digest(identifier)),
            _ => Err(Error::custom(format!(
                "Invalid value {identifier:?}, expected URL or 16-char hex id"
            ))),
        }
    }
}
impl AsRef<Option<GboxLink>> for OptionalGboxLink {
    fn as_ref(&self) -> &Option<GboxLink> {
        &self.0
    }
}

impl Deref for OptionalGboxLink {
    type Target = Option<GboxLink>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(clippy::unnecessary_wraps, reason = "Serde API")]
fn deserialize_option<'de, D>(deserializer: D) -> Result<Option<GboxLink>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(GboxLink::deserialize(deserializer).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_link() {
        let json = r#"[
        "https://example.com",
        "9426e3601c4821ad"
        ]"#;

        let links: Vec<Option<GboxLink>> = serde_json::from_str(json).unwrap();
        assert!(matches!(links[0], Some(GboxLink::Url(_))));
        assert!(matches!(links[1], Some(GboxLink::Digest(_))));
    }

    #[test]
    fn invalid_link() {
        let json = r#"[
        "htt\\example.com",
        "",
        "gggg"
        ]"#;

        let links: Vec<OptionalGboxLink> = serde_json::from_str(json).unwrap();
        assert!(links[0].0.is_none());
        assert!(links[1].0.is_none());
        assert!(links[2].0.is_none());
    }
}
