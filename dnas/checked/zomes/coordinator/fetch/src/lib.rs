use fetch_integrity::prelude::*;
use hdk::prelude::hash_type::AnyLinkable;
use hdk::prelude::*;
use signing_keys_types::*;

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
    signature: Vec<u8>,
    reason: FetchCheckSignatureReason,
}

#[hdk_extern]
fn prepare_fetch(request: PrepareFetchRequest) -> ExternResult<Vec<FetchCheckSignature>> {
    url::Url::parse(&request.fetch_url)
        .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?;

    let inner: UnsafeBytes = request.fetch_url.as_bytes().to_vec().into();
    let asset_base = hash_entry(Entry::App(AppEntryBytes(inner.into())))?;

    // We're online anyway to do a download so go looking for new data.
    let links = get_links(
        GetLinksInputBuilder::try_new(asset_base, LinkTypes::AssetUrlToSignature)?
            .get_options(GetStrategy::Network)
            .build(),
    )?;

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

    Ok(pick_signatures(signatures, key_collections))
}

fn pick_signatures(possible_signatures: Vec<Record>, key_collections: Vec<KeyCollectionWithKeys>) -> Vec<FetchCheckSignature> {
    let possible_signatures: Vec<(Action, AssetSignature)> = possible_signatures
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

    picked_signatures
}
