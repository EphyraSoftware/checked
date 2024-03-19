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

    /// Sign a file
    Sign(SignArgs),
}

#[derive(clap::Args)]
pub struct GenerateArgs {
    /// Name of the key, used to identify the key when used.
    ///
    /// Defaults to `default`.
    #[arg(long, short, default_value_t = String::from("default"))]
    pub name: String,

    /// Provide a password on the command line instead of prompting for it on platforms
    /// where a prompt isn't supported.
    #[cfg(not(any(windows, unix)))]
    pub password: String,

    /// The directory to save the key in.
    ///
    /// Defaults to `.config/checked` in your home directory.
    #[arg(long, short)]
    pub path: Option<PathBuf>,
}

#[derive(clap::Args)]
pub struct SignArgs {
    /// Name of the key to use for signing.
    ///
    /// Defaults to `default`.
    #[arg(long, short, default_value_t = String::from("default"))]
    pub name: String,

    /// Provide a password on the command line instead of prompting for it on platforms
    /// where a prompt isn't supported.
    #[cfg(not(any(windows, unix)))]
    pub password: String,

    /// The directory to save the key in.
    ///
    /// Defaults to `.config/checked` in your home directory.
    #[arg(long, short)]
    pub path: Option<PathBuf>,

    /// The file to sign.
    #[arg(long, short)]
    pub file: PathBuf,

    /// The file to save the signature in.
    ///
    /// Defaults to the file to sign, with `.sig` extension appended.
    #[arg(long, short)]
    pub output: Option<PathBuf>,
}
