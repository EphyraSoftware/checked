use crate::{convert_to_app_entry_type, key_collection::get_key_collections_reference_count};
use hdk::prelude::*;
use signing_keys_integrity::prelude::*;

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

    let record = get(vf_key_dist_action_hash.clone(), GetOptions::local())?.ok_or(wasm_error!(
        WasmErrorInner::Guest("Could not find the newly created VerificationKeyDist".to_string())
    ))?;

    create_vf_key_dist_discovery_links(&vf_key_dist_action_hash)?;

    Ok(record)
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
        let author = r.action().author().clone();
        let created_at = r.action().timestamp();
        let key_dist_address = r.action_address().clone();
        let vf_key_dist: VerificationKeyDist = convert_to_app_entry_type(r)?;
        let marks = get_key_marks(key_dist_address.clone(), GetOptions::local())?;
        let reference_count =
            get_key_collections_reference_count(key_dist_address.clone(), &GetOptions::local())?;
        out.push(VfKeyResponse {
            verification_key_dist: (vf_key_dist, marks).into(),
            key_dist_address,
            reference_count,
            author,
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
    search_keys_with_get_options(request, GetOptions::network())
}

#[hdk_extern]
pub fn search_keys_local(request: SearchKeysRequest) -> ExternResult<Vec<VfKeyResponse>> {
    search_keys_with_get_options(request, GetOptions::local())
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
                .flat_map(|l| ActionHash::try_from(l.target).ok())
            {
                let reference_count =
                    get_key_collections_reference_count(key_dist_address.clone(), &get_options)?;

                match get(key_dist_address.clone(), GetOptions::network())? {
                    Some(r) => {
                        let author = r.action().author().clone();
                        let created_at = r.action().timestamp();
                        let marks = get_key_marks(key_dist_address.clone(), get_options.clone())?;
                        let vf_key_dist: VerificationKeyDist = convert_to_app_entry_type(r)?;
                        out.push(VfKeyResponse {
                            verification_key_dist: (vf_key_dist, marks).into(),
                            key_dist_address,
                            reference_count,
                            author,
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
        None => Err(wasm_error!(WasmErrorInner::Guest(
            "No fields on the request to perform a search on".to_string()
        ))),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, SerializedBytes)]
pub struct MarkVfKeyDistRequest {
    pub verification_key_dist_address: ActionHash,
    pub mark: MarkVfKeyDistOpt,
}

/// A mark is how the owner of a key can attach metadata to the key to describe its state.
#[hdk_extern]
pub fn mark_verification_key_dist(request: MarkVfKeyDistRequest) -> ExternResult<ActionHash> {
    let record = get(
        request.verification_key_dist_address.clone(),
        GetOptions::local(),
    )?
    .ok_or_else(|| {
        wasm_error!(WasmErrorInner::Guest(format!(
            "Could not find the VerificationKeyDist: {:?}",
            request.verification_key_dist_address
        )))
    })?;

    // To check it really is a VerificationKeyDist
    let _: VerificationKeyDist = convert_to_app_entry_type(record)?;

    let mark_action = create_entry(EntryTypes::VerificationKeyDistMark(
        VerificationKeyDistMark {
            verification_key_dist_address: request.verification_key_dist_address.clone(),
            mark: request.mark,
        },
    ))?;

    let mark_entry = get(mark_action.clone(), GetOptions::local())?.ok_or_else(|| {
        wasm_error!(WasmErrorInner::Guest(format!(
            "Could not find the VerificationKeyDistMark: {:?}",
            mark_action
        )))
    })?;

    create_link(
        request.verification_key_dist_address,
        mark_entry
            .signed_action
            .hashed
            .content
            .entry_hash()
            .ok_or_else(|| {
                wasm_error!(WasmErrorInner::Guest(format!(
                    "Could not find the entry hash for VerificationKeyDistMark: {:?}",
                    mark_action
                )))
            })?
            .clone(),
        LinkTypes::VfKeyDistToMark,
        (),
    )?;

    Ok(mark_action)
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

/// Creates discovery links for the VerificationKeyDist:
/// - From the agent's agent_pub_key to the entry hash of the VerificationKeyDist with type [LinkTypes::AgentToVfKeyDist]
/// - From the entry hash of the VerificationKeyDist to the agent's agent_pub_key with type [LinkTypes::AgentToVfKeyDist]
fn create_vf_key_dist_discovery_links(action_hash: &ActionHash) -> ExternResult<()> {
    let agent_info = agent_info()?;

    create_link(
        agent_info.agent_initial_pubkey.clone(),
        action_hash.clone(),
        LinkTypes::AgentToVfKeyDist,
        (),
    )?;

    create_link(
        action_hash.clone(),
        agent_info.agent_initial_pubkey,
        LinkTypes::VfKeyDistToAgent,
        (),
    )?;

    Ok(())
}

pub fn get_key_marks(
    vf_key_dist_address: ActionHash,
    get_options: GetOptions,
) -> ExternResult<Vec<VerificationKeyDistMark>> {
    let links = get_links(
        GetLinksInputBuilder::try_new(vf_key_dist_address, LinkTypes::VfKeyDistToMark)?
            .get_options(get_options.strategy)
            .build(),
    )?;

    let mut out = Vec::with_capacity(links.len());
    for link in links {
        let target_addr: AnyDhtHash = link.target.clone().try_into().map_err(|_| {
            wasm_error!(WasmErrorInner::Guest(format!(
                "Failed to convert link target to AnyDhtHash: {:?}",
                link.target
            )))
        })?;
        if let Some(r) = get(target_addr, get_options.clone())? {
            let mark: VerificationKeyDistMark = convert_to_app_entry_type(r)?;
            out.push(mark);
        }
    }

    Ok(out)
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
