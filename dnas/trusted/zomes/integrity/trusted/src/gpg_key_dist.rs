use crate::LinkTypes;
use chrono::{DateTime, Utc};
use hdi::prelude::{hash_type::AnyLinkable, *};

#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct GpgKeyDist {
    pub public_key: String,
    pub fingerprint: String,
    pub name: String,
    pub email: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
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
    Ok(ValidateCallbackResult::Invalid(String::from(
        "Gpg key distributions cannot be updated",
    )))
}

pub fn validate_delete_gpg_key_dist(
    _action: Delete,
    _original_action: EntryCreationAction,
    _original_gpg_key: GpgKeyDist,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(String::from(
        "Gpg key distributions cannot be deleted",
    )))
}

pub fn validate_create_gpg_key_dist_link(
    target_address: HoloHash<AnyLinkable>,
    link_type: LinkTypes,
) -> ExternResult<ValidateCallbackResult> {
    let entry_hash = match target_address.clone().try_into() {
        Ok(entry_hash) => entry_hash,
        Err(_) => {
            return Ok(ValidateCallbackResult::Invalid(format!(
                "The target address for {:?} must be an entry hash",
                link_type
            )));
        }
    };
    let entry = must_get_entry(entry_hash)?;
    match entry.as_app_entry() {
        Some(app_entry) => {
            match <SerializedBytes as TryInto<crate::gpg_key_dist::GpgKeyDist>>::try_into(
                app_entry.clone().into_sb(),
            ) {
                Ok(_) => Ok(ValidateCallbackResult::Valid),
                Err(_) => Ok(ValidateCallbackResult::Invalid(format!(
                    "The target for {:?} must be a {}",
                    link_type,
                    std::any::type_name::<crate::gpg_key_dist::GpgKeyDist>()
                ))),
            }
        }
        None => Ok(ValidateCallbackResult::Invalid(format!(
            "The target for {:?} must be an app entry",
            link_type
        ))),
    }
}
