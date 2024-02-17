use hdk::prelude::*;
use trusted_integrity::prelude::*;

use crate::gpg_util::{try_extract_public_key, PublicKeySummary};

#[derive(Serialize, Deserialize, Debug, Clone, SerializedBytes)]
pub struct DistributeGpgKeyRequest {
    pub public_key: String,
}

#[hdk_extern]
pub fn distribute_gpg_key(gpg_key: DistributeGpgKeyRequest) -> ExternResult<Record> {
    let public_key = try_extract_public_key(gpg_key.public_key.clone())?;

    let summary = PublicKeySummary::try_from_public_key(&public_key)?;

    let gpg_key_hash = create_entry(&EntryTypes::GpgKeyDist(GpgKeyDist {
        public_key: gpg_key.public_key,
        fingerprint: summary.fingerprint,
        user_id: summary.user_id,
        email: summary.email,
    }))?;
    let record = get(gpg_key_hash.clone(), GetOptions::default())?
        .ok_or(
            wasm_error!(
                WasmErrorInner::Guest(String::from("Could not find the newly created GpgKey"))
            ),
        )?;
    Ok(record)
}

#[hdk_extern]
pub fn get_my_keys(_: ()) -> ExternResult<Vec<Record>> {
    let q = ChainQueryFilter::default()
        .action_type(ActionType::Create)
        .entry_type(EntryType::App(UnitEntryTypes::GpgKeyDist.try_into()?))
        .include_entries(true)
        .ascending();

    let gpg_key_dist_entries = query(q)?;
    Ok(gpg_key_dist_entries)
}

#[hdk_extern]
pub fn get_gpg_key_dist(gpg_key_hash: ActionHash) -> ExternResult<Option<Record>> {
    let Some(details) = get_details(gpg_key_hash, GetOptions::default())? else {
        return Ok(None);
    };
    match details {
        Details::Record(details) => Ok(Some(details.record)),
        _ => {
            Err(
                wasm_error!(
                    WasmErrorInner::Guest(String::from("Malformed get details response"))
                ),
            )
        }
    }
}
