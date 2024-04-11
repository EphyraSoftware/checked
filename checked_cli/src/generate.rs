use crate::cli::{DistributeArgs, GenerateArgs};
use crate::common::{get_signing_key_path, get_store_dir, get_verification_key_path, open_file};
use crate::distribute::distribute;
use crate::interactive::GetPassword;
use minisign::KeyPair;
use std::io::Write;
use std::path::PathBuf;

/// Information about the result of generating a new keypair.
#[derive(Debug)]
pub struct GenerateInfo {
    /// Path to the secret key.
    pub sk_path: PathBuf,
    /// Path to the public key.
    pub vk_path: PathBuf,
}

/// Generate a new signing keypair and optionally distribute the verification (public) key on Holochain.
pub async fn generate(generate_args: GenerateArgs) -> anyhow::Result<GenerateInfo> {
    let store_dir = get_store_dir(generate_args.config_dir.clone())?;

    // Signing key
    let sk_path = get_signing_key_path(&store_dir, &generate_args.name);
    let mut sk_file = open_file(&sk_path)?;

    // Verification key
    let vk_path = get_verification_key_path(&store_dir, &generate_args.name);
    let mut vk_file = open_file(&vk_path)?;

    let password = generate_args.get_password()?;

    let _pk = KeyPair::generate_and_write_encrypted_keypair(
        &mut vk_file,
        &mut sk_file,
        None,
        Some(password.clone()),
    )?
    .pk;

    sk_file.flush()?;
    vk_file.flush()?;

    println!(
        "\nThe secret key was saved as {} - Keep it secret!",
        sk_path.display()
    );
    println!(
        "The public key was saved as {} - That one can be public.\n",
        vk_path.display()
    );

    let should_distribute = match generate_args.distribute {
        Some(distribute) => distribute,
        None => dialoguer::Confirm::new()
            .with_prompt("Would you like to distribute this key on Holochain?")
            .interact()?,
    };

    if should_distribute {
        dispatch_distribute(generate_args, password).await?;
    }

    Ok(GenerateInfo { sk_path, vk_path })
}

async fn dispatch_distribute(generate_args: GenerateArgs, password: String) -> anyhow::Result<()> {
    let admin_port = match generate_args.port {
        Some(port) => port,
        None => dialoguer::Input::<u16>::new()
            .with_prompt("Admin port for Holochain")
            .interact()?,
    };

    distribute(DistributeArgs {
        port: admin_port,
        name: generate_args.name,
        password: Some(password),
        config_dir: generate_args.config_dir,
        app_id: generate_args.app_id,
    })
    .await?;

    Ok(())
}
