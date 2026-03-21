use super::{GlobalOptions, gbox_file::LinkMapReader};
use crate::CliCommand;
use crate::input_file::{InputFile, InputFileParser, InputFileReader, PlainReader};
use anyhow::{Result, anyhow};
use clap::Parser;
use isignkit::gbox::{CryptKeys, EncryptedLinksMap, LinksMap, links_map::LinkMapResponse};
use std::path::Path;
use url::Url;

#[derive(Parser)]
#[clap(visible_alias = "pm")]
/// Print links map contents
pub(crate) struct PrintMapCommand {
    #[clap(flatten)]
    global_options: GlobalOptions,

    /// Access code which will be used to make remote KVP requests
    #[arg(long, short)]
    code: Option<String>,

    /// Main URL of the repository. Will be used to make KVP request
    #[arg(long, short)]
    repo_url: Option<String>,

    /// Local input file, stdin (-) or remote URL
    #[arg(value_parser = InputFileParser)]
    links_map: InputFile,
}

impl PrintMapCommand {
    fn read_map(&self, keys: &CryptKeys) -> Result<LinksMap> {
        let links_map = if self.links_map.is_remote() {
            let udid = self.global_options.udid.as_ref();
            let code = self.code.as_ref();
            let url = self.repo_url.as_ref();
            let (Some(udid), Some(code), Some(url)) = (udid, code, url) else {
                return Err(anyhow!(
                    "Links map is a remote URL but no udid, code or url is provided"
                ));
            };

            let reader = LinkMapReader::new(udid, code, Url::parse(url)?);
            let response = reader.read_json(&self.links_map, true)?;
            match response {
                LinkMapResponse::Success { map, .. } => map,
                LinkMapResponse::Error { error, .. } => return Err(anyhow!("{error}")),
            }
        } else {
            PlainReader.read_json::<EncryptedLinksMap>(&self.links_map, true)?
        };

        let decrypted = links_map.decrypt(keys)?;

        Ok(decrypted)
    }
}

impl CliCommand for PrintMapCommand {
    fn run(&self) -> Result<()> {
        let keys = self.global_options.suitable_keys()?;
        let map = self.read_map(&keys)?;
        let result = serde_json::to_string_pretty(&map)?;

        let output = &self.global_options.output;
        if output == Path::new("-") {
            println!("{result}");
        } else {
            std::fs::write(output, result)?;
        }

        Ok(())
    }
}
