pub(crate) mod gpg_key_dist;
pub(crate) mod key_collection;

use hdi::prelude::*;

pub mod prelude {
    pub use crate::gpg_key_dist::*;
    pub use crate::key_collection::*;
    pub use crate::LinkTypes;
    pub use crate::{EntryTypes, UnitEntryTypes};
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[hdk_entry_types]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    GpgKeyDist(gpg_key_dist::GpgKeyDist),
    #[entry_type(visibility = "private")]
    KeyCollection(key_collection::KeyCollection),
}

#[hdk_link_types]
pub enum LinkTypes {
    UserIdToGpgKeyDist,
    EmailToGpgKeyDist,
    FingerprintToGpgKeyDist,
    KeyCollection,
}

// Validation you perform during the genesis process. Nobody else on the network performs it, only you.
// There *is no* access to network calls in this callback
#[hdk_extern]
pub fn genesis_self_check(_data: GenesisSelfCheckData) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}

// Validation the network performs when you try to join, you can't perform this validation yourself as you are not a member yet.
// There *is* access to network calls in this function
pub fn validate_agent_joining(
    _agent_pub_key: AgentPubKey,
    _membrane_proof: &Option<MembraneProof>,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}

// This is the unified validation callback for all entries and link types in this integrity zome
// Below is a match template for all of the variants of `DHT Ops` and entry and link types
//
// Holochain has already performed the following validation for you:
// - The action signature matches on the hash of its content and is signed by its author
// - The previous action exists, has a lower timestamp than the new action, and incremented sequence number
// - The previous action author is the same as the new action author
// - The timestamp of each action is after the DNA's origin time
// - AgentActivity authorities check that the agent hasn't forked their chain
// - The entry hash in the action matches the entry content
// - The entry type in the action matches the entry content
// - The entry size doesn't exceed the maximum entry size (currently 4MB)
// - Private entry types are not included in the Op content, and public entry types are
// - If the `Op` is an update or a delete, the original action exists and is a `Create` or `Update` action
// - If the `Op` is an update, the original entry exists and is of the same type as the new one
// - If the `Op` is a delete link, the original action exists and is a `CreateLink` action
// - Link tags don't exceed the maximum tag size (currently 1KB)
// - Countersigned entries include an action from each required signer
//
// You can read more about validation here: https://docs.rs/hdi/latest/hdi/index.html#data-validation
#[hdk_extern]
pub fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
    match op.flattened::<EntryTypes, LinkTypes>()? {
        FlatOp::StoreEntry(store_entry) => match store_entry {
            OpEntry::CreateEntry { app_entry, action } => match app_entry {
                EntryTypes::GpgKeyDist(gpg_key) => gpg_key_dist::validate_create_gpg_key_dist(
                    EntryCreationAction::Create(action),
                    gpg_key,
                ),
                EntryTypes::KeyCollection(key_collection) => {
                    key_collection::validate_create_key_collection(EntryCreationAction::Create(action), key_collection)
                }
            },
            OpEntry::UpdateEntry {
                app_entry, action, ..
            } => match app_entry {
                EntryTypes::GpgKeyDist(gpg_key) => gpg_key_dist::validate_create_gpg_key_dist(
                    EntryCreationAction::Update(action),
                    gpg_key,
                ),
                _ => {
                    Ok(ValidateCallbackResult::Invalid("todo: update entry".to_string()))
                }
            },
            _ => Ok(ValidateCallbackResult::Valid),
        },
        FlatOp::RegisterUpdate(update_entry) => match update_entry {
            OpUpdate::Entry {
                original_action,
                original_app_entry,
                app_entry,
                action,
            } => match (app_entry, original_app_entry) {
                (EntryTypes::GpgKeyDist(gpg_key), EntryTypes::GpgKeyDist(original_gpg_key)) => {
                    gpg_key_dist::validate_update_gpg_key_dist(
                        action,
                        gpg_key,
                        original_action,
                        original_gpg_key,
                    )
                }
                _ => Ok(ValidateCallbackResult::Invalid("todo: register update".to_string())),
            },
            _ => Ok(ValidateCallbackResult::Valid),
        },
        FlatOp::RegisterDelete(delete_entry) => match delete_entry {
            OpDelete::Entry {
                original_action,
                original_app_entry,
                action,
            } => match original_app_entry {
                EntryTypes::GpgKeyDist(gpg_key) => {
                    gpg_key_dist::validate_delete_gpg_key_dist(action, original_action, gpg_key)
                }
                _ => Ok(ValidateCallbackResult::Invalid("todo: register delete".to_string())),
            },
            _ => Ok(ValidateCallbackResult::Valid),
        },
        FlatOp::RegisterCreateLink {
            base_address,
            target_address,
            link_type,
            action,
            ..
        } => match link_type {
            LinkTypes::FingerprintToGpgKeyDist
            | LinkTypes::UserIdToGpgKeyDist
            | LinkTypes::EmailToGpgKeyDist => {
                gpg_key_dist::validate_create_gpg_key_dist_link(target_address, link_type)
            }
            LinkTypes::KeyCollection => {
                key_collection::validate_key_collection_link(action, base_address, target_address, link_type)
            },
        },
        FlatOp::RegisterDeleteLink { .. } => Ok(ValidateCallbackResult::Invalid(String::from(
            "There are no link types in this integrity zome",
        ))),
        FlatOp::StoreRecord(store_record) => {
            match store_record {
                // Complementary validation to the `StoreEntry` Op, in which the record itself is validated
                // If you want to optimize performance, you can remove the validation for an entry type here and keep it in `StoreEntry`
                // Notice that doing so will cause `must_get_valid_record` for this record to return a valid record even if the `StoreEntry` validation failed
                OpRecord::CreateEntry { app_entry, action } => match app_entry {
                    EntryTypes::GpgKeyDist(gpg_key) => gpg_key_dist::validate_create_gpg_key_dist(
                        EntryCreationAction::Create(action),
                        gpg_key,
                    ),
                    _ => Ok(ValidateCallbackResult::Invalid("todo: store record".to_string())),
                },
                // Complementary validation to the `RegisterUpdate` Op, in which the record itself is validated
                // If you want to optimize performance, you can remove the validation for an entry type here and keep it in `StoreEntry` and in `RegisterUpdate`
                // Notice that doing so will cause `must_get_valid_record` for this record to return a valid record even if the other validations failed
                OpRecord::UpdateEntry {
                    original_action_hash,
                    app_entry,
                    action,
                    ..
                } => {
                    let original_record = must_get_valid_record(original_action_hash)?;
                    let original_action = original_record.action().clone();
                    let original_action = match original_action {
                        Action::Create(create) => EntryCreationAction::Create(create),
                        Action::Update(update) => EntryCreationAction::Update(update),
                        _ => {
                            return Ok(ValidateCallbackResult::Invalid(
                                "Original action for an update must be a Create or Update action"
                                    .to_string(),
                            ));
                        }
                    };
                    match app_entry {
                        EntryTypes::GpgKeyDist(gpg_key) => {
                            let result = gpg_key_dist::validate_create_gpg_key_dist(
                                EntryCreationAction::Update(action.clone()),
                                gpg_key.clone(),
                            )?;
                            if let ValidateCallbackResult::Valid = result {
                                let original_gpg_key: Option<gpg_key_dist::GpgKeyDist> =
                                    original_record
                                        .entry()
                                        .to_app_option()
                                        .map_err(|e| wasm_error!(e))?;
                                let original_gpg_key = match original_gpg_key {
                                    Some(gpg_key) => gpg_key,
                                    None => {
                                        return Ok(
                                            ValidateCallbackResult::Invalid(
                                                "The updated entry type must be the same as the original entry type"
                                                    .to_string(),
                                            ),
                                        );
                                    }
                                };
                                gpg_key_dist::validate_update_gpg_key_dist(
                                    action,
                                    gpg_key,
                                    original_action,
                                    original_gpg_key,
                                )
                            } else {
                                Ok(result)
                            }
                        }
                        _ => Ok(ValidateCallbackResult::Invalid("todo".to_string())),
                    }
                }
                // Complementary validation to the `RegisterDelete` Op, in which the record itself is validated
                // If you want to optimize performance, you can remove the validation for an entry type here and keep it in `RegisterDelete`
                // Notice that doing so will cause `must_get_valid_record` for this record to return a valid record even if the `RegisterDelete` validation failed
                OpRecord::DeleteEntry {
                    original_action_hash,
                    action,
                    ..
                } => {
                    let original_record = must_get_valid_record(original_action_hash)?;
                    let original_action = original_record.action().clone();
                    let original_action = match original_action {
                        Action::Create(create) => EntryCreationAction::Create(create),
                        Action::Update(update) => EntryCreationAction::Update(update),
                        _ => {
                            return Ok(ValidateCallbackResult::Invalid(
                                "Original action for a delete must be a Create or Update action"
                                    .to_string(),
                            ));
                        }
                    };
                    let app_entry_type = match original_action.entry_type() {
                        EntryType::App(app_entry_type) => app_entry_type,
                        _ => {
                            return Ok(ValidateCallbackResult::Valid);
                        }
                    };
                    let entry = match original_record.entry().as_option() {
                        Some(entry) => entry,
                        None => {
                            if original_action.entry_type().visibility().is_public() {
                                return Ok(
                                    ValidateCallbackResult::Invalid(
                                        "Original record for a delete of a public entry must contain an entry"
                                            .to_string(),
                                    ),
                                );
                            } else {
                                return Ok(ValidateCallbackResult::Valid);
                            }
                        }
                    };
                    let original_app_entry = match EntryTypes::deserialize_from_type(
                        app_entry_type.zome_index,
                        app_entry_type.entry_index,
                        entry,
                    )? {
                        Some(app_entry) => app_entry,
                        None => {
                            return Ok(
                                ValidateCallbackResult::Invalid(
                                    "Original app entry must be one of the defined entry types for this zome"
                                        .to_string(),
                                ),
                            );
                        }
                    };
                    match original_app_entry {
                        EntryTypes::GpgKeyDist(original_gpg_key) => {
                            gpg_key_dist::validate_delete_gpg_key_dist(
                                action,
                                original_action,
                                original_gpg_key,
                            )
                        }
                        _ => Ok(ValidateCallbackResult::Invalid("todo".to_string())),
                    }
                }
                // Complementary validation to the `RegisterCreateLink` Op, in which the record itself is validated
                // If you want to optimize performance, you can remove the validation for an entry type here and keep it in `RegisterCreateLink`
                // Notice that doing so will cause `must_get_valid_record` for this record to return a valid record even if the `RegisterCreateLink` validation failed
                OpRecord::CreateLink {
                    base_address,
                    target_address,
                    link_type,
                    action,
                    ..
                } => match link_type {
                    LinkTypes::FingerprintToGpgKeyDist
                    | LinkTypes::UserIdToGpgKeyDist
                    | LinkTypes::EmailToGpgKeyDist => {
                        gpg_key_dist::validate_create_gpg_key_dist_link(target_address, link_type)
                    }
                    LinkTypes::KeyCollection => {
                        key_collection::validate_key_collection_link(action, base_address, target_address, link_type)
                    }
                },
                // Complementary validation to the `RegisterDeleteLink` Op, in which the record itself is validated
                // If you want to optimize performance, you can remove the validation for an entry type here and keep it in `RegisterDeleteLink`
                // Notice that doing so will cause `must_get_valid_record` for this record to return a valid record even if the `RegisterDeleteLink` validation failed
                OpRecord::DeleteLink { .. } => Ok(ValidateCallbackResult::Invalid(
                    "There are no link types in this integrity zome".to_string(),
                )),
                OpRecord::CreatePrivateEntry { .. } => Ok(ValidateCallbackResult::Valid),
                OpRecord::UpdatePrivateEntry { .. } => Ok(ValidateCallbackResult::Valid),
                OpRecord::CreateCapClaim { .. } => Ok(ValidateCallbackResult::Valid),
                OpRecord::CreateCapGrant { .. } => Ok(ValidateCallbackResult::Valid),
                OpRecord::UpdateCapClaim { .. } => Ok(ValidateCallbackResult::Valid),
                OpRecord::UpdateCapGrant { .. } => Ok(ValidateCallbackResult::Valid),
                OpRecord::Dna { .. } => Ok(ValidateCallbackResult::Valid),
                OpRecord::OpenChain { .. } => Ok(ValidateCallbackResult::Valid),
                OpRecord::CloseChain { .. } => Ok(ValidateCallbackResult::Valid),
                OpRecord::InitZomesComplete { .. } => Ok(ValidateCallbackResult::Valid),
                _ => Ok(ValidateCallbackResult::Valid),
            }
        }
        FlatOp::RegisterAgentActivity(agent_activity) => match agent_activity {
            OpActivity::CreateAgent { agent, action } => {
                let previous_action = must_get_action(action.prev_action)?;
                match previous_action.action() {
                        Action::AgentValidationPkg(
                            AgentValidationPkg { membrane_proof, .. },
                        ) => validate_agent_joining(agent, membrane_proof),
                        _ => {
                            Ok(
                                ValidateCallbackResult::Invalid(
                                    "The previous action for a `CreateAgent` action must be an `AgentValidationPkg`"
                                        .to_string(),
                                ),
                            )
                        }
                    }
            }
            _ => Ok(ValidateCallbackResult::Valid),
        },
    }
}
