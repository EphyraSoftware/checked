use crate::{convert_to_app_entry_type, key_collection::get_key_collections_reference_count};
use chrono::{DateTime, Utc};
use hdk::prelude::*;
use trusted_integrity::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone, SerializedBytes)]
pub struct DistributeVfKeyRequest {
    pub name: String,
    pub verification_key: String,
    pub key_type: VerificationKeyType,
    pub proof: String,
    pub proof_signature: Vec<u8>,
}

#[hdk_extern]
pub fn distribute_verification_key(request: DistributeVfKeyRequest) -> ExternResult<Record> {
    // The key will be checked by validation but we need to hash it here so we can link it, so ensure that is
    // is in a reasonably canonical form. I.e. no leading/trailing blank characters and parses as a public key.
    let checked_vf_key = checked_vf_key(&request.verification_key, &request.key_type)?;

    verify_key_not_distributed_by_me(&checked_vf_key, &request.key_type)?;
    try_verify_key_not_distributed_by_somebody_else(&checked_vf_key)?;

    let vf_key_dist_action_hash =
        create_entry(EntryTypes::VerificationKeyDist(VerificationKeyDist {
            verification_key: checked_vf_key,
            key_type: request.key_type,
            proof: request.proof,
            proof_signature: request.proof_signature,
            name: request.name,
            // Not supported by MiniSign, ignore for now.
            expires_at: None,
        }))?;

    let record = get(vf_key_dist_action_hash.clone(), GetOptions::content())?.ok_or(
        wasm_error!(WasmErrorInner::Guest(
            "Could not find the newly created VerificationKeyDist".to_string()
        )),
    )?;

    let entry_hash = record
        .action()
        .entry_hash()
        .ok_or_else(|| wasm_error!(WasmErrorInner::Guest(String::from("Missing entry hash"))))?;

    create_vf_key_dist_discovery_links(entry_hash)?;

    Ok(record)
}

/// Reduced form of [VerificationKeyDist] to avoid returning fields that shouldn't be needed by the caller.
#[derive(Serialize, Deserialize, Debug, Clone, SerializedBytes)]
pub struct VerificationKeyDistResponse {
    pub verification_key: String,
    pub key_type: VerificationKeyType,
    pub name: String,
    pub expires_at: Option<DateTime<Utc>>,
}

impl From<VerificationKeyDist> for VerificationKeyDistResponse {
    fn from(vf_key_dist: VerificationKeyDist) -> Self {
        Self {
            verification_key: vf_key_dist.verification_key,
            key_type: vf_key_dist.key_type,
            name: vf_key_dist.name,
            expires_at: vf_key_dist.expires_at,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, SerializedBytes)]
pub struct VfKeyResponse {
    pub verification_key_dist: VerificationKeyDistResponse,
    pub key_dist_address: EntryHash,
    pub reference_count: usize,
    pub created_at: Timestamp,
}

#[hdk_extern]
pub fn get_my_verification_key_distributions(_: ()) -> ExternResult<Vec<VfKeyResponse>> {
    let q = ChainQueryFilter::default()
        .action_type(ActionType::Create)
        .entry_type(EntryType::App(
            UnitEntryTypes::VerificationKeyDist.try_into()?,
        ))
        .include_entries(true)
        .ascending();

    let vf_key_dist_entries = query(q)?;

    let mut out = Vec::with_capacity(vf_key_dist_entries.len());
    for r in vf_key_dist_entries.into_iter() {
        let created_at = r.action().timestamp();
        let key_dist_address = r
            .action()
            .entry_hash()
            .ok_or_else(|| {
                wasm_error!(WasmErrorInner::Guest(
                    "Missing entry hash for VerificationKeyDist".to_string()
                ))
            })?
            .clone();
        let vf_key_dist: VerificationKeyDist = convert_to_app_entry_type(r)?;
        let reference_count =
            get_key_collections_reference_count(key_dist_address.clone(), &GetOptions::content())?;
        out.push(VfKeyResponse {
            verification_key_dist: vf_key_dist.into(),
            key_dist_address,
            reference_count,
            created_at,
        });
    }

    Ok(out)
}

#[derive(Serialize, Deserialize, Debug, Clone, SerializedBytes)]
pub struct SearchKeysRequest {
    pub agent_pub_key: Option<AgentPubKey>,
}

#[hdk_extern]
pub fn search_keys(request: SearchKeysRequest) -> ExternResult<Vec<VfKeyResponse>> {
    search_keys_with_get_options(request, GetOptions::latest())
}

#[hdk_extern]
pub fn search_keys_local(request: SearchKeysRequest) -> ExternResult<Vec<VfKeyResponse>> {
    search_keys_with_get_options(request, GetOptions::content())
}

fn search_keys_with_get_options(
    request: SearchKeysRequest,
    get_options: GetOptions,
) -> ExternResult<Vec<VfKeyResponse>> {
    match request.agent_pub_key {
        Some(agent_pub_key) => {
            let links = get_links(
                GetLinksInputBuilder::try_new(agent_pub_key, LinkTypes::AgentToVfKeyDist)?
                    .get_options(get_options.strategy)
                    .build(),
            )?;

            let mut out = Vec::with_capacity(links.len());
            for key_dist_address in links
                .into_iter()
                .flat_map(|l| EntryHash::try_from(l.target).ok())
            {
                let reference_count =
                    get_key_collections_reference_count(key_dist_address.clone(), &get_options)?;

                match get(key_dist_address.clone(), GetOptions::latest())? {
                    Some(r) => {
                        let created_at = r.action().timestamp();
                        let vf_key_dist: VerificationKeyDist = convert_to_app_entry_type(r)?;
                        out.push(VfKeyResponse {
                            verification_key_dist: vf_key_dist.into(),
                            key_dist_address,
                            reference_count,
                            created_at,
                        });
                    }
                    _ => {
                        // Link target not found
                    }
                }
            }

            Ok(out)
        }
        None => {
            Err(wasm_error!(WasmErrorInner::Guest(
                "No fields on the request to perform a search on".to_string()
            )))
        }
    }
}

fn checked_vf_key(verification_key: &str, key_type: &VerificationKeyType) -> ExternResult<String> {
    match key_type {
        VerificationKeyType::MiniSignEd25519 => {
            let checked_key = verification_key.trim();
            minisign_verify::PublicKey::decode(checked_key).map_err(|e| {
                wasm_error!(WasmErrorInner::Guest(format!(
                    "Failed to parse MiniSign verification key: {}",
                    e
                )))
            })?;
            Ok(checked_key.to_string())
        }
    }
}

/// Builds a dummy hash from a string input.
///
/// This is useful for working with baseless links, is there something in the HDK I'm missing that can do this?
pub fn make_base_hash(input: &str) -> ExternResult<EntryHash> {
    hash_entry(Entry::App(
        AppEntryBytes::try_from(SerializedBytes::from(UnsafeBytes::from(
            input.as_bytes().to_vec(),
        )))
        .map_err(|e| {
            wasm_error!(WasmErrorInner::Guest(format!(
                "Cannot create base hash from {}: {}",
                input, e
            )))
        })?,
    ))
}

/// Check our own source chain to see if we already have this key.
/// If we do, we can't distribute it again so return an error.
fn verify_key_not_distributed_by_me(
    vf_key: &str,
    key_type: &VerificationKeyType,
) -> ExternResult<()> {
    // Check that we haven't already distributed this key, that would never be valid and will be checked by our peers.
    let has_key = get_my_verification_key_distributions(())?
        .iter()
        .any(|response| {
            response.verification_key_dist.verification_key == vf_key
                && &response.verification_key_dist.key_type == key_type
        });
    if has_key {
        return Err(wasm_error!(WasmErrorInner::Guest(
            "You have already distributed this key".to_string()
        )));
    }

    Ok(())
}

/// A point in time check that we don't know of somebody else having distributed this key. Somebody could distribute this
/// key using other code or we might just not have seen it yet.
///
/// While this isn't an integrity guarantee, it might help out a somebody who is trying to distribute a key and hasn't realised
/// they're using a different agent key than they originally distributed the key with.
fn try_verify_key_not_distributed_by_somebody_else(vf_key: &str) -> ExternResult<()> {
    let other_has_key = get_links(
        GetLinksInputBuilder::try_new(make_base_hash(vf_key)?, LinkTypes::VfKeyDistToAgent)?
            .build(),
    )?;
    if !other_has_key.is_empty() {
        return Err(wasm_error!(WasmErrorInner::Guest(
            "This key has already been distributed by somebody else".to_string()
        )));
    }

    Ok(())
}

/// Creates discovery links for the VerificationKeyDist:
/// - From the agent's agent_pub_key to the entry hash of the VerificationKeyDist with type [LinkTypes::AgentToVfKeyDist]
/// - From the entry hash of the VerificationKeyDist to the agent's agent_pub_key with type [LinkTypes::AgentToVfKeyDist]
fn create_vf_key_dist_discovery_links(entry_hash: &EntryHash) -> ExternResult<()> {
    let agent_info = agent_info()?;

    create_link(
        agent_info.agent_initial_pubkey.clone(),
        entry_hash.clone(),
        LinkTypes::AgentToVfKeyDist,
        (),
    )?;

    create_link(
        entry_hash.clone(),
        agent_info.agent_initial_pubkey,
        LinkTypes::VfKeyDistToAgent,
        (),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dump_distribute_vf_key_request() {
        let request = DistributeVfKeyRequest {
            name: "test".to_string(),
            verification_key: "test".to_string(),
            key_type: VerificationKeyType::MiniSignEd25519,
            proof: "test".to_string(),
            proof_signature: vec![1, 2, 3],
        };

        let encoded = ExternIO::encode(request).unwrap();
        println!("{:?}", encoded);
    }
}
