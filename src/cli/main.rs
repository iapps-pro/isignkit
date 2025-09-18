mod gbox;

use anyhow::Result;
use clap::Parser;

trait CliCommand {
    fn run(&self) -> Result<()>;
}

#[derive(Parser)]
#[clap(version)]
enum Command {
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
    match cli_options.command {
        Command::Gbox(command) => command.run()?,
    }

    Ok(())
}
