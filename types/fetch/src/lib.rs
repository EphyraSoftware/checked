use hdi::prelude::*;

#[hdk_entry_helper]
pub struct AssetSignature {
    /// The URL that the asset was fetched from.
    ///
    /// Note that there is a link created from this `fetch_url` to the `AssetSignature` by the
    /// author of the signature. This field helps find other signatures for the same asset.
    pub fetch_url: String,

    /// The signature of the asset. Detached from the asset itself since we don't want to store that
    /// on the DHT.
    pub signature: String,

    /// The address of the public key that signed this asset.
    ///
    /// NOTE: This action hash refers to the `PublicKeyDist` entry which is defined in a different
    /// zome (signing_keys) but the same DHT.
    pub key_dist_address: ActionHash,
}
