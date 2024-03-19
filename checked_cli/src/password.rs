use crate::cli::{GenerateArgs, SignArgs};

pub trait GetPassword {
    fn get_password(&self) -> anyhow::Result<String>;
}

impl GetPassword for GenerateArgs {
    fn get_password(&self) -> anyhow::Result<String> {
        #[cfg(not(any(windows, unix)))]
            return Ok(self.password);
        #[cfg(any(windows, unix))]
            return Ok(rpassword::prompt_password("New password: ")?);
    }
}

impl GetPassword for SignArgs {
    fn get_password(&self) -> anyhow::Result<String> {
        #[cfg(not(any(windows, unix)))]
            return Ok(self.password);
        #[cfg(any(windows, unix))]
            return Ok(rpassword::prompt_password(format!("Password for '{}': ", self.name))?);
    }
}
