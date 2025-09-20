use super::input_file::{InputFile, InputFileParser};
use crate::CliCommand;
use anyhow::{Context, Result, anyhow};
use clap::{Parser, ValueEnum};
use ios_signers_types::gbox::{CryptKeysStorage, LinksMap, Repository, encrypted};
use std::path::{Path, PathBuf};

#[derive(ValueEnum, Clone)]
enum Command {
    Encrypt,
    Decrypt,
    CreateMap,
    PrintMap,
}

#[derive(Parser)]
#[clap(about, version)]
pub(crate) struct GBoxCommand {
    /// GBox target version.
    ///
    /// This will be used for getting compatible crypt keys
    #[arg(long, short, default_value = "5.6.6")]
    gbox_version: String,

    /// Output file. Can be stdout (-) or local file
    #[arg(long, short, default_value = "-")]
    output: PathBuf,

    /// Links map file or remote URL. Use it to normalize urls
    #[arg(long, short = 'm', value_parser = InputFileParser)]
    links_map: Option<InputFile>,

    /// URL to the repo. Required if creating links map
    #[arg(long, short = 'r')]
    repo_url: Option<String>,

    #[arg(value_enum)]
    command: Command,

    /// Local input file, stdin (-) or remote URL
    #[arg(value_parser = InputFileParser)]
    repo_json: InputFile,
}

impl CliCommand for GBoxCommand {
    fn run(&self) -> Result<()> {
        let input = self.repo_json.read_to_string(true)?;

        let keys_storage = CryptKeysStorage::default();
        let keys = keys_storage
            .keys_for_version(&self.gbox_version)
            .with_context(|| format!("No keys are available for {}", self.gbox_version))?;

        let result = match self.command {
            Command::Encrypt => {
                let mut repo: Repository = serde_json::from_str(&input)?;
                drop(input);

                repo.obfuscate_links();

                let repo = repo.encrypt(keys)?;
                serde_json::to_string_pretty(&repo)?
            }
            Command::Decrypt => {
                let repo: encrypted::Repository = serde_json::from_str(&input)?;
                drop(input);

                let mut repo = repo.decrypt(keys)?;

                if let Some(links_map) = &self.links_map {
                    let links_map = links_map.read_to_string(false)?;
                    let links_map = LinksMap::from_encrypted(&links_map, keys)?;
                    repo.normalize_links(&links_map);
                }

                serde_json::to_string_pretty(&repo)?
            }
            Command::CreateMap => {
                let repo: Repository = serde_json::from_str(&input)?;
                drop(input);

                let Some(repo_url) = &self.repo_url else {
                    return Err(anyhow!("Repo URL was not provided!"));
                };

                let map = repo.create_links_map();
                let map = map.encrypt(&keys, repo_url)?;

                serde_json::to_string_pretty(&map)?
            }
            Command::PrintMap => {
                let map = LinksMap::from_encrypted(&input, keys)?;
                drop(input);

                serde_json::to_string_pretty(&map)?
            }
        };

        if self.output != Path::new("-") {
            std::fs::write(&self.output, result)?;
        } else {
            println!("{result}");
        }

        Ok(())
    }
}
