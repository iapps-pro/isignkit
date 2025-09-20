use super::{GlobalOptions, gbox_file::LinkMapReader};
use crate::CliCommand;
use crate::input_file::{InputFile, InputFileParser, InputFileReader, PlainReader};
use anyhow::{Result, anyhow};
use clap::Parser;
use ios_signers_types::gbox::{CryptKeys, LinksMap};
use std::path::Path;
use url::Url;

#[derive(Parser)]
#[clap(about, version)]
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
        let input = if self.links_map.is_remote() {
            let udid = self.global_options.udid.as_ref();
            let code = self.code.as_ref();
            let url = self.repo_url.as_ref();
            let (Some(udid), Some(code), Some(url)) = (udid, code, url) else {
                return Err(anyhow!(
                    "Links map is a remote URL but no udid, code or url is provided"
                ));
            };

            let reader = LinkMapReader::new(udid, code, Url::parse(url)?);
            reader.read_to_string(&self.links_map, true)?
        } else {
            PlainReader.read_to_string(&self.links_map, true)?
        };

        LinksMap::from_encrypted(input, keys)
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
