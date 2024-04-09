//! A library intended to be embedded in a CLI application for:
//! - Fetching assets from URLs, with signatures checks.
//! - Managing signing keys and publishing the verification keys on Holochain.
//! - Signing assets and distributing the signatures on Holochain.
//!
//! Assets are any content that can be stored in a file. The intention is for authors to create
//! signatures for their own assets locally with [sign](crate::sign::sign) and distribute those
//! signatures on Holochain. When other users [fetch](crate::fetch::fetch) the asset,
//! they can verify the signature and the signatures of other users who have signed the asset.
//! Users can also publish their own signatures as they use the tool to fetch and check assets.

mod common;

mod cli;
mod distribute;
mod fetch;
mod generate;
pub(crate) mod hc_client;
mod interactive;
mod sign;
mod verify;

/// Flattened exports for public use.
pub mod prelude {
    pub use crate::cli::*;
    pub use crate::distribute::distribute;
    pub use crate::fetch::{fetch, FetchInfo};
    pub use crate::generate::{generate, GenerateInfo};
    pub use crate::sign::sign;
    pub use crate::verify::verify;
    pub use crate::interactive::GetPassword;
}
