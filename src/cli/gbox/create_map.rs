use super::{GlobalOptions, gbox_file::RepoReader};
use crate::CliCommand;
use crate::input_file::{InputFile, InputFileParser, InputFileReader};
use anyhow::Result;
use clap::Parser;
use ios_signers_types::gbox::Repository;
use std::path::Path;

#[derive(Parser)]
#[clap(about, version)]
pub(crate) struct CreateMapCommand {
    #[clap(flatten)]
    global_options: GlobalOptions,

    /// Local input file, stdin (-) or remote URL
    #[arg(value_parser = InputFileParser)]
    repo_json: InputFile,
}

impl CreateMapCommand {
    fn read_repo(&self) -> Result<Repository> {
        let reader = RepoReader::new(self.global_options.udid.as_deref());
        let input = reader.read_to_string(&self.repo_json, true)?;
        let repo = serde_json::from_str(&input)?;

        Ok(repo)
    }
}

impl CliCommand for CreateMapCommand {
    fn run(&self) -> Result<()> {
        let keys = self.global_options.suitable_keys()?;

        let repo: Repository = self.read_repo()?;
        let map = repo.create_links_map();
        let map = map.encrypt(&keys)?;
        log::info!("sourceHash is `{}`", map.mapping_hash);
        log::info!("Please edit input file manually.");

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
