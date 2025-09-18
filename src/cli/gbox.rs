use crate::CliCommand;
use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use ios_signers_types::gbox::{CryptKeysStorage, Repository, encrypted};
use std::{
    fs::File,
    io::{read_to_string, stdin},
    path::{Path, PathBuf},
};

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

    #[arg(long, short = 'g', default_value = "5.6.6")]
    gbox_version: String,

    #[arg(long, short)]
    output: Option<PathBuf>,

    file: PathBuf,
}

impl CliCommand for GBoxCommand {
    fn run(&self) -> Result<()> {
        let input = if self.file == Path::new("-") {
            read_to_string(stdin())?
        } else {
            read_to_string(File::open(&self.file)?)?
        };

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

        match &self.output {
            Some(path) if path != Path::new("-") => {
                std::fs::write(path, crypt_result)?;
            }
            _ => {
                println!("{crypt_result}");
            }
        }

        Ok(())
    }
}
