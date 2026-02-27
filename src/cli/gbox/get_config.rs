use super::GlobalOptions;
use crate::CliCommand;
use anyhow::{Result, anyhow};
use clap::Parser;
use isignkit::gbox::config::{GBoxAppType, RemoteConfigRequest, RemoteConfigResponse};
use reqwest::blocking::Client;
use serde_json::{Map, Value};
use std::path::Path;

#[derive(Parser)]
#[clap(visible_alias = "conf")]
/// Fetches remote app config
pub(crate) struct GetConfigCommand {
    #[clap(flatten)]
    global_options: GlobalOptions,
}

impl GetConfigCommand {
    fn get_config(&self) -> Result<Map<String, Value>> {
        let request = RemoteConfigRequest {
            app_type: GBoxAppType::Ios,
            version: self.global_options.gbox_version.as_str().into(),
        };

        let client = Client::new();
        let request = client
            .get("https://api.gbox.run/util/appConfig")
            .query(&request);

        let response = request
            .send()?
            .error_for_status()?
            .json::<RemoteConfigResponse>()?;

        let RemoteConfigResponse::Success { data } = response else {
            return Err(anyhow!("Got invalid response: {:#?}", response));
        };

        let keys = self.global_options.suitable_keys()?;
        data.decrypt(&keys)
    }
}

impl CliCommand for GetConfigCommand {
    fn run(&self) -> Result<()> {
        let config = self.get_config()?;
        let result = serde_json::to_string_pretty(&config)?;

        let output = &self.global_options.output;
        if output == Path::new("-") {
            println!("{result}");
        } else {
            std::fs::write(output, result)?;
        }

        Ok(())
    }
}
