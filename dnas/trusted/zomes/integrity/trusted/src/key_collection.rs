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

pub fn validate_key_collection_link(
    action: CreateLink,
    base_address: AnyLinkableHash,
    target_address: AnyLinkableHash,
    link_type: LinkTypes,
) -> ExternResult<ValidateCallbackResult> {
    let action_author_anylinkable: AnyLinkableHash = action.author.clone().into();

    if action_author_anylinkable != base_address {
        return Ok(ValidateCallbackResult::Invalid(
            "Key collection must be linked from the author".to_string(),
        ));
    }

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
            match <SerializedBytes as TryInto<crate::key_collection::KeyCollection>>::try_into(app_entry.clone().into_sb()) {
                Ok(_) => Ok(ValidateCallbackResult::Valid),
                Err(_) => Ok(ValidateCallbackResult::Invalid(format!(
                    "The target for {:?} must be a {}",
                    link_type,
                    std::any::type_name::<crate::key_collection::KeyCollection>()
                ))),
            }
        }
        None => Ok(ValidateCallbackResult::Invalid(format!(
            "The target for {:?} must be an app entry",
            link_type
        ))),
    }
}
