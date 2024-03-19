use crate::cli::SignArgs;
use crate::common::{
    get_signing_key_path, get_store_dir, get_verification_key_path, open_file, unix_timestamp,
};
use crate::password::GetPassword;
use minisign::{PublicKey, SecretKey};
use std::io::{BufReader, Write};

pub fn sign(sign_args: SignArgs) -> anyhow::Result<()> {
    if !sign_args.file.exists() {
        anyhow::bail!("File to sign does not exist - {:?}", sign_args.file);
    }

    if !sign_args.file.metadata()?.is_file() {
        anyhow::bail!(
            "Value provided for `--file` is not a file - {:?}",
            sign_args.file
        );
    }

    let output_path = sign_args.output.clone().unwrap_or_else(|| {
        let p = sign_args.file.clone();
        p.with_extension(format!(
            "{}.minisig",
            sign_args
                .file
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
        ))
    });

    let store_dir = get_store_dir(sign_args.path.clone())?;

    // Signing key
    let sk_path = get_signing_key_path(&store_dir, &sign_args.name);
    let sk = SecretKey::from_file(sk_path, Some(sign_args.get_password()?))?;

    // Verification key
    let vk_path = get_verification_key_path(&store_dir, &sign_args.name);
    let vk = match PublicKey::from_file(vk_path) {
        Ok(vk) => Some(vk),
        Err(e) => {
            println!("Verification key not found, signature will not be checked after it is created: {:?}", e);
            None
        }
    };

    let trusted_comment = format!(
        "timestamp:{}\tfile:{}\tprehashed",
        unix_timestamp(),
        sign_args.file.display()
    );

    let mut data_reader = BufReader::new(std::fs::File::open(&sign_args.file)?);

    let sig = minisign::sign(
        vk.as_ref(),
        &sk,
        &mut data_reader,
        Some(trusted_comment.as_str()),
        None,
    )?;

    let mut sig_file = open_file(&output_path)?;
    sig_file.write_all(&sig.to_bytes())?;
    sig_file.flush()?;

    println!("Signature created and saved in: {}", output_path.display());

    Ok(())
}
