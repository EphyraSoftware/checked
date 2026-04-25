//! Sweettest port of the old tryorama `signing_keys` suite.
//!
//! Covers: key distribution validation, searching, marks, key collections,
//! linking, reference counting, and the remote-validation smoke checks.

use checked_integration_tests::*;
use signing_keys_types::MarkVfKeyDistOpt;

// ---------- validation-key-dist ----------

#[tokio::test(flavor = "multi_thread")]
async fn distribute_a_key() -> anyhow::Result<()> {
    let (conductor, alice) = setup_one().await?;

    let record = distribute_verification_key(
        &conductor,
        &alice,
        sample_minisign_key(),
        sample_minisign_proof(),
        sample_minisign_proof_signature(),
    )
    .await?;
    assert!(!record.action_address().get_raw_36().is_empty());
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn distribute_a_key_with_invalid_proof() -> anyhow::Result<()> {
    let (conductor, alice) = setup_one().await?;

    let err = distribute_verification_key(
        &conductor,
        &alice,
        sample_minisign_key(),
        // Changing the proof makes the signature invalid.
        &format!("{}invalid", sample_minisign_proof()),
        sample_minisign_proof_signature(),
    )
    .await
    .unwrap_err();

    assert!(
        err.to_string().contains("Failed to verify proof signature"),
        "unexpected error: {err}"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn get_my_keys() -> anyhow::Result<()> {
    let (conductor, alice) = setup_one().await?;

    distribute_verification_key(
        &conductor,
        &alice,
        sample_minisign_key(),
        sample_minisign_proof(),
        sample_minisign_proof_signature(),
    )
    .await?;

    let keys = get_my_verification_key_distributions(&conductor, &alice).await?;

    assert_eq!(keys.len(), 1);
    assert_eq!(
        keys[0].verification_key_dist.verification_key,
        sample_minisign_key().trim()
    );
    assert_eq!(keys[0].reference_count, 0);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn search_for_a_key() -> anyhow::Result<()> {
    let (conductor, players) = setup_players(2).await?;
    let alice = &players[0];
    let bob = &players[1];

    distribute_verification_key(
        &conductor,
        alice,
        sample_minisign_key(),
        sample_minisign_proof(),
        sample_minisign_proof_signature(),
    )
    .await?;

    sync(&[alice, bob]).await?;

    let responses = search_keys(&conductor, bob, alice.agent_pub_key().clone()).await?;
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0].verification_key_dist.name, "test");
    assert_eq!(
        responses[0].verification_key_dist.verification_key,
        sample_minisign_key().trim()
    );

    let offline = search_keys_local(&conductor, bob, alice.agent_pub_key().clone()).await?;
    assert_eq!(offline.len(), 1);
    assert_eq!(offline[0].verification_key_dist.name, "test");
    assert_eq!(
        offline[0].verification_key_dist.verification_key,
        sample_minisign_key().trim()
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn mark_a_key_as_compromised() -> anyhow::Result<()> {
    let (conductor, players) = setup_players(2).await?;
    let alice = &players[0];
    let bob = &players[1];

    let record = distribute_verification_key(
        &conductor,
        alice,
        sample_minisign_key(),
        sample_minisign_proof(),
        sample_minisign_proof_signature(),
    )
    .await?;
    let vf_key_dist_address = record.action_address().clone();

    sync(&[alice, bob]).await?;

    let compromised_since = holochain::prelude::Timestamp::now();
    mark_verification_key_dist(
        &conductor,
        alice,
        vf_key_dist_address.clone(),
        MarkVfKeyDistOpt::Compromised {
            note: "I think someone is using my private key!".to_string(),
            since: compromised_since,
        },
    )
    .await?;

    // Bob's search call hits the DHT, but links may need a moment — force a sync.
    sync(&[alice, bob]).await?;

    let responses = search_keys(&conductor, bob, alice.agent_pub_key().clone()).await?;
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0].verification_key_dist.marks.len(), 1);

    match &responses[0].verification_key_dist.marks[0] {
        MarkVfKeyDistOpt::Compromised { note, since } => {
            assert_eq!(note, "I think someone is using my private key!");
            assert_eq!(*since, compromised_since);
        }
        other => panic!("expected Compromised mark, got {other:?}"),
    }

    sync(&[alice, bob]).await?;
    Ok(())
}

// ---------- key-collection ----------

#[tokio::test(flavor = "multi_thread")]
async fn create_key_collection_basic() -> anyhow::Result<()> {
    let (conductor, alice) = setup_one().await?;
    create_key_collection(&conductor, &alice, "a test").await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn create_key_collection_limit() -> anyhow::Result<()> {
    let (conductor, alice) = setup_one().await?;

    for i in 0..10 {
        create_key_collection(&conductor, &alice, &format!("a test {i}")).await?;
    }

    let err = create_key_collection(&conductor, &alice, "a test too many")
        .await
        .unwrap_err();
    assert!(err.to_string().contains("maximum"), "unexpected: {err}");
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn get_my_key_collections_basic() -> anyhow::Result<()> {
    let (conductor, alice) = setup_one().await?;

    for i in 0..2 {
        create_key_collection(&conductor, &alice, &format!("a test {i}")).await?;
    }

    let collections = get_my_key_collections(&conductor, &alice).await?;
    assert_eq!(collections.len(), 2);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn link_verification_key_distribution_to_collection() -> anyhow::Result<()> {
    let (conductor, players) = setup_players(2).await?;
    let alice = &players[0];
    let bob = &players[1];

    let vf_record = distribute_verification_key(
        &conductor,
        alice,
        sample_minisign_key(),
        sample_minisign_proof(),
        sample_minisign_proof_signature(),
    )
    .await?;
    let vf_key_dist_address = vf_record.action_address().clone();

    create_key_collection(&conductor, bob, "a test").await?;

    sync(&[alice, bob]).await?;

    link_verification_key_to_key_collection(&conductor, bob, vf_key_dist_address, "a test").await?;

    let collections = get_my_key_collections(&conductor, bob).await?;
    assert_eq!(collections.len(), 1);
    assert_eq!(collections[0].verification_keys.len(), 1);
    Ok(())
}

// Delete-link validation calls `must_get_agent_activity` with `until_hash`,
// which is unreliable in sweettest with the `transport-tx5-backend-go-pion`
// backend. Holochain's own consistency tests are gated on `transport-iroh` for
// the same reason (see `holochain::sweettest::sweet_consistency` tests).
#[tokio::test(flavor = "multi_thread")]
#[ignore = "delete-link via must_get_agent_activity is flaky on tx5/go-pion; run locally with --ignored"]
async fn unlink_verification_key_from_collection() -> anyhow::Result<()> {
    let (conductor, players) = setup_players(2).await?;
    let alice = &players[0];
    let bob = &players[1];

    let vf_record = distribute_verification_key(
        &conductor,
        alice,
        sample_minisign_key(),
        sample_minisign_proof(),
        sample_minisign_proof_signature(),
    )
    .await?;
    let vf_key_dist_address = vf_record.action_address().clone();

    create_key_collection(&conductor, bob, "a test").await?;

    sync(&[alice, bob]).await?;

    with_retry(|| async {
        link_verification_key_to_key_collection(
            &conductor,
            bob,
            vf_key_dist_address.clone(),
            "a test",
        )
        .await
        .map(|_| ())
    })
    .await?;

    sync(&[alice, bob]).await?;

    with_retry(|| async {
        unlink_verification_key_from_key_collection(
            &conductor,
            bob,
            vf_key_dist_address.clone(),
            "a test",
        )
        .await
    })
    .await?;

    let collections = get_my_key_collections(&conductor, bob).await?;
    assert_eq!(collections.len(), 1);
    assert_eq!(collections[0].verification_keys.len(), 0);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "delete-link via must_get_agent_activity is flaky on tx5/go-pion; run locally with --ignored"]
async fn key_collection_remote_validation() -> anyhow::Result<()> {
    let (conductor, players) = setup_players(2).await?;
    let alice = &players[0];
    let bob = &players[1];

    for i in 0..3 {
        create_key_collection(&conductor, bob, &format!("a test {i}")).await?;
    }

    // Each sync asserts that remote validation hasn't stalled propagation.
    sync(&[alice, bob]).await?;

    let vf_record = distribute_verification_key(
        &conductor,
        alice,
        sample_minisign_key(),
        sample_minisign_proof(),
        sample_minisign_proof_signature(),
    )
    .await?;
    let vf_key_dist_address = vf_record.action_address().clone();

    sync(&[alice, bob]).await?;

    with_retry(|| async {
        link_verification_key_to_key_collection(
            &conductor,
            bob,
            vf_key_dist_address.clone(),
            "a test 1",
        )
        .await
        .map(|_| ())
    })
    .await?;

    sync(&[alice, bob]).await?;

    with_retry(|| async {
        unlink_verification_key_from_key_collection(
            &conductor,
            bob,
            vf_key_dist_address.clone(),
            "a test 1",
        )
        .await
    })
    .await?;

    sync(&[alice, bob]).await?;
    Ok(())
}

// ---------- reference-count ----------

#[tokio::test(flavor = "multi_thread")]
async fn get_my_keys_with_remote_reference() -> anyhow::Result<()> {
    let (conductor, players) = setup_players(2).await?;
    let alice = &players[0];
    let bob = &players[1];

    let record = distribute_verification_key(
        &conductor,
        alice,
        sample_minisign_key(),
        sample_minisign_proof(),
        sample_minisign_proof_signature(),
    )
    .await?;
    let vf_key_dist_address = record.action_address().clone();

    sync(&[alice, bob]).await?;

    create_key_collection(&conductor, bob, "a test").await?;
    link_verification_key_to_key_collection(&conductor, bob, vf_key_dist_address, "a test").await?;

    sync(&[alice, bob]).await?;

    let responses = get_my_verification_key_distributions(&conductor, alice).await?;
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0].reference_count, 1);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn search_for_a_key_with_remote_reference() -> anyhow::Result<()> {
    let (conductor, players) = setup_players(2).await?;
    let alice = &players[0];
    let bob = &players[1];

    let record = distribute_verification_key(
        &conductor,
        alice,
        sample_minisign_key(),
        sample_minisign_proof(),
        sample_minisign_proof_signature(),
    )
    .await?;
    let vf_key_dist_address = record.action_address().clone();

    sync(&[alice, bob]).await?;

    create_key_collection(&conductor, bob, "a test").await?;
    link_verification_key_to_key_collection(&conductor, bob, vf_key_dist_address, "a test").await?;

    sync(&[alice, bob]).await?;

    let responses = search_keys(&conductor, alice, alice.agent_pub_key().clone()).await?;
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0].reference_count, 1);
    Ok(())
}

// The old suite skipped this test in CI (GITHUB_ACTIONS). Preserve that guard.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "three-player test is flaky on CI; run locally with --ignored"]
async fn get_my_key_collections_with_multi_reference() -> anyhow::Result<()> {
    let (conductor, players) = setup_players(3).await?;
    let alice = &players[0];
    let bob = &players[1];
    let carol = &players[2];

    let record = distribute_verification_key(
        &conductor,
        alice,
        sample_minisign_key(),
        sample_minisign_proof(),
        sample_minisign_proof_signature(),
    )
    .await?;
    let vf_key_dist_address = record.action_address().clone();

    sync(&[alice, bob, carol]).await?;

    create_key_collection(&conductor, bob, "bob test").await?;
    link_verification_key_to_key_collection(
        &conductor,
        bob,
        vf_key_dist_address.clone(),
        "bob test",
    )
    .await?;

    create_key_collection(&conductor, carol, "carol test").await?;
    link_verification_key_to_key_collection(&conductor, carol, vf_key_dist_address, "carol test")
        .await?;

    sync(&[alice, bob, carol]).await?;

    let responses = get_my_key_collections(&conductor, bob).await?;
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0].name, "bob test");
    assert_eq!(responses[0].verification_keys.len(), 1);
    assert_eq!(responses[0].verification_keys[0].reference_count, 2);
    Ok(())
}
