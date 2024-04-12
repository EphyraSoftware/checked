use crate::cli::SignArgs;
use crate::common::{
    get_signing_key_path, get_store_dir, get_verification_key_path, open_file, unix_timestamp,
};
use crate::hc_client;
use crate::interactive::GetPassword;
use checked_types::{CreateAssetSignature, VerificationKeyType};
use holochain_client::ZomeCallTarget;
use holochain_types::prelude::{ActionHash, ExternIO};
use minisign::{PublicKey, SecretKey};
use std::io::{BufReader, Write};
use std::path::PathBuf;

/// Sign a file and optionally distribute the signature on Holochain.
pub async fn sign(sign_args: SignArgs) -> anyhow::Result<PathBuf> {
    if !sign_args.file.exists() {
        anyhow::bail!("File to sign does not exist - {:?}", sign_args.file);
    }

    if !sign_args.file.metadata()?.is_file() {
        anyhow::bail!(
            "Value provided for `--file` is not a file - {:?}",
            sign_args.file
        );
    }

    let mut data_reader = BufReader::new(std::fs::File::open(&sign_args.file)?);

    let sig_path = sign_args.output.clone().unwrap_or_else(|| {
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

    let mut sig_file = open_file(&sig_path)?;

    let store_dir = get_store_dir(sign_args.config_dir.clone())?;

    // Signing key
    let sk_path = get_signing_key_path(&store_dir, &sign_args.name);
    let sk = SecretKey::from_file(sk_path, Some(sign_args.get_password()?))?;

    // Verification key
    let vk_path = get_verification_key_path(&store_dir, &sign_args.name);
    let vk = match PublicKey::from_file(&vk_path) {
        Ok(vk) => Some(vk),
        Err(e) => {
            println!("Verification key not found, signature will not be checked after it is created: {:?}", e);
            None
        }
    };

    let trusted_comment = format!(
        "timestamp:{}\tfile:{}\tprehashed",
        unix_timestamp(),
        sign_args
            .file
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
    );

    let sig = minisign::sign(
        vk.as_ref(),
        &sk,
        &mut data_reader,
        Some(trusted_comment.as_str()),
        None,
    )?;

    sig_file.write_all(&sig.to_bytes())?;
    sig_file.flush()?;

    println!("Signature created and saved in: {}", sig_path.display());

    if !sign_args.distribute {
        return Ok(sig_path);
    }

    let admin_port = sign_args.admin_port()?;

    let mut app_client = hc_client::get_authenticated_app_agent_client(
        admin_port,
        sign_args.config_dir.clone(),
        sign_args.app_id,
    )
    .await?;

    println!(
        "Distributing signature to Holochain: {:?}",
        std::fs::read_to_string(&sig_path)?
    );

    let response = app_client
        .call_zome(
            ZomeCallTarget::RoleName("checked".to_string()),
            "fetch".into(),
            "create_asset_signature".into(),
            ExternIO::encode(CreateAssetSignature {
                fetch_url: sign_args
                    .url
                    .ok_or_else(|| anyhow::anyhow!("URL is required for distribution"))?,
                signature: std::fs::read_to_string(&sig_path)?,
                key_type: VerificationKeyType::MiniSignEd25519,
                verification_key: std::fs::read_to_string(vk_path)?,
            })?,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to report signature to Holochain: {:?}", e))?;

    let asset_signature_address: ActionHash = response.decode()?;
    println!(
        "Signature stored on Holochain at: {:?}",
        asset_signature_address
    );

    Ok(sig_path)
}
