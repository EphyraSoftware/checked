use checked_types::*;
use fetch_integrity::prelude::*;
use hdk::prelude::hash_type::AnyLinkable;
use hdk::prelude::*;
use rand::prelude::IteratorRandom;
use rand::thread_rng;
use signing_keys_types::*;
use std::ops::{Add, Deref, Sub};
use std::time::Duration;

#[hdk_extern]
fn prepare_fetch(request: PrepareFetchRequest) -> ExternResult<Vec<FetchCheckSignature>> {
    let asset_base = make_asset_url_address(&request.fetch_url)?;

    info!(
        "Fetching signatures for: {}, using as base: {:?}",
        request.fetch_url, asset_base
    );

    // We're online anyway to do a download so go looking for new data.
    let links = get_links(
        GetLinksInputBuilder::try_new(asset_base, LinkTypes::AssetUrlToSignature)?
            .get_options(GetStrategy::Network)
            .build(),
    )?;

    info!("Got {} links", links.len());

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

    info!("Got {} key collections", key_collections.len());

    pick_signatures(signatures, key_collections)
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

    signatures
        .into_iter()
        .map(|sig| {
            let (action, entry) = sig.into_inner();

            let entry = entry
                .into_option()
                .ok_or_else(|| wasm_error!(WasmErrorInner::Guest("No entry found".to_string())))?;

            let signature = AssetSignature::try_from(entry).map_err(|_| {
                wasm_error!(WasmErrorInner::Guest(
                    "Failed to deserialize AssetSignature".to_string()
                ))
            })?;

            Ok(AssetSignatureResponse {
                fetch_url: signature.fetch_url,
                signature: signature.signature,
                key_dist_address: signature.key_dist_address,
                created_at: action.action().timestamp(),
            })
        })
        .collect::<ExternResult<_>>()
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

fn pick_signatures(
    possible_signatures: Vec<Record>,
    key_collections: Vec<KeyCollectionWithKeys>,
) -> ExternResult<Vec<FetchCheckSignature>> {
    let mut possible_signatures: Vec<(Action, AssetSignature)> = possible_signatures
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

    let mut picked_signatures = Vec::new();

    // Search key collections for signatures from agents we've chosen to reference.
    for key_collection in key_collections {
        'keys: for key in key_collection.verification_keys {
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
                    reason: FetchCheckSignatureReason::Pinned(FetchCheckSignaturePinned {
                        author: action.author().clone(),
                        key_collection: key_collection.name.clone(),
                        key_name: key.verification_key_dist.name.clone(),
                    }),
                });
            }
        }
    }

    // Drop signatures that we've already picked from the possible set.
    possible_signatures.retain(|(_, asset_signature)| {
        !picked_signatures
            .iter()
            .any(|p| p.signature == asset_signature.signature)
    });

    possible_signatures.sort_by(|(a, _), (b, _)| a.timestamp().cmp(&b.timestamp()));

    picked_signatures.extend(select_early_signatures(&possible_signatures));

    // Drop signatures that we've already picked from the possible set.
    possible_signatures.retain(|(_, asset_signature)| {
        !picked_signatures
            .iter()
            .any(|p| p.signature == asset_signature.signature)
    });

    picked_signatures.extend(select_recent_signatures(sys_time()?, &possible_signatures)?);

    Ok(picked_signatures)
}

/// Tries to select up to 5 random signatures from the first week of signatures.
/// If there were fewer than 30 signatures in the first week it defaults to selecting from the first 30.
///
/// This function assumes that the input is sorted by the [Action] timestamp.
///
/// The reason on the [FetchCheckSignature] will be [FetchCheckSignatureReason::RandomHistorical].
fn select_early_signatures(
    possible_signatures: &[(Action, AssetSignature)],
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
        Some(x) if x < 30 => 30,
        Some(x) => x,
    };

    let mut rng = &mut thread_rng();
    possible_signatures
        .iter()
        .take(take_many)
        .choose_multiple(&mut rng, 5)
        .iter()
        .map(|(_, sig)| FetchCheckSignature {
            signature: sig.signature.clone(),
            reason: FetchCheckSignatureReason::RandomHistorical,
        })
        .collect()
}

/// Tries to select up to 5 random signatures from the last week of signatures.
/// If there were fewer than 30 signatures in the last week it defaults to selecting from the last 30.
///
/// This function assumes that the input is sorted by the [Action] timestamp.
///
/// The reason on the [FetchCheckSignature] will be [FetchCheckSignatureReason::RandomRecent].
fn select_recent_signatures(
    current_time: Timestamp,
    possible_signatures: &[(Action, AssetSignature)],
) -> ExternResult<Vec<FetchCheckSignature>> {
    let take_after = current_time
        .sub(Duration::from_secs(60 * 60 * 24 * 7))
        .unwrap();

    let take_many = match possible_signatures
        .iter()
        .rev()
        .position(|(a, _)| a.timestamp() < take_after)
    {
        // None or too few found, then default to 30
        None => 30,
        Some(x) if x < 30 => 30,
        Some(x) => x,
    };

    let mut rng = &mut thread_rng();

    Ok(possible_signatures
        .iter()
        .rev()
        .take(take_many)
        .choose_multiple(&mut rng, 5)
        .iter()
        .map(|(_, sig)| FetchCheckSignature {
            signature: sig.signature.clone(),
            reason: FetchCheckSignatureReason::RandomRecent,
        })
        .collect())
}

fn make_asset_url_address(asset_url: &str) -> ExternResult<ExternalHash> {
    let mut url = url::Url::parse(asset_url)
        .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?;

    url.set_password(None).ok();
    url.set_username("").ok();

    let mut hash = blake2b_256(url.as_str().as_bytes());
    hash.extend_from_slice(&[0, 0, 0, 0]);
    Ok(ExternalHash::from_raw_36(hash))
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
    use crate::{select_early_signatures, select_recent_signatures};
    use fetch_types::AssetSignature;
    use hdk::prelude::{
        Action, ActionHash, AgentPubKey, AppEntryDef, Create, EntryHash, EntryRateWeight,
        EntryType, EntryVisibility, Timestamp,
    };
    use std::ops::Sub;
    use std::time::Duration;

    #[test]
    fn select_early_signatures_empty() {
        let picked = select_early_signatures(&[]);
        assert_eq!(0, picked.len());
    }

    #[test]
    fn select_early_signatures_all_recent() {
        // Time in seconds
        let mut time = chrono::prelude::Utc::now()
            .sub(Duration::from_secs(60 * 60 * 24))
            .timestamp(); // 1 day ago

        let possible_signatures = std::iter::repeat_with(|| {
            time += 5; // +5 seconds
            action_at_time(time)
        })
        .take(100)
        .enumerate()
        .map(|(idx, a)| {
            (
                a,
                AssetSignature {
                    signature: vec![idx as u8],
                    key_dist_address: ActionHash::from_raw_36(vec![0; 36]),
                },
            )
        })
        .collect::<Vec<_>>();

        let picked = select_early_signatures(&possible_signatures);

        // Picked 5
        assert_eq!(5, picked.len());

        // No need to assert, can pick from anywhere because they're all close to the first signature in time.
    }

    #[test]
    fn select_early_signatures_time_spread() {
        println!("Time now {:?}", chrono::prelude::Utc::now());

        // Time in seconds
        let mut time = chrono::prelude::Utc::now()
            .sub(Duration::from_secs(60 * 60 * 24 * 100))
            .timestamp(); // 100 days ago

        let possible_signatures = std::iter::repeat_with(|| {
            time += 60 * 60 * 24; // +1 day
            action_at_time(time)
        })
        .take(100)
        .enumerate()
        .map(|(idx, a)| {
            (
                a,
                AssetSignature {
                    signature: vec![idx as u8],
                    key_dist_address: ActionHash::from_raw_36(vec![0; 36]),
                },
            )
        })
        .collect::<Vec<_>>();

        let picked = select_early_signatures(&possible_signatures);

        // Picked 5
        assert_eq!(5, picked.len());

        println!("Picked signatures: {:?}", picked);

        // All from the first 30
        assert!(picked.iter().all(|sig| { sig.signature[0] <= 30 }));
    }

    #[test]
    fn select_recent_signatures_empty() {
        let picked = select_recent_signatures(current_time(), &[]).unwrap();
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
            action_at_time(time)
        })
        .take(100)
        .enumerate()
        .map(|(idx, a)| {
            (
                a,
                AssetSignature {
                    signature: vec![idx as u8],
                    key_dist_address: ActionHash::from_raw_36(vec![0; 36]),
                },
            )
        })
        .collect::<Vec<_>>();

        let picked = select_recent_signatures(current_time(), &possible_signatures).unwrap();

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
            action_at_time(time)
        })
        .take(100)
        .enumerate()
        .map(|(idx, a)| {
            (
                a,
                AssetSignature {
                    signature: vec![idx as u8],
                    key_dist_address: ActionHash::from_raw_36(vec![0; 36]),
                },
            )
        })
        .collect::<Vec<_>>();

        let picked = select_recent_signatures(current_time(), &possible_signatures).unwrap();

        // Picked 5
        assert_eq!(5, picked.len());

        // All from the last 30
        assert!(picked.iter().all(|sig| { sig.signature[0] >= 70 }));
    }

    #[test]
    fn select_recent_signatures_all_old() {
        // Time in seconds
        let time = chrono::prelude::Utc::now()
            .sub(Duration::from_secs(60 * 60 * 24 * 100))
            .timestamp(); // 100 days ago

        let possible_signatures = std::iter::repeat_with(|| action_at_time(time))
            .take(100)
            .enumerate()
            .map(|(idx, a)| {
                (
                    a,
                    AssetSignature {
                        signature: vec![idx as u8],
                        key_dist_address: ActionHash::from_raw_36(vec![0; 36]),
                    },
                )
            })
            .collect::<Vec<_>>();

        let picked = select_recent_signatures(current_time(), &possible_signatures).unwrap();

        // Picked 5
        assert_eq!(5, picked.len());

        // All from the last 30
        assert!(picked.iter().all(|sig| { sig.signature[0] >= 70 }));
    }

    fn current_time() -> Timestamp {
        Timestamp(chrono::Utc::now().timestamp() * 1_000_000)
    }

    fn action_at_time(time: i64) -> Action {
        Action::Create(Create {
            author: AgentPubKey::from_raw_36(vec![0; 36]),
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
}
