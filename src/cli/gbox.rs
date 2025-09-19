use super::input_file::{InputFile, InputFileParser};
use crate::CliCommand;
use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use ios_signers_types::gbox::{CryptKeysStorage, Repository, encrypted};
use std::path::{Path, PathBuf};

#[derive(ValueEnum, Clone)]
enum Command {
    Encrypt,
    Decrypt,
}

#[derive(Parser)]
#[clap(about, version)]
pub(crate) struct GBoxCommand {
    #[arg(value_enum)]
    command: Command,

    /// GBox target version.
    ///
    /// This will be used for getting compatible crypt keys
    #[arg(long, short = 'g', default_value = "5.6.6")]
    gbox_version: String,

    /// Output file. Can be stdout (-) or local file
    #[arg(long, short, default_value = "-")]
    output: PathBuf,

    /// Local input file, stdin (-) or remote URL
    #[arg(value_parser = InputFileParser)]
    repo_json: InputFile,
}

impl CliCommand for GBoxCommand {
    fn run(&self) -> Result<()> {
        let input = self.repo_json.read_to_string()?;

        let keys_storage = CryptKeysStorage::default();
        let keys = keys_storage
            .keys_for_version(&self.gbox_version)
            .with_context(|| format!("No keys are available for {}", self.gbox_version))?;

        let crypt_result = match self.command {
            Command::Encrypt => {
                let repo: Repository = serde_json::from_str(&input)?;
                drop(input);

                let repo = repo.encrypt(keys)?;
                serde_json::to_string_pretty(&repo)?
            }
            Command::Decrypt => {
                let repo: encrypted::Repository = serde_json::from_str(&input)?;
                drop(input);

                let repo = repo.decrypt(keys)?;
                serde_json::to_string_pretty(&repo)?
            }
        };

        if self.output != Path::new("-") {
            std::fs::write(&self.output, crypt_result)?;
        } else {
            println!("{crypt_result}");
        }

        Ok(())
    }
}
