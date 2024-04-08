use hdi::prelude::*;

pub fn make_asset_url_address(asset_url: &str) -> ExternResult<ExternalHash> {
    let mut url = url::Url::parse(asset_url)
        .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?;

    url.set_password(None).ok();
    url.set_username("").ok();

    let mut hash = holo_hash::blake2b_256(url.as_str().as_bytes());
    hash.extend_from_slice(&[0, 0, 0, 0]);
    Ok(ExternalHash::from_raw_36(hash))
}
