import {CallableCell} from "@holochain/tryorama";
import {AgentPubKey, Record} from "@holochain/client";

const utf8Encode = new TextEncoder();

export interface PrepareFetchRequest {
    fetch_url: string;
}

export interface FetchCheckSignaturePinned {
    author: AgentPubKey;
    key_collection: string;
    key_name: string;
}

export type FetchCheckSignatureReason =
    | { RandomRecent: null }
    | { RandomHistorical: null }
    | { Pinned: FetchCheckSignaturePinned };

export interface FetchCheckSignature {
    signature: Uint8Array;
    reason: FetchCheckSignatureReason;
}

export interface CreateAssetSignature {
    fetch_url: string;
    signature: number[];
    key_type: { MiniSignEd25519: null };
    verification_key: string;
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
}

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

export const sampleFetchKey = () => {
    return `
untrusted comment: minisign public key: B7ED5BB003A859F1
RWTxWagDsFvtt+e4V5KJAahwTh381E6PMTFvgvGYsnWLXCwIxe4YE/sM
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

export const sampleFetchAsset = () => {
    // The formatting of this must be EXACT, since this content is being signed
    return "hello asset\n";
}

export const sampleFetchAssetSignature = () => {
    // Similar with the signature, this must be EXACT. Removing whitespace is permitted but extra whitespace is not.
    return Array.from(
        utf8Encode.encode(`untrusted comment: signature from rsign secret key
RUTxWagDsFvtt7zP24HuGOxNlWk93OYpP9dJJ3k6y+ZEQ7Ym56Loy5/KusNLnWHi0PCOB5ore+kK+wfImf+ZwFewvLuBPE3tYAU=
trusted comment: timestamp:1711592494\tfile:sample-asset.txt\tprehashed
TlTxfjqY85HlrHFPjOgCmhFIKfH5Jz2MoE+zJMju9iLJ150Zdidzm3ucRsk4wU3B/OEifaL6clxmJGSWZ3bcAg==
`),
    );
}
