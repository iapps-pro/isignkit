use super::{CryptKeys, GboxLink};
use crate::{serde_support::chrono_string_seconds, unit_false::False, unit_true::True};
use anyhow::{Result, anyhow};
use base64::{Engine, prelude::BASE64_STANDARD};
use chrono::{DateTime, Utc};
use rncryptor::v3::{decrypt as rndecrypt, encrypt as rnencrypt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LinksMapRequest {
    #[serde(rename = "pwd")]
    pub access_code: String,
    pub udid: String,
    #[serde(with = "chrono_string_seconds")]
    pub timestamp: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum LinkMapResponse {
    Success {
        /// A dummy field. Used only for deserialization.
        #[allow(dead_code)]
        success: True,

        #[serde(rename = "data")]
        map: EncryptedLinksMap,
    },
    Error {
        /// A dummy field. Used only for deserialization.
        #[allow(dead_code)]
        success: False,

        /// Error message
        #[serde(rename = "message")]
        error: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EncryptedLinksMap {
    #[serde(rename = "hash")]
    pub mapping_hash: String,

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

    pub fn encrypt(&self, keys: &CryptKeys) -> Result<EncryptedLinksMap> {
        let mapping = serde_json::to_vec(&self)?;
        let mapping_hash = hex::encode(*md5::compute(&mapping));

        let payload = rnencrypt(&keys.links_map, &mapping).map_err(|error| anyhow!("{error:?}"))?;
        let links_map = BASE64_STANDARD.encode(payload);

        Ok(EncryptedLinksMap {
            mapping_hash,
            links_map,
        })
    }
}

impl EncryptedLinksMap {
    pub fn decrypt(&self, keys: &CryptKeys) -> Result<LinksMap> {
        let payload = BASE64_STANDARD.decode(&self.links_map)?;
        let payload = rndecrypt(&keys.links_map, &payload).map_err(|error| anyhow!("{error:?}"))?;

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

impl LinksMapRequest {
    pub fn encrypt(&self, repo_url: &Url) -> Result<String> {
        let password = hex::encode(*md5::compute(repo_url.as_str()));

        let request = serde_json::to_vec(&self)?;
        let request = rnencrypt(&password, &request).map_err(|error| anyhow!("{error:?}"))?;

        Ok(BASE64_STANDARD.encode(request))
    }

    pub fn decrypt(encrypted: impl AsRef<str>, repo_url: &Url) -> Result<Self> {
        let password = hex::encode(*md5::compute(repo_url.as_str()));

        let encrypted = BASE64_STANDARD.decode(encrypted.as_ref())?;
        let decrypted = rndecrypt(&password, &encrypted).map_err(|error| anyhow!("{error:?}"))?;

        let request = serde_json::from_slice(&decrypted)?;
        Ok(request)
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

    #[test]
    fn invalid_response() {
        let json = r#"
        {
	"success": false,
	"message": "\u041d :D"
}
        "#;

        let response: LinkMapResponse = serde_json::from_str(json).unwrap();
        assert!(matches!(response, LinkMapResponse::Error { .. }));
    }

    #[test]
    fn valid_response() {
        let json = r#"
        {
	"success": true,
	"data": {
		"hash": "328a65a13de4a99f8218ec777587296d",
		"kvp": "really_long_kvp_contents"
	}
}
        "#;

        let response: LinkMapResponse = serde_json::from_str(json).unwrap();
        let LinkMapResponse::Success { map: data, .. } = response else {
            panic!("Got invalid response {response:?}");
        };
        assert_eq!(data.mapping_hash, "328a65a13de4a99f8218ec777587296d");
        assert_eq!(data.links_map, "really_long_kvp_contents");
    }
}
