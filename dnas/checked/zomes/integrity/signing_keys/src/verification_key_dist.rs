//! Distribute public keys to be used for verification.

use crate::convert::try_extract_entry_to_app_type;
use crate::prelude::*;
use checked_types::VerificationKeyType;
use hdi::prelude::*;
use signing_keys_types::{MarkVfKeyDistOpt, VerificationKeyDist, VerificationKeyDistMark};

pub const VERIFICATION_KEY_NAME_MIN_LENGTH: usize = 3;
pub const MAX_VF_KEY_DIST_COMPROMISED_NOTE_LENGTH: usize = 120;

// TODO validate creation rate?
pub fn validate_create_vf_key_dist(
    create_action: EntryCreationAction,
    vf_key: VerificationKeyDist,
) -> ExternResult<ValidateCallbackResult> {
    if vf_key.name.len() < VERIFICATION_KEY_NAME_MIN_LENGTH {
        return Ok(ValidateCallbackResult::Invalid(format!(
            "Verification key name must be at least {} characters",
            VERIFICATION_KEY_NAME_MIN_LENGTH
        )));
    }

    match vf_key.key_type {
        // Checks that:
        // - The key can be parsed as a MiniSign verification key
        // - The key does not have an expiry
        // - The proof signature is valid
        VerificationKeyType::MiniSignEd25519 => {
            let key = match try_read_mini_sign_vf_key(&vf_key.verification_key) {
                Ok(key) => key,
                Err(e) => {
                    return Ok(ValidateCallbackResult::Invalid(format!(
                        "Failed to read MiniSign verification key: {}",
                        e
                    )));
                }
            };

            if vf_key.expires_at.is_some() {
                return Ok(ValidateCallbackResult::Invalid(String::from(
                    "MiniSign verification keys do not support expiry",
                )));
            }

            if let Err(e) =
                check_mini_sign_proof(&key, vf_key.proof.as_bytes(), vf_key.proof_signature)
            {
                return Ok(ValidateCallbackResult::Invalid(format!(
                    "Failed to verify proof signature: {}",
                    e
                )));
            }
        }
    }

    let activity = {
        // Must check from the previous action otherwise this create action will show up and appear as an
        // existing distribution of the key we're checking isn't already present.
        must_get_agent_activity(
            create_action.author().clone(),
            ChainFilter::new(create_action.prev_action().clone()),
        )?
    };

    let entry_def: AppEntryDef = UnitEntryTypes::VerificationKeyDist.try_into()?;
    let entry_hashes = activity
        .into_iter()
        .filter_map(|activity| match activity.action.action() {
            Action::Create(Create {
                entry_type: EntryType::App(entry_type),
                entry_hash,
                ..
            }) if entry_type == &entry_def => Some(entry_hash.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();

    for entry_hash in entry_hashes {
        let entry: VerificationKeyDist =
            try_extract_entry_to_app_type(must_get_entry(entry_hash)?)?;
        if entry.verification_key == vf_key.verification_key {
            return Ok(ValidateCallbackResult::Invalid(String::from(
                "Verification key already distributed by this agent",
            )));
        }
    }

    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_update_vf_key_dist(
    _action: Update,
    _vf_key_dist: VerificationKeyDist,
    _original_action: EntryCreationAction,
    _original_vf_key_dist: VerificationKeyDist,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(String::from(
        "Verification key distributions cannot be updated",
    )))
}

pub fn validate_delete_vf_key_dist(
    _action: Delete,
    _original_action: EntryCreationAction,
    _original_vf_key_dist: VerificationKeyDist,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(String::from(
        "Verification key distributions cannot be deleted",
    )))
}

/// Validation for links of type [LinkTypes::AgentToVfKeyDist]
pub fn validate_create_agent_to_vf_key_dist_link(
    create_action: CreateLink,
    base_address: AnyLinkableHash,
    target_address: AnyLinkableHash,
    link_type: LinkTypes,
) -> ExternResult<ValidateCallbackResult> {
    // Should never be hit, would imply a mistake in `lib.rs`.
    assert_eq!(
        link_type,
        LinkTypes::AgentToVfKeyDist,
        "Wrong link type: {:?}",
        link_type
    );

    //
    // Check the base
    //
    let base_agent_pub_key: AgentPubKey = match base_address.try_into() {
        Ok(agent_pub_key) => agent_pub_key,
        Err(_) => {
            return Ok(ValidateCallbackResult::Invalid(format!(
                "The base address for {:?} must be an agent public key",
                link_type
            )));
        }
    };

    // Only permit links from 'me'
    if create_action.author != base_agent_pub_key {
        return Ok(ValidateCallbackResult::Invalid(format!(
            "The base address for {:?} must be the author of the link",
            link_type
        )));
    }

    //
    // Check the target
    // - is owned by the same author
    // - is a VerificationKeyDist
    //
    let vf_key_dist_address = match target_address.clone().try_into() {
        Ok(action_hash) => action_hash,
        Err(_) => {
            return Ok(ValidateCallbackResult::Invalid(format!(
                "The target address for {:?} must be an action hash",
                link_type
            )));
        }
    };
    let record = must_get_valid_record(vf_key_dist_address)?;

    if record.action().author() != &create_action.author {
        return Ok(ValidateCallbackResult::Invalid(format!(
            "The target for {:?} must be owned by the author of the link",
            link_type
        )));
    }

    if try_extract_entry_to_app_type::<_, VerificationKeyDist>(record).is_err() {
        return Ok(ValidateCallbackResult::Invalid(format!(
            "The target for {:?} must be a {}",
            link_type,
            std::any::type_name::<VerificationKeyDist>()
        )));
    }

    Ok(ValidateCallbackResult::Valid)
}

/// Validation for links of type [LinkTypes::VfKeyDistToAgent]
pub fn validate_create_vf_key_dist_to_agent_link(
    create_action: CreateLink,
    base_address: AnyLinkableHash,
    target_address: AnyLinkableHash,
    link_type: LinkTypes,
) -> ExternResult<ValidateCallbackResult> {
    // Should never be hit, would imply a mistake in `lib.rs`.
    assert_eq!(
        link_type,
        LinkTypes::VfKeyDistToAgent,
        "Wrong link type: {:?}",
        link_type
    );

    //
    // Check the base
    // - is owned by the same author
    // - is a VerificationKeyDist
    //
    let vf_key_dist_address = match base_address.clone().try_into() {
        Ok(action_hash) => action_hash,
        Err(_) => {
            return Ok(ValidateCallbackResult::Invalid(format!(
                "The base address for {:?} must be a verification key dist address",
                link_type
            )));
        }
    };
    let record = must_get_valid_record(vf_key_dist_address)?;

    if record.action().author() != &create_action.author {
        return Ok(ValidateCallbackResult::Invalid(format!(
            "The base for {:?} must be owned by the author of the link",
            link_type
        )));
    }

    if try_extract_entry_to_app_type::<_, VerificationKeyDist>(record).is_err() {
        return Ok(ValidateCallbackResult::Invalid(format!(
            "The base for {:?} must be a {}",
            link_type,
            std::any::type_name::<VerificationKeyDist>()
        )));
    }

    //
    // Check the target
    //
    let target_agent_pub_key: AgentPubKey = match target_address.try_into() {
        Ok(agent_pub_key) => agent_pub_key,
        Err(_) => {
            return Ok(ValidateCallbackResult::Invalid(format!(
                "The target address for {:?} must be an agent public key",
                link_type
            )));
        }
    };

    // Only permit links to 'me'
    if create_action.author != target_agent_pub_key {
        return Ok(ValidateCallbackResult::Invalid(format!(
            "The target address for {:?} must be the author of the link",
            link_type
        )));
    }

    Ok(ValidateCallbackResult::Valid)
}

pub(crate) fn validate_create_vf_key_dist_mark(
    create_action: EntryCreationAction,
    mark: VerificationKeyDistMark,
) -> ExternResult<ValidateCallbackResult> {
    //
    // If the mark is 'compromised' then:
    // - the note must be less than 120 characters.
    // - the since time must be before the action time. (i.e. you can't mark a key as compromised in the future!)
    //
    if let MarkVfKeyDistOpt::Compromised { note, since } = &mark.mark {
        if note.len() > MAX_VF_KEY_DIST_COMPROMISED_NOTE_LENGTH {
            return Ok(ValidateCallbackResult::Invalid(
                "The note for a compromised mark must be less than 120 characters".to_string(),
            ));
        }

        if since > create_action.timestamp() {
            return Ok(ValidateCallbackResult::Invalid(
                "The 'since' time for a compromised mark must be before the action time"
                    .to_string(),
            ));
        }
    }

    //
    // The mark must be applied to a verification key distribution that is owned by the same author.
    //
    let vf_key_dist_record = must_get_valid_record(mark.verification_key_dist_address.clone())?;
    if create_action.author() != vf_key_dist_record.signed_action.hashed.author() {
        return Ok(ValidateCallbackResult::Invalid(
            "Only the author of the verification key distribution can mark it".to_string(),
        ));
    }

    //
    // The address to mark MUST point to a VerificationKeyDist entry.
    //
    try_extract_entry_to_app_type::<_, VerificationKeyDist>(vf_key_dist_record.clone())?;

    //
    // If the mark is 'rotated' then the new_verification_key_dist_address must point to a VerificationKeyDist, owned by the same author.
    //
    if let MarkVfKeyDistOpt::Rotated {
        new_verification_key_dist_address,
    } = &mark.mark
    {
        let new_vf_key_dist_record =
            must_get_valid_record(new_verification_key_dist_address.clone())?;
        try_extract_entry_to_app_type::<_, VerificationKeyDist>(new_vf_key_dist_record.clone())?;

        if vf_key_dist_record.action().author() != new_vf_key_dist_record.action().author() {
            return Ok(ValidateCallbackResult::Invalid(
                "The new verification key distribution must be owned by the same author"
                    .to_string(),
            ));
        }
    }

    //
    // Check that there is only one mark of each type for a key.
    //
    // It's valid to rotate a compromised key because the old key does not need to be signing_keys in order
    // to do that. It's your agent key that is signing_keys to rotate the key.
    // It's valid to mark a key as compromised because you might not know it had been compromised until
    // after you rotated it.
    //

    // Want all activity previous to this action.
    let activity = must_get_agent_activity(
        create_action.author().clone(),
        ChainFilter::new(create_action.prev_action().clone()),
    )?;

    // Usually expect only 1 mark for a key, so don't allocate memory here until we need it.
    let entry_def: AppEntryDef = UnitEntryTypes::VerificationKeyDistMark.try_into()?;
    let mut other_marks_for_vf_key_dist = Vec::with_capacity(0);
    for activity in activity.into_iter() {
        match activity.action.action() {
            Action::Create(Create {
                entry_type: EntryType::App(entry_type),
                entry_hash,
                ..
            }) if entry_type == &entry_def => {
                let existing_mark: VerificationKeyDistMark =
                    try_extract_entry_to_app_type(must_get_entry(entry_hash.clone())?)?;

                // We only want the marks that apply to the same key
                if existing_mark.verification_key_dist_address == mark.verification_key_dist_address
                {
                    other_marks_for_vf_key_dist.push(existing_mark);
                }
            }
            _ => continue,
        };
    }

    match mark.mark {
        MarkVfKeyDistOpt::Rotated { .. } => {
            if other_marks_for_vf_key_dist
                .iter()
                .any(|m| matches!(m.mark, MarkVfKeyDistOpt::Rotated { .. }))
            {
                return Ok(ValidateCallbackResult::Invalid(
                    "A key can only be rotated once".to_string(),
                ));
            }
        }
        MarkVfKeyDistOpt::Compromised { .. } => {
            if other_marks_for_vf_key_dist
                .iter()
                .any(|m| matches!(m.mark, MarkVfKeyDistOpt::Compromised { .. }))
            {
                return Ok(ValidateCallbackResult::Invalid(
                    "A key can only be marked as compromised once".to_string(),
                ));
            }
        }
    }

    Ok(ValidateCallbackResult::Valid)
}

pub fn verify_vf_key_dist_to_mark_link(
    _link: CreateLink,
    _base: AnyLinkableHash,
    _target: AnyLinkableHash,
    _link_type: LinkTypes,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}
