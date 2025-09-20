use super::{GlobalOptions, gbox_file::RepoReader};
use crate::CliCommand;
use crate::gbox::gbox_file::LinkMapReader;
use crate::input_file::{InputFile, InputFileParser, InputFileReader, PlainReader};
use anyhow::Result;
use clap::Parser;
use ios_signers_types::gbox::{CryptKeys, LinksMap, Repository, encrypted};
use std::path::Path;
use url::Url;

#[derive(Parser)]
#[clap(about, version)]
pub(crate) struct DecryptCommand {
    #[clap(flatten)]
    global_options: GlobalOptions,

    /// Links map file or remote URL. Use it to normalize urls
    #[arg(long, short = 'm', value_parser = InputFileParser)]
    links_map: Option<InputFile>,

    /// Access code which will be used to make remote KVP requests
    #[arg(long, short)]
    code: Option<String>,

    /// Main URL of the repository. Will be used to make KVP request
    #[arg(long, short)]
    repo_url: Option<String>,

    /// Local input file, stdin (-) or remote URL
    #[arg(value_parser = InputFileParser)]
    repo_json: InputFile,
}

impl DecryptCommand {
    fn read_repo(&self) -> Result<encrypted::Repository> {
        let reader = RepoReader::new(self.global_options.udid.as_deref());
        let input = reader.read_to_string(&self.repo_json, true)?;
        let repo = serde_json::from_str(&input)?;

        Ok(repo)
    }

    fn normalize_links(&self, repo: &mut Repository, keys: &CryptKeys) -> Result<()> {
        let Some(links_map) = &self.links_map else {
            log::warn!("Links map was not provided, skipping links normalization...");
            return Ok(());
        };

        let links_map = if links_map.is_remote() {
            let udid = self.global_options.udid.as_ref();
            let code = self.code.as_ref();
            let url = self.repo_url.as_ref();
            let (Some(udid), Some(code), Some(url)) = (udid, code, url) else {
                return Ok(());
            };

            let reader = LinkMapReader::new(udid, code, Url::parse(url)?);
            reader.read_to_string(links_map, false)?
        } else {
            PlainReader.read_to_string(links_map, false)?
        };

        let links_map = LinksMap::from_encrypted(&links_map, keys)?;
        repo.normalize_links(&links_map);

        Ok(())
    }
}

impl CliCommand for DecryptCommand {
    fn run(&self) -> Result<()> {
        let keys = self.global_options.suitable_keys()?;

        let repo = self.read_repo()?;
        let mut repo = repo.decrypt(&keys)?;
        self.normalize_links(&mut repo, &keys)?;

        let result = serde_json::to_string_pretty(&repo)?;

        let output = &self.global_options.output;
        if output == Path::new("-") {
            println!("{result}");
        } else {
            std::fs::write(output, result)?;
        }

        Ok(())
    }
}
