use checked_cli::prelude::{generate, GenerateArgs};
use holochain::sweettest::{SweetAgents, SweetConductor, SweetZome};
use holochain_conductor_api::{AdminInterfaceConfig, InterfaceDriver};
use holochain_types::app::InstallAppPayload;
use holochain_types::prelude::AppBundleSource;
use holochain_types::websocket::AllowedOrigins;
use signing_keys_types::VfKeyResponse;
use std::collections::HashMap;

#[tokio::test(flavor = "multi_thread")]
async fn test_generate() -> anyhow::Result<()> {
    let conductor = SweetConductor::from_standard_config().await;

    let agent = SweetAgents::one(conductor.keystore()).await;

    conductor
        .clone()
        .install_app_bundle(InstallAppPayload {
            source: AppBundleSource::Path("../workdir/checked.happ".into()),
            agent_key: agent,
            installed_app_id: Some("checked".into()),
            membrane_proofs: HashMap::with_capacity(0),
            network_seed: None,
        })
        .await?;

    conductor.clone().enable_app("checked".to_string()).await?;

    let admin_port = conductor
        .clone()
        .add_admin_interfaces(vec![AdminInterfaceConfig {
            driver: InterfaceDriver::Websocket {
                port: 0,
                allowed_origins: AllowedOrigins::Any,
            },
        }])
        .await?;
    let admin_port = admin_port.first().unwrap();

    let dir = tempfile::tempdir()?;

    generate(GenerateArgs {
        name: "test_generate".to_string(),
        port: Some(*admin_port),
        password: Some("test".to_string()),
        distribute: Some(true),
        path: Some(dir.as_ref().to_path_buf()),
    })
    .await?;

    let cell_ids = conductor.running_cell_ids();
    let cell_id = cell_ids.iter().next().unwrap();

    let zome = SweetZome::new(cell_id.clone(), "signing_keys".into());

    let keys: Vec<VfKeyResponse> = conductor
        .call_fallible(&zome, "get_my_verification_key_distributions", ())
        .await?;

    assert_eq!(keys.len(), 1);

    println!("Keys: {:?}", keys);

    Ok(())
}
