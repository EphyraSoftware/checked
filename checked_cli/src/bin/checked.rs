use checked_cli::cli::{Cli, Commands};
use checked_cli::generate::generate;
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate(generate_args) => {
            generate(generate_args)?;
        }
    }

    Ok(())
}
