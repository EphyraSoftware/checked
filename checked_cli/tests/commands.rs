use checked_cli::cli::VerifyArgs;
use checked_cli::prelude::{generate, GenerateArgs, SignArgs};
use checked_cli::sign::sign;
use checked_cli::verify::verify;
use std::fs::File;
use std::io::Write;

// Generate a signing keypair, do not distribute
#[tokio::test(flavor = "multi_thread")]
async fn generate_signing_keypair() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;

    let info = generate(GenerateArgs {
        name: "test_generate".to_string(),
        port: None,
        password: Some("test".to_string()),
        distribute: Some(false),
        path: Some(dir.as_ref().to_path_buf()),
    })
    .await?;

    assert!(info.sk_path.exists());
    assert!(info.vk_path.exists());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn sign_file() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;

    let name = "test_generate".to_string();
    generate(GenerateArgs {
        name: name.clone(),
        port: None,
        password: Some("test".to_string()),
        distribute: Some(false),
        path: Some(dir.as_ref().to_path_buf()),
    })
    .await?;

    let test_file = dir.path().join("test.txt");
    File::options()
        .write(true)
        .create_new(true)
        .open(&test_file)?
        .write_all(b"test")?;

    let sig_path = sign(SignArgs {
        name: name.clone(),
        password: Some("test".to_string()),
        path: Some(dir.as_ref().to_path_buf()),
        file: test_file.clone(),
        output: None,
    })?;

    assert!(sig_path.exists());
    assert_eq!(
        test_file.to_str().unwrap().to_string() + ".minisig",
        sig_path.to_str().unwrap()
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn verify_signed_file() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;

    let name = "test_generate".to_string();
    let info = generate(GenerateArgs {
        name: name.clone(),
        port: None,
        password: Some("test".to_string()),
        distribute: Some(false),
        path: Some(dir.as_ref().to_path_buf()),
    })
    .await?;

    let test_file = dir.path().join("test.txt");
    File::options()
        .write(true)
        .create_new(true)
        .open(&test_file)?
        .write_all(b"test")?;

    sign(SignArgs {
        name: name.clone(),
        password: Some("test".to_string()),
        path: Some(dir.as_ref().to_path_buf()),
        file: test_file.clone(),
        output: None,
    })?;

    verify(VerifyArgs {
        file: test_file,
        verification_key: info.vk_path,
        signature: None,
    })?;

    Ok(())
}
