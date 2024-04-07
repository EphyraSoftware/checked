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

    /// Verify a file
    Verify(VerifyArgs),

    /// Distribute a verification key on Holochain
    Distribute(DistributeArgs),

    /// Fetch an asset from a URL and check signatures for it
    Fetch(FetchArgs),
}

#[derive(clap::Args)]
pub struct GenerateArgs {
    /// Name of the key, used to identify the key when used.
    ///
    /// Defaults to `default`.
    #[arg(long, short, default_value_t = String::from("default"))]
    pub name: String,

    /// The admin port for Holochain.
    #[arg(long, short)]
    pub port: Option<u16>,

    /// Provide a password on the command line instead of prompting for it.
    ///
    /// If this flag is not provided, then an interactive prompt is used to get the password.
    ///
    /// This is not recommended when using as a CLI flag because the password may stay in your
    /// shell history. Use the interactive prompt instead if possible!
    #[arg(long)]
    pub password: Option<String>,

    /// Whether to distribute the key on Holochain after generating it.
    ///
    /// If this flag is not provided, then an interactive prompt is used to confirm.
    #[arg(long, short)]
    pub distribute: Option<bool>,

    /// The directory to save the key in.
    ///
    /// Defaults to `.config/checked` in your home directory.
    #[arg(long)]
    pub path: Option<PathBuf>,

    /// Override the app id that `checked` has been installed with on Holochain.
    #[arg(long)]
    pub app_id: Option<String>,
}

#[derive(clap::Args)]
pub struct SignArgs {
    /// The URL that this asset belongs at.
    pub url: Option<String>,

    /// Name of the key to use for signing.
    ///
    /// Defaults to `default`.
    #[arg(long, short, default_value_t = String::from("default"))]
    pub name: String,

    /// The admin port for Holochain.
    #[arg(long, short)]
    pub port: Option<u16>,

    /// Provide a password on the command line instead of prompting for it.
    ///
    /// If this flag is not provided, then an interactive prompt is used to get the password.
    ///
    /// This is not recommended when using as a CLI flag because the password may stay in your
    /// shell history. Use the interactive prompt instead if possible!
    #[arg(long)]
    pub password: Option<String>,

    /// The directory to find the signing key in.
    ///
    /// Defaults to `.config/checked` in your home directory.
    #[arg(long, short)]
    pub path: Option<PathBuf>,

    /// The file to sign.
    #[arg(long, short)]
    pub file: PathBuf,

    /// The file to save the signature in.
    ///
    /// Defaults to the file to sign, with `.minisig` extension appended.
    #[arg(long, short)]
    pub output: Option<PathBuf>,

    /// Whether to distribute the asset signature
    #[arg(long, short, default_value_t = true)]
    pub distribute: bool,

    /// Override the app id that `checked` has been installed with on Holochain.
    #[arg(long)]
    pub app_id: Option<String>,
}

#[derive(clap::Args)]
pub struct VerifyArgs {
    /// The file to verify.
    #[arg(long, short)]
    pub file: PathBuf,

    /// The file containing the verification key.
    #[arg(long, short)]
    pub verification_key: PathBuf,

    /// The file containing the signature.
    ///
    /// Defaults to the `--file` path with `.minisig` appended.
    #[arg(long, short)]
    pub signature: Option<PathBuf>,
}

#[derive(clap::Args)]
pub struct DistributeArgs {
    /// The admin port for Holochain
    #[arg(long, short)]
    pub port: u16,

    /// Name of the key to distribute.
    ///
    /// Defaults to `default`.
    #[arg(long, short, default_value_t = String::from("default"))]
    pub name: String,

    /// Provide a password on the command line instead of prompting for it.
    ///
    /// If this flag is not provided, then an interactive prompt is used to get the password.
    ///
    /// This is not recommended when using as a CLI flag because the password may stay in your
    /// shell history. Use the interactive prompt instead if possible!
    #[arg(long)]
    pub password: Option<String>,

    /// The directory to find the verification key in.
    ///
    /// Defaults to `.config/checked` in your home directory.
    #[arg(long, short)]
    pub path: Option<PathBuf>,

    /// Override the app id that `checked` has been installed with on Holochain.
    #[arg(long)]
    pub app_id: Option<String>,
}

#[derive(clap::Args)]
pub struct FetchArgs {
    /// URL to fetch the asset from.
    pub url: String,

    /// The admin port for Holochain
    #[arg(long, short)]
    pub port: u16,

    /// Name of the key to use for signing.
    ///
    /// Defaults to `default`.
    #[arg(long, short, default_value_t = String::from("default"))]
    pub name: String,

    /// The directory or file to save the fetched asset to.
    ///
    /// When a directory is provided:
    /// - The directory must exist
    /// - The filename is taken from the last component in the fetch URL's path.
    ///
    /// When a file is provided:
    /// - The directory containing the file, and any required parent directories, will be created.
    ///
    /// Defaults to the directory that the CLI is running in.
    #[arg(long, short)]
    pub output: Option<PathBuf>,

    /// Provide a password on the command line instead of prompting for it.
    ///
    /// If this flag is not provided, then an interactive prompt is used to get the password.
    ///
    /// This is not recommended when using as a CLI flag because the password may stay in your
    /// shell history. Use the interactive prompt instead if possible!
    #[arg(long)]
    pub password: Option<String>,

    /// The directory to find the signing key in.
    ///
    /// Defaults to `.config/checked` in your home directory.
    #[arg(long, short)]
    pub path: Option<PathBuf>,

    /// Continue if no existing signatures are found.
    ///
    /// If this flag is not provided, then an interactive prompt is used to confirm.
    #[arg(long, short)]
    pub allow_no_signatures: Option<bool>,

    /// Sign the asset after downloading and publish the signature on Holochain.
    ///
    /// If this flag is not provided, then an interactive prompt is used to confirm.
    #[arg(long, short)]
    pub sign: Option<bool>,

    /// Override the app id that `checked` has been installed with on Holochain.
    #[arg(long)]
    pub app_id: Option<String>,

    /// Automatically approve the fetched asset, ignoring the results of the signature report.
    ///
    /// Using this flag is NOT recommended for interactive use. It is provided to allow automation
    /// of this library or for scripts calling the CLI. If you choose to skip the interactive
    /// confirmation then you are responsible for checking the signature report some other way.
    #[arg(long)]
    pub approve: Option<bool>,
}
