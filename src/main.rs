use anyhow::Result;
use clap::{Parser, Subcommand};
use opw::RunOptions;

#[derive(Parser)]
#[command(version = env!("CARGO_PKG_VERSION"), about = "Wraps op and injects environment variables")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Daemon,
    Run(RunOptions),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Daemon => opw::start_daemon(),
        Commands::Run(options) => opw::run_command(options),
    }
}
