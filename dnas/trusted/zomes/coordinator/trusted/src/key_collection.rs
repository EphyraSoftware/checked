use crate::gpg_key_dist::make_base_hash;
use hdk::prelude::*;
use nanoid::nanoid;
use trusted_integrity::prelude::*;

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
    pub gpg_keys: Vec<GpgKeyDist>,
}

#[hdk_extern]
pub fn get_my_key_collections(_: ()) -> ExternResult<Vec<KeyCollectionWithKeys>> {
    let mut key_collections = Vec::new();
    for record in inner_get_my_key_collections()? {
        let mut key_collection = KeyCollectionWithKeys {
            // TODO If it worked, walidation would ensure these are all KeyCollections, so this could not fail
            name: record
                .entry()
                .as_option()
                .and_then(|e| e.as_app_entry())
                .and_then(|e| {
                    tracing::info!("entry: {:?}", e);
                    let key_collection: KeyCollection = e.clone().into_sb().try_into().ok()?;
                    Some(key_collection.name)
                })
                .ok_or(wasm_error!(WasmErrorInner::Guest(String::from(
                    "Could not convert entry to KeyCollection"
                ))))?,
            gpg_keys: Vec::new(),
        };

        let linked_keys = get_links(
            GetLinksInputBuilder::try_new(
                record.action_hashed().as_hash().clone(),
                LinkTypes::KeyCollectionToGpgKeyDist,
            )?
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
            make_base_hash(&request.gpg_key_fingerprint)?,
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

    let my_key_collections = inner_get_my_key_collections()?;

    let matched_collection = my_key_collections.into_iter().find(|r| {
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

    let tag_id = nanoid!();

    // Link from the key collection to the GpgKeyDist so that we can find them from the key collection later
    create_link(
        key_collection.action_hashed().as_hash().clone(),
        fingerprint_link.target.clone(), // entry address
        LinkTypes::KeyCollectionToGpgKeyDist,
        tag_id.as_bytes().to_vec(),
    )?;

    // Link from the key fingerprint to the key collection so that we can report how many key collections a key is in
    create_link(
        fingerprint_link.target, // entry address
        key_collection.action_hashed().as_hash().clone(),
        LinkTypes::GpgKeyDistToKeyCollection,
        tag_id.as_bytes().to_vec(),
    )
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
