//! Sweettest port of the old tryorama `fetch` suite.

use checked_integration_tests::*;
use checked_types::{CreateAssetSignature, FetchCheckSignatureReason, VerificationKeyType};

fn asset_signature_req(verification_key: &str, signature: &str) -> CreateAssetSignature {
    CreateAssetSignature {
        fetch_url: "https://example.com/sample.csv".to_string(),
        signature: signature.to_string(),
        key_type: VerificationKeyType::MiniSignEd25519,
        verification_key: verification_key.to_string(),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn prepare_fetch_with_no_existing_signatures() -> anyhow::Result<()> {
    let (conductor, alice) = setup_one().await?;

    let check_signatures =
        prepare_fetch(&conductor, &alice, "https://example.com/sample.csv").await?;
    assert_eq!(check_signatures.len(), 0);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn create_asset_signature_basic() -> anyhow::Result<()> {
    let (conductor, alice) = setup_one().await?;

    let record = distribute_verification_key(
        &conductor,
        &alice,
        sample_fetch_key(),
        sample_fetch_key_proof(),
        sample_fetch_key_proof_signature(),
    )
    .await?;
    let vf_key_dist_address = record.action_address().clone();

    create_asset_signature(
        &conductor,
        &alice,
        asset_signature_req(sample_fetch_key(), sample_fetch_asset_signature()),
    )
    .await?;

    let my_asset_signatures = get_my_asset_signatures(&conductor, &alice).await?;
    assert_eq!(my_asset_signatures.len(), 1);
    assert_eq!(my_asset_signatures[0].key_dist_address, vf_key_dist_address);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn get_my_asset_signatures_per_agent() -> anyhow::Result<()> {
    let (conductor, players) = setup_players(2).await?;
    let alice = &players[0];
    let bob = &players[1];

    distribute_verification_key(
        &conductor,
        alice,
        sample_fetch_key(),
        sample_fetch_key_proof(),
        sample_fetch_key_proof_signature(),
    )
    .await?;
    distribute_verification_key(
        &conductor,
        bob,
        sample_fetch_key_other(),
        sample_fetch_key_proof(),
        sample_fetch_key_other_proof_signature(),
    )
    .await?;

    create_asset_signature(
        &conductor,
        alice,
        asset_signature_req(sample_fetch_key(), sample_fetch_asset_signature()),
    )
    .await?;
    create_asset_signature(
        &conductor,
        bob,
        asset_signature_req(
            sample_fetch_key_other(),
            sample_fetch_other_asset_signature(),
        ),
    )
    .await?;

    let alice_signatures = get_my_asset_signatures(&conductor, alice).await?;
    assert_eq!(alice_signatures.len(), 1);
    assert_eq!(
        alice_signatures[0].fetch_url,
        "https://example.com/sample.csv"
    );

    let bob_signatures = get_my_asset_signatures(&conductor, bob).await?;
    assert_eq!(bob_signatures.len(), 1);
    assert_eq!(
        bob_signatures[0].fetch_url,
        "https://example.com/sample.csv"
    );

    assert_ne!(
        alice_signatures[0].key_dist_address, bob_signatures[0].key_dist_address,
        "the signatures should hang off different verification keys"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_an_asset_signature() -> anyhow::Result<()> {
    let (conductor, alice) = setup_one().await?;

    distribute_verification_key(
        &conductor,
        &alice,
        sample_fetch_key(),
        sample_fetch_key_proof(),
        sample_fetch_key_proof_signature(),
    )
    .await?;

    create_asset_signature(
        &conductor,
        &alice,
        asset_signature_req(sample_fetch_key(), sample_fetch_asset_signature()),
    )
    .await?;

    delete_asset_signature(&conductor, &alice, "https://example.com/sample.csv").await?;

    let signatures = get_my_asset_signatures(&conductor, &alice).await?;
    assert_eq!(signatures.len(), 0);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn cannot_resign_an_asset() -> anyhow::Result<()> {
    let (conductor, alice) = setup_one().await?;

    distribute_verification_key(
        &conductor,
        &alice,
        sample_fetch_key(),
        sample_fetch_key_proof(),
        sample_fetch_key_proof_signature(),
    )
    .await?;

    create_asset_signature(
        &conductor,
        &alice,
        asset_signature_req(sample_fetch_key(), sample_fetch_asset_signature()),
    )
    .await?;
    assert_eq!(get_my_asset_signatures(&conductor, &alice).await?.len(), 1);

    let err = create_asset_signature(
        &conductor,
        &alice,
        asset_signature_req(sample_fetch_key(), sample_fetch_asset_signature()),
    )
    .await
    .unwrap_err();
    assert!(
        err.to_string()
            .contains("An asset signature with the same fetch URL already exists"),
        "unexpected error: {err}"
    );

    assert_eq!(get_my_asset_signatures(&conductor, &alice).await?.len(), 1);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn cannot_resign_an_asset_after_deleting_the_original_signature() -> anyhow::Result<()> {
    let (conductor, alice) = setup_one().await?;

    distribute_verification_key(
        &conductor,
        &alice,
        sample_fetch_key(),
        sample_fetch_key_proof(),
        sample_fetch_key_proof_signature(),
    )
    .await?;

    create_asset_signature(
        &conductor,
        &alice,
        asset_signature_req(sample_fetch_key(), sample_fetch_asset_signature()),
    )
    .await?;
    assert_eq!(get_my_asset_signatures(&conductor, &alice).await?.len(), 1);

    delete_asset_signature(&conductor, &alice, "https://example.com/sample.csv").await?;

    let err = create_asset_signature(
        &conductor,
        &alice,
        asset_signature_req(sample_fetch_key(), sample_fetch_asset_signature()),
    )
    .await
    .unwrap_err();
    assert!(
        err.to_string()
            .contains("An asset signature with the same fetch URL already exists"),
        "unexpected error: {err}"
    );

    assert_eq!(get_my_asset_signatures(&conductor, &alice).await?.len(), 0);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn signatures_from_multiple_selection_strategies() -> anyhow::Result<()> {
    let (conductor, players) = setup_players(2).await?;
    let alice = &players[0];
    let bob = &players[1];

    let alice_record = distribute_verification_key(
        &conductor,
        alice,
        sample_fetch_key(),
        sample_fetch_key_proof(),
        sample_fetch_key_proof_signature(),
    )
    .await?;
    let vf_key_dist_address_alice = alice_record.action_address().clone();

    sync(&[alice, bob]).await?;

    distribute_verification_key(
        &conductor,
        bob,
        sample_fetch_key_other(),
        sample_fetch_key_proof(),
        sample_fetch_key_other_proof_signature(),
    )
    .await?;

    create_key_collection(&conductor, bob, "bob collection").await?;
    link_verification_key_to_key_collection(
        &conductor,
        bob,
        vf_key_dist_address_alice,
        "bob collection",
    )
    .await?;

    create_asset_signature(
        &conductor,
        alice,
        asset_signature_req(sample_fetch_key(), sample_fetch_asset_signature()),
    )
    .await?;
    create_asset_signature(
        &conductor,
        bob,
        asset_signature_req(
            sample_fetch_key_other(),
            sample_fetch_other_asset_signature(),
        ),
    )
    .await?;

    sync(&[alice, bob]).await?;

    let check_signatures_alice =
        prepare_fetch(&conductor, alice, "https://example.com/sample.csv").await?;
    assert_eq!(check_signatures_alice.len(), 2);
    assert_eq!(
        check_signatures_alice[0].reason,
        FetchCheckSignatureReason::Mine
    );
    assert_eq!(
        check_signatures_alice[1].reason,
        FetchCheckSignatureReason::RandomRecent
    );

    let check_signatures_bob =
        prepare_fetch(&conductor, bob, "https://example.com/sample.csv").await?;
    assert_eq!(check_signatures_bob.len(), 2);
    assert_eq!(&check_signatures_bob[0].author, bob.agent_pub_key());
    assert_eq!(
        check_signatures_bob[0].reason,
        FetchCheckSignatureReason::Mine
    );
    assert_eq!(&check_signatures_bob[1].author, alice.agent_pub_key());
    match &check_signatures_bob[1].reason {
        FetchCheckSignatureReason::Pinned(pinned) => {
            assert_eq!(pinned.key_name, "test");
            assert_eq!(pinned.key_collection, "bob collection");
        }
        other => panic!("expected Pinned reason, got {other:?}"),
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn fetch_remote_validation() -> anyhow::Result<()> {
    let (conductor, players) = setup_players(2).await?;
    let alice = &players[0];
    let bob = &players[1];

    distribute_verification_key(
        &conductor,
        alice,
        sample_fetch_key(),
        sample_fetch_key_proof(),
        sample_fetch_key_proof_signature(),
    )
    .await?;
    distribute_verification_key(
        &conductor,
        bob,
        sample_fetch_key_other(),
        sample_fetch_key_proof(),
        sample_fetch_key_other_proof_signature(),
    )
    .await?;

    create_asset_signature(
        &conductor,
        alice,
        asset_signature_req(sample_fetch_key(), sample_fetch_asset_signature()),
    )
    .await?;
    create_asset_signature(
        &conductor,
        bob,
        asset_signature_req(
            sample_fetch_key_other(),
            sample_fetch_other_asset_signature(),
        ),
    )
    .await?;

    sync(&[alice, bob]).await?;

    delete_asset_signature(&conductor, alice, "https://example.com/sample.csv").await?;

    sync(&[alice, bob]).await?;

    let bob_signatures = get_my_asset_signatures(&conductor, bob).await?;
    assert_eq!(bob_signatures.len(), 1);

    delete_asset_signature(&conductor, bob, "https://example.com/sample.csv").await?;

    sync(&[alice, bob]).await?;
    Ok(())
}
