#![deny(clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::doc_markdown)]

#[cfg(not(any(feature = "openssl", feature = "openssl-vendored")))]
compile_error!(
    "You need to select openssl backend. Available backend features: openssl, openssl-vendored."
);

mod gbox;
mod input_file;

use anyhow::Result;
use clap::Parser;

trait CliCommand {
    fn run(&self) -> Result<()>;
}

#[derive(Parser)]
enum Command {
    #[clap(subcommand)]
    Gbox(gbox::GBoxCommand),
}

#[derive(Parser)]
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
