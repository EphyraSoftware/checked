use anyhow::Context;
use checked_cli::cli::{DistributeArgs, FetchArgs};
use checked_cli::prelude::{distribute, fetch, generate, GenerateArgs, SignArgs};
use checked_cli::sign::sign;
use checked_types::{AssetSignatureResponse, FetchCheckSignatureReason};
use holochain::core::AgentPubKey;
use holochain::sweettest::{SweetAgents, SweetConductor, SweetConductorHandle, SweetZome};
use holochain_conductor_api::{AdminInterfaceConfig, AppStatusFilter, CellInfo, InterfaceDriver};
use holochain_types::app::InstallAppPayload;
use holochain_types::prelude::AppBundleSource;
use holochain_types::websocket::AllowedOrigins;
use signing_keys_types::VfKeyResponse;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::net::SocketAddr;
use tokio::task::AbortHandle;

// Generate a signing keypair, distribute it on Holochain
#[tokio::test(flavor = "multi_thread")]
async fn generate_signing_keypair() -> anyhow::Result<()> {
    let conductor = SweetConductor::from_standard_config().await;

    install_checked_app(conductor.sweet_handle(), "checked").await?;
    let admin_port = add_admin_port(conductor.sweet_handle()).await?;

    let dir = tempfile::tempdir()?;

    generate(GenerateArgs {
        name: "test_generate".to_string(),
        port: Some(admin_port),
        password: Some("test".to_string()),
        distribute: Some(true),
        path: Some(dir.as_ref().to_path_buf()),
        app_id: None,
    })
    .await?;

    let zome = get_zome_handle(&conductor, "checked", "signing_keys").await;
    let keys: Vec<VfKeyResponse> = conductor
        .call_fallible(&zome, "get_my_verification_key_distributions", ())
        .await?;

    assert_eq!(1, keys.len());
    assert_eq!("test_generate", keys[0].verification_key_dist.name);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn distribute_existing_keypair() -> anyhow::Result<()> {
    let conductor = SweetConductor::from_standard_config().await;

    install_checked_app(conductor.sweet_handle(), "checked").await?;
    let admin_port = add_admin_port(conductor.sweet_handle()).await?;

    let dir = tempfile::tempdir()?;

    let name = "test_generate".to_string();
    generate(GenerateArgs {
        name: name.clone(),
        port: None,
        password: Some("test".to_string()),
        distribute: Some(false),
        path: Some(dir.as_ref().to_path_buf()),
        app_id: None,
    })
    .await?;

    let zome = get_zome_handle(&conductor, "checked", "signing_keys").await;

    let keys: Vec<VfKeyResponse> = conductor
        .call_fallible(&zome, "get_my_verification_key_distributions", ())
        .await?;

    assert_eq!(0, keys.len());

    distribute(DistributeArgs {
        port: admin_port,
        name,
        password: Some("test".to_string()),
        path: Some(dir.as_ref().to_path_buf()),
        app_id: None,
    })
    .await?;

    let keys: Vec<VfKeyResponse> = conductor
        .call_fallible(&zome, "get_my_verification_key_distributions", ())
        .await?;

    assert_eq!(1, keys.len());
    assert_eq!("test_generate", keys[0].verification_key_dist.name);

    Ok(())
}

// TODO this test should `sign` not `fetch`
#[tokio::test(flavor = "multi_thread")]
async fn create_first_asset_signature() -> anyhow::Result<()> {
    let conductor = SweetConductor::from_standard_config().await;

    install_checked_app(conductor.sweet_handle(), "checked").await?;
    let admin_port = add_admin_port(conductor.sweet_handle()).await?;

    let dir = tempfile::tempdir()?;

    let name = "test_generate".to_string();
    generate(GenerateArgs {
        name: name.clone(),
        port: Some(admin_port),
        password: Some("test".to_string()),
        distribute: Some(true),
        path: Some(dir.as_ref().to_path_buf()),
        app_id: None,
    })
    .await?;

    let (addr, _fs_abort_handle) = start_sample_file_server().await;
    let url = format!("http://{}:{}/test.txt", addr.ip(), addr.port());

    let fetch_info = fetch(FetchArgs {
        url: url.clone(),
        port: admin_port,
        name,
        output: Some(dir.as_ref().to_path_buf()),
        password: Some("test".to_string()),
        path: Some(dir.as_ref().to_path_buf()),
        allow_no_signatures: Some(true),
        sign: Some(true),
        app_id: None,
        approve: Some(true),
    })
    .await?;

    assert!(fetch_info.signature_path.is_some());

    let zome = get_zome_handle(&conductor, "checked", "fetch").await;

    let signatures: Vec<AssetSignatureResponse> = conductor
        .call_fallible(&zome, "get_my_asset_signatures", ())
        .await?;

    assert_eq!(1, signatures.len());
    assert_eq!(url, signatures[0].fetch_url);
    assert_eq!(
        std::fs::read_to_string(fetch_info.signature_path.unwrap())?,
        signatures[0].signature
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn fetch_asset_signed_by_others() -> anyhow::Result<()> {
    let conductor = SweetConductor::from_standard_config().await;

    let (addr, _fs_abort_handle) = start_sample_file_server().await;
    let url = format!("http://{}:{}/test.txt", addr.ip(), addr.port());

    let admin_port = add_admin_port(conductor.sweet_handle()).await?;

    for i in 0..6 {
        publish_asset_signature(
            conductor.sweet_handle(),
            &url,
            admin_port,
            format!("checked-{i}"),
            false,
        )
        .await
        .context("Couldn't publish asset signature")?;
    }

    install_checked_app(conductor.sweet_handle(), "checked").await?;

    let dir = tempfile::tempdir()?;

    let name = "test_generate".to_string();
    generate(GenerateArgs {
        name: name.clone(),
        port: Some(admin_port),
        password: Some("test".to_string()),
        distribute: Some(true),
        path: Some(dir.as_ref().to_path_buf()),
        app_id: None,
    })
    .await?;

    let fetch_info = fetch(FetchArgs {
        url: url.clone(),
        port: admin_port,
        name,
        output: Some(dir.as_ref().to_path_buf()),
        password: Some("test".to_string()),
        path: Some(dir.as_ref().to_path_buf()),
        allow_no_signatures: Some(false),
        sign: Some(true),
        app_id: None,
        approve: Some(true),
    })
    .await?;

    assert!(fetch_info.signature_path.is_some());

    let zome = get_zome_handle(&conductor, "checked", "fetch").await;

    let signatures: Vec<AssetSignatureResponse> = conductor
        .call_fallible(&zome, "get_my_asset_signatures", ())
        .await?;

    assert_eq!(1, signatures.len());
    assert_eq!(url, signatures[0].fetch_url);
    assert_eq!(
        std::fs::read_to_string(fetch_info.signature_path.unwrap())?,
        signatures[0].signature
    );

    let recent_signatures = fetch_info
        .reports
        .iter()
        .find(|r| r.reason == FetchCheckSignatureReason::RandomRecent)
        .unwrap();
    assert_eq!(5, recent_signatures.passed_signatures.len());
    assert!(recent_signatures.failed_signatures.is_empty());

    assert!(fetch_info
        .reports
        .iter()
        .find(|r| r.reason == FetchCheckSignatureReason::RandomHistorical)
        .is_none());

    assert!(fetch_info
        .reports
        .iter()
        .find(|r| matches!(r.reason, FetchCheckSignatureReason::Pinned(_)))
        .is_none());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn fetch_asset_signed_by_others_with_mismatches() -> anyhow::Result<()> {
    let conductor = SweetConductor::from_standard_config().await;

    let (addr, _fs_abort_handle) = start_sample_file_server().await;
    let url = format!("http://{}:{}/test.txt", addr.ip(), addr.port());

    let admin_port = add_admin_port(conductor.sweet_handle()).await?;

    for i in 0..5 {
        publish_asset_signature(
            conductor.sweet_handle(),
            &url,
            admin_port,
            format!("checked-{i}"),
            i == 2 || i == 4,
        )
            .await
            .context("Couldn't publish asset signature")?;
    }

    install_checked_app(conductor.sweet_handle(), "checked").await?;

    let dir = tempfile::tempdir()?;

    let name = "test_generate".to_string();
    generate(GenerateArgs {
        name: name.clone(),
        port: Some(admin_port),
        password: Some("test".to_string()),
        distribute: Some(true),
        path: Some(dir.as_ref().to_path_buf()),
        app_id: None,
    })
        .await?;

    let fetch_info = fetch(FetchArgs {
        url: url.clone(),
        port: admin_port,
        name,
        output: Some(dir.as_ref().to_path_buf()),
        password: Some("test".to_string()),
        path: Some(dir.as_ref().to_path_buf()),
        allow_no_signatures: Some(false),
        sign: Some(true),
        app_id: None,
        approve: Some(true),
    })
        .await?;

    assert!(fetch_info.signature_path.is_some());

    let zome = get_zome_handle(&conductor, "checked", "fetch").await;

    let signatures: Vec<AssetSignatureResponse> = conductor
        .call_fallible(&zome, "get_my_asset_signatures", ())
        .await?;

    assert_eq!(1, signatures.len());
    assert_eq!(url, signatures[0].fetch_url);
    assert_eq!(
        std::fs::read_to_string(fetch_info.signature_path.unwrap())?,
        signatures[0].signature
    );

    let recent_signatures = fetch_info
        .reports
        .iter()
        .find(|r| r.reason == FetchCheckSignatureReason::RandomRecent)
        .unwrap();
    assert_eq!(3, recent_signatures.passed_signatures.len());
    assert_eq!(2, recent_signatures.failed_signatures.len());

    assert!(fetch_info
        .reports
        .iter()
        .find(|r| r.reason == FetchCheckSignatureReason::RandomHistorical)
        .is_none());

    assert!(fetch_info
        .reports
        .iter()
        .find(|r| matches!(r.reason, FetchCheckSignatureReason::Pinned(_)))
        .is_none());

    Ok(())
}

async fn install_checked_app(
    conductor: SweetConductorHandle,
    app_id: &str,
) -> anyhow::Result<AgentPubKey> {
    let agent = SweetAgents::one(conductor.keystore().clone()).await;

    conductor
        .clone()
        .install_app_bundle(InstallAppPayload {
            source: AppBundleSource::Path("../workdir/checked.happ".into()),
            agent_key: agent.clone(),
            installed_app_id: Some(app_id.into()),
            membrane_proofs: HashMap::with_capacity(0),
            network_seed: None,
        })
        .await?;

    conductor.clone().enable_app(app_id.to_string()).await?;

    Ok(agent)
}

async fn add_admin_port(conductor: SweetConductorHandle) -> anyhow::Result<u16> {
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

    Ok(*admin_port)
}

async fn get_zome_handle(conductor: &SweetConductor, app_id: &str, zome_name: &str) -> SweetZome {
    let apps = conductor
        .list_apps(Some(AppStatusFilter::Running))
        .await
        .unwrap();

    let app = apps
        .iter()
        .find(|app| app.installed_app_id == app_id)
        .unwrap();

    let cells = app.cell_info.get("checked").unwrap();

    let cell_id = cells
        .into_iter()
        .filter_map(|cell| match cell {
            CellInfo::Provisioned(cell) => Some(cell.cell_id.clone()),
            _ => None,
        })
        .next()
        .unwrap();

    let zome = SweetZome::new(cell_id.clone(), zome_name.into());
    zome
}

async fn start_sample_file_server() -> (SocketAddr, DropAbortHandle) {
    use warp::Filter;

    let (tx, rx) = tokio::sync::oneshot::channel::<SocketAddr>();

    let join_handle = tokio::task::spawn(async move {
        let test_txt = warp::path!("test.txt").map(|| "test");

        let (addr, srv) = warp::serve(test_txt).bind_ephemeral(([127, 0, 0, 1], 0));

        tx.send(addr).unwrap();

        srv.await;
    });

    let addr = rx.await.unwrap();

    (addr, DropAbortHandle(join_handle.abort_handle()))
}

struct DropAbortHandle(AbortHandle);

impl Drop for DropAbortHandle {
    fn drop(&mut self) {
        self.0.abort();
    }
}

async fn publish_asset_signature(
    conductor: SweetConductorHandle,
    url: &str,
    admin_port: u16,
    app_id: String,
    bad_signature: bool,
) -> anyhow::Result<()> {
    install_checked_app(conductor, &app_id)
        .await
        .context("Installed app with different app id failed")?;

    let dir = tempfile::tempdir()?;

    let name = "test_generate".to_string();
    generate(GenerateArgs {
        name: name.clone(),
        port: Some(admin_port),
        password: Some("test".to_string()),
        distribute: Some(true),
        path: Some(dir.as_ref().to_path_buf()),
        app_id: Some(app_id.clone()),
    })
    .await
    .context("Generating signing keypair failed")?;

    let fetch_info = fetch(FetchArgs {
        url: url.to_string(),
        port: admin_port,
        name: name.clone(),
        output: Some(dir.as_ref().to_path_buf()),
        password: Some("test".to_string()),
        path: Some(dir.as_ref().to_path_buf()),
        allow_no_signatures: Some(true),
        sign: Some(!bad_signature),
        app_id: Some(app_id.clone()),
        approve: Some(true),
    })
    .await
    .context("Fetch failed")?;

    if bad_signature {
        let output_path = fetch_info.output_path.clone().unwrap();
        let mut f = File::options().append(true).open(&output_path).unwrap();
        f.write_all("bad signature".as_bytes()).unwrap();

        sign(SignArgs {
            url: Some(url.to_string()),
            name: name.clone(),
            port: Some(admin_port),
            password: Some("test".to_string()),
            path: Some(dir.as_ref().to_path_buf()),
            file: output_path,
            output: None,
            distribute: true,
            app_id: Some(app_id.clone()),
        })
        .await
        .context("Publishing bad signature failed")?;
    }

    Ok(())
}
