use hdi::prelude::*;

#[hdk_entry_helper]
pub struct AssetSignature {
    pub signature: Vec<u8>,

    /// The address of the public key that signed this asset.
    ///
    /// NOTE: This action hash refers to the `PublicKeyDist` entry which is defined in a different
    /// zome (signing_keys) but the same DHT.
    pub key_dist_address: ActionHash,
}
