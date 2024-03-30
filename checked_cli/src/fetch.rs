use crate::cli::FetchArgs;
use anyhow::Context;

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

    while let Some(c) = res.chunk()? {
        tmp_file.as_file_mut().write_all(&c)?;
    }

    println!("Did initial request");

    let content = res.bytes().await?;

    println!("Reading body");

    std::io::copy(&mut content.as_ref(), &mut tmp_file.as_file())?;

    println!("Downloaded to {:?}", tmp_file.path());

    Ok(())
}
