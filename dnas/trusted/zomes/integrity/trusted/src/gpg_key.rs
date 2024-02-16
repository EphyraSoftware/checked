use hdi::prelude::*;
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct GpgKey {
    pub public_key: Vec<u32>,
    pub fingerprint: String,
}
pub fn validate_create_gpg_key(
    _action: EntryCreationAction,
    _gpg_key: GpgKey,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}
pub fn validate_update_gpg_key(
    _action: Update,
    _gpg_key: GpgKey,
    _original_action: EntryCreationAction,
    _original_gpg_key: GpgKey,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(String::from("Gpg Keys cannot be updated")))
}
pub fn validate_delete_gpg_key(
    _action: Delete,
    _original_action: EntryCreationAction,
    _original_gpg_key: GpgKey,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(String::from("Gpg Keys cannot be deleted")))
}
