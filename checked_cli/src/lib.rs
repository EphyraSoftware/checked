mod common;

pub mod cli;
mod distribute;
mod fetch;
pub mod generate;
pub(crate) mod hc_client;
mod password;
pub mod sign;
pub mod verify;

pub mod prelude {
    pub use crate::cli::{Cli, Commands, GenerateArgs, SignArgs};
    pub use crate::distribute::distribute;
    pub use crate::fetch::fetch;
    pub use crate::generate::generate;
    pub use crate::sign::sign;
    pub use crate::verify::verify;
}
