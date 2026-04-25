//! Shared helpers for the sweettest integration tests.
//!
//! These mirror the helpers that the old tryorama JS tests carried: sample
//! verification keys/proofs, zome wrappers, and a multi-agent setup utility
//! that loads the `checked` DNA and installs it on a batch of conductors.

use anyhow::{Context, Result};
use checked_types::{
    AssetSignatureResponse, CreateAssetSignature, DeleteAssetSignatureRequest,
    DistributeVfKeyRequest, FetchCheckSignature, PrepareFetchRequest, VerificationKeyType,
};
use holochain::prelude::{ActionHash, AgentPubKey, DnaFile, Record};
use holochain::sweettest::{
    SweetAgents, SweetCell, SweetConductor, SweetDnaFile, SweetZome, await_consistency,
};
use serde::{Deserialize, Serialize};
use signing_keys_types::{KeyCollection, KeyCollectionWithKeys, MarkVfKeyDistOpt, VfKeyResponse};
use std::path::PathBuf;

/// DNA bundle path (relative to the tests crate). `npm run build:happ` creates
/// this by packing the zomes + `dna.yaml` under `dnas/checked/workdir`.
pub const DNA_PATH: &str = "../dnas/checked/workdir/checked.dna";

/// Role name from `workdir/happ.yaml` — the sweettest cell is found by this
/// role.
pub const CHECKED_ROLE: &str = "checked";

/// A handle to a provisioned cell with convenience accessors for the two
/// coordinator zomes the tests call into.
#[derive(Debug, Clone)]
pub struct CheckedPlayer {
    pub cell: SweetCell,
    pub signing_keys: SweetZome,
    pub fetch: SweetZome,
}

impl CheckedPlayer {
    pub fn agent_pub_key(&self) -> &AgentPubKey {
        self.cell.agent_pubkey()
    }
}

fn player_from_cell(cell: SweetCell) -> CheckedPlayer {
    let signing_keys = cell.zome("signing_keys");
    let fetch = cell.zome("fetch");
    CheckedPlayer {
        cell,
        signing_keys,
        fetch,
    }
}

async fn load_dna() -> Result<DnaFile> {
    SweetDnaFile::from_bundle(&PathBuf::from(DNA_PATH))
        .await
        .context("load checked DNA bundle")
}

/// Set up a single conductor with the checked hApp installed.
pub async fn setup_one() -> Result<(SweetConductor, CheckedPlayer)> {
    let dna = load_dna().await?;
    let mut conductor = SweetConductor::from_standard_config().await;
    let app = conductor
        .setup_app("checked", &[(CHECKED_ROLE.into(), dna)])
        .await
        .context("setup_app")?;
    let cell = app.cells().first().context("app has no cells")?.clone();
    Ok((conductor, player_from_cell(cell)))
}

/// Set up `n` agents on a single in-process conductor, each with their own
/// installed hApp. Mirrors the pattern used by the CLI integration tests. A
/// single conductor provides a locally-consistent DHT so multi-agent tests
/// don't race with the in-process network backend.
pub async fn setup_players(n: usize) -> Result<(SweetConductor, Vec<CheckedPlayer>)> {
    let dna = load_dna().await?;
    let mut conductor = SweetConductor::from_standard_config().await;
    let agents = SweetAgents::get(conductor.keystore().clone(), n).await;

    let apps = conductor
        .setup_app_for_agents("checked-", &agents, &[(CHECKED_ROLE.into(), dna)])
        .await
        .context("setup_app_for_agents")?;
    let players = apps
        .cells_flattened()
        .into_iter()
        .map(player_from_cell)
        .collect();
    Ok((conductor, players))
}

/// Wait for the DHT to be consistent across the given players.
pub async fn sync(players: &[&CheckedPlayer]) -> Result<()> {
    let cells: Vec<&SweetCell> = players.iter().map(|p| &p.cell).collect();
    await_consistency(cells)
        .await
        .map_err(|e| anyhow::anyhow!("dht consistency timed out: {e}"))
}

/// Retry a fallible async operation while its error string looks like a
/// transient DHT lookup failure. Sweettest's in-process network can surface
/// `DepMissingFromDht` even shortly after `await_consistency` returns if the
/// cascade hasn't yet warmed its caches; production callers would observe the
/// conductor's own retry loop swallow these.
pub async fn with_retry<F, Fut, T>(mut op: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    const MAX_ATTEMPTS: usize = 10;
    const DELAY_MS: u64 = 500;

    let mut last_err = None;
    for _ in 0..MAX_ATTEMPTS {
        match op().await {
            Ok(value) => return Ok(value),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("DepMissingFromDht") || msg.contains("may be retried") {
                    last_err = Some(e);
                    tokio::time::sleep(std::time::Duration::from_millis(DELAY_MS)).await;
                    continue;
                }
                return Err(e);
            }
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("retry loop exhausted with no error")))
}

// ---------- signing_keys zome wrappers ----------

pub async fn distribute_verification_key(
    conductor: &SweetConductor,
    player: &CheckedPlayer,
    verification_key: &str,
    proof: &str,
    proof_signature: Vec<u8>,
) -> Result<Record> {
    conductor
        .call_fallible(
            &player.signing_keys,
            "distribute_verification_key",
            DistributeVfKeyRequest {
                name: "test".to_string(),
                verification_key: verification_key.to_string(),
                key_type: VerificationKeyType::MiniSignEd25519,
                proof: proof.to_string(),
                proof_signature,
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn create_key_collection(
    conductor: &SweetConductor,
    player: &CheckedPlayer,
    name: &str,
) -> Result<Record> {
    conductor
        .call_fallible(
            &player.signing_keys,
            "create_key_collection",
            KeyCollection {
                name: name.to_string(),
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn get_my_key_collections(
    conductor: &SweetConductor,
    player: &CheckedPlayer,
) -> Result<Vec<KeyCollectionWithKeys>> {
    conductor
        .call_fallible(&player.signing_keys, "get_my_key_collections", ())
        .await
        .map_err(Into::into)
}

pub async fn link_verification_key_to_key_collection(
    conductor: &SweetConductor,
    player: &CheckedPlayer,
    verification_key_dist_address: ActionHash,
    key_collection_name: &str,
) -> Result<ActionHash> {
    conductor
        .call_fallible(
            &player.signing_keys,
            "link_verification_key_to_key_collection",
            LinkVfKeyDistToKeyCollectionRequest {
                verification_key_dist_address,
                key_collection_name: key_collection_name.to_string(),
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn unlink_verification_key_from_key_collection(
    conductor: &SweetConductor,
    player: &CheckedPlayer,
    verification_key_dist_address: ActionHash,
    key_collection_name: &str,
) -> Result<()> {
    conductor
        .call_fallible(
            &player.signing_keys,
            "unlink_verification_key_from_key_collection",
            UnlinkVfKeyFromKeyCollectionRequest {
                verification_key_dist_address,
                key_collection_name: key_collection_name.to_string(),
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn get_my_verification_key_distributions(
    conductor: &SweetConductor,
    player: &CheckedPlayer,
) -> Result<Vec<VfKeyResponse>> {
    conductor
        .call_fallible(
            &player.signing_keys,
            "get_my_verification_key_distributions",
            (),
        )
        .await
        .map_err(Into::into)
}

pub async fn search_keys(
    conductor: &SweetConductor,
    player: &CheckedPlayer,
    agent_pub_key: AgentPubKey,
) -> Result<Vec<VfKeyResponse>> {
    conductor
        .call_fallible(
            &player.signing_keys,
            "search_keys",
            SearchKeysRequest {
                agent_pub_key: Some(agent_pub_key),
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn search_keys_local(
    conductor: &SweetConductor,
    player: &CheckedPlayer,
    agent_pub_key: AgentPubKey,
) -> Result<Vec<VfKeyResponse>> {
    conductor
        .call_fallible(
            &player.signing_keys,
            "search_keys_local",
            SearchKeysRequest {
                agent_pub_key: Some(agent_pub_key),
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn mark_verification_key_dist(
    conductor: &SweetConductor,
    player: &CheckedPlayer,
    verification_key_dist_address: ActionHash,
    mark: MarkVfKeyDistOpt,
) -> Result<ActionHash> {
    conductor
        .call_fallible(
            &player.signing_keys,
            "mark_verification_key_dist",
            MarkVfKeyDistRequest {
                verification_key_dist_address,
                mark,
            },
        )
        .await
        .map_err(Into::into)
}

// ---------- fetch zome wrappers ----------

pub async fn prepare_fetch(
    conductor: &SweetConductor,
    player: &CheckedPlayer,
    fetch_url: &str,
) -> Result<Vec<FetchCheckSignature>> {
    conductor
        .call_fallible(
            &player.fetch,
            "prepare_fetch",
            PrepareFetchRequest {
                fetch_url: fetch_url.to_string(),
            },
        )
        .await
        .map_err(Into::into)
}

pub async fn create_asset_signature(
    conductor: &SweetConductor,
    player: &CheckedPlayer,
    request: CreateAssetSignature,
) -> Result<ActionHash> {
    conductor
        .call_fallible(&player.fetch, "create_asset_signature", request)
        .await
        .map_err(Into::into)
}

pub async fn get_my_asset_signatures(
    conductor: &SweetConductor,
    player: &CheckedPlayer,
) -> Result<Vec<AssetSignatureResponse>> {
    conductor
        .call_fallible(&player.fetch, "get_my_asset_signatures", ())
        .await
        .map_err(Into::into)
}

pub async fn delete_asset_signature(
    conductor: &SweetConductor,
    player: &CheckedPlayer,
    fetch_url: &str,
) -> Result<()> {
    conductor
        .call_fallible(
            &player.fetch,
            "delete_asset_signature",
            DeleteAssetSignatureRequest {
                fetch_url: fetch_url.to_string(),
            },
        )
        .await
        .map_err(Into::into)
}

// ---------- Local request mirrors ----------
//
// These are declared privately inside the coordinator zomes (not exported from
// a shared types crate), so we re-declare them here with the same field shape
// so they serialise identically.

#[derive(Serialize, Deserialize, Debug, Clone)]
struct LinkVfKeyDistToKeyCollectionRequest {
    verification_key_dist_address: ActionHash,
    key_collection_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct UnlinkVfKeyFromKeyCollectionRequest {
    verification_key_dist_address: ActionHash,
    key_collection_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SearchKeysRequest {
    agent_pub_key: Option<AgentPubKey>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MarkVfKeyDistRequest {
    verification_key_dist_address: ActionHash,
    mark: MarkVfKeyDistOpt,
}

// ---------- Sample data (ported verbatim from the old tryorama suite) ----------

pub fn sample_minisign_key() -> &'static str {
    "\nuntrusted comment: minisign public key 5DDF4BB342787FB5\nRWS1f3hCs0vfXeaPCLyiQt9NDQ+MzReDNLz+kaw+hK9NV8nb9G7opa7q\n"
}

pub fn sample_minisign_proof() -> &'static str {
    "some test data\n"
}

pub fn sample_minisign_proof_signature() -> Vec<u8> {
    b"untrusted comment: signature from minisign secret key\nRUS1f3hCs0vfXb4ExmkOtLWNkqaPkEyzEIRrcmHWyoJuSMUR3U7jx08hri3cr8EYyBNVnH1LOSdjY3Hfk2BQU15jMD25ub5sBAU=\ntrusted comment: timestamp:1709423483\tfile:test.txt\thashed\nGjpn4nbsrDPysp3Nl63GZO5YWaB0aiJljBlUOQWIYE6tgUL7inOyiYcx5EWb2yOKvwbIjRk3u0ShhgqBIwM7Dg==\n"
        .to_vec()
}

pub fn sample_minisign_key_2() -> &'static str {
    "\nuntrusted comment: minisign public key 20AFFC89E77E7A09\nRWQJen7nifyvIJgut8+D3v1aV+khU16lvgTWkz85fwM6wGKxGREH6jSh\n"
}

pub fn sample_minisign_proof_2() -> &'static str {
    "some other test data"
}

pub fn sample_minisign_proof_signature_2() -> Vec<u8> {
    b"untrusted comment: signature from minisign secret key\nRUQJen7nifyvIAFpF3HuKaf34HochzUTI0lquynL1q+UdDIsdnI73D7n5sLkynkcfUbxjHcj1Jgrxl0kyC6ftEdD5VWpi6uadw0=\ntrusted comment: timestamp:1709495926\tfile:test.txt\thashed\nZ9BIZ44mZzediyne0UqMhzz4wnKMINUGKp/gL0g5rixo+N+4mAbfK4caAoBWVVMRq172jw5EmKCYiaorK72uCQ==\n"
        .to_vec()
}

pub fn sample_fetch_key() -> &'static str {
    "\nuntrusted comment: minisign public key: B7ED5BB003A859F1\nRWTxWagDsFvtt+e4V5KJAahwTh381E6PMTFvgvGYsnWLXCwIxe4YE/sM\n"
}

pub fn sample_fetch_key_other() -> &'static str {
    "\nuntrusted comment: minisign public key: F6B62CB533B02416\nRWQWJLAztSy29s6SlUzvA2lV3X7+CMJRSeFQSjvW/xwetDEQi5arNq/J\n"
}

pub fn sample_fetch_key_proof() -> &'static str {
    "proof\n"
}

pub fn sample_fetch_key_proof_signature() -> Vec<u8> {
    b"untrusted comment: signature from rsign secret key\nRUTxWagDsFvtt0iU8ObmWDVb70Bzh8xJ3oaJ0bADndtOdFpf9LmsPlZlcnqfQ7kVnFNE8T8phqo2ieK/L/ajPt5kRwYInCoWSQE=\ntrusted comment: timestamp:1711592643\tfile:proof.txt\tprehashed\n1Qj9ogn/ieaUlJVXD6m3E88EeytS9fmJlh3Phcvq9yxI/eEmpiNH/culPFxAEXEXRwJ9jPmU0tqdoFOX3wU4Bw==\n"
        .to_vec()
}

pub fn sample_fetch_key_other_proof_signature() -> Vec<u8> {
    b"untrusted comment: signature from rsign secret key\nRUQWJLAztSy29u8kzptxH22hL+snYIoV9OHdggAmqdQOCUZpaFc9awKHR//VA/6w7iTCiK07U/MhXCwbJoVffobj+EIp4JovkAY=\ntrusted comment: timestamp:1711770738\tfile:proof.txt\tprehashed\nzvqpLC4ZziJ5Z8DRtugyjkDn/oHtKu6o71acc0H9dnCmpJVdYsyXxVcvKCCUSNTe7dQDm3Dc7LGvsgYXk0UWBg==\n"
        .to_vec()
}

pub fn sample_fetch_asset_signature() -> &'static str {
    "untrusted comment: signature from rsign secret key\nRUTxWagDsFvtt7zP24HuGOxNlWk93OYpP9dJJ3k6y+ZEQ7Ym56Loy5/KusNLnWHi0PCOB5ore+kK+wfImf+ZwFewvLuBPE3tYAU=\ntrusted comment: timestamp:1711592494\tfile:sample-asset.txt\tprehashed\nTlTxfjqY85HlrHFPjOgCmhFIKfH5Jz2MoE+zJMju9iLJ150Zdidzm3ucRsk4wU3B/OEifaL6clxmJGSWZ3bcAg==\n"
}

pub fn sample_fetch_other_asset_signature() -> &'static str {
    "untrusted comment: signature from rsign secret key\nRUQWJLAztSy29pIrQ81/cjlDxoU4docg3ox7hU241qV4G4IApnBMBLuk6mkrWWk4eQPF1IdVkL78y0yLepHpXerf4aJzuWIPPg0=\ntrusted comment: timestamp:1711757915\tfile:sample-asset.txt\tprehashed\nnmLg2TZIzWELS+DY87/dd/SFzhqrE6LCww5+prCeoa4aNJVKeNLAwNcj3NBF9PmRcXMWb6Adw1UL2MUvL+NzBw==\n"
}
