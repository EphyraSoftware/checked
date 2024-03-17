use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate a new signing key
    Generate(GenerateArgs),
}

#[derive(clap::Args)]
pub struct GenerateArgs {
    /// Name of the key, used to identify the key when used.
    pub name: String,

    #[cfg(not(any(windows, unix)))]
    pub password: String,

    /// The directory to save the key in.
    ///
    /// Defaults to `.config/checked` in your home directory.
    #[arg(long)]
    pub path: Option<PathBuf>,
}
