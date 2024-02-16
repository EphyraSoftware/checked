use hdk::prelude::*;
use trusted_integrity::*;
#[hdk_extern]
pub fn create_gpg_key(gpg_key: GpgKey) -> ExternResult<Record> {
    let gpg_key_hash = create_entry(&EntryTypes::GpgKey(gpg_key.clone()))?;
    let record = get(gpg_key_hash.clone(), GetOptions::default())?
        .ok_or(
            wasm_error!(
                WasmErrorInner::Guest(String::from("Could not find the newly created GpgKey"))
            ),
        )?;
    Ok(record)
}
#[hdk_extern]
pub fn get_gpg_key(gpg_key_hash: ActionHash) -> ExternResult<Option<Record>> {
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
