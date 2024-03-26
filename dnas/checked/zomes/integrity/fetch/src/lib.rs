mod asset_signature;

use hdi::prelude::*;

pub mod prelude {
    pub use crate::asset_signature::*;
    pub use crate::LinkTypes;
    pub use crate::{EntryTypes, UnitEntryTypes};
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[hdk_entry_types]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    AssetSignature(asset_signature::AssetSignature),
}

#[hdk_link_types]
pub enum LinkTypes {
    AssetUrlToSignature,
}
