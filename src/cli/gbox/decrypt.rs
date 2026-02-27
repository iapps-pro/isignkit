use super::{GlobalOptions, gbox_file::RepoReader};
use crate::CliCommand;
use crate::gbox::gbox_file::LinkMapReader;
use crate::input_file::{InputFile, InputFileParser, InputFileReader, PlainReader};
use anyhow::{Context, Result, anyhow};
use clap::Parser;
use isignkit::gbox::{
    CryptKeys, EncryptedLinksMap, GboxRepo, Repository, SourceProcessor, encrypted,
    links_map::LinkMapResponse,
};
use std::path::Path;
use url::Url;

#[derive(Parser)]
#[clap(visible_alias = "d")]
/// Perform repo decryption
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

    /// Print raw contents of encrypted payload
    #[arg(long = "raw", default_value_t = false)]
    print_raw: bool,

    /// Local input file, stdin (-) or remote URL
    #[arg(value_parser = InputFileParser)]
    repo_json: InputFile,
}

impl DecryptCommand {
    fn read_repo(&self) -> Result<encrypted::Repository> {
        let reader = RepoReader::new(self.global_options.udid.as_deref());
        let repo = reader
            .read_json::<GboxRepo>(&self.repo_json, true)
            .with_context(|| format!("Can't read: {:?}", self.repo_json.as_str()))?;

        if let GboxRepo::Encrypted(repo) = repo {
            Ok(repo)
        } else {
            Err(anyhow!(
                "Your repository seems to be decrypted, expected to get encrypted"
            ))
        }
    }

    fn normalize_links(&self, repo: &mut Repository, keys: &CryptKeys) -> Result<()> {
        let links_map = self.links_map.clone().or_else(|| {
            if let Some(SourceProcessor::AppsUnlock(unlock)) = &repo.info.source_processor {
                Some(InputFile::Remote(unlock.auth_url.clone()))
            } else {
                None
            }
        });

        let Some(links_map) = links_map else {
            log::info!("Links map was not provided, skipping links normalization...");
            return Ok(());
        };

        let Some(unlock_hash) = &repo.info.items_unlock_hash else {
            log::info!("Repository does not contain unlock hash...");
            return Ok(());
        };

        let links_map = if links_map.is_remote() {
            let udid = self.global_options.udid.as_ref();
            let code = self.code.as_ref();
            let url = self.repo_url.as_deref().or_else(|| {
                if self.repo_json.is_remote() {
                    self.repo_json.as_str()
                } else {
                    None
                }
            });

            let (Some(udid), Some(code), Some(url)) = (udid, code, url) else {
                return Ok(());
            };

            let reader = LinkMapReader::new(udid, code, Url::parse(url)?);
            let response: LinkMapResponse = reader.read_json(&links_map, false)?;
            match response {
                LinkMapResponse::Success { map, .. } => map,
                LinkMapResponse::Error { error, .. } => return Err(anyhow!("{error}")),
            }
        } else {
            PlainReader.read_json::<EncryptedLinksMap>(&links_map, false)?
        };

        if links_map.mapping_hash.as_str() != unlock_hash {
            log::info!("Expected {unlock_hash}, got {}", links_map.mapping_hash);
            log::info!("Unlock hash by mapping and in repo doesn't match! Is mapping valid?");
            return Ok(());
        }

        let links_map = links_map.decrypt(keys)?;
        repo.normalize_links(&links_map);

        Ok(())
    }
}

impl CliCommand for DecryptCommand {
    fn run(&self) -> Result<()> {
        let keys = self.global_options.suitable_keys()?;

        let repo = self.read_repo()?;
        let result = if self.print_raw {
            serde_json::to_string_pretty(&repo.decrypt_payload(&keys)?)?
        } else {
            let mut repo = repo.decrypt(&keys)?;
            self.normalize_links(&mut repo, &keys)?;

            serde_json::to_string_pretty(&repo)?
        };

        let output = &self.global_options.output;
        if output == Path::new("-") {
            println!("{result}");
        } else {
            std::fs::write(output, result)?;
        }

        Ok(())
    }
}
