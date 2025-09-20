use crate::input_file::InputFileReader;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue, USER_AGENT},
};
use url::Url;

pub(crate) struct GBoxFileReader<'a> {
    udid: Option<&'a str>,
    client: Client,
}

impl<'a> GBoxFileReader<'a> {
    pub(crate) fn new(udid: Option<&'a str>) -> Self {
        Self {
            udid,
            client: Client::new(),
        }
    }
}

impl InputFileReader for GBoxFileReader<'_> {
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
