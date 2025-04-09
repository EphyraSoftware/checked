use std::cmp::min;
use std::ops::{Add, Deref, Sub};
use std::time::Duration;

use hdk::prelude::hash_type::AnyLinkable;
use hdk::prelude::*;
use rand::prelude::{IteratorRandom, SliceRandom};

use checked_types::*;
use fetch_integrity::prelude::*;
use signing_keys_types::*;

#[hdk_extern]
fn prepare_fetch(request: PrepareFetchRequest) -> ExternResult<Vec<FetchCheckSignature>> {
    let asset_base = make_asset_url_address(&request.fetch_url)?;

    info!(
        "Fetching signatures for: {}, using as base: {:?}",
        request.fetch_url, asset_base
    );

    // We're online anyway to do a download so go looking for new data.
    let links = get_links(
        GetLinksInputBuilder::try_new(asset_base.clone(), LinkTypes::AssetUrlToSignature)?
            .get_options(GetStrategy::Network)
            .build(),
    )?;

    info!("Found {} signature links", links.len());

    let mut signatures = Vec::new();
    for link in links {
        let signature_action: ActionHash = link.target.try_into().map_err(
            |e: HashConversionError<AnyLinkable, hash_type::Action>| {
                wasm_error!(WasmErrorInner::Guest(format!(
                    "Signature target is not an action: {:?}",
                    e
                )))
            },
        )?;

        if let Some(r) = get(signature_action, GetOptions::network())? {
            info!("Got a signature record");
            signatures.push(r);
        }
    }

    // Now we have a list of signatures, we know who created them and when. Next is figuring out which ones we want to keep.

    let key_collections: Vec<KeyCollectionWithKeys> = match call(
        CallTargetCell::Local,
        "signing_keys".to_string(),
        "get_my_key_collections".into(),
        None,
        (),
    )? {
        ZomeCallResponse::Ok(response) => response
            .decode()
            .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?,
        _ => {
            return Err(wasm_error!(WasmErrorInner::Guest(
                "Unexpected response from signing_keys".into()
            )))
        }
    };

    info!("Found {} key collections", key_collections.len());

    let signatures: Vec<(Action, AssetSignature)> = signatures
        .into_iter()
        .filter_map(|record| {
            let action = record.signed_action.action().clone();

            record
                .entry
                .to_app_option()
                .ok()
                .flatten()
                .map(|sig| (action, sig))
        })
        .collect();

    let my_agent = agent_info()?.agent_initial_pubkey;

    let my_existing_signature = find_my_existing_signature(asset_base)?;

    Ok(pick_signatures(
        signatures,
        key_collections,
        my_existing_signature,
        get_vf_key_dist,
        sys_time()?,
        &my_agent,
    ))
}

#[hdk_extern]
pub fn create_asset_signature(
    create_asset_signature: CreateAssetSignature,
) -> ExternResult<ActionHash> {
    let my_keys: Vec<VfKeyResponse> = match call(
        CallTargetCell::Local,
        "signing_keys".to_string(),
        "get_my_verification_key_distributions".into(),
        None,
        (),
    )? {
        ZomeCallResponse::Ok(response) => response
            .decode()
            .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?,
        _ => {
            return Err(wasm_error!(WasmErrorInner::Guest(
                "Unexpected response from signing_keys".into()
            )))
        }
    };

    let key_dist_address = match create_asset_signature.key_type {
        VerificationKeyType::MiniSignEd25519 => {
            let verification_key =
                minisign_verify::PublicKey::decode(create_asset_signature.verification_key.trim())
                    .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?;
            find_key_address(create_asset_signature.key_type, verification_key, &my_keys)
        }
    }
    .ok_or_else(|| {
        wasm_error!(WasmErrorInner::Guest(
            "Key not found in your distributed keys".to_string()
        ))
    })?;

    let asset_sig_address = create_entry(EntryTypes::AssetSignature(AssetSignature {
        fetch_url: create_asset_signature.fetch_url.clone(),
        signature: create_asset_signature.signature,
        key_dist_address,
    }))?;

    info!(
        "Linking from {:?}",
        make_asset_url_address(&create_asset_signature.fetch_url)?
    );

    create_link(
        make_asset_url_address(&create_asset_signature.fetch_url)?,
        asset_sig_address.clone(),
        LinkTypes::AssetUrlToSignature,
        (),
    )?;

    Ok(asset_sig_address)
}

#[hdk_extern]
pub fn get_my_asset_signatures() -> ExternResult<Vec<AssetSignatureResponse>> {
    let signatures = query(
        ChainQueryFilter::new()
            .entry_type(UnitEntryTypes::AssetSignature.try_into()?)
            .include_entries(true)
            .ascending(),
    )?;

    // Can't query deletes by entry type (any good reason!?), so get all deletes and use their delete
    // address to filter creates.
    let all_deletes = query(
        ChainQueryFilter::new()
            .action_type(ActionType::Delete)
            .ascending(),
    )?;
    let deleted = all_deletes
        .iter()
        .filter_map(|sig| match sig.signed_action().action() {
            Action::Delete(delete) => Some(delete.deletes_address.clone()),
            _ => None,
        })
        .collect::<HashSet<ActionHash>>();

    Ok(signatures
        .into_iter()
        .map(|sig| {
            if deleted.contains(&sig.signed_action().hashed.hash) {
                return Ok(None);
            }

            let (action, entry) = sig.into_inner();

            let entry = entry
                .into_option()
                .ok_or_else(|| wasm_error!(WasmErrorInner::Guest("No entry found".to_string())))?;

            let signature = AssetSignature::try_from(entry).map_err(|_| {
                wasm_error!(WasmErrorInner::Guest(
                    "Failed to deserialize AssetSignature".to_string()
                ))
            })?;

            Ok(Some(AssetSignatureResponse {
                fetch_url: signature.fetch_url,
                signature: signature.signature,
                key_dist_address: signature.key_dist_address,
                created_at: action.action().timestamp(),
            }))
        })
        .collect::<ExternResult<Vec<Option<AssetSignatureResponse>>>>()?
        .into_iter()
        .flatten()
        .collect::<_>())
}

#[hdk_extern]
pub fn delete_asset_signature(request: DeleteAssetSignatureRequest) -> ExternResult<()> {
    let asset_base = make_asset_url_address(&request.fetch_url)?;

    let links = get_links(
        GetLinksInputBuilder::try_new(asset_base, LinkTypes::AssetUrlToSignature)?
            .get_options(GetStrategy::Local)
            .build(),
    )?;

    trace!("Found {} links from the asset fetch url", links.len());

    let my_agent = agent_info()?.agent_initial_pubkey;

    // This will fail if multiple links exist for the same fetch_url which shouldn't have been
    // allowed by validation.
    for link in links {
        if link.author != my_agent {
            continue;
        }

        debug!("Found a link to delete: {:?}", link);

        let target: ActionHash = link.target.try_into().map_err(|_| {
            wasm_error!(WasmErrorInner::Guest("Target is not an action".to_string()))
        })?;

        // Delete the asset signature and the link to it
        delete(target)?;
        delete_link(link.create_link_hash)?;
    }

    debug!("Deleted asset signature for: {}", request.fetch_url);

    Ok(())
}

fn find_key_address<'a, K>(
    key_type: VerificationKeyType,
    verification_key: K,
    search_keys: &'a [VfKeyResponse],
) -> Option<ActionHash>
where
    &'a str: Into<KeyConvertible<K>>,
    K: PartialEq,
{
    let verification_key = Some(verification_key);

    search_keys
        .iter()
        .find(|key| {
            key.verification_key_dist.key_type == key_type
                && *key.verification_key_dist.verification_key.as_str().into() == verification_key
        })
        .map(|key| key.key_dist_address.clone())
}

pub const MAX_SIGNATURES_FROM_CATEGORY: usize = 5;
pub const MIN_SIGNATURES: usize = 30;

fn pick_signatures(
    mut possible_signatures: Vec<(Action, AssetSignature)>,
    key_collections: Vec<KeyCollectionWithKeys>,
    my_existing_signature: Option<AssetSignature>,
    fetcher: VfKeyDistFetcher,
    current_time: Timestamp,
    my_agent: &AgentPubKey,
) -> Vec<FetchCheckSignature> {
    info!(
        "Selecting from {} possible signatures",
        possible_signatures.len()
    );

    let mut picked_signatures = Vec::new();

    let filter_picked = |possible_signatures: &mut Vec<(Action, AssetSignature)>,
                         picked_signatures: &Vec<FetchCheckSignature>| {
        possible_signatures.retain(|(_, asset_signature)| {
            !picked_signatures
                .iter()
                .any(|p: &FetchCheckSignature| p.signature == asset_signature.signature)
        });
    };

    if let Some(sig) = my_existing_signature {
        match fetcher(&sig.key_dist_address) {
            Ok(Some(vf_key_dist)) => {
                picked_signatures.push(FetchCheckSignature {
                    signature: sig.signature.clone(),
                    key_type: vf_key_dist.verification_key_dist.key_type,
                    verification_key: vf_key_dist.verification_key_dist.verification_key,
                    author: my_agent.clone(),
                    key_dist_address: sig.key_dist_address.clone(),
                    reason: FetchCheckSignatureReason::Mine,
                });

                filter_picked(&mut possible_signatures, &picked_signatures);
            }
            _ => {
                warn!("Discarding my existing asset signature signature because the key distribution could not be fetched: {:?}", sig.key_dist_address);
            }
        }
    }

    picked_signatures.extend(select_pinned_signatures(
        &possible_signatures,
        key_collections,
        my_agent,
    ));

    debug!("Picked {} signatures for pinned", picked_signatures.len());

    // Drop signatures that we've already picked from the possible set.
    filter_picked(&mut possible_signatures, &picked_signatures);

    debug!(
        "Have {} signatures to search for recent and historical signatures",
        possible_signatures.len()
    );

    possible_signatures.sort_by(|(a, _), (b, _)| a.timestamp().cmp(&b.timestamp()));

    picked_signatures.extend(select_historical_signatures(
        &possible_signatures,
        current_time,
        my_agent,
        fetcher,
    ));

    debug!(
        "After adding historical, have {} signatures",
        picked_signatures.len()
    );

    // Drop signatures that we've already picked from the possible set.
    filter_picked(&mut possible_signatures, &picked_signatures);

    debug!(
        "Have {} signatures to search for recent signatures",
        possible_signatures.len()
    );

    picked_signatures.extend(select_recent_signatures(
        &possible_signatures,
        current_time,
        my_agent,
        fetcher,
    ));

    debug!(
        "After adding recent, have {} signatures",
        picked_signatures.len()
    );

    picked_signatures
}

/// If the calling agent has signed this asset before then this function will find and return that
/// signature.
///
/// Unlike the main signature lookup, this will return the signature even if it has been deleted.
/// This allows a client application to avoid creating a duplicate signature even if the previous
/// one has been deleted for some reason.
fn find_my_existing_signature(asset_base: ExternalHash) -> ExternResult<Option<AssetSignature>> {
    let my_agent = agent_info()?.agent_initial_pubkey;

    let mut my_link_creates = get_link_details(
        asset_base,
        LinkTypes::AssetUrlToSignature,
        None,
        GetOptions::local(),
    )?
    .into_inner()
    .into_iter()
    .filter_map(|(create, _)| match &create.hashed.content {
        Action::CreateLink(create_link @ CreateLink { author, .. }) if author == &my_agent => {
            Some(create_link.clone())
        }
        _ => None,
    })
    .collect::<Vec<_>>();

    if my_link_creates.is_empty() {
        return Ok(None);
    }

    my_link_creates.sort_by_key(|create| create.timestamp);
    let first_create = my_link_creates.first().unwrap();

    let target_address_action_hash: ActionHash = first_create
        .target_address
        .clone()
        .try_into()
        .map_err(|_| wasm_error!(WasmErrorInner::Guest("Target is not an action".to_string())))?;
    let asset_signature_record =
        get(target_address_action_hash, GetOptions::local())?.ok_or_else(|| {
            wasm_error!(WasmErrorInner::Guest(
                "No asset signature found".to_string()
            ))
        })?;

    let asset_signature: AssetSignature = asset_signature_record
        .entry
        .to_app_option()
        .map_err(|e| {
            wasm_error!(WasmErrorInner::Guest(format!(
                "Failed to deserialize asset signature: {:?}",
                e
            )))
        })?
        .ok_or_else(|| wasm_error!(WasmErrorInner::Guest("No entry found".to_string())))?;

    Ok(Some(asset_signature))
}

/// Searches for signatures that were created by keys that are found in the key collections. It will
/// return up to 5 signatures that match. The selection is randomised.
///
/// Any keys that have been marked as compromised by their owner will be ignored.
///
/// The reason on the [FetchCheckSignature] will be [FetchCheckSignatureReason::Pinned].
fn select_pinned_signatures(
    possible_signatures: &[(Action, AssetSignature)],
    key_collections: Vec<KeyCollectionWithKeys>,
    my_agent: &AgentPubKey,
) -> Vec<FetchCheckSignature> {
    let mut picked_signatures = Vec::new();

    let mut rng = &mut rand::thread_rng();

    // Search key collections for signatures from agents we've chosen to reference.
    for mut key_collection in key_collections {
        if picked_signatures.len() >= MAX_SIGNATURES_FROM_CATEGORY {
            break;
        }

        key_collection.verification_keys.shuffle(&mut rng);

        'keys: for key in key_collection.verification_keys {
            if picked_signatures.len() >= MAX_SIGNATURES_FROM_CATEGORY {
                break;
            }

            if &key.author == my_agent {
                continue;
            }

            for mark in key.verification_key_dist.marks {
                if let MarkVfKeyDistOpt::Compromised { .. } = mark {
                    continue 'keys;
                }
            }

            // Look for a signature produced by this key and additionally check the author even though
            // that really should match anyway. One person could appear as two agents using
            // the same signing key, so it makes sense to check.
            let matched_signature = possible_signatures.iter().find(|(action, sig)| {
                action.author() == &key.author && sig.key_dist_address == key.key_dist_address
            });

            if let Some((action, sig)) = matched_signature {
                picked_signatures.push(FetchCheckSignature {
                    signature: sig.signature.clone(),
                    key_type: key.verification_key_dist.key_type,
                    verification_key: key.verification_key_dist.verification_key,
                    author: action.author().clone(),
                    key_dist_address: sig.key_dist_address.clone(),
                    reason: FetchCheckSignatureReason::Pinned(FetchCheckSignaturePinned {
                        key_collection: key_collection.name.clone(),
                        key_name: key.verification_key_dist.name.clone(),
                    }),
                });
            }
        }
    }

    picked_signatures
}

/// Tries to select up to [MAX_SIGNATURES_FROM_CATEGORY] random signatures from the first week of signatures.
///
/// If there were fewer than [MIN_SIGNATURES] signatures in the first week it defaults to selecting from the first [MIN_SIGNATURES].
/// If any signatures in the expanded set of signatures would overlap with recent signatures then those will be filtered out.
///
/// This function assumes that the input is sorted by the [Action] timestamp.
///
/// The reason on the [FetchCheckSignature] will be [FetchCheckSignatureReason::RandomHistorical].
fn select_historical_signatures(
    possible_signatures: &[(Action, AssetSignature)],
    current_time: Timestamp,
    my_agent: &AgentPubKey,
    fetcher: VfKeyDistFetcher,
) -> Vec<FetchCheckSignature> {
    let earliest = match possible_signatures.first().map(|(a, _)| a.timestamp()) {
        Some(earliest) => earliest,
        None => return Vec::with_capacity(0),
    };

    let take_before = earliest.add(Duration::from_secs(60 * 60 * 24 * 7)).unwrap(); // 1 week

    let take_many = match possible_signatures
        .iter()
        .position(|(a, _)| a.timestamp() > take_before)
    {
        // None means all are within the time period, take all
        None => possible_signatures.len(),
        // Too few found, use default
        Some(x) if x < MIN_SIGNATURES => min(MIN_SIGNATURES, possible_signatures.len()),
        Some(x) => x,
    };

    info!(
        "Selecting up to {} signatures randomly from {} possible historical signatures",
        MAX_SIGNATURES_FROM_CATEGORY, take_many
    );

    // Would be recent signatures, don't want to overlap with them
    let ignore_after = current_time
        .sub(Duration::from_secs(60 * 60 * 24 * 7)) // 1 week
        .unwrap();

    let mut rng = &mut rand::thread_rng();
    possible_signatures
        .iter()
        .filter_map(|(action, sig)| {
            if action.timestamp() > ignore_after {
                return None;
            }

            if action.author() == my_agent {
                return None;
            }

            match fetcher(&sig.key_dist_address) {
                Ok(Some(vf_key_dist)) => {
                    Some(FetchCheckSignature {
                        signature: sig.signature.clone(),
                        key_type: vf_key_dist.verification_key_dist.key_type,
                        verification_key: vf_key_dist.verification_key_dist.verification_key,
                        author: action.author().clone(),
                        key_dist_address: sig.key_dist_address.clone(),
                        reason: FetchCheckSignatureReason::RandomHistorical,
                    })
                },
                _ => {
                    warn!("Discarding possible signature because the key distribution could not be fetched: {:?}", sig.key_dist_address);
                    None
                },
            }
        })
        .take(take_many)
        .choose_multiple(&mut rng, MAX_SIGNATURES_FROM_CATEGORY)
}

/// Tries to select up to [MAX_SIGNATURES_FROM_CATEGORY] random signatures from the last week of signatures.
/// If there were fewer than [MIN_SIGNATURES] signatures in the last week it defaults to selecting from the last [MIN_SIGNATURES].
///
/// This function assumes that the input is sorted by the [Action] timestamp.
///
/// The reason on the [FetchCheckSignature] will be [FetchCheckSignatureReason::RandomRecent].
fn select_recent_signatures(
    possible_signatures: &[(Action, AssetSignature)],
    current_time: Timestamp,
    my_agent: &AgentPubKey,
    fetcher: VfKeyDistFetcher,
) -> Vec<FetchCheckSignature> {
    let take_after = current_time
        .sub(Duration::from_secs(60 * 60 * 24 * 7)) // 1 week
        .unwrap();

    let take_many = match possible_signatures
        .iter()
        .rev()
        .position(|(a, _)| a.timestamp() < take_after)
    {
        // None or too few found, then default to 30
        None => min(MIN_SIGNATURES, possible_signatures.len()),
        Some(x) if x < MIN_SIGNATURES => min(MIN_SIGNATURES, possible_signatures.len()),
        Some(x) => x,
    };

    let mut rng = &mut rand::thread_rng();

    info!(
        "Selecting up to {} signatures randomly from {} possible recent signatures",
        MAX_SIGNATURES_FROM_CATEGORY, take_many
    );

    possible_signatures
        .iter()
        .rev()
        .filter_map(|(action, sig)| {
            if action.author() == my_agent {
                return None;
            }

            match fetcher(&sig.key_dist_address) {
                Ok(Some(vf_key_dist)) => Some(FetchCheckSignature {
                    signature: sig.signature.clone(),
                    key_type: vf_key_dist.verification_key_dist.key_type,
                    verification_key: vf_key_dist.verification_key_dist.verification_key,
                    author: action.author().clone(),
                    key_dist_address: sig.key_dist_address.clone(),
                    reason: FetchCheckSignatureReason::RandomRecent,
                }),
                _ => {
                    warn!("Discarding possible signature because the key distribution could not be fetched: {:?}", sig.key_dist_address);
                    None
                },
            }
        })
        .take(take_many)
        .choose_multiple(&mut rng, MAX_SIGNATURES_FROM_CATEGORY)
}

type VfKeyDistFetcher = fn(&ActionHash) -> ExternResult<Option<VfKeyResponse>>;

fn get_vf_key_dist(vf_key_dist_address: &ActionHash) -> ExternResult<Option<VfKeyResponse>> {
    let response = call(
        CallTargetCell::Local,
        "signing_keys".to_string(),
        "get_verification_key_dist".into(),
        None,
        vf_key_dist_address.clone(),
    )?;

    match response {
        ZomeCallResponse::Ok(response) => {
            let response: Option<VfKeyResponse> = response.decode().map_err(|e| {
                wasm_error!(WasmErrorInner::Guest(format!(
                    "Failed to decode get_verification_key_dist response: {:?}",
                    e
                )))
            })?;
            Ok(response)
        }
        _ => Err(wasm_error!(WasmErrorInner::Guest(
            "Unexpected response from get_verification_key_dist".to_string()
        ))),
    }
}

struct KeyConvertible<T>(Option<T>);

impl<T> Deref for KeyConvertible<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for KeyConvertible<minisign_verify::PublicKey> {
    fn from(s: &str) -> Self {
        KeyConvertible(minisign_verify::PublicKey::decode(s.trim()).ok())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::ops::{Add, Sub};
    use std::time::Duration;

    use hdk::prelude::{
        Action, ActionHash, AgentPubKey, AppEntryDef, Create, EntryHash, EntryRateWeight,
        EntryType, EntryVisibility, Timestamp,
    };

    use checked_types::{FetchCheckSignatureReason, VerificationKeyType};
    use fetch_types::AssetSignature;
    use signing_keys_types::{
        KeyCollectionWithKeys, MarkVfKeyDistOpt, VerificationKeyDistResponse, VfKeyResponse,
    };

    use super::{
        pick_signatures, select_historical_signatures, select_pinned_signatures,
        select_recent_signatures, MAX_SIGNATURES_FROM_CATEGORY,
    };

    #[test]
    fn select_pinned_empty() {
        let picked =
            select_pinned_signatures(&[], Vec::new(), &AgentPubKey::from_raw_36(vec![0; 36]));
        assert_eq!(0, picked.len());
    }

    #[test]
    fn select_matching_pinned() {
        let possible_signatures = vec![
            (
                action_at_time(0, 0),
                AssetSignature {
                    fetch_url: "http://example.com".to_string(),
                    signature: "1".to_string(),
                    key_dist_address: ActionHash::from_raw_36(vec![0; 36]),
                },
            ),
            (
                action_at_time(0, 1),
                AssetSignature {
                    fetch_url: "http://example.com".to_string(),
                    signature: "2".to_string(),
                    key_dist_address: ActionHash::from_raw_36(vec![1; 36]),
                },
            ),
            (
                action_at_time(0, 2),
                AssetSignature {
                    fetch_url: "http://example.com".to_string(),
                    signature: "3".to_string(),
                    key_dist_address: ActionHash::from_raw_36(vec![2; 36]),
                },
            ),
        ];

        let key_collections = vec![KeyCollectionWithKeys {
            name: "test".to_string(),
            verification_keys: vec![test_vf_key_response(0), test_vf_key_response(2)],
        }];

        let selected = select_pinned_signatures(
            &possible_signatures,
            key_collections,
            &AgentPubKey::from_raw_36(vec![130; 36]),
        );

        assert_eq!(2, selected.len());

        let picked = selected
            .iter()
            .map(|s| s.signature.as_str())
            .collect::<HashSet<_>>();
        assert!(picked.contains("1"));
        assert!(picked.contains("3"));
    }

    // Not supposed to happen, you are supposed to pin other keys rather than your own. But best
    // to filter here anyway.
    #[test]
    fn ignore_pinned_mine() {
        let possible_signatures = vec![
            (
                action_at_time(0, 0),
                AssetSignature {
                    fetch_url: "http://example.com".to_string(),
                    signature: "1".to_string(),
                    key_dist_address: ActionHash::from_raw_36(vec![0; 36]),
                },
            ),
            (
                action_at_time(0, 1),
                AssetSignature {
                    fetch_url: "http://example.com".to_string(),
                    signature: "2".to_string(),
                    key_dist_address: ActionHash::from_raw_36(vec![1; 36]),
                },
            ),
            (
                action_at_time(0, 2),
                AssetSignature {
                    fetch_url: "http://example.com".to_string(),
                    signature: "3".to_string(),
                    key_dist_address: ActionHash::from_raw_36(vec![2; 36]),
                },
            ),
        ];

        let key_collections = vec![KeyCollectionWithKeys {
            name: "test".to_string(),
            verification_keys: vec![
                test_vf_key_response(0),
                test_vf_key_response(1),
                test_vf_key_response(2),
            ],
        }];

        let selected = select_pinned_signatures(
            &possible_signatures,
            key_collections,
            &AgentPubKey::from_raw_36(vec![1; 36]),
        );

        assert_eq!(2, selected.len());

        let picked = selected
            .iter()
            .map(|s| s.signature.as_str())
            .collect::<HashSet<_>>();
        assert!(picked.contains("1"));
        assert!(picked.contains("3"));
    }

    #[test]
    fn ignore_compromised_pinned() {
        let possible_signatures = vec![
            (
                action_at_time(0, 0),
                AssetSignature {
                    fetch_url: "http://example.com".to_string(),
                    signature: "1".to_string(),
                    key_dist_address: ActionHash::from_raw_36(vec![0; 36]),
                },
            ),
            (
                action_at_time(0, 1),
                AssetSignature {
                    fetch_url: "http://example.com".to_string(),
                    signature: "2".to_string(),
                    key_dist_address: ActionHash::from_raw_36(vec![1; 36]),
                },
            ),
            (
                action_at_time(0, 2),
                AssetSignature {
                    fetch_url: "http://example.com".to_string(),
                    signature: "3".to_string(),
                    key_dist_address: ActionHash::from_raw_36(vec![2; 36]),
                },
            ),
        ];

        let key_collections = vec![KeyCollectionWithKeys {
            name: "test".to_string(),
            verification_keys: vec![
                test_vf_key_response(0),
                vf_key_response_add_compromised_mark(test_vf_key_response(1)),
                test_vf_key_response(2),
            ],
        }];

        let selected = select_pinned_signatures(
            &possible_signatures,
            key_collections,
            &AgentPubKey::from_raw_36(vec![130; 36]),
        );

        assert_eq!(2, selected.len());

        let picked = selected
            .iter()
            .map(|s| s.signature.as_str())
            .collect::<HashSet<_>>();
        assert!(picked.contains("1"));
        assert!(picked.contains("3"));
    }

    #[test]
    fn select_limited_pinned() {
        let possible_signatures = vec![
            (
                action_at_time(0, 0),
                AssetSignature {
                    fetch_url: "http://example.com".to_string(),
                    signature: "1".to_string(),
                    key_dist_address: ActionHash::from_raw_36(vec![0; 36]),
                },
            ),
            (
                action_at_time(0, 1),
                AssetSignature {
                    fetch_url: "http://example.com".to_string(),
                    signature: "2".to_string(),
                    key_dist_address: ActionHash::from_raw_36(vec![1; 36]),
                },
            ),
            (
                action_at_time(0, 2),
                AssetSignature {
                    fetch_url: "http://example.com".to_string(),
                    signature: "3".to_string(),
                    key_dist_address: ActionHash::from_raw_36(vec![2; 36]),
                },
            ),
            (
                action_at_time(0, 3),
                AssetSignature {
                    fetch_url: "http://example.com".to_string(),
                    signature: "3".to_string(),
                    key_dist_address: ActionHash::from_raw_36(vec![3; 36]),
                },
            ),
            (
                action_at_time(0, 4),
                AssetSignature {
                    fetch_url: "http://example.com".to_string(),
                    signature: "3".to_string(),
                    key_dist_address: ActionHash::from_raw_36(vec![4; 36]),
                },
            ),
            (
                action_at_time(0, 5),
                AssetSignature {
                    fetch_url: "http://example.com".to_string(),
                    signature: "3".to_string(),
                    key_dist_address: ActionHash::from_raw_36(vec![5; 36]),
                },
            ),
        ];

        let key_collections = vec![KeyCollectionWithKeys {
            name: "test".to_string(),
            verification_keys: vec![
                test_vf_key_response(0),
                test_vf_key_response(1),
                test_vf_key_response(2),
                test_vf_key_response(3),
                test_vf_key_response(4),
                test_vf_key_response(5),
            ],
        }];

        let selected = select_pinned_signatures(
            &possible_signatures,
            key_collections,
            &AgentPubKey::from_raw_36(vec![130; 36]),
        );

        assert_eq!(MAX_SIGNATURES_FROM_CATEGORY, selected.len());
    }

    #[test]
    fn select_historical_signatures_empty() {
        let picked = select_historical_signatures(
            &[],
            Timestamp::now(),
            &AgentPubKey::from_raw_36(vec![0; 36]),
            test_fetcher,
        );
        assert_eq!(0, picked.len());
    }

    #[test]
    fn select_historical_signatures_all_recent() {
        // Time in seconds
        let mut time = chrono::prelude::Utc::now()
            .sub(Duration::from_secs(60 * 60 * 24))
            .timestamp(); // 1 day ago

        let possible_signatures = std::iter::repeat_with(|| {
            time += 5; // +5 seconds
            action_at_time(time, 0)
        })
        .take(100)
        .enumerate()
        .map(|(idx, a)| {
            (
                a,
                AssetSignature {
                    fetch_url: "http://example.com".to_string(),
                    signature: String::from_utf8(vec![idx as u8]).unwrap(),
                    key_dist_address: ActionHash::from_raw_36(vec![0; 36]),
                },
            )
        })
        .collect::<Vec<_>>();

        let picked = select_historical_signatures(
            &possible_signatures,
            Timestamp::now(),
            &AgentPubKey::from_raw_36(vec![130; 36]),
            test_fetcher,
        );

        // Should not return anything, leave these for recent selection
        assert_eq!(0, picked.len());
    }

    #[test]
    fn select_historical_signatures_time_spread() {
        // Time in seconds
        let mut time = chrono::prelude::Utc::now()
            .sub(Duration::from_secs(60 * 60 * 24 * 100))
            .timestamp(); // 100 days ago

        let possible_signatures = std::iter::repeat_with(|| {
            time += 60 * 60 * 24; // +1 day
            action_at_time(time, 0)
        })
        .take(100)
        .enumerate()
        .map(|(idx, a)| {
            (
                a,
                AssetSignature {
                    fetch_url: "http://example.com".to_string(),
                    signature: String::from_utf8(vec![idx as u8]).unwrap(),
                    key_dist_address: ActionHash::from_raw_36(vec![0; 36]),
                },
            )
        })
        .collect::<Vec<_>>();

        let picked = select_historical_signatures(
            &possible_signatures,
            Timestamp::now(),
            &AgentPubKey::from_raw_36(vec![130; 36]),
            test_fetcher,
        );

        // Picked 5
        assert_eq!(5, picked.len());

        // All from the first 30
        assert!(picked
            .iter()
            .all(|sig| { sig.signature.as_bytes()[0] <= 30 }));
    }

    #[test]
    fn select_historical_ignore_mine() {
        // Time in seconds
        let mut time = chrono::prelude::Utc::now()
            .sub(Duration::from_secs(60 * 60 * 24 * 100))
            .timestamp(); // 100 days ago
        let mut author = 0;

        let possible_signatures = std::iter::repeat_with(|| {
            time += 5; // 5s
            author += 1;
            action_at_time(time, author)
        })
        .take(5)
        .enumerate()
        .map(|(idx, a)| {
            (
                a,
                AssetSignature {
                    fetch_url: "http://example.com".to_string(),
                    signature: String::from_utf8(vec![idx as u8]).unwrap(),
                    key_dist_address: ActionHash::from_raw_36(vec![0; 36]),
                },
            )
        })
        .collect::<Vec<_>>();

        let picked = select_historical_signatures(
            &possible_signatures,
            Timestamp::now(),
            &AgentPubKey::from_raw_36(vec![3; 36]),
            test_fetcher,
        );

        // Picked 4 of the possible 5 valid options
        assert_eq!(4, picked.len());
        assert!(!picked.iter().any(|p| p.author.get_raw_36()[0] == 3))
    }

    #[test]
    fn select_recent_signatures_empty() {
        let picked = select_recent_signatures(
            &[],
            current_time(),
            &AgentPubKey::from_raw_36(vec![0; 36]),
            test_fetcher,
        );
        assert_eq!(0, picked.len());
    }

    #[test]
    fn select_recent_signatures_all_recent() {
        // Time in seconds
        let mut time = chrono::prelude::Utc::now()
            .sub(Duration::from_secs(60 * 60 * 24))
            .timestamp(); // 1 day ago

        let possible_signatures = std::iter::repeat_with(|| {
            time += 5; // +5 seconds
            action_at_time(time, 0)
        })
        .take(100)
        .enumerate()
        .map(|(idx, a)| {
            (
                a,
                AssetSignature {
                    fetch_url: "http://example.com".to_string(),
                    signature: String::from_utf8(vec![idx as u8]).unwrap(),
                    key_dist_address: ActionHash::from_raw_36(vec![0; 36]),
                },
            )
        })
        .collect::<Vec<_>>();

        let picked = select_recent_signatures(
            &possible_signatures,
            current_time(),
            &AgentPubKey::from_raw_36(vec![130; 36]),
            test_fetcher,
        );

        // Picked 5
        assert_eq!(5, picked.len());

        // No need to assert, can pick from anywhere because they're all recent.
    }

    #[test]
    fn select_recent_signatures_time_spread() {
        // Time in seconds
        let mut time = chrono::prelude::Utc::now()
            .sub(Duration::from_secs(60 * 60 * 24 * 100))
            .timestamp(); // 100 days ago

        let possible_signatures = std::iter::repeat_with(|| {
            time += 60 * 60 * 24; // +1 day
            action_at_time(time, 0)
        })
        .take(100)
        .enumerate()
        .map(|(idx, a)| {
            (
                a,
                AssetSignature {
                    fetch_url: "http://example.com".to_string(),
                    signature: String::from_utf8(vec![idx as u8]).unwrap(),
                    key_dist_address: ActionHash::from_raw_36(vec![0; 36]),
                },
            )
        })
        .collect::<Vec<_>>();

        let picked = select_recent_signatures(
            &possible_signatures,
            current_time(),
            &AgentPubKey::from_raw_36(vec![130; 36]),
            test_fetcher,
        );

        // Picked 5
        assert_eq!(5, picked.len());

        // All from the last 30
        assert!(picked
            .iter()
            .all(|sig| { sig.signature.as_bytes()[0] >= 70 }));
    }

    #[test]
    fn select_recent_signatures_all_old() {
        // Time in seconds
        let time = chrono::prelude::Utc::now()
            .sub(Duration::from_secs(60 * 60 * 24 * 100))
            .timestamp(); // 100 days ago

        let possible_signatures = std::iter::repeat_with(|| action_at_time(time, 0))
            .take(100)
            .enumerate()
            .map(|(idx, a)| {
                (
                    a,
                    AssetSignature {
                        fetch_url: "http://example.com".to_string(),
                        signature: String::from_utf8(vec![idx as u8]).unwrap(),
                        key_dist_address: ActionHash::from_raw_36(vec![0; 36]),
                    },
                )
            })
            .collect::<Vec<_>>();

        let picked = select_recent_signatures(
            &possible_signatures,
            current_time(),
            &AgentPubKey::from_raw_36(vec![130; 36]),
            test_fetcher,
        );

        // Picked 5
        assert_eq!(5, picked.len());

        // All from the last 30
        assert!(picked
            .iter()
            .all(|sig| { sig.signature.as_bytes()[0] >= 70 }));
    }

    #[test]
    fn select_recent_ignore_mine() {
        // Time in seconds
        let mut time = chrono::prelude::Utc::now()
            .sub(Duration::from_secs(60 * 60 * 24))
            .timestamp(); // 1 day ago
        let mut author = 0;

        let possible_signatures = std::iter::repeat_with(|| {
            time += 5; // +5 seconds
            author += 1;
            action_at_time(time, author)
        })
        .take(5)
        .enumerate()
        .map(|(idx, a)| {
            (
                a,
                AssetSignature {
                    fetch_url: "http://example.com".to_string(),
                    signature: String::from_utf8(vec![idx as u8]).unwrap(),
                    key_dist_address: ActionHash::from_raw_36(vec![0; 36]),
                },
            )
        })
        .collect::<Vec<_>>();

        let picked = select_recent_signatures(
            &possible_signatures,
            current_time(),
            &AgentPubKey::from_raw_36(vec![3; 36]),
            test_fetcher,
        );

        // Picked 4 of the possible 5 valid options
        assert_eq!(4, picked.len());
        assert!(!picked.iter().any(|p| p.author.get_raw_36()[0] == 3));
    }

    #[test]
    fn no_duplicates_between_categories() {
        // Time in seconds
        let mut time = chrono::prelude::Utc::now().timestamp();

        let possible_signatures = std::iter::repeat_with(|| {
            let t = time;
            time += 60 * 60 * 24; // +1 day

            t
        })
        .take(13)
        .enumerate()
        .map(|(idx, time)| {
            (
                action_at_time(time, idx as u8),
                AssetSignature {
                    fetch_url: "http://example.com".to_string(),
                    signature: format!("{idx}"),
                    key_dist_address: ActionHash::from_raw_36(vec![idx as u8; 36]),
                },
            )
        })
        .collect::<Vec<_>>();

        let key_responses = std::iter::repeat(3)
            .take(6)
            .enumerate()
            .map(|(idx, offset)| test_vf_key_response((idx + offset) as u8))
            .collect();

        let selected = pick_signatures(
            possible_signatures,
            vec![KeyCollectionWithKeys {
                name: "test".to_string(),
                verification_keys: key_responses,
            }],
            Some(AssetSignature {
                fetch_url: "http://example.com".to_string(),
                signature: "4".to_string(),
                key_dist_address: ActionHash::from_raw_36(vec![4; 36]),
            }),
            test_fetcher,
            Timestamp::now()
                .add(Duration::from_secs(60 * 60 * 24 * 15))
                .unwrap(),
            &AgentPubKey::from_raw_36(vec![4; 36]),
        );

        assert_eq!(13, selected.len());

        assert_eq!(
            1,
            selected
                .iter()
                .filter(|s| s.reason == FetchCheckSignatureReason::Mine)
                .count()
        );
        assert_eq!(
            5,
            selected
                .iter()
                .filter(|s| matches!(s.reason, FetchCheckSignatureReason::Pinned { .. }))
                .count()
        );
        assert_eq!(
            4,
            selected
                .iter()
                .filter(|s| matches!(s.reason, FetchCheckSignatureReason::RandomRecent))
                .count()
        );
        assert_eq!(
            3,
            selected
                .iter()
                .filter(|s| matches!(s.reason, FetchCheckSignatureReason::RandomHistorical))
                .count()
        );

        let selected_sigs_unique = selected
            .iter()
            .map(|s| s.signature.as_str())
            .collect::<HashSet<_>>();
        assert_eq!(13, selected_sigs_unique.len());
    }

    fn current_time() -> Timestamp {
        Timestamp(chrono::Utc::now().timestamp() * 1_000_000)
    }

    fn test_vf_key_response(id: u8) -> VfKeyResponse {
        VfKeyResponse {
            verification_key_dist: VerificationKeyDistResponse {
                verification_key: format!("test key {id}"),
                key_type: VerificationKeyType::MiniSignEd25519,
                name: format!("test {id}"),
                expires_at: None,
                marks: vec![],
            },
            key_dist_address: ActionHash::from_raw_36(vec![id; 36]),
            reference_count: 1,
            author: AgentPubKey::from_raw_36(vec![id; 36]),
            created_at: Timestamp(0),
        }
    }

    fn action_at_time(time: i64, author_id: u8) -> Action {
        Action::Create(Create {
            author: AgentPubKey::from_raw_36(vec![author_id; 36]),
            timestamp: Timestamp(time * 1_000_000), // Time in seconds to microseconds
            action_seq: 1,
            prev_action: ActionHash::from_raw_36(vec![0; 36]),
            entry_type: EntryType::App(AppEntryDef {
                entry_index: 0.into(),
                zome_index: 0.into(),
                visibility: EntryVisibility::Public,
            }),
            entry_hash: EntryHash::from_raw_36(vec![0; 36]),
            weight: EntryRateWeight::default(),
        })
    }

    fn test_fetcher(_: &ActionHash) -> crate::ExternResult<Option<VfKeyResponse>> {
        Ok(Some(VfKeyResponse {
            verification_key_dist: VerificationKeyDistResponse {
                verification_key: "test key".to_string(),
                key_type: VerificationKeyType::MiniSignEd25519,
                name: "test".to_string(),
                expires_at: None,
                marks: vec![],
            },
            key_dist_address: ActionHash::from_raw_36(vec![0; 36]),
            reference_count: 0,
            author: AgentPubKey::from_raw_36(vec![0; 36]),
            created_at: Timestamp(0),
        }))
    }

    fn vf_key_response_add_compromised_mark(mut response: VfKeyResponse) -> VfKeyResponse {
        response
            .verification_key_dist
            .marks
            .push(MarkVfKeyDistOpt::Compromised {
                note: "Compromised".to_string(),
                since: Timestamp(0),
            });

        response
    }
}
