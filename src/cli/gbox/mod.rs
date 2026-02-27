mod create_map;
mod decrypt;
mod encrypt;
mod gbox_file;
mod get_config;
mod print_map;
mod schema;
mod validate;

use self::{
    create_map::CreateMapCommand, decrypt::DecryptCommand, encrypt::EncryptCommand,
    get_config::GetConfigCommand, print_map::PrintMapCommand, schema::SchemaCommand,
    validate::ValidateCommand,
};
use crate::CliCommand;
use anyhow::{Context, Result};
use isignkit::gbox::{CryptKeys, CryptKeysStorage};
use std::path::PathBuf;

// TODO: Refacror when https://github.com/clap-rs/clap/issues/5525 will be closed
#[derive(clap::Args, Clone)]
pub(crate) struct GlobalOptions {
    /// Output file. Can be stdout (-) or local file
    #[arg(long, short, default_value = "-")]
    #[arg(help_heading = "GLOBAL OPTIONS", global = true)]
    output: PathBuf,

    /// GBox target version.
    ///
    /// This will be used for getting compatible crypt keys
    #[arg(long, short, default_value = "5.6.6")]
    #[arg(help_heading = "GLOBAL OPTIONS", global = true)]
    gbox_version: String,

    /// UDID which will be used to make remote requests
    #[arg(long, short = 'U')]
    #[arg(help_heading = "GLOBAL OPTIONS", global = true)]
    udid: Option<String>,
}

#[derive(clap::Subcommand)]
#[clap(visible_alias = "g")]
/// GBox related utils
pub(crate) enum GBoxCommand {
    Encrypt(EncryptCommand),
    Decrypt(DecryptCommand),
    PrintMap(PrintMapCommand),
    CreateMap(CreateMapCommand),
    GetConfig(GetConfigCommand),
    Validate(ValidateCommand),
    Schema(SchemaCommand),
}

impl CliCommand for GBoxCommand {
    fn run(&self) -> Result<()> {
        match self {
            Self::Encrypt(command) => command.run(),
            Self::Decrypt(command) => command.run(),
            Self::PrintMap(command) => command.run(),
            Self::CreateMap(command) => command.run(),
            Self::GetConfig(command) => command.run(),
            Self::Validate(command) => command.run(),
            Self::Schema(command) => command.run(),
        }
    }
}

impl GlobalOptions {
    pub(crate) fn suitable_keys(&self) -> Result<CryptKeys> {
        let keys_storage = CryptKeysStorage::default();
        let keys = keys_storage
            .keys_for_version(&self.gbox_version)
            .with_context(|| format!("No keys are available for {}", self.gbox_version))?;

        Ok(keys.clone())
    }
}
