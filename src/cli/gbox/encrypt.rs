use super::{GlobalOptions, gbox_file::RepoReader};
use crate::CliCommand;
use crate::input_file::{InputFile, InputFileParser, InputFileReader};
use anyhow::{Context, Result, anyhow};
use clap::Parser;
use ios_signers_types::gbox::{GboxRepo, Repository};
use std::path::Path;

#[derive(Parser)]
#[clap(visible_alias = "e")]
/// Perform repo encryption
pub(crate) struct EncryptCommand {
    #[clap(flatten)]
    global_options: GlobalOptions,

    /// Local input file, stdin (-) or remote URL
    #[arg(value_parser = InputFileParser)]
    repo_json: InputFile,
}

impl EncryptCommand {
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

impl CliCommand for EncryptCommand {
    fn run(&self) -> Result<()> {
        let keys = self.global_options.suitable_keys()?;

        let mut repo: Repository = self.read_repo()?;
        repo.obfuscate_links();

        let repo = repo.encrypt(&keys)?;
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
