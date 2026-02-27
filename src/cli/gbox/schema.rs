use crate::CliCommand;
use anyhow::Result;
use clap::Parser;
use isignkit::gbox::GboxRepo;

#[derive(Parser)]
/// Prints current repository schema
pub(crate) struct SchemaCommand;

impl CliCommand for SchemaCommand {
    fn run(&self) -> Result<()> {
        println!("{}", serde_json::to_string_pretty(&GboxRepo::schema())?);

        Ok(())
    }
}
