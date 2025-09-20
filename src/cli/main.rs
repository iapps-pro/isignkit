#![deny(clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::doc_markdown)]

mod gbox;
mod input_file;

use anyhow::Result;
use clap::Parser;

trait CliCommand {
    fn run(&self) -> Result<()>;
}

#[derive(Parser)]
#[clap(version)]
enum Command {
    /// GBox related utils
    #[clap(disable_version_flag = true, subcommand)]
    Gbox(gbox::GBoxCommand),
}

#[derive(Parser)]
#[clap(about, version)]
struct CLIOptions {
    #[command(subcommand)]
    command: Command,
}

fn main() -> Result<()> {
    let cli_options = CLIOptions::parse();
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .without_timestamps()
        .env()
        .init()?;

    match cli_options.command {
        Command::Gbox(command) => command.run()?,
    }

    Ok(())
}
