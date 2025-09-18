use super::GboxLink;
use crate::serde_support::chrono_string_seconds;
use anyhow::{Result, anyhow};
use base64::{Engine, prelude::BASE64_STANDARD};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct KVPRequest {
    #[serde(rename = "pwd")]
    pub access_code: String,
    pub udid: String,
    #[serde(deserialize_with = "chrono_string_seconds")]
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Kvp {
    pub hash: String,
    #[serde(rename = "kvp")]
    pub payload: String,
}

impl Kvp {
    pub fn new<L>(apps_links: L, password: &str) -> Result<Self>
    where
        L: Iterator<Item = GboxLink>,
    {
        let map: HashMap<_, _> = apps_links
            .map(|link| {
                let url = link.to_string();
                (link.into_digest(), url)
            })
            .collect();
        let payload = serde_json::to_string(&map)?;
        let hash = hex::encode(*md5::compute(&payload));

        let payload = rncryptor::v3::encrypt(password, payload.as_bytes())
            .map_err(|error| anyhow!("{error:?}"))?;
        let payload = BASE64_STANDARD.encode(payload);

        Ok(Self { hash, payload })
    }
}

// #[async_trait]
// impl<'r> FromData<'r> for KVPRequest {
//     type Error = ();
//
//     async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
//         use rocket::request::Outcome as ROutcome;
//         let ROutcome::Success(config) = req.guard::<&State<Arc<Config>>>().await else {
//             return Outcome::Error((Status::InternalServerError, ()));
//         };
//
//         let repo_url = req.uri().parent();
//         let Ok(repo_url) = config.base_uri.join(&repo_url.to_string()) else {
//             return Outcome::Error((Status::InternalServerError, ()));
//         };
//
//         let data_pass = md5::compute(repo_url.as_str());
//         let data_pass = hex::encode(*data_pass);
//
//         let Ok(data) = data.open(1.mebibytes()).into_bytes().await else {
//             return Outcome::Error((Status::PayloadTooLarge, ()));
//         };
//
//         let Ok(data) = BASE64_STANDARD.decode(data.into_inner()) else {
//             return Outcome::Error((Status::UnprocessableEntity, ()));
//         };
//
//         let Ok(data) = rncryptor::v3::decrypt(&data_pass, &data) else {
//             return Outcome::Error((Status::UnprocessableEntity, ()));
//         };
//
//         match serde_json::from_slice::<Self>(&data) {
//             Ok(request) => Outcome::Success(request),
//             Err(error) => {
//                 log::warn!("Decode kvp request error: {error:?}");
//                 Outcome::Error((Status::UnprocessableEntity, ()))
//             }
//         }
//     }
// }
