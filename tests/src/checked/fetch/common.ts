import {CallableCell} from "@holochain/tryorama";
import {AgentPubKey, Record} from "@holochain/client";

export interface PrepareFetchRequest {
    fetch_url: string;
}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct FetchCheckSignaturePinned {
//     pub author: AgentPubKey,
//     pub key_collection: String,
//     pub key_name: String,
// }
//
// #[derive(Serialize, Deserialize, Debug)]
// pub enum FetchCheckSignatureReason {
//     RandomRecent,
//     RandomHistorical,
//     Pinned(FetchCheckSignaturePinned),
// }
//
// #[derive(Serialize, Deserialize, Debug)]
// pub struct FetchCheckSignature {
//     signature: Vec<u8>,
//     reason: FetchCheckSignatureReason,
// }

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
