mod common;

pub mod cli;
pub mod generate;
mod password;
pub mod sign;
pub mod verify;

pub mod prelude {
    pub use crate::cli::{Cli, Commands, GenerateArgs, SignArgs};
    pub use crate::generate::generate;
    pub use crate::sign::sign;
    pub use crate::verify::verify;
}