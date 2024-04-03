use hdi::prelude::*;

pub mod prelude {
    pub use crate::LinkTypes;
    pub use crate::{EntryTypes, UnitEntryTypes};
    pub use fetch_types::*;
}

// TODO validation for asset signatures and links

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[hdk_entry_types]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    AssetSignature(fetch_types::AssetSignature),
}

#[hdk_link_types]
pub enum LinkTypes {
    // TODO create links to make my own signatures discoverable in the UI
    AssetUrlToSignature,
}
