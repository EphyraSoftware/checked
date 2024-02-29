use hdi::prelude::*;

use crate::LinkTypes;
use crate::UnitEntryTypes;

pub const KEY_COLLECTION_LIMIT: usize = 10;
pub const KEY_COLLECTION_NAME_MIN_LENGTH: usize = 3;

#[hdk_entry_helper]
pub struct KeyCollection {
    pub name: String,
}

pub fn validate_create_key_collection(
    create_action: EntryCreationAction,
    key_collection: KeyCollection,
) -> ExternResult<ValidateCallbackResult> {
    if key_collection.name.len() < KEY_COLLECTION_NAME_MIN_LENGTH {
        return Ok(ValidateCallbackResult::Invalid(
            format!(
                "Key collection name must be at least {} characters long",
                KEY_COLLECTION_NAME_MIN_LENGTH
            )
            .to_string(),
        ));
    }

    let entry_def: AppEntryDef = UnitEntryTypes::KeyCollection.try_into()?;

    let action: Action = create_action.clone().into();
    let action_hash = hash_action(action.clone())?;
    let activity = must_get_agent_activity(action.author().clone(), ChainFilter::new(action_hash))?;

    // Find all key collection creates
    let mut key_collection_creates: HashSet<_> = activity
        .iter()
        .filter_map(|activity| match activity.action.action() {
            Action::Create(Create {
                entry_type: EntryType::App(app_entry),
                entry_hash,
                ..
            }) if app_entry == &entry_def => Some(entry_hash),
            _ => None,
        })
        .collect();

    // Run through every delete and grab the entry hash that it deletes, then remove those entries from the key collection set
    activity
        .iter()
        .filter_map(|activity| match activity.action.action() {
            Action::Delete(Delete {
                deletes_entry_address,
                ..
            }) => Some(deletes_entry_address),
            _ => None,
        })
        .for_each(|entry_address| {
            key_collection_creates.remove(&entry_address);
        });

    // Now check the remaining number of key collections is under the limit
    // Note that being at the limit is allowed because the newly created key collection is already in the agent activity.
    if key_collection_creates.len() > KEY_COLLECTION_LIMIT {
        return Ok(ValidateCallbackResult::Invalid(
            "Exceeded the maximum number of key collections".to_string(),
        ));
    }

    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_key_collection_to_gpg_key_dist_link(
    target_address: AnyLinkableHash,
    link_type: LinkTypes,
) -> ExternResult<ValidateCallbackResult> {
    let gpg_key_dist_hash: EntryHash = match target_address.clone().try_into() {
        Ok(entry_hash) => entry_hash,
        Err(_) => {
            return Ok(ValidateCallbackResult::Invalid(format!(
                "The target address for {:?} must be an entry hash",
                link_type
            )));
        }
    };

    let maybe_gpg_key_dist_entry = must_get_entry(gpg_key_dist_hash)?;

    match maybe_gpg_key_dist_entry.as_app_entry() {
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

pub fn validate_gpg_key_dist_to_key_collection_link(
    action: CreateLink,
    base_address: AnyLinkableHash,
    target_address: AnyLinkableHash,
    link_type: LinkTypes,
    tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    // Check that the base is a GpgKeyDist entry hash
    let base_entry_hash = base_address.try_into().map_err(|_| {
        wasm_error!(WasmErrorInner::Guest(format!(
            "The base address for {:?} must be an entry hash",
            link_type
        )))
    })?;
    let maybe_gpg_key_dist_entry = must_get_entry(base_entry_hash)?;

    match maybe_gpg_key_dist_entry.as_app_entry() {
        Some(app_entry) => {
            match <SerializedBytes as TryInto<crate::gpg_key_dist::GpgKeyDist>>::try_into(
                app_entry.clone().into_sb(),
            ) {
                Ok(_) => {
                    // Valid
                }
                Err(_) => {
                    return Ok(ValidateCallbackResult::Invalid(format!(
                        "The base for {:?} must be a {}",
                        link_type,
                        std::any::type_name::<crate::gpg_key_dist::GpgKeyDist>()
                    )))
                }
            }
        }
        None => {
            return Ok(ValidateCallbackResult::Invalid(format!(
                "The base for {:?} must be an app entry",
                link_type
            )))
        }
    }

    // Check that the target is a KeyCollection typed action hash
    let target_action_hash = target_address.try_into().map_err(|_| {
        wasm_error!(WasmErrorInner::Guest(format!(
            "The target address for {:?} must be an action hash",
            link_type
        )))
    })?;
    let key_collection_action = must_get_action(target_action_hash)?;

    let entry_def: AppEntryDef = UnitEntryTypes::KeyCollection.try_into()?;
    match key_collection_action.action() {
        Action::Create(Create {
            entry_type: EntryType::App(def),
            ..
        }) if def == &entry_def => {
            // Valid
        }
        _ => {
            return Ok(ValidateCallbackResult::Invalid(format!(
                "The target for {:?} must be a {}",
                link_type,
                std::any::type_name::<crate::key_collection::KeyCollection>()
            )))
        }
    }

    // Check that the link in the opposite direction to this one exists
    let scoped_link_type: ScopedLinkType = link_type.try_into()?;
    let action_hash = hash_action(Action::CreateLink(action.clone()))?;
    let activity = must_get_agent_activity(action.author.clone(), ChainFilter::new(action_hash))?;
    let found_reverse_link = activity.into_iter().any(|a| {
        matches!(a.action.action(), Action::CreateLink(CreateLink { link_type, tag: t, .. }) if link_type == &scoped_link_type.zome_type && t == &tag)
    });

    if !found_reverse_link {
        return Ok(ValidateCallbackResult::Invalid(format!(
            "The reverse link for {:?} must exist",
            link_type
        )));
    }

    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_delete_gpg_key_dist_to_key_collection_link(
    original_action: CreateLink,
    action: DeleteLink,
) -> ExternResult<ValidateCallbackResult> {
    if original_action.author != action.author {
        return Ok(ValidateCallbackResult::Invalid(
            "The author of the create link and the delete link must be the same".to_string(),
        ));
    }

    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_delete_key_collection_to_gpg_key_dist_link(
    original_action: CreateLink,
    action: DeleteLink,
) -> ExternResult<ValidateCallbackResult> {
    if original_action.author != action.author {
        return Ok(ValidateCallbackResult::Invalid(
            "The author of the create link and the delete link must be the same".to_string(),
        ));
    }

    let original_action_hash = {
        let action: Action = original_action.clone().into();
        hash_action(action)?
    };
    let action_hash = {
        let action: Action = action.clone().into();
        hash_action(action)?
    };
    let activity = must_get_agent_activity(
        action.author,
        ChainFilter::new(action_hash).until(original_action_hash),
    )?;

    // Look for the reverse create link, needed to find the associated delete.
    let reverse_link_type = {
        let scoped_link_type: ScopedLinkType = LinkTypes::GpgKeyDistToKeyCollection.try_into()?;
        scoped_link_type.zome_type
    };
    let matched_gpg_key_dist_to_collection_creates = activity
        .iter()
        .filter(|agent_activity| matches!(agent_activity.action.action(), Action::CreateLink(CreateLink { link_type, tag, .. }) if *link_type == reverse_link_type && *tag == original_action.tag))
        .collect::<Vec<_>>();

    if matched_gpg_key_dist_to_collection_creates.len() > 1 {
        return Ok(ValidateCallbackResult::Invalid(
            "Found duplicate create links to delete, this should have been prevented on create"
                .to_string(),
        ));
    }

    if matched_gpg_key_dist_to_collection_creates.is_empty() {
        return Ok(ValidateCallbackResult::Invalid(
            "The create link to delete does not exist".to_string(),
        ));
    }

    let reverse_create = matched_gpg_key_dist_to_collection_creates
        .first()
        .ok_or_else(|| {
            wasm_error!(WasmErrorInner::Guest(
                "The create link to delete does not exist".to_string()
            ))
        })?;

    let reverse_create_hash = reverse_create.action.as_hash();
    let match_gpg_key_dist_to_collection_deletes = activity
        .iter()
        .filter(|agent_activity| {
            matches!(agent_activity.action.action(), Action::DeleteLink(DeleteLink {
                             link_add_address, ..
                         }) if link_add_address == reverse_create_hash)
        })
        .collect::<Vec<_>>();

    if match_gpg_key_dist_to_collection_deletes.len() > 1 {
        return Ok(ValidateCallbackResult::Invalid(
            "Found duplicate delete links, this is pointless".to_string(),
        ));
    }

    if match_gpg_key_dist_to_collection_deletes.is_empty() {
        return Ok(ValidateCallbackResult::Invalid(
            "Missing associated reverse deletion".to_string(),
        ));
    }

    // At this point, there is exactly one matched reverse delete and we're convinced that the 'public' link associated with out 'private' link has been removed.

    Ok(ValidateCallbackResult::Valid)
}
