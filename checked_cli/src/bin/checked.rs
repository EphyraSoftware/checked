use checked_cli::prelude::*;
use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate(generate_args) => {
            generate(generate_args).await?;
        }
        Commands::Sign(sign_args) => {
            sign(sign_args)?;
        }
        Commands::Verify(verify_args) => verify(verify_args)?,
        Commands::Distribute(distribute_args) => distribute(distribute_args).await?,
        Commands::Fetch(fetch_args) => {
            fetch(fetch_args).await?;
        },
    }

    Ok(())
}
