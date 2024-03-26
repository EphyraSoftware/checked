use hdi::prelude::*;

pub mod prelude {
    pub use fetch_types::*;
    pub use crate::LinkTypes;
    pub use crate::{EntryTypes, UnitEntryTypes};
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[hdk_entry_types]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    AssetSignature(fetch_types::AssetSignature),
}

#[hdk_link_types]
pub enum LinkTypes {
    AssetUrlToSignature,
}
