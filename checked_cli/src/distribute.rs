use anyhow::Context;
use holochain_client::ZomeCallTarget;
use holochain_types::prelude::ExternIO;
use rand::Rng;

use checked_types::{DistributeVfKeyRequest, VerificationKeyType};

use crate::cli::DistributeArgs;
use crate::common::{get_store_dir, get_verification_key_path};
use crate::hc_client::{get_authenticated_app_agent_client, maybe_handle_holochain_error};
use crate::interactive::GetPassword;
use crate::prelude::SignArgs;
use crate::sign::sign;

const PROOF_WORDS: [&str; 40] = [
    "handle",
    "under",
    "send",
    "sample",
    "eagle",
    "wind",
    "clamber",
    "cycle",
    "run",
    "sunset",
    "clock",
    "week",
    "lion",
    "fender",
    "the",
    "crack",
    "crumble",
    "butterfly",
    "sail",
    "butter",
    "fly",
    "wipe",
    "off",
    "pen",
    "grape",
    "fruit",
    "sugar",
    "cane",
    "dog",
    "flan",
    "cherry",
    "pie",
    "candy",
    "can",
    "amused",
    "kettle",
    "bottle",
    "candle",
    "high",
    "tide",
];

/// Distribute a verification key on Holochain. This can be done as part of [generate](crate::generate::generate) but if you
/// have an existing key or wish to generate it using another tool then you can distribute it
/// using this operation.
pub async fn distribute(distribute_args: DistributeArgs) -> anyhow::Result<()> {
    println!("Distributing key: {}", distribute_args.name);

    let mut app_client = get_authenticated_app_agent_client(
        distribute_args.port,
        distribute_args.config_dir.clone(),
        distribute_args.app_id.clone(),
    )
    .await?;

    let store_dir = get_store_dir(distribute_args.config_dir.clone())?;
    let vk_path = get_verification_key_path(&store_dir, &distribute_args.name);

    let proof = generate_proof();

    let tmp_file = tempfile::Builder::new()
        .prefix("proof")
        .suffix(".txt")
        .tempfile()
        .context("Could not create temporary file")?;

    std::fs::write(tmp_file.path(), proof.as_bytes())
        .context("Could not write proof to temporary file")?;

    let sig_path = sign(SignArgs {
        url: None,
        name: distribute_args.name.clone(),
        port: Some(distribute_args.port),
        password: Some(distribute_args.get_password()?),
        config_dir: distribute_args.config_dir.clone(),
        file: tmp_file.path().to_path_buf(),
        output: None,
        // This is a temporary file for demonstrating private key access, should not be distributed as an asset signature
        distribute: false,
        app_id: distribute_args.app_id,
    })
    .await?;

    app_client
        .call_zome(
            ZomeCallTarget::RoleName("checked".to_string()),
            "signing_keys".into(),
            "distribute_verification_key".into(),
            ExternIO::encode(DistributeVfKeyRequest {
                name: distribute_args.name.clone(),
                verification_key: std::fs::read_to_string(vk_path)
                    .map_err(|e| anyhow::anyhow!("Failed to read verification key: {:?}", e))?,
                key_type: VerificationKeyType::MiniSignEd25519,
                proof,
                proof_signature: std::fs::read(sig_path)
                    .map_err(|e| anyhow::anyhow!("Failed to read proof signature: {:?}", e))?,
            })
            .unwrap(),
        )
        .await
        .map_err(|e| {
            maybe_handle_holochain_error(&e, distribute_args.config_dir);
            anyhow::anyhow!("Failed to get signatures for the asset: {:?}", e)
        })?;

    println!("Successfully distributed on Holochain!");

    Ok(())
}

fn generate_proof() -> String {
    let mut rng = rand::thread_rng();
    let mut proof = String::new();
    for _ in 0..10 {
        proof.push_str(PROOF_WORDS[rng.gen_range(0..PROOF_WORDS.len())]);
        proof.push(' ');
    }
    proof
}
