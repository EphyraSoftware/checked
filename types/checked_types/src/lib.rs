use serde::{Serialize, Deserialize};
use holochain_zome_types::prelude::AgentPubKey;

/// Supported key types for verification keys.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum VerificationKeyType {
    /// MiniSign verification key, using the Ed25519 algorithm. See the [minisign](https://jedisct1.github.io/minisign/) documentation for more information.
    MiniSignEd25519,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct PrepareFetchRequest {
    pub fetch_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FetchCheckSignaturePinned {
    pub author: AgentPubKey,
    pub key_collection: String,
    pub key_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum FetchCheckSignatureReason {
    RandomRecent,
    RandomHistorical,
    Pinned(FetchCheckSignaturePinned),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FetchCheckSignature {
    pub signature: Vec<u8>,
    pub reason: FetchCheckSignatureReason,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateAssetSignature {
    pub fetch_url: String,
    pub signature: Vec<u8>,
    pub key_type: VerificationKeyType,
    pub verification_key: String,
}
