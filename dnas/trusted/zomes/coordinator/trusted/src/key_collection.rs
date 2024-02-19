use crate::gpg_key_dist::make_base_hash;
use hdk::prelude::*;
use trusted_integrity::prelude::*;

#[hdk_extern]
pub fn create_key_collection(key_collection: KeyCollection) -> ExternResult<Record> {
    check_key_collection_create(&key_collection)?;

    let entry = EntryTypes::KeyCollection(key_collection);
    let action_hash = create_entry(entry)?;

    let record = get(action_hash.clone(), GetOptions::content())?.ok_or(wasm_error!(
        WasmErrorInner::Guest(String::from(
            "Could not find the newly created KeyCollection"
        ))
    ))?;

    let entry_hash = hash_entry(
        record
            .entry()
            .as_option()
            .ok_or_else(|| wasm_error!(WasmErrorInner::Guest(String::from("Missing entry hash"))))?
            .clone(),
    )?;
    let my_agent_info = agent_info()?;
    create_link(
        my_agent_info.agent_latest_pubkey,
        entry_hash,
        LinkTypes::KeyCollection,
        (),
    )?;

    Ok(record)
}

fn check_key_collection_create(key_collection: &KeyCollection) -> ExternResult<()> {
    let existing_key_collections = query(
        ChainQueryFilter::default()
            .entry_type(EntryType::App(UnitEntryTypes::KeyCollection.try_into()?)),
    )?;

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

#[derive(Serialize, Deserialize, Debug, Clone, SerializedBytes)]
pub struct KeyCollectionWithKeys {
    pub name: String,
    pub gpg_keys: Vec<GpgKeyDist>,
}

#[hdk_extern]
pub fn get_my_key_collections(_: ()) -> ExternResult<Vec<KeyCollectionWithKeys>> {
    let mut key_collections = Vec::new();
    for (entry_hash, record) in get_key_collections()? {
        let mut key_collection = KeyCollectionWithKeys {
            // TODO If it worked, walidation would ensure these are all KeyCollections, so this could not fail
            name: record
                .entry()
                .as_option()
                .and_then(|e| e.as_app_entry())
                .and_then(|e| {
                    let key_collection: KeyCollection = e.clone().into_sb().try_into().ok()?;
                    Some(key_collection.name)
                })
                .ok_or(wasm_error!(WasmErrorInner::Guest(String::from(
                    "Could not convert entry to KeyCollection"
                ))))?,
            gpg_keys: Vec::new(),
        };

        let linked_keys = get_links(
            GetLinksInputBuilder::try_new(entry_hash, LinkTypes::KeyCollectionToGpgKeyDist)?
                .build(),
        )?;

        for link in linked_keys {
            let gpg_key_dist: Option<GpgKeyDist> = get(
                <AnyLinkableHash as TryInto<AnyDhtHash>>::try_into(link.target).map_err(|_| {
                    wasm_error!(WasmErrorInner::Guest(String::from("Not a DHT hash")))
                })?,
                GetOptions::content(),
            )?
            .ok_or(wasm_error!(WasmErrorInner::Guest(String::from(
                "Could not find the GpgKeyDist"
            ))))?
            .entry()
            .as_option()
            .and_then(|e| e.as_app_entry())
            .and_then(|e| e.clone().into_sb().try_into().ok());

            if let Some(gpg_key_dist) = gpg_key_dist {
                key_collection.gpg_keys.push(gpg_key_dist);
            }
        }

        key_collections.push(key_collection);
    }

    Ok(key_collections)
}

#[derive(Serialize, Deserialize, Debug, Clone, SerializedBytes)]
pub struct LinkGpgKeyToKeyCollectionRequest {
    pub gpg_key_fingerprint: String,
    pub key_collection_name: String,
}

// TODO prevent duplicate links? Could also be done in the UI fairly easily
#[hdk_extern]
pub fn link_gpg_key_to_key_collection(
    request: LinkGpgKeyToKeyCollectionRequest,
) -> ExternResult<ActionHash> {
    let fingerprint_links = get_links(
        GetLinksInputBuilder::try_new(
            make_base_hash(request.gpg_key_fingerprint)?,
            LinkTypes::FingerprintToGpgKeyDist,
        )?
        .build(),
    )?;

    if fingerprint_links.is_empty() {
        return Err(wasm_error!(WasmErrorInner::Guest(String::from(
            "No GPG key found with the given fingerprint"
        ))));
    }

    if fingerprint_links.len() > 1 {
        return Err(wasm_error!(WasmErrorInner::Guest(String::from(
            "Multiple GPG keys found with the given fingerprint"
        ))));
    }

    // Target is the entry hash of the GpgKeyDist
    let fingerprint_link = fingerprint_links[0].clone();

    let my_key_collections = get_key_collections()?;

    let matched_collection = my_key_collections.into_iter().find(|(_, r)| {
        r.entry
            .as_option()
            .and_then(|e| e.as_app_entry())
            .and_then(|e| {
                let key_collection: KeyCollection = e.clone().into_sb().try_into().ok()?;
                Some(key_collection.name == request.key_collection_name)
            })
            .unwrap_or(false)
    });

    let key_collection = matched_collection.ok_or(wasm_error!(WasmErrorInner::Guest(
        String::from("No key collection found with the given name")
    )))?;

    create_link(
        key_collection.0,
        fingerprint_link.target,
        LinkTypes::KeyCollectionToGpgKeyDist,
        (),
    )
}

fn get_key_collections() -> ExternResult<Vec<(EntryHash, Record)>> {
    let my_agent_info = agent_info()?;
    let key_collection_links = get_links(
        GetLinksInputBuilder::try_new(my_agent_info.agent_latest_pubkey, LinkTypes::KeyCollection)?
            .build(),
    )?;

    let mut records = Vec::with_capacity(key_collection_links.len());
    for link in key_collection_links {
        let entry_hash: EntryHash = link.target.clone().try_into().map_err(|_| {
            wasm_error!(WasmErrorInner::Guest(String::from(
                "Could not convert link target to EntryHash"
            )))
        })?;
        let record = get(entry_hash.clone(), GetOptions::content())?.ok_or(wasm_error!(
            WasmErrorInner::Guest(String::from("Could not find the KeyCollection"))
        ))?;

        records.push((entry_hash, record));
    }

    Ok(records)
}
