use crate::cli::FetchArgs;
use anyhow::Context;
use indicatif::{ProgressFinish, ProgressStyle};
use std::io::{BufWriter, Write};
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

struct FetchState {
    asset_size: AtomicUsize,
    downloaded_size: AtomicUsize,
}

pub async fn fetch(fetch_args: FetchArgs) -> anyhow::Result<()> {
    let fetch_url = url::Url::parse(&fetch_args.url).context("Invalid URL")?;
    println!("Fetching from {}", fetch_url);

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

    let run_download_handle = tokio::task::spawn(async move {
        let mut writer = BufWriter::new(tmp_file.as_file_mut());
        run_download(
            fetch_url,
            &mut writer,
            state,
        ).await
    });

    let handle_err = || {
        // If the download failed, remove the temporary file
        let _ = std::fs::remove_file(&path);
        // Kill the progress bar
        progress_handle.abort();
    };

    match run_download_handle.await {
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
        _ => {
            // Download ok
        }
    }

    progress_handle.await??;

    println!("Downloaded to {:?}", path);

    Ok(())
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
    if let Err(_) = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        while state
            .asset_size
            .load(std::sync::atomic::Ordering::Relaxed)
            == 0
        {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    })
    .await
    {
        return Ok(());
    }

    let asset_size = state
        .asset_size
        .load(std::sync::atomic::Ordering::Acquire);

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
