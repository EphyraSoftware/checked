use hdi::prelude::*;

///! Conversions that are fiddly to code by hand again and again that should probably be affordances
///! of the SDK rather than being here.

pub fn try_extract_entry_to_app_type<T: TryFrom<SerializedBytes>>(entry: EntryHashed) -> ExternResult<T> {
    match entry.as_app_entry() {
        Some(app_entry) => {
            match <SerializedBytes as TryInto<T>>::try_into(app_entry.clone().into_sb()) {
                Ok(t) => Ok(t),
                Err(_) => Err(wasm_error!(WasmErrorInner::Guest(format!(
                    "Entry {:?} is not a {}",
                    entry.hash,
                    std::any::type_name::<T>()
                )))),
            }
        }
        None => Err(wasm_error!(WasmErrorInner::Guest(format!(
            "Entry {:?} is not an app entry of type {}",
            entry.hash,
            std::any::type_name::<T>()
        )))),
    }
}
