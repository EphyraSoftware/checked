use hdi::prelude::*;
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct GpgKeyDist {
    pub public_key: String,
    pub fingerprint: String,
}

pub fn validate_create_gpg_key_dist(
    _action: EntryCreationAction,
    _gpg_key: GpgKeyDist,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_update_gpg_key_dist(
    _action: Update,
    _gpg_key: GpgKeyDist,
    _original_action: EntryCreationAction,
    _original_gpg_key: GpgKeyDist,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(String::from("Gpg key distributions cannot be updated")))
}

pub fn validate_delete_gpg_key_dist(
    _action: Delete,
    _original_action: EntryCreationAction,
    _original_gpg_key: GpgKeyDist,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(String::from("Gpg key distributions cannot be deleted")))
}
