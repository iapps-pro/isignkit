use super::{GlobalOptions, gbox_file::RepoReader};
use crate::CliCommand;
use crate::input_file::{InputFile, InputFileParser, InputFileReader};
use anyhow::{Context, Result, anyhow};
use clap::Parser;
use isignkit::gbox::{GboxRepo, Repository};
use std::path::Path;

#[derive(Parser)]
#[clap(visible_alias = "cm")]
/// Create new links map
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
        let repo = reader
            .read_json::<GboxRepo>(&self.repo_json, true)
            .with_context(|| format!("Can't read: {:?}", self.repo_json.as_str()))?;

        if let GboxRepo::Plain(repo) = repo {
            Ok(repo)
        } else {
            Err(anyhow!(
                "Your repository seems to be encrypted, expected to get decrypted"
            ))
        }
    }
}

impl CliCommand for CreateMapCommand {
    fn run(&self) -> Result<()> {
        let keys = self.global_options.suitable_keys()?;

        let repo: Repository = self.read_repo()?;
        let map = repo.create_links_map();
        let map = map.encrypt(&keys)?;

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
