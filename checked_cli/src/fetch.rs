use crate::cli::FetchArgs;
use crate::common::{get_store_dir, get_verification_key_path};
use crate::hc_client;
use crate::interactive::GetPassword;
use crate::prelude::SignArgs;
use crate::sign::sign;
use anyhow::Context;
use checked_types::{
    CreateAssetSignature, FetchCheckSignature, PrepareFetchRequest, VerificationKeyType,
};
use holochain_client::ZomeCallTarget;
use holochain_types::prelude::ExternIO;
use indicatif::{ProgressFinish, ProgressStyle};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tempfile::NamedTempFile;
use url::Url;
use crate::hc_client::maybe_handle_holochain_error;

pub struct FetchInfo {
    pub signature_path: Option<PathBuf>,
}

struct FetchState {
    asset_size: AtomicUsize,
    downloaded_size: AtomicUsize,
}

pub async fn fetch(fetch_args: FetchArgs) -> anyhow::Result<FetchInfo> {
    let fetch_url = url::Url::parse(&fetch_args.url).context("Invalid URL")?;
    println!("Fetching from {}", fetch_url);

    let output_path = get_output_path(&fetch_args, &fetch_url)?;

    let mut app_client =
        hc_client::get_authenticated_app_agent_client(fetch_args.port, fetch_args.path.clone())
            .await?;

    // TODO if this fails because the credentials are no longer valid then we need a recovery mechanism that isn't `rm ~/.checked/credentials.json`
    let response = app_client
        .call_zome(
            ZomeCallTarget::RoleName("checked".to_string()),
            "fetch".into(),
            "prepare_fetch".into(),
            ExternIO::encode(PrepareFetchRequest {
                fetch_url: fetch_args.url.clone(),
            })
            .unwrap(),
        )
        .await
        .map_err(|e| {
            maybe_handle_holochain_error(&e, fetch_args.path.clone());
            anyhow::anyhow!("Failed to get signatures for the asset: {:?}", e)
        })?;

    let response: Vec<FetchCheckSignature> = response.decode()?;

    if response.is_empty() {
        println!("No signatures found for this asset. This is normal but please consider asking the author to create a signature!");

        let allow = fetch_args.allow_no_signatures()?;
        if !allow {
            return Ok(FetchInfo { signature_path: None });
        }
    }

    println!("Found {} signatures to check against", response.len());

    let mut tmp_file = tempfile::Builder::new()
        .prefix("checked-")
        .suffix(".unverified")
        .tempfile()
        .context("Could not create temporary file")?;

    let path = tmp_file.path().to_owned();

    let state = Arc::new(FetchState {
        asset_size: AtomicUsize::new(0),
        downloaded_size: AtomicUsize::new(0),
    });

    let progress_handle = tokio::task::spawn(report_progress(state.clone()));

    let run_download_handle = tokio::task::spawn({
        let fetch_url = fetch_url.clone();
        async move {
            {
                let mut writer = BufWriter::new(tmp_file.as_file_mut());
                run_download(fetch_url, &mut writer, state).await?;
            }
            anyhow::Result::<NamedTempFile>::Ok(tmp_file)
        }
    });

    let handle_err = || {
        // If the download failed, remove the temporary file
        let _ = std::fs::remove_file(&path);
        // Kill the progress bar
        progress_handle.abort();
    };

    // If the download succeeds then keep the reference to the tmp_file so it doesn't get deleted
    let _tmp_file = match run_download_handle.await {
        Err(e) => {
            println!("Download failed: {:?}", e);
            handle_err();
            return Err(anyhow::anyhow!("Download failed"));
        }
        Ok(Err(e)) => {
            println!("Download failed: {:?}", e);
            handle_err();
            return Err(anyhow::anyhow!("Download failed"));
        }
        Ok(Ok(tmp_file)) => {
            // Download ok
            tmp_file
        }
    };

    progress_handle.await??;

    println!("Downloaded to {:?}", path);

    // TODO validate the signatures here and report

    std::fs::rename(path.clone(), &output_path)?;

    let should_sign = fetch_args.sign_asset()?;
    if !should_sign {
        return Ok(FetchInfo { signature_path: None });
    }

    let signature_path = sign(SignArgs {
        name: fetch_args.name.clone(),
        password: Some(fetch_args.get_password()?),
        path: fetch_args.path.clone(),
        file: output_path,
        output: None,
    })?;

    let store_dir = get_store_dir(fetch_args.path)?;
    let vk_path = get_verification_key_path(&store_dir, &fetch_args.name);
    app_client
        .call_zome(
            ZomeCallTarget::RoleName("checked".to_string()),
            "fetch".into(),
            "create_asset_signature".into(),
            ExternIO::encode(CreateAssetSignature {
                fetch_url: fetch_args.url.clone(),
                signature: std::fs::read(&signature_path)?,
                key_type: VerificationKeyType::MiniSignEd25519,
                verification_key: std::fs::read_to_string(vk_path)?,
            })
            .unwrap(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to report signature to Holochain: {:?}", e))?;

    println!("Created signature!");

    Ok(FetchInfo {
        signature_path: Some(signature_path),
    })
}

fn get_output_path(fetch_args: &FetchArgs, fetch_url: &Url) -> anyhow::Result<PathBuf> {
    let guessed_file_name = fetch_url
        .path_segments()
        .ok_or_else(|| anyhow::anyhow!("Invalid URL"))?
        .last()
        .ok_or_else(|| anyhow::anyhow!("Invalid URL"))?;

    let output_path = match &fetch_args.output {
        Some(output) => {
            if output.is_dir() {
                output.join(guessed_file_name)
            } else {
                let mut out = output.clone();
                out.pop();
                std::fs::create_dir_all(&out)?;

                output.clone()
            }
        }
        None => {
            let mut out = std::env::current_dir()?;
            out.push(guessed_file_name);
            out
        }
    };

    Ok(output_path)
}

/// Download from `fetch_url` into `writer` and update `state` with the download progress.
async fn run_download<W>(
    fetch_url: url::Url,
    writer: &mut BufWriter<W>,
    state: Arc<FetchState>,
) -> anyhow::Result<()>
where
    W: Write,
{
    let mut response = reqwest::get(fetch_url).await?;

    let content_length = response
        .headers()
        .get("content-length")
        .and_then(|l| l.to_str().ok())
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(0);

    state
        .asset_size
        .store(content_length, std::sync::atomic::Ordering::Relaxed);

    while let Some(c) = response.chunk().await? {
        writer.write_all(&c)?;
        state
            .downloaded_size
            .fetch_add(c.len(), std::sync::atomic::Ordering::Release);
    }

    writer.flush()?;

    Ok(())
}

async fn report_progress(state: Arc<FetchState>) -> anyhow::Result<()> {
    if tokio::time::timeout(std::time::Duration::from_secs(5), async {
        while state.asset_size.load(std::sync::atomic::Ordering::Relaxed) == 0 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    })
    .await
    .is_err()
    {
        return Ok(());
    }

    let asset_size = state.asset_size.load(std::sync::atomic::Ordering::Acquire);

    let progress_bar = indicatif::ProgressBar::new(asset_size as u64)
        .with_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:40.white/magenta} {bytes}/{total_bytes} @ {bytes_per_sec} {msg}").context("Could not create progress bar style")?)
        .with_finish(ProgressFinish::WithMessage("✅".into()));

    loop {
        let downloaded = state
            .downloaded_size
            .load(std::sync::atomic::Ordering::Acquire);

        progress_bar.set_position(downloaded as u64);

        if downloaded >= asset_size {
            break;
        }

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    progress_bar.finish_using_style();

    Ok(())
}
