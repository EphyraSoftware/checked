use crate::cli::{DistributeArgs, FetchArgs, GenerateArgs, SignArgs};

/// Common trait to allow for passwords to be retrieved from the user.
pub trait GetPassword {
    /// Retrieve a password.
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
    /// Whether the asset should be downloaded even if no signatures are found. This is primarily
    /// an interactive prompt but can be forced with [FetchArgs::allow_no_signatures].
    pub fn allow_no_signatures(&self) -> anyhow::Result<bool> {
        match self.allow_no_signatures {
            Some(allow_no_signatures) => Ok(allow_no_signatures),
            None => Ok(dialoguer::Confirm::new()
                .with_prompt("Download anyway?")
                .interact()?),
        }
    }

    /// Whether the signature checks are acceptable and the asset should be retained. This is
    /// primarily an interactive prompt but can be forced with [FetchArgs::approve].
    pub fn approve_signatures_report(&self) -> anyhow::Result<bool> {
        match self.approve {
            Some(approve_signature) => Ok(approve_signature),
            None => Ok(dialoguer::Confirm::new()
                .with_prompt("Approve the download?")
                .interact()?),
        }
    }

    /// Whether the asset should be signed according to [FetchArgs::sign] or use an interactive
    /// prompt if not specified.
    pub fn sign_asset(&self) -> anyhow::Result<bool> {
        match self.sign {
            Some(sign) => Ok(sign),
            None => Ok(dialoguer::Confirm::new()
                .with_prompt("Sign this asset?")
                .interact()?),
        }
    }
}
