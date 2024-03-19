mod common;

pub mod cli;
pub mod generate;
pub mod sign;
mod password;

pub mod prelude {
    pub use crate::cli::{Cli, Commands, GenerateArgs, SignArgs};
    pub use crate::generate::generate;
    pub use crate::sign::sign;
}
