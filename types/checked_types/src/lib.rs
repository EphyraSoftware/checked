use holochain_zome_types::prelude::{ActionHash, AgentPubKey, Timestamp};
use serde::{Deserialize, Serialize};

/// Supported key types for verification keys.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum VerificationKeyType {
    /// MiniSign verification key, using the Ed25519 algorithm. See the [minisign](https://jedisct1.github.io/minisign/) documentation for more information.
    MiniSignEd25519,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DistributeVfKeyRequest {
    pub name: String,
    pub verification_key: String,
    pub key_type: VerificationKeyType,
    pub proof: String,
    pub proof_signature: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PrepareFetchRequest {
    pub fetch_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteAssetSignatureRequest {
    pub fetch_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FetchCheckSignaturePinned {
    pub key_collection: String,
    pub key_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum FetchCheckSignatureReason {
    RandomRecent,
    RandomHistorical,
    Pinned(FetchCheckSignaturePinned),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FetchCheckSignature {
    pub signature: String,
    pub key_type: VerificationKeyType,
    pub verification_key: String,
    pub author: AgentPubKey,
    pub key_dist_address: ActionHash,
    pub reason: FetchCheckSignatureReason,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateAssetSignature {
    pub fetch_url: String,
    pub signature: String,
    pub key_type: VerificationKeyType,
    pub verification_key: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AssetSignatureResponse {
    /// The URL that the asset was fetched from.
    pub fetch_url: String,

    /// The signature of the asset.
    pub signature: String,

    /// The address of the public key that signed this asset.
    pub key_dist_address: ActionHash,

    /// When the signature was published on Holochain.
    pub created_at: Timestamp,
}
