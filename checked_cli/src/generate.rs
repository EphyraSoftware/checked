use crate::cli::GenerateArgs;
use crate::common::{get_signing_key_path, get_store_dir, get_verification_key_path, open_file};
use minisign::KeyPair;
use std::io::Write;

pub fn generate(generate_args: GenerateArgs) -> anyhow::Result<()> {
    let store_dir = get_store_dir(generate_args.path)?;

    // Signing key
    let sk_path = get_signing_key_path(&store_dir, &generate_args.name);
    let mut sk_file = open_file(&sk_path)?;

    // Verification key
    let vk_path = get_verification_key_path(&store_dir, &generate_args.name);
    let mut vk_file = open_file(&vk_path)?;

    #[cfg(not(any(windows, unix)))]
    let password = generate_args.password;
    #[cfg(any(windows, unix))]
    let password = rpassword::prompt_password("New password: ")?;

    let _pk = KeyPair::generate_and_write_encrypted_keypair(
        &mut vk_file,
        &mut sk_file,
        None,
        Some(password),
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
    // println!("Files signed using this key can be verified with the following command:\n");
    // println!("checked verify <file> -P {}", _pk.to_base64());

    let distribute = dialoguer::Confirm::new()
        .with_prompt("Would you like to distribute this key on Holochain?")
        .interact()?;

    if !distribute {
        return Ok(());
    }

    Ok(())
}
