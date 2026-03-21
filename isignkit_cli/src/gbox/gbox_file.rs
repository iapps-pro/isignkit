use crate::input_file::InputFileReader;
use chrono::Utc;
use isignkit::gbox::links_map::LinksMapRequest;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue, USER_AGENT},
};
use url::Url;

pub(crate) struct RepoReader<'a> {
    udid: Option<&'a str>,
    client: Client,
}

impl<'a> RepoReader<'a> {
    pub(crate) fn new(udid: Option<&'a str>) -> Self {
        Self {
            udid,
            client: Client::new(),
        }
    }
}

impl InputFileReader for RepoReader<'_> {
    fn read_remote(&self, url: &Url) -> anyhow::Result<String> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("GBox/576"));
        if let Some(udid) = self.udid {
            headers.insert("udid", HeaderValue::from_str(udid)?);
        }

        let response = self.client.get(url.as_str()).headers(headers).send()?;
        Ok(response.error_for_status()?.text()?)
    }
}

pub(crate) struct LinkMapReader<'a> {
    udid: &'a str,
    code: &'a str,
    repo_url: Url,
    client: Client,
}

impl<'a> LinkMapReader<'a> {
    pub(crate) fn new(udid: &'a str, code: &'a str, repo_url: Url) -> Self {
        Self {
            udid,
            code,
            repo_url,
            client: Client::new(),
        }
    }
}

impl InputFileReader for LinkMapReader<'_> {
    fn read_remote(&self, url: &Url) -> anyhow::Result<String> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("GBox/576"));
        headers.insert("udid", HeaderValue::from_str(self.udid)?);

        let body = LinksMapRequest {
            access_code: self.code.to_string(),
            udid: self.udid.to_string(),
            timestamp: Utc::now(),
        };

        let response = self
            .client
            .get(url.as_str())
            .body(body.encrypt(&self.repo_url)?)
            .headers(headers)
            .send()?;

        Ok(response.error_for_status()?.text()?)
    }
}
