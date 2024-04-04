use crate::cli::{DistributeArgs, FetchArgs, GenerateArgs, SignArgs};

pub trait GetPassword {
    fn get_password(&self) -> anyhow::Result<String>;
}

impl GetPassword for GenerateArgs {
    fn get_password(&self) -> anyhow::Result<String> {
        get_password_common(self.password.as_ref(), "New password: ")
    }
}

impl GetPassword for SignArgs {
    fn get_password(&self) -> anyhow::Result<String> {
        get_password_common(
            self.password.as_ref(),
            format!("Password for '{}': ", self.name),
        )
    }
}

impl GetPassword for DistributeArgs {
    fn get_password(&self) -> anyhow::Result<String> {
        get_password_common(
            self.password.as_ref(),
            format!("Password for '{}': ", self.name),
        )
    }
}

impl GetPassword for FetchArgs {
    fn get_password(&self) -> anyhow::Result<String> {
        get_password_common(
            self.password.as_ref(),
            format!("Password for '{}': ", self.name),
        )
    }
}

fn get_password_common(
    maybe_password: Option<&String>,
    prompt: impl ToString,
) -> anyhow::Result<String> {
    match maybe_password {
        Some(password) => Ok(password.clone()),
        None => Ok(rpassword::prompt_password(prompt)?),
    }
}

impl FetchArgs {
    pub fn allow_no_signatures(&self) -> anyhow::Result<bool> {
        match self.allow_no_signatures {
            Some(allow_no_signatures) => Ok(allow_no_signatures),
            None => Ok(dialoguer::Confirm::new()
                .with_prompt("Download anyway?")
                .interact()?),
        }
    }

    pub fn sign_asset(&self) -> anyhow::Result<bool> {
        match self.sign {
            Some(sign) => Ok(sign),
            None => Ok(dialoguer::Confirm::new()
                .with_prompt("Sign this asset?")
                .interact()?),
        }
    }
}
