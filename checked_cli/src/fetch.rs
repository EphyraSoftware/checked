use std::fs::File;
use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use anyhow::Context;
use holochain_client::ZomeCallTarget;
use holochain_types::prelude::{ActionHash, AgentPubKey, ExternIO};
use indicatif::{ProgressFinish, ProgressStyle};
use itertools::Itertools;
use minisign::PublicKeyBox;
use tempfile::NamedTempFile;
use url::Url;

use checked_types::{
    FetchCheckSignature, FetchCheckSignatureReason, PrepareFetchRequest, VerificationKeyType,
};

use crate::cli::FetchArgs;
use crate::hc_client;
use crate::hc_client::maybe_handle_holochain_error;
use crate::interactive::GetPassword;
use crate::prelude::SignArgs;
use crate::sign::sign;

/// Information about the result of fetching an asset.
#[derive(Debug)]
pub struct FetchInfo {
    /// The path to the fetched asset. This is only present if the user decided to keep the asset
    /// and it was copied from its temporary location to the output path.
    pub output_path: Option<PathBuf>,
    /// The results of checking existing signatures for the asset.
    pub reports: Vec<SignatureCheckReport>,
    /// The path to the signature created by the user performing the fetch. This is only present if
    /// the user decided to sign the asset and the fetch process made it far enough to create the
    /// signature.
    pub signature_path: Option<PathBuf>,
}

#[derive(Debug)]
struct FetchState {
    asset_size: AtomicUsize,
    downloaded_size: AtomicUsize,
}

/// Fetch an asset from a URL, verify a selection of signatures for it and optionally distribute
/// your own signature for it.
///
/// First attempts to find existing signatures for the asset on Holochain. A mixture of historical,
/// recent and pinned keys from the user's key collections. You can proceed even if no signatures
/// are found because this is normal, but the user is prompted to make it clear that this is the case.
///
/// Next, the file is downloaded to a temporary location. Each of the signatures are run against it
/// and a report is printed. The user is prompted to check the report and decide whether to continue.
///
/// If the decides to reject the asset then the temporary file is deleted and the process ends.
///
/// Otherwise, the file is moved to the output location and the user is prompted to sign the asset.
/// Unlike with [sign] where the user is is prompted about whether to distribute the signature, here
/// the signature is always distributed after being created.
pub async fn fetch(fetch_args: FetchArgs) -> anyhow::Result<FetchInfo> {
    let fetch_url = url::Url::parse(&fetch_args.url).context("Invalid URL")?;
    println!("Fetching from {}", fetch_url);

    let output_path = get_output_path(&fetch_args, &fetch_url)?;

    let admin_port = fetch_args.admin_port().await?;

    let mut app_client = hc_client::get_authenticated_app_agent_client(
        admin_port,
        fetch_args.config_dir.clone(),
        fetch_args.app_id.clone(),
    )
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
            maybe_handle_holochain_error(&e, fetch_args.config_dir.clone());
            anyhow::anyhow!("Failed to get signatures for the asset: {:?}", e)
        })?;

    let response: Vec<FetchCheckSignature> = response.decode()?;

    if response.is_empty() {
        println!("No signatures found for this asset. This is normal but please consider asking the author to create a signature!");

        let allow = fetch_args.allow_no_signatures()?;
        if !allow {
            return Ok(FetchInfo {
                output_path: None,
                signature_path: None,
                reports: vec![],
            });
        }
    } else {
        println!("Found {} signatures to check against", response.len());
    }

    let has_mine_signature = response
        .iter()
        .any(|s| s.reason == FetchCheckSignatureReason::Mine);

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
            // Only retain the file if the download was successful, otherwise it will be deleted
            // when tmp_file goes out of scope
            anyhow::Result::<NamedTempFile>::Ok(tmp_file)
        }
    });

    let handle_err = || {
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

    // No point running the check and report if there are no signatures
    let reports = if !response.is_empty() {
        let reports = check_signatures(path.clone(), response)?;
        show_report(&reports);

        if !fetch_args.approve_signatures_report()? {
            println!("Discarding temporary asset...");
            std::fs::remove_file(path.clone())?;

            println!("Done");
            return Ok(FetchInfo {
                output_path: None,
                signature_path: None,
                reports,
            });
        }

        reports
    } else {
        vec![]
    };

    std::fs::rename(path.clone(), &output_path)?;

    let should_sign = !has_mine_signature && fetch_args.sign_asset()?;
    if !should_sign {
        return Ok(FetchInfo {
            output_path: Some(output_path),
            signature_path: None,
            reports,
        });
    }

    let signature_path = sign(SignArgs {
        url: Some(fetch_args.url.clone()),
        name: fetch_args.name.clone(),
        port: Some(admin_port),
        password: Some(fetch_args.get_password()?),
        config_dir: fetch_args.config_dir.clone(),
        file: output_path.clone(),
        output: None,
        distribute: true,
        app_id: fetch_args.app_id,
    })
    .await?;

    println!("Created signature!");

    Ok(FetchInfo {
        output_path: Some(output_path),
        signature_path: Some(signature_path),
        reports,
    })
}

#[derive(Debug)]
pub struct CheckedSignature {
    pub key_dist_address: ActionHash,
    pub author: AgentPubKey,
}

#[derive(Debug)]
pub struct SignatureCheckReport {
    pub reason: FetchCheckSignatureReason,
    pub passed_signatures: Vec<CheckedSignature>,
    pub failed_signatures: Vec<CheckedSignature>,
}

fn check_signatures(
    check_file: PathBuf,
    signatures: Vec<FetchCheckSignature>,
) -> anyhow::Result<Vec<SignatureCheckReport>> {
    let check_file = File::options().read(true).open(check_file)?;
    let mut check_file_reader = BufReader::new(check_file);

    let mut signature_reports = Vec::new();
    for (reason, sigs) in signatures.iter().group_by(|s| s.reason.clone()).into_iter() {
        let mut group_report = SignatureCheckReport {
            reason,
            passed_signatures: vec![],
            failed_signatures: vec![],
        };
        for sig in sigs {
            println!("Checking signature from {:?}... ", sig.author);
            match check_one_signature(&mut check_file_reader, sig) {
                Ok(true) => group_report.passed_signatures.push(CheckedSignature {
                    key_dist_address: sig.key_dist_address.clone(),
                    author: sig.author.clone(),
                }),
                Ok(false) => group_report.failed_signatures.push(CheckedSignature {
                    key_dist_address: sig.key_dist_address.clone(),
                    author: sig.author.clone(),
                }),
                Err(e) => {
                    println!("Error during verification: {:?}", e);
                    group_report.failed_signatures.push(CheckedSignature {
                        key_dist_address: sig.key_dist_address.clone(),
                        author: sig.author.clone(),
                    })
                }
            }
            check_file_reader.seek(SeekFrom::Start(0))?;
        }
        signature_reports.push(group_report);
    }

    Ok(signature_reports)
}

fn check_one_signature(
    check_file_reader: &mut BufReader<File>,
    sig: &FetchCheckSignature,
) -> anyhow::Result<bool> {
    match sig.key_type {
        VerificationKeyType::MiniSignEd25519 => {
            let vf_key = PublicKeyBox::from_string(&sig.verification_key)?;
            let sig = minisign::SignatureBox::from_string(&sig.signature)?;

            match minisign::verify(&vf_key.into(), &sig, check_file_reader, true, false, false) {
                Ok(()) => Ok(true),
                Err(_) => Ok(false),
            }
        }
    }
}

fn show_report(report: &[SignatureCheckReport]) {
    println!("\nLooking for existing signature:");
    let maybe_mine_report = report
        .iter()
        .find(|r| r.reason == FetchCheckSignatureReason::Mine);
    if let Some(mine_report) = maybe_mine_report {
        // Always only 1 so no else case required
        if !mine_report.passed_signatures.is_empty() && mine_report.failed_signatures.is_empty() {
            println!("Your signature passed verification. This means you have fetched this asset before and got the same content.");
        } else if mine_report.passed_signatures.is_empty()
            && !mine_report.failed_signatures.is_empty()
        {
            println!("Your signature failed verification. This is very likely to mean that the asset you have fetched is different to the one you got previously.");
        }
    } else {
        println!("No signature from you was found.");
    }

    println!("\nLooking for historical signatures:");
    let maybe_historical_report = report
        .iter()
        .find(|r| r.reason == FetchCheckSignatureReason::RandomHistorical);
    if let Some(historical_report) = maybe_historical_report {
        if !historical_report.passed_signatures.is_empty()
            && historical_report.failed_signatures.is_empty()
        {
            println!("{} historical signature{} passed verification. This means that you are likely to have the same asset that was originally published.", historical_report.passed_signatures.len(), if historical_report.passed_signatures.len() == 1 { "" } else { "s" });
        } else if historical_report.passed_signatures.is_empty()
            && !historical_report.failed_signatures.is_empty()
        {
            println!("{} historical signature{} failed verification. This means that you may not have the same asset that was originally published.", historical_report.failed_signatures.len(), if historical_report.failed_signatures.len() == 1 { "" } else { "s" });
        } else {
            println!("{}/{} historical signatures failed verification. Inconsistent signatures do not mean that the asset you have fetched is valid or invalid but provides you with a piece of information you can use in making a judgement for yourself.", historical_report.passed_signatures.len(), historical_report.passed_signatures.len() + historical_report.failed_signatures.len());
        }
    } else {
        println!("No historical signatures were found.");
    }

    println!("\nLooking for signatures from pinned keys:");
    let maybe_pinned_report = report
        .iter()
        .find(|r| matches!(r.reason, FetchCheckSignatureReason::Pinned(_)));
    if let Some(pinned_report) = maybe_pinned_report {
        for checked_sig in &pinned_report.passed_signatures {
            println!(
                "Signature from author {:?} with key {:?}: ✅",
                checked_sig.author, checked_sig.key_dist_address
            );
        }
        for checked_sig in &pinned_report.failed_signatures {
            println!(
                "Signature from author {:?} with key {:?}: ❌",
                checked_sig.author, checked_sig.key_dist_address
            );
        }

        if !pinned_report.passed_signatures.is_empty() && pinned_report.failed_signatures.is_empty()
        {
            println!("{} pinned signature{} passed verification. This means that the asset you have fetched is likely to be the same as other pinned signatories are seeing.", pinned_report.passed_signatures.len(), if pinned_report.passed_signatures.len() == 1 { "" } else { "s" });
        } else if pinned_report.passed_signatures.is_empty()
            && !pinned_report.failed_signatures.is_empty()
        {
            println!("{} pinned signature{} failed verification. This means that the asset you have fetched is likely not the same as other pinned signatories are seeing.", pinned_report.failed_signatures.len(), if pinned_report.failed_signatures.len() == 1 { "" } else { "s" });
        } else {
            println!("{}/{} pinned signatures failed verification. Please ensure that your key collections only contain keys from signatories you trust. If you are happy with your pinned keys then consider contacting the author to see if you have received different assets.", pinned_report.passed_signatures.len(), pinned_report.passed_signatures.len() + pinned_report.failed_signatures.len());
        }
    } else {
        println!("No pinned signatures were found.");
    }

    println!("\nLooking for recent signatures:");
    let maybe_recent_report = report
        .iter()
        .find(|r| r.reason == FetchCheckSignatureReason::RandomRecent);
    if let Some(recent_report) = maybe_recent_report {
        if !recent_report.passed_signatures.is_empty() && recent_report.failed_signatures.is_empty()
        {
            println!("{} recent signature{} passed verification. This means that the asset you have fetched is likely to be the same as the one that others have been getting recently.", recent_report.passed_signatures.len(), if recent_report.passed_signatures.len() == 1 { "" } else { "s" });
        } else if recent_report.passed_signatures.is_empty()
            && !recent_report.failed_signatures.is_empty()
        {
            println!("{} recent signature{} failed verification. This means that the asset you have fetched is likely not the same as the one that others have been getting recently.", recent_report.failed_signatures.len(), if recent_report.failed_signatures.len() == 1 { "" } else { "s" });
        } else {
            println!("{}/{} recent signatures failed verification. Inconsistent signatures do not mean that the asset you have fetched is valid or invalid but provides you with a piece of information you can use in making a judgement for yourself.", recent_report.passed_signatures.len(), recent_report.passed_signatures.len() + recent_report.failed_signatures.len());
        }
    } else {
        println!("No recent signatures were found.");
    }

    println!();
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
                if !out.pop() {
                    anyhow::bail!("Output path does not have a parent directory");
                }
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
    fetch_url: Url,
    writer: &mut BufWriter<W>,
    state: Arc<FetchState>,
) -> anyhow::Result<()>
where
    W: Write,
{
    let mut response = reqwest::get(fetch_url).await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to fetch asset: {:?}", response.status());
    }

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
