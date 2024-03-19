use checked_cli::prelude::*;
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate(generate_args) => {
            generate(generate_args)?;
        }
        Commands::Sign(sign_args) => sign(sign_args)?,
        Commands::Verify(verify_args) => verify(verify_args)?,
    }

    Ok(())
}
