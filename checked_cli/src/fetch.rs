use std::io::{BufWriter, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, AtomicUsize};
use crate::cli::FetchArgs;
use anyhow::Context;

struct FetchState {
    asset_size: AtomicUsize,
    downloaded_size: AtomicI64,
}

pub async fn fetch(fetch_args: FetchArgs) -> anyhow::Result<()> {
    let fetch_url = url::Url::parse(&fetch_args.url).context("Invalid URL")?;
    let tmp_file = tempfile::Builder::new()
        .prefix("checked-")
        .suffix(".unverified")
        .tempfile()
        .context("Could not create temporary file")?;

    let client = reqwest::Client::new();

    let resp = client
        .head(fetch_url.clone())
        .send()
        .await?
        .error_for_status()?;

    let content_length = resp
        .headers()
        .get("content-length")
        .and_then(|l| l.to_str().ok())
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(0);

    println!("Content length: {}", content_length);

    let mut res = reqwest::get(fetch_args.url).await?;

    let mut writer = BufWriter::new(tmp_file.as_file());
    while let Some(c) = res.chunk().await? {
        writer.write_all(&c)?;
    }

    println!("Did initial request");

    let content = res.bytes().await?;

    println!("Reading body");

    std::io::copy(&mut content.as_ref(), &mut tmp_file.as_file())?;

    println!("Downloaded to {:?}", tmp_file.path());

    Ok(())
}

async fn run_download(fetch_url: String, state: Arc<FetchState>) {

}
