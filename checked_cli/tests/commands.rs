use checked_cli::prelude::{generate, GenerateArgs};

// Generate a signing keypair, do not distribute
#[tokio::test(flavor = "multi_thread")]
async fn test_generate() -> anyhow::Result<()> {
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
