use serde::{Serialize, Deserialize};
use holochain_zome_types::prelude::AgentPubKey;

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
