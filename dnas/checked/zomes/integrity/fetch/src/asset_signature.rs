use super::UnitEntryTypes;
use crate::prelude::make_asset_url_address;
use crate::LinkTypes;
use fetch_types::AssetSignature;
use hdi::prelude::*;

pub(crate) fn validate_create_asset_signature(
    create_action: EntryCreationAction,
    asset_signature: AssetSignature,
) -> ExternResult<ValidateCallbackResult> {
    let agent_activity = must_get_agent_activity(
        create_action.author().clone(),
        ChainFilter::new(create_action.prev_action().clone()),
    )?;

    // Not a perfect match because it's a type defined in another zome. We just check that the key
    // dist address points to a real action on this agent's chain.
    let maybe_found_key_dist = agent_activity.iter().find(|activity| {
        activity.action.action_address() == &asset_signature.key_dist_address
            && match activity.action.action() {
                Action::Create(create) => create_action.author() == &create.author,
                _ => false,
            }
    });

    if maybe_found_key_dist.is_none() {
        return Ok(ValidateCallbackResult::Invalid(
            "The key dist address does not point to a valid action on the author's chain"
                .to_string(),
        ));
    }

    let entry_type: EntryType = UnitEntryTypes::AssetSignature.try_into()?;

    let previous_asset_signatures =
        agent_activity
            .iter()
            .filter(|activity| match activity.action.action() {
                Action::Create(create) => {
                    create_action.author() == &create.author && entry_type == create.entry_type
                }
                _ => false,
            });

    for activity in previous_asset_signatures {
        match activity.action.action() {
            Action::Create(create) => {
                // TODO any loss of entry data would prevent future validation, is that possible?
                let entry = must_get_entry(create.entry_hash.clone())?;
                let entry: AssetSignature = entry.try_into()?;
                if entry.fetch_url == asset_signature.fetch_url {
                    return Ok(ValidateCallbackResult::Invalid(
                        "An asset signature with the same fetch URL already exists".to_string(),
                    ));
                }
            }
            _ => {
                // Only care about other creates. Note that ignoring deletes means that you can
                // delete a signature but not re-add it.
            }
        }
    }

    Ok(ValidateCallbackResult::Valid)
}

pub(crate) fn validate_delete_asset_signature(
    _original_action: EntryCreationAction,
    _delete: Delete,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}

pub(crate) fn validate_create_asset_url_to_signature_link(
    action: CreateLink,
    base_address: AnyLinkableHash,
    target_address: AnyLinkableHash,
    link_type: LinkTypes,
) -> ExternResult<ValidateCallbackResult> {
    if link_type != LinkTypes::AssetUrlToSignature {
        return Ok(ValidateCallbackResult::Invalid(
            "The link type is not AssetUrlToSignature".to_string(),
        ));
    }

    let target = must_get_valid_record(target_address.try_into().map_err(|_| {
        wasm_error!(WasmErrorInner::Guest(
            "Target address is not an action hash".to_string()
        ))
    })?)?;

    if target.signed_action.hashed.author() != &action.author {
        return Ok(ValidateCallbackResult::Invalid(
            "The target entry is not authored by the link author".to_string(),
        ));
    }

    let entry_type: EntryType = UnitEntryTypes::AssetSignature.try_into()?;

    if target
        .signed_action
        .hashed
        .content
        .entry_type()
        .ok_or_else(|| wasm_error!(WasmErrorInner::Guest("No entry type".to_string())))?
        != &entry_type
    {
        return Ok(ValidateCallbackResult::Invalid(
            "The target entry is not an AssetSignature".to_string(),
        ));
    }

    let asset_signature: AssetSignature = target
        .entry
        .into_option()
        .ok_or_else(|| wasm_error!(WasmErrorInner::Guest("No entry".to_string())))?
        .try_into()?;
    let expected_base_address = make_asset_url_address(&asset_signature.fetch_url)?;

    let base_address: ExternalHash = base_address.try_into().map_err(|_| {
        wasm_error!(WasmErrorInner::Guest(
            "Base address is not an ExternalHash".to_string()
        ))
    })?;

    if base_address != expected_base_address {
        return Ok(ValidateCallbackResult::Invalid(
            "The base address does not match the expected address for the asset URL".to_string(),
        ));
    }

    Ok(ValidateCallbackResult::Valid)
}

pub(crate) fn validate_delete_asset_url_to_signature_link(
    original_action: CreateLink,
    delete: DeleteLink,
) -> ExternResult<ValidateCallbackResult> {
    if original_action.author != delete.author {
        return Ok(ValidateCallbackResult::Invalid(
            "The delete author does not match the original author".to_string(),
        ));
    }

    let agent_activity = must_get_agent_activity(
        original_action.author.clone(),
        ChainFilter::new(delete.prev_action.clone()),
    )?;

    let original_asset_signature_address: ActionHash =
        original_action.target_address.try_into().map_err(|_| {
            wasm_error!(WasmErrorInner::Guest(
                "Original target address is not an ActionHash".to_string()
            ))
        })?;

    let maybe_delete_for_asset_signature =
        agent_activity
            .iter()
            .find(|activity| match activity.action.action() {
                Action::Delete(delete) => {
                    delete.deletes_address == original_asset_signature_address
                }
                _ => false,
            });

    if maybe_delete_for_asset_signature.is_none() {
        return Ok(ValidateCallbackResult::Invalid(
            "Refusing to remove link when target has not been deleted".to_string(),
        ));
    }

    Ok(ValidateCallbackResult::Valid)
}
