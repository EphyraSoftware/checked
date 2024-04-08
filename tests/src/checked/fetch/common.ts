import { CallableCell } from "@holochain/tryorama";
import { ActionHash, AgentPubKey } from "@holochain/client";

const utf8Encode = new TextEncoder();

export interface PrepareFetchRequest {
  fetch_url: string;
}

export interface FetchCheckSignaturePinned {
  key_collection: string;
  key_name: string;
}

export type FetchCheckSignatureReason =
  | { RandomRecent: null }
  | { RandomHistorical: null }
  | { Pinned: FetchCheckSignaturePinned };

export interface FetchCheckSignature {
  signature: Uint8Array;
  key_type: { MiniSignEd25519: null };
  verification_key: string;
  author: AgentPubKey;
  key_dist_address: ActionHash;
  reason: FetchCheckSignatureReason;
}

export interface CreateAssetSignature {
  fetch_url: string;
  signature: string;
  key_type: { MiniSignEd25519: null };
  verification_key: string;
}

export interface DeleteAssetSignatureRequest {
    fetch_url: string;
}

export interface AssetSignatureResponse {
    fetch_url: string;
    signature: string;
    key_dist_address: ActionHash;
    created_at: number;
}

export const prepareFetch = async (
  cell: CallableCell,
  request: PrepareFetchRequest,
): Promise<FetchCheckSignature[]> => {
  return cell.callZome({
    zome_name: "fetch",
    fn_name: "prepare_fetch",
    payload: request,
  });
};

export const createAssetSignature = async (
  cell: CallableCell,
  request: CreateAssetSignature,
) => {
  return cell.callZome({
    zome_name: "fetch",
    fn_name: "create_asset_signature",
    payload: request,
  });
};

export const getMyAssetSignatures = async (cell: CallableCell): Promise<AssetSignatureResponse[]> => {
    return cell.callZome({
        zome_name: "fetch",
        fn_name: "get_my_asset_signatures",
        payload: null,
    });
}

export const deleteAssetSignature = async (cell: CallableCell, request: DeleteAssetSignatureRequest): Promise<null> => {
    return cell.callZome({
        zome_name: "fetch",
        fn_name: "delete_asset_signature",
        payload: request,
    });
}

export const sampleFetchKey = () => {
  return `
untrusted comment: minisign public key: B7ED5BB003A859F1
RWTxWagDsFvtt+e4V5KJAahwTh381E6PMTFvgvGYsnWLXCwIxe4YE/sM
`;
};

export const sampleFetchKeyOther = () => {
  return `
untrusted comment: minisign public key: F6B62CB533B02416
RWQWJLAztSy29s6SlUzvA2lV3X7+CMJRSeFQSjvW/xwetDEQi5arNq/J
`;
};

export const sampleFetchKeyProof = () => {
  // The formatting of this must be EXACT, since this content is being signed
  return "proof\n";
};

export const sampleFetchKeyProofSignature = () => {
  // Similar with the signature, this must be EXACT. Removing whitespace is permitted but extra whitespace is not.
  return Array.from(
    utf8Encode.encode(`untrusted comment: signature from rsign secret key
RUTxWagDsFvtt0iU8ObmWDVb70Bzh8xJ3oaJ0bADndtOdFpf9LmsPlZlcnqfQ7kVnFNE8T8phqo2ieK/L/ajPt5kRwYInCoWSQE=
trusted comment: timestamp:1711592643\tfile:proof.txt\tprehashed
1Qj9ogn/ieaUlJVXD6m3E88EeytS9fmJlh3Phcvq9yxI/eEmpiNH/culPFxAEXEXRwJ9jPmU0tqdoFOX3wU4Bw==
`),
  );
};

// Proof for `sampleFetchKeyOther`
export const sampleFetchKeyOtherProofSignature = () => {
  // Similar with the signature, this must be EXACT. Removing whitespace is permitted but extra whitespace is not.
  return Array.from(
    utf8Encode.encode(`untrusted comment: signature from rsign secret key
RUQWJLAztSy29u8kzptxH22hL+snYIoV9OHdggAmqdQOCUZpaFc9awKHR//VA/6w7iTCiK07U/MhXCwbJoVffobj+EIp4JovkAY=
trusted comment: timestamp:1711770738\tfile:proof.txt\tprehashed
zvqpLC4ZziJ5Z8DRtugyjkDn/oHtKu6o71acc0H9dnCmpJVdYsyXxVcvKCCUSNTe7dQDm3Dc7LGvsgYXk0UWBg==
`),
  );
};

// Primary signature for `sampleFetchAsset`
export const sampleFetchAssetSignature = () => {
  // Similar with the signature, this must be EXACT. Removing whitespace is permitted but extra whitespace is not.
  return `untrusted comment: signature from rsign secret key
RUTxWagDsFvtt7zP24HuGOxNlWk93OYpP9dJJ3k6y+ZEQ7Ym56Loy5/KusNLnWHi0PCOB5ore+kK+wfImf+ZwFewvLuBPE3tYAU=
trusted comment: timestamp:1711592494\tfile:sample-asset.txt\tprehashed
TlTxfjqY85HlrHFPjOgCmhFIKfH5Jz2MoE+zJMju9iLJ150Zdidzm3ucRsk4wU3B/OEifaL6clxmJGSWZ3bcAg==
`;
};

// Alternate signature that will match `sampleFetchAsset`
export const sampleFetchOtherAssetSignature = () => {
  // Similar with the signature, this must be EXACT. Removing whitespace is permitted but extra whitespace is not.
  return `untrusted comment: signature from rsign secret key
RUQWJLAztSy29pIrQ81/cjlDxoU4docg3ox7hU241qV4G4IApnBMBLuk6mkrWWk4eQPF1IdVkL78y0yLepHpXerf4aJzuWIPPg0=
trusted comment: timestamp:1711757915\tfile:sample-asset.txt\tprehashed
nmLg2TZIzWELS+DY87/dd/SFzhqrE6LCww5+prCeoa4aNJVKeNLAwNcj3NBF9PmRcXMWb6Adw1UL2MUvL+NzBw==
`;
};
