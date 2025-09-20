use super::{CryptKeys, GboxLink};
use crate::serde_support::chrono_string_seconds;
use anyhow::{Result, anyhow};
use base64::{Engine, prelude::BASE64_STANDARD};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct LinksMapRequest {
    #[serde(rename = "pwd")]
    pub access_code: String,
    pub udid: String,
    #[serde(deserialize_with = "chrono_string_seconds")]
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EncryptedLinksMap {
    #[serde(rename = "hash")]
    pub source_url_hash: String,

    #[serde(rename = "kvp")]
    pub links_map: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LinksMap(HashMap<String, GboxLink>);

impl LinksMap {
    #[must_use]
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn from_encrypted(str: impl AsRef<str>, keys: &CryptKeys) -> Result<Self> {
        let encrypted: EncryptedLinksMap = serde_json::from_str(str.as_ref())?;
        encrypted.decrypt(keys)
    }

    pub fn insert(&mut self, link: GboxLink) {
        if link.is_url() {
            let digest = link.to_digest().into_string();
            self.0.insert(digest, link);
        }
    }

    pub fn get(&self, digest: impl AsRef<str>) -> Option<&GboxLink> {
        self.0.get(digest.as_ref())
    }

    pub fn encrypt(
        &self,
        keys: &CryptKeys,
        source_url: impl AsRef<str>,
    ) -> Result<EncryptedLinksMap> {
        let payload = serde_json::to_vec(&self)?;

        let payload = rncryptor::v3::encrypt(&keys.links_map, &payload)
            .map_err(|error| anyhow!("{error:?}"))?;
        let payload = BASE64_STANDARD.encode(payload);

        Ok(EncryptedLinksMap {
            source_url_hash: hex::encode(*md5::compute(source_url.as_ref())),
            links_map: payload,
        })
    }
}

impl EncryptedLinksMap {
    pub fn decrypt(&self, keys: &CryptKeys) -> Result<LinksMap> {
        let payload = BASE64_STANDARD.decode(&self.links_map)?;
        let payload = rncryptor::v3::decrypt(&keys.links_map, &payload)
            .map_err(|error| anyhow!("{error:?}"))?;

        let map = serde_json::from_slice(&payload)?;

        Ok(map)
    }
}

impl Default for LinksMap {
    fn default() -> Self {
        Self::new()
    }
}

impl<I> From<I> for LinksMap
where
    I: Iterator<Item = GboxLink>,
{
    fn from(links: I) -> Self {
        let map = links
            .filter_map(|link| {
                if link.is_url() {
                    Some((link.to_digest().into_string(), link))
                } else {
                    None
                }
            })
            .collect();

        Self(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gbox::CryptKeysStorage;

    #[test]
    fn decrypt() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/gbox/kvp.json");
        let encrypted_map = std::fs::read_to_string(path).expect("Can't read file");

        let keys_storage = CryptKeysStorage::default();
        let keys = keys_storage.latest_keys();

        let links_map = LinksMap::from_encrypted(encrypted_map, keys).expect("decrypt failed");

        let link = links_map.get("4e3596273689baa1");
        let expected =
            "https://storage.iapps.rejail.ru/s/HfpSH3FrPajqDNE/download/HfpSH3FrPajqDNE.ipa";
        assert_eq!(link, Some(&GboxLink::new_url(expected).unwrap()));
    }
}
