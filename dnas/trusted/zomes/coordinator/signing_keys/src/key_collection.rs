use crate::verification_key_dist::get_key_marks;
use crate::{convert_to_app_entry_type, verification_key_dist::VfKeyResponse};
use hdk::prelude::*;
use nanoid::nanoid;
use signing_keys_integrity::prelude::*;

#[hdk_extern]
pub fn create_key_collection(key_collection: KeyCollection) -> ExternResult<Record> {
    verify_key_collection_create(&key_collection)?;

    let entry = EntryTypes::KeyCollection(key_collection);
    let action_hash = create_entry(entry)?;

    let record = get(action_hash.clone(), GetOptions::content())?.ok_or(wasm_error!(
        WasmErrorInner::Guest(String::from(
            "Could not find the newly created KeyCollection"
        ))
    ))?;

    Ok(record)
}

#[derive(Serialize, Deserialize, Debug, Clone, SerializedBytes)]
pub struct KeyCollectionWithKeys {
    pub name: String,
    pub verification_keys: Vec<VfKeyResponse>,
}

#[hdk_extern]
pub fn get_my_key_collections(_: ()) -> ExternResult<Vec<KeyCollectionWithKeys>> {
    let mut key_collections = Vec::new();
    for record in inner_get_my_key_collections()? {
        let collection_action_hash = record.action_hashed().as_hash().clone();
        let key_collection: KeyCollection = convert_to_app_entry_type(record)?;
        let mut key_collection = KeyCollectionWithKeys {
            name: key_collection.name,
            verification_keys: Vec::new(),
        };

        let linked_vf_keys = get_links(
            GetLinksInputBuilder::try_new(
                collection_action_hash,
                LinkTypes::KeyCollectionToVfKeyDist,
            )?
            // We created these links so only look locally.
            .get_options(GetStrategy::Content)
            .build(),
        )?;

        for link in linked_vf_keys {
            let key_dist_address: ActionHash = link.target.try_into().map_err(|_| {
                wasm_error!(WasmErrorInner::Guest(String::from(
                    "Not a valid verification key dist address"
                )))
            })?;

            let vf_key_dist_record = get(key_dist_address.clone(), GetOptions::content())?;

            let (created_at, vf_key_dist) = if let Some(vf_key_dist_record) = vf_key_dist_record {
                let vf_key_dist: VerificationKeyDist =
                    convert_to_app_entry_type(vf_key_dist_record.clone())?;
                (vf_key_dist_record.action().timestamp(), vf_key_dist)
            } else {
                continue;
            };

            // Must be latest because we are looking for marks on the key dist created by *other* agents.
            let marks = get_key_marks(key_dist_address.clone(), GetOptions::latest())?;
            let reference_count = get_key_collections_reference_count(
                key_dist_address.clone(),
                // This is collective across the network, so prefer latest.
                &GetOptions::latest(),
            )?;
            key_collection.verification_keys.push(VfKeyResponse {
                verification_key_dist: (vf_key_dist, marks).into(),
                key_dist_address,
                reference_count,
                created_at,
            });
        }

        key_collections.push(key_collection);
    }

    Ok(key_collections)
}

#[derive(Serialize, Deserialize, Debug, Clone, SerializedBytes)]
pub struct LinkVfKeyDistToKeyCollectionRequest {
    pub verification_key_dist_address: ActionHash,
    pub key_collection_name: String,
}

#[hdk_extern]
pub fn link_verification_key_to_key_collection(
    request: LinkVfKeyDistToKeyCollectionRequest,
) -> ExternResult<ActionHash> {
    let (kc_action, _) = find_key_collection(&request.key_collection_name)?.ok_or(wasm_error!(
        WasmErrorInner::Guest(String::from("No key collection found with the given name"))
    ))?;

    let tag_id = nanoid!();

    // Link from the key collection ActionHash to the VerificationKeyDist EntryHash so that we can find them from the key collection later
    create_link(
        kc_action.clone(),
        request.verification_key_dist_address.clone(),
        LinkTypes::KeyCollectionToVfKeyDist,
        tag_id.as_bytes().to_vec(),
    )?;

    // Link from the VerificationKeyDist EntryHash to the key collection so that we can report how many key collections a key is in
    create_link(
        request.verification_key_dist_address,
        kc_action,
        LinkTypes::VfKeyDistToKeyCollection,
        tag_id.as_bytes().to_vec(),
    )
}

#[derive(Serialize, Deserialize, Debug, Clone, SerializedBytes)]
pub struct UnlinkVfKeyFromKeyCollectionRequest {
    pub verification_key_dist_address: ActionHash,
    pub key_collection_name: String,
}

#[hdk_extern]
pub fn unlink_verification_key_from_key_collection(
    request: UnlinkVfKeyFromKeyCollectionRequest,
) -> ExternResult<()> {
    let (kc_action, _) = find_key_collection(&request.key_collection_name)?.ok_or(wasm_error!(
        WasmErrorInner::Guest(String::from("No key collection found with the given name"))
    ))?;

    let agent_info = agent_info()?;

    let links_from_vf_key_dist = get_links(
        GetLinksInputBuilder::try_new(
            request.verification_key_dist_address.clone(),
            LinkTypes::VfKeyDistToKeyCollection.try_into_filter()?,
        )?
        // Always created by the current agent so only look locally.
        .get_options(GetStrategy::Content)
        .author(agent_info.agent_initial_pubkey.clone())
        .build(),
    )?;

    // Unlink the key fingerprint from the key collection
    let mut removing_tags = HashSet::new();
    for link in links_from_vf_key_dist.into_iter() {
        // Find the links from this collection that target the key to remove
        if link.target == kc_action.clone().into() {
            removing_tags.insert(link.tag);
            delete_link(link.create_link_hash)?;
        }
    }

    let links_from_selected_collection = get_links(
        GetLinksInputBuilder::try_new(
            kc_action,
            LinkTypes::KeyCollectionToVfKeyDist.try_into_filter()?,
        )?
        .author(agent_info.agent_initial_pubkey)
        .build(),
    )?;

    let target_as_any: AnyLinkableHash = request.verification_key_dist_address.clone().into();
    // Unlink the the key collection from the VerificationKeyDist
    for link in links_from_selected_collection.into_iter() {
        // Find the links from this collection that target the key to remove
        if link.target == target_as_any {
            if !removing_tags.remove(&link.tag) {
                return Err(wasm_error!(WasmErrorInner::Guest(format!(
                    "Link from verification key dist to key collection has tag {:?} but no corresponding link from key collection to verification key dist was deleted",
                    link.tag
                ))));
            }
            delete_link(link.create_link_hash)?;
        }
    }

    if !removing_tags.is_empty() {
        warn!("There were links from the verification key dist that did not correspond to a link from the key collection. Validation is supposed to prevent this. {:?}", removing_tags);
    }

    Ok(())
}

/// Checks if the key collection can be created.
///
/// - Ensures the name is at least [KEY_COLLECTION_NAME_MIN_LENGTH] characters long. Also checked by validation.
/// - Limits the number of key collections a user can have to [KEY_COLLECTION_LIMIT]. Also checked by validation.
/// - Ensures the name is unique among the user's key collections. Not checked by validation.
fn verify_key_collection_create(key_collection: &KeyCollection) -> ExternResult<()> {
    // This is enforced by validation, but checked here for faster feedback
    if key_collection.name.len() < KEY_COLLECTION_NAME_MIN_LENGTH {
        return Err(wasm_error!(WasmErrorInner::Guest(format!(
            "Key collection name must be at least {} characters long",
            KEY_COLLECTION_NAME_MIN_LENGTH
        ))));
    }

    let existing_key_collections = inner_get_my_key_collections()?;

    // This is enforced by validation, but checked here for faster feedback
    if existing_key_collections.len() >= KEY_COLLECTION_LIMIT {
        return Err(wasm_error!(WasmErrorInner::Guest(String::from(
            "Exceeded the maximum number of key collections",
        ))));
    }

    let names: HashSet<_> = existing_key_collections
        .into_iter()
        .filter_map(
            |kc| match kc.entry().as_option().and_then(|e| e.as_app_entry()) {
                Some(entry_bytes) => {
                    let key_collection: KeyCollection =
                        entry_bytes.clone().into_sb().try_into().ok()?;
                    Some(key_collection.name)
                }
                None => None,
            },
        )
        .collect();

    // Not checked by validation, other users do not care about your key collection names being unique. The entries are private
    // so they can't actually see them!
    if names.contains(&key_collection.name) {
        return Err(wasm_error!(WasmErrorInner::Guest(String::from(
            "Key collection with the same name already exists",
        ))));
    }

    Ok(())
}

fn inner_get_my_key_collections() -> ExternResult<Vec<Record>> {
    query(
        ChainQueryFilter::default()
            .include_entries(true)
            .entry_type(EntryType::App(UnitEntryTypes::KeyCollection.try_into()?)),
    )
}

fn find_key_collection(name: &str) -> ExternResult<Option<(ActionHash, KeyCollection)>> {
    let my_key_collections = inner_get_my_key_collections()?;

    Ok(my_key_collections
        .into_iter()
        .filter_map(|r| -> Option<(ActionHash, KeyCollection)> {
            let action_hash = r.action_hashed().hash.clone();

            convert_to_app_entry_type(r)
                .ok()
                .map(|kc| (action_hash, kc))
        })
        .find(|(_, kc)| kc.name == name))
}

/// Counts the number of references from a [VerificationKeyDist] to [KeyCollection]s.
///
/// Each author may put the same key in multiple collections, but that is only counted once.
/// That means this is the number of unique agents who are referencing this key.
pub fn get_key_collections_reference_count(
    key_dist_address: ActionHash,
    get_options: &GetOptions,
) -> ExternResult<usize> {
    let links = get_links(
        GetLinksInputBuilder::try_new(key_dist_address, LinkTypes::VfKeyDistToKeyCollection)?
            .get_options(get_options.strategy)
            .build(),
    )?;
    Ok(links
        .into_iter()
        .map(|l| l.author)
        .collect::<HashSet<_>>()
        .len())
}
