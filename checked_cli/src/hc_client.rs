use crate::common::get_store_dir;
use anyhow::Context;
use holochain_client::{
    AdminWebsocket, AppAgentWebsocket, AppStatusFilter, AuthorizeSigningCredentialsPayload,
    ClientAgentSigner, ConductorApiError, SigningCredentials,
};
use holochain_conductor_api::CellInfo;
use holochain_types::prelude::{AgentPubKey, CapSecret, CellId};
use holochain_types::websocket::AllowedOrigins;
use serde::{Deserialize, Serialize};
use std::fs::{File, Permissions};
use std::io::Write;
use std::path::PathBuf;

const DEFAULT_INSTALLED_APP_ID: &str = "checked";

pub async fn get_authenticated_app_agent_client(
    admin_port: u16,
    path: Option<PathBuf>,
    installed_app_id: Option<String>,
) -> anyhow::Result<AppAgentWebsocket> {
    // TODO connect timeout not configurable! Really slow if Holochain is not running.
    let mut admin_client = AdminWebsocket::connect(format!("127.0.0.1:{admin_port}"))
        .await
        .with_context(|| {
            format!("Failed to connect to Holochain admin interface at {admin_port}")
        })?;

    let mut signer = ClientAgentSigner::new();
    load_or_create_signing_credentials(
        &mut admin_client,
        &mut signer,
        path,
        installed_app_id.clone(),
    )
    .await
    .context("Failed to load Holochain call credentials")?;

    let app_port = find_or_create_app_interface(&mut admin_client)
        .await
        .context("Failed to find or create an app port on Holochain")?;

    let app_id = installed_app_id.unwrap_or_else(|| DEFAULT_INSTALLED_APP_ID.to_string());
    AppAgentWebsocket::connect(
        format!("127.0.0.1:{app_port}"),
        app_id.clone(),
        signer.into(),
    )
    .await.with_context(|| {
        format!(
            "Failed to connect to Holochain app interface at `127.0.0.1:{app_port}` with app_id {app_id}"
        )
    })
}

pub fn maybe_handle_holochain_error(
    conductor_api_error: &ConductorApiError,
    path: Option<PathBuf>,
) {
    match conductor_api_error {
        // TODO brittle, would be nice if the errors for some important failures were more specific.
        ConductorApiError::SignZomeCallError(e) if e == "Provenance not found" => {
            eprintln!("Saved credentials for Holochain appear invalid, removing them. Please re-run this command");
            if let Ok(e) = get_credentials_path(path) {
                if std::fs::remove_file(e).is_ok() {
                    println!("Successfully removed credentials");
                    return;
                }
            }

            eprintln!("Failed to remove");
        }
        _ => {
            // No special handling required
        }
    }
}

async fn find_or_create_app_interface(admin_client: &mut AdminWebsocket) -> anyhow::Result<u16> {
    let app_interfaces = admin_client
        .list_app_interfaces()
        .await
        .map_err(|e| anyhow::anyhow!("Error listing app interfaces: {:?}", e))?;

    // The client doesn't tell us what origins are set for each app interface so we have to pick one.
    let app_port = match app_interfaces.first() {
        Some(app_port) => *app_port,
        None => admin_client
            .attach_app_interface(0, AllowedOrigins::Any)
            .await
            .map_err(|e| anyhow::anyhow!("Error attaching app interface: {:?}", e))?,
    };
    Ok(app_port)
}

async fn load_or_create_signing_credentials(
    admin_client: &mut AdminWebsocket,
    signer: &mut ClientAgentSigner,
    path: Option<PathBuf>,
    installed_app_id: Option<String>,
) -> anyhow::Result<()> {
    match try_load_credentials(path.clone())? {
        Some((cell_id, credentials)) => {
            signer.add_credentials(cell_id, credentials);
        }
        None => {
            let (cell_id, credentials) =
                create_new_credentials(admin_client, installed_app_id).await?;
            dump_credentials(cell_id.clone(), &credentials, path)?;
            signer.add_credentials(cell_id, credentials);
        }
    }
    Ok(())
}

async fn create_new_credentials(
    client: &mut AdminWebsocket,
    installed_app_id: Option<String>,
) -> anyhow::Result<(CellId, SigningCredentials)> {
    let apps = client
        .list_apps(Some(AppStatusFilter::Running))
        .await
        .map_err(|e| anyhow::anyhow!("Error listing apps: {:?}", e))?;

    let app_id = installed_app_id.unwrap_or_else(|| DEFAULT_INSTALLED_APP_ID.to_string());
    let app = apps
        .iter()
        .find(|app| app.installed_app_id == app_id)
        .ok_or_else(|| anyhow::anyhow!("App `checked` not found"))?;

    let cells = app
        .cell_info
        .get("checked")
        .ok_or_else(|| anyhow::anyhow!("Role `checked` not found"))?;

    let cell = cells
        .iter()
        .find_map(|cell| match cell {
            CellInfo::Provisioned(cell) if cell.name == "checked" => Some(cell),
            _ => None,
        })
        .ok_or_else(|| anyhow::anyhow!("Cell `checked` not found"))?;

    let credentials = client
        .authorize_signing_credentials(AuthorizeSigningCredentialsPayload {
            cell_id: cell.cell_id.clone(),
            functions: None, // For all, not documented!
        })
        .await
        .map_err(|e| anyhow::anyhow!("Error authorizing signing credentials: {:?}", e))?;

    Ok((cell.cell_id.clone(), credentials))
}

#[derive(Serialize, Deserialize)]
struct SavedCredentials {
    cell_id: CellId,
    signing_agent_key: AgentPubKey,
    keypair: Vec<u8>,
    cap_secret: CapSecret,
}

fn dump_credentials(
    cell_id: CellId,
    signing_credentials: &SigningCredentials,
    path: Option<PathBuf>,
) -> anyhow::Result<()> {
    let saved = SavedCredentials {
        cell_id: cell_id.clone(),
        signing_agent_key: signing_credentials.signing_agent_key.clone(),
        keypair: signing_credentials.keypair.to_keypair_bytes().to_vec(),
        cap_secret: signing_credentials.cap_secret,
    };

    let serialized = serde_json::to_string(&saved)
        .map_err(|e| anyhow::anyhow!("Error serializing credentials: {:?}", e))?;

    let credentials_path = get_credentials_path(path)?;

    let mut f = File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open(credentials_path)
        .map_err(|e| anyhow::anyhow!("Error opening credentials file: {:?}", e))?;

    #[cfg(target_family = "unix")]
    {
        use std::os::unix::fs::PermissionsExt;
        f.set_permissions(Permissions::from_mode(0o660))
            .map_err(|e| {
                anyhow::anyhow!("Error setting permissions on credentials file: {:?}", e)
            })?;
    }

    f.write_all(serialized.as_bytes())
        .map_err(|e| anyhow::anyhow!("Error writing credentials file: {:?}", e))?;

    Ok(())
}

fn try_load_credentials(
    path: Option<PathBuf>,
) -> anyhow::Result<Option<(CellId, SigningCredentials)>> {
    let credentials_path = get_credentials_path(path)?;

    let f = match File::open(credentials_path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(None);
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Error reading credentials file: {:?}", e));
        }
    };

    let saved: SavedCredentials = match serde_json::from_reader(f) {
        Ok(saved) => saved,
        Err(e) => {
            eprintln!("Saved credentials file is corrupt: {:?}", e);
            return Ok(None);
        }
    };

    let keypair = match ed25519_dalek::SigningKey::from_keypair_bytes(
        saved.keypair.as_slice().try_into().unwrap(),
    ) {
        Ok(keypair) => keypair,
        Err(e) => {
            eprintln!("Saved credentials file is corrupt: {:?}", e);
            return Ok(None);
        }
    };

    Ok(Some((
        saved.cell_id,
        SigningCredentials {
            signing_agent_key: saved.signing_agent_key,
            keypair,
            cap_secret: saved.cap_secret,
        },
    )))
}

fn get_credentials_path(path: Option<PathBuf>) -> anyhow::Result<std::path::PathBuf> {
    Ok(get_store_dir(path)?.join("credentials.json"))
}
