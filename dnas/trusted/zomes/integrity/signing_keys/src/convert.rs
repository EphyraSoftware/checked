use hdi::prelude::*;

pub struct AppTypeConvertible(AppEntryBytes);

impl TryFrom<EntryHashed> for AppTypeConvertible {
    type Error = WasmError;

    fn try_from(value: EntryHashed) -> ExternResult<Self> {
        value
            .as_app_entry()
            .cloned()
            .ok_or(wasm_error!(WasmErrorInner::Guest(
                "Not an app entry".into()
            )))
            .map(AppTypeConvertible)
    }
}

impl TryFrom<Record> for AppTypeConvertible {
    type Error = WasmError;

    fn try_from(value: Record) -> ExternResult<Self> {
        value
            .entry
            .as_option()
            .and_then(|e| e.as_app_entry())
            .cloned()
            .ok_or(wasm_error!(WasmErrorInner::Guest(
                "Not an app entry".into()
            )))
            .map(AppTypeConvertible)
    }
}

pub fn try_extract_entry_to_app_type<
    I: TryInto<AppTypeConvertible, Error = WasmError>,
    T: TryFrom<SerializedBytes>,
>(
    input: I,
) -> ExternResult<T> {
    let app_entry_bytes: AppTypeConvertible = input.try_into()?;
    match <SerializedBytes as TryInto<T>>::try_into(app_entry_bytes.0.into_sb()) {
        Ok(t) => Ok(t),
        Err(_) => Err(wasm_error!(WasmErrorInner::Guest(format!(
            "Input is not a {}",
            std::any::type_name::<T>()
        )))),
    }
}
