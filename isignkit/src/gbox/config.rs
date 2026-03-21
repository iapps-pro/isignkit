#![allow(unused)]

use super::CryptKeys;
use crate::error::GboxError;
use base64::{Engine as _, prelude::BASE64_STANDARD};
#[cfg(feature = "openssl")]
use openssl::{
    error::ErrorStack,
    pkey::Public,
    rsa::{Padding, Rsa},
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::borrow::Cow;

#[derive(Serialize, Debug, Clone, Eq, PartialEq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum GBoxAppType {
    Ios,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct RemoteConfigRequest<'a> {
    #[serde(rename = "type")]
    pub app_type: GBoxAppType,

    #[serde(rename = "versionCode")]
    pub version: Cow<'a, str>,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
#[serde(untagged)]
pub enum RemoteConfigResponse {
    Success { data: EncryptedConfig },
    Error { message: String },
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct EncryptedConfig {
    app_config: String,
}

impl EncryptedConfig {
    /// Decrypts encrypted config data
    ///
    /// `OpenSSL` is required because data is encrypted by RSA private key
    /// and is being decrypted by public one. `rsa` crate doesn't implement this feature
    /// so the only way is to use `OpenSSL`
    #[cfg(feature = "openssl")]
    pub fn decrypt(&self, keys: &CryptKeys) -> Result<Map<String, Value>, GboxError> {
        let decoded = BASE64_STANDARD.decode(&self.app_config)?;

        let pubkey = BASE64_STANDARD.decode(&keys.config_pubkey)?;
        let rsa = Rsa::<Public>::public_key_from_der(&pubkey)?;

        let chunk_size = rsa.size() as usize;
        let mut buf = vec![0; chunk_size];
        let decrypted = decoded
            .chunks(chunk_size)
            .try_fold(Vec::new(), |mut result, chunk| {
                let len = rsa.public_decrypt(chunk, &mut buf, Padding::PKCS1)?;
                result.extend(&buf[..len]);

                Ok::<_, ErrorStack>(result)
            })?;

        Ok(serde_json::from_slice(&decrypted)?)
    }
}
