pub mod config;
mod crypt_keys;
pub mod encrypted;
mod gbox_link;
pub mod links_map;
mod schema;
mod unencrypted;

pub use self::{
    config::EncryptedConfig,
    crypt_keys::{CryptKeys, CryptKeysStorage},
    gbox_link::{GboxLink, OptionalGboxLink},
    links_map::{EncryptedLinksMap, LinksMap},
    schema::SchemaVersion,
    unencrypted::*,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
#[serde(untagged)]
pub enum GboxRepo {
    Plain(Repository),
    Encrypted(encrypted::Repository),
}
