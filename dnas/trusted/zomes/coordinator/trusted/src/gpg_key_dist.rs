use hdk::prelude::*;
use trusted_integrity::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone, SerializedBytes)]
pub struct DistributeGpgKeyRequest {
    pub public_key: String,
}

#[hdk_extern]
pub fn distribute_gpg_key(request: DistributeGpgKeyRequest) -> ExternResult<Record> {
    let public_key = try_extract_public_key(request.public_key.clone())?;
    let summary = PublicKeySummary::try_from_public_key(&public_key)?;

    verify_key_not_distributed_by_me(&summary)?;

    try_verify_key_not_distributed_by_somebody_else(&summary)?;

    let gpg_key_dist_action_hash = create_entry(&EntryTypes::GpgKeyDist(GpgKeyDist {
        public_key: request.public_key.trim().to_string(),
        fingerprint: summary.fingerprint.clone(),
        name: summary.name.clone(),
        email: summary.email.clone(),
        expires_at: summary.expires_at,
    }))?;

    let record = get(gpg_key_dist_action_hash.clone(), GetOptions::content())?.ok_or(wasm_error!(
        WasmErrorInner::Guest(String::from("Could not find the newly created GpgKeyDist"))
    ))?;

    let entry_hash = record
        .action()
        .entry_hash()
        .ok_or_else(|| wasm_error!(WasmErrorInner::Guest(String::from("Missing entry hash"))))?;

    create_gpg_key_dist_discovery_links(&summary, entry_hash)?;
    
    Ok(record)
}

#[hdk_extern]
pub fn get_my_gpg_key_dists(_: ()) -> ExternResult<Vec<Record>> {
    let q = ChainQueryFilter::default()
        .action_type(ActionType::Create)
        .entry_type(EntryType::App(UnitEntryTypes::GpgKeyDist.try_into()?))
        .include_entries(true)
        .ascending();

    let gpg_key_dist_entries = query(q)?;
    Ok(gpg_key_dist_entries)
}

#[derive(Serialize, Deserialize, Debug, Clone, SerializedBytes)]
pub struct SearchKeysRequest {
    pub query: String,
}

#[hdk_extern]
pub fn search_keys(request: SearchKeysRequest) -> ExternResult<Vec<Record>> {
    let mut links = get_links(
        GetLinksInputBuilder::try_new(
            make_base_hash(&request.query)?,
            LinkTypes::UserIdToGpgKeyDist,
        )?
        .build(),
    )?;
    let email_links = get_links(
        GetLinksInputBuilder::try_new(
            make_base_hash(&request.query)?,
            LinkTypes::EmailToGpgKeyDist,
        )?
        .build(),
    )?;
    let fingerprint_links = get_links(
        GetLinksInputBuilder::try_new(
            make_base_hash(&request.query)?,
            LinkTypes::FingerprintToGpgKeyDist,
        )?
        .build(),
    )?;

    links.extend(email_links);
    links.extend(fingerprint_links.clone());

    let mut out = Vec::with_capacity(links.len());
    for target in links
        .into_iter()
        .flat_map(|l| AnyDhtHash::try_from(l.target).ok())
    {
        match get(target, GetOptions::default())? {
            Some(r) => {
                out.push(r);
            }
            _ => {
                // Link target not found
            }
        }
    }

    Ok(out)
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
fn verify_key_not_distributed_by_me(summary: &PublicKeySummary) -> ExternResult<()> {
    // Check that we haven't already distributed this key, that would never be valid and will be checked by our peers.
    let has_key = get_my_gpg_key_dists(())?
        .iter()
        .any(|record| match record.entry.as_option() {
            Some(Entry::App(app_entry)) => {
                let gpg_key_dist: GpgKeyDist = app_entry.clone().into_sb().try_into().unwrap();
                gpg_key_dist.fingerprint == summary.fingerprint
            }
            _ => false,
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
fn try_verify_key_not_distributed_by_somebody_else(summary: &PublicKeySummary) -> ExternResult<()> {
    let other_has_key = get_links(
        GetLinksInputBuilder::try_new(
            make_base_hash(&summary.fingerprint)?,
            LinkTypes::FingerprintToGpgKeyDist,
        )?
        .build(),
    )?;
    if !other_has_key.is_empty() {
        return Err(wasm_error!(WasmErrorInner::Guest(
            "This key has already been distributed by somebody else".to_string()
        )));
    }

    Ok(())
}

/// Create links from the name, email (if present) and fingerprint to the GpgKeyDist entry.
fn create_gpg_key_dist_discovery_links(summary: &PublicKeySummary, entry_hash: &EntryHash) -> ExternResult<()> {
    create_link(
        make_base_hash(&summary.name)?,
        entry_hash.clone(),
        LinkTypes::UserIdToGpgKeyDist,
        (),
    )?;

    if let Some(email) = &summary.email {
        create_link(
            make_base_hash(email)?,
            entry_hash.clone(),
            LinkTypes::EmailToGpgKeyDist,
            (),
        )?;
    }

    create_link(
        make_base_hash(&summary.fingerprint)?,
        entry_hash.clone(),
        LinkTypes::FingerprintToGpgKeyDist,
        (),
    )?;

    Ok(())
}
