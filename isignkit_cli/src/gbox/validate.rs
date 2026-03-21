use super::{GlobalOptions, gbox_file::RepoReader};
use crate::CliCommand;
use crate::input_file::{InputFile, InputFileParser, InputFileReader};
use anyhow::Result;
use clap::Parser;
use isignkit::gbox::{GboxRepo, SchemaVersion};

#[derive(Parser)]
/// Perform repo validation
pub(crate) struct ValidateCommand {
    #[clap(flatten)]
    global_options: GlobalOptions,

    /// Local input file, stdin (-) or remote URL
    #[arg(value_parser = InputFileParser)]
    repo_json: InputFile,
}

impl CliCommand for ValidateCommand {
    fn run(&self) -> Result<()> {
        let reader = RepoReader::new(self.global_options.udid.as_deref());
        let repo_json = reader.read_to_string(&self.repo_json, true)?;

        if let GboxRepo::Encrypted(repo) = GboxRepo::validate_and_parse(&repo_json)? {
            let keys = self.global_options.suitable_keys()?;
            eprintln!("Encrypted repository is valid, validating encrypted items...");
            repo.validate_items(&keys)?;
        }

        println!(
            "This repository is fully valid for {} schema!",
            SchemaVersion::current()
        );

        Ok(())
    }
}
