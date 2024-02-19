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

#[hdk_extern]
pub fn get_my_key_collections(_: ()) -> ExternResult<Vec<Record>> {
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
        let record = get(entry_hash, GetOptions::content())?.ok_or(wasm_error!(
            WasmErrorInner::Guest(String::from("Could not find the KeyCollection"))
        ))?;
        // No need to type check these records, validation ensures they are all KeyCollections
        records.push(record);
    }

    Ok(records)
}
