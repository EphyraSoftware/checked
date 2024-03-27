import { CallableCell } from "@holochain/tryorama";
import {ActionHash, AgentPubKey, Record} from "@holochain/client";
import { decode } from "@msgpack/msgpack";

const utf8Encode = new TextEncoder();

export const testAppPath = process.cwd() + "/../workdir/checked.happ";

export interface VerificationKeyDistMarkRotated {
  new_verification_key_dist_address: Uint8Array;
}

export interface VerificationKeyDistMarkCompromised {
  note: string;
  since: number;
}

export type VfKeyDistMark =
    | { Rotated: VerificationKeyDistMarkRotated }
    | { Compromised: VerificationKeyDistMarkCompromised };

export interface VerificationKeyDist {
  verification_key: string;
  key_type: { MiniSignEd25519: null };
  name: string;
  expires_at: number;
  marks: VfKeyDistMark[];
}

export interface VerificationKeyResponse {
  verification_key_dist: VerificationKeyDist;
  key_dist_address: number[];
  reference_count: number;
  created_at: number;
}

export interface KeyCollectionWithKeys {
  name: string;
  verification_keys: VerificationKeyResponse[];
}

export const decodeRecord = <T>(record: Record): T => {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return decode((record.entry as any).Present.entry) as T;
};

export const sampleMiniSignKey = () => {
  return `
untrusted comment: minisign public key 5DDF4BB342787FB5
RWS1f3hCs0vfXeaPCLyiQt9NDQ+MzReDNLz+kaw+hK9NV8nb9G7opa7q
`;
};

export const sampleMiniSignProof = () => {
  // The formatting of this must be EXACT, since this content is being signed
  return "some test data\n";
};

export const sampleMiniSignProofSignature = () => {
  // Similar with the signature, this must be EXACT. Removing whitespace is permitted but extra whitespace is not.
  return Array.from(
    utf8Encode.encode(`untrusted comment: signature from minisign secret key
RUS1f3hCs0vfXb4ExmkOtLWNkqaPkEyzEIRrcmHWyoJuSMUR3U7jx08hri3cr8EYyBNVnH1LOSdjY3Hfk2BQU15jMD25ub5sBAU=
trusted comment: timestamp:1709423483\tfile:test.txt\thashed
Gjpn4nbsrDPysp3Nl63GZO5YWaB0aiJljBlUOQWIYE6tgUL7inOyiYcx5EWb2yOKvwbIjRk3u0ShhgqBIwM7Dg==
`),
  );
};

export function sampleMiniSignKey2() {
  return `
untrusted comment: minisign public key 20AFFC89E77E7A09
RWQJen7nifyvIJgut8+D3v1aV+khU16lvgTWkz85fwM6wGKxGREH6jSh
`;
}

export const sampleMiniSignProof2 = () => {
  // The formatting of this must be EXACT, since this content is being signed
  return "some other test data";
};

export const sampleMiniSignProofSignature2 = () => {
  // Similar with the signature, this must be EXACT. Removing whitespace is permitted but extra whitespace is not.
  return Array.from(
    utf8Encode.encode(`untrusted comment: signature from minisign secret key
RUQJen7nifyvIAFpF3HuKaf34HochzUTI0lquynL1q+UdDIsdnI73D7n5sLkynkcfUbxjHcj1Jgrxl0kyC6ftEdD5VWpi6uadw0=
trusted comment: timestamp:1709495926\tfile:test.txt\thashed
Z9BIZ44mZzediyne0UqMhzz4wnKMINUGKp/gL0g5rixo+N+4mAbfK4caAoBWVVMRq172jw5EmKCYiaorK72uCQ==
`),
  );
};

export const distributeVerificationKey = async (
  cell: CallableCell,
  verificationKey: string,
  proof: string,
  proofSignature: number[],
): Promise<Record> => {
  return cell.callZome({
    zome_name: "signing_keys",
    fn_name: "distribute_verification_key",
    payload: {
      name: "test",
      verification_key: verificationKey,
      key_type: { MiniSignEd25519: null },
      proof,
      proof_signature: proofSignature,
    },
  });
};

export async function createKeyCollection(
  cell: CallableCell,
  name: string,
): Promise<Record> {
  return cell.callZome({
    zome_name: "signing_keys",
    fn_name: "create_key_collection",
    payload: { name },
  });
}

export const getMyKeyCollections = async (
    cell: CallableCell,
): Promise<KeyCollectionWithKeys[]> => {
  return cell.callZome({
    zome_name: "signing_keys",
    fn_name: "get_my_key_collections",
    payload: null,
  });
}

export const linkVerificationKeyToKeyCollection = async (cell: CallableCell, verification_key_dist_address: ActionHash, key_collection_name: string): Promise<ActionHash> => {
  return cell.callZome({
    zome_name: "signing_keys",
    fn_name: "link_verification_key_to_key_collection",
    payload: {
      verification_key_dist_address,
      key_collection_name,
    },
  })
}

export const unlinkVerificationKeyToKeyCollection = async (cell: CallableCell, verification_key_dist_address: ActionHash, key_collection_name: string): Promise<ActionHash> => {
  return cell.callZome({
    zome_name: "signing_keys",
    fn_name: "unlink_verification_key_from_key_collection",
    payload: {
      verification_key_dist_address,
      key_collection_name,
    },
  })
}

export const getMyVerificationKeyDistributions = async (cell: CallableCell): Promise<VerificationKeyResponse[]> => {
  return cell.callZome({
      zome_name: "signing_keys",
      fn_name: "get_my_verification_key_distributions",
      payload: null,
    });
}

export const searchKeys = async (cell: CallableCell, agent_pub_key: AgentPubKey): Promise<VerificationKeyResponse[]> => {
  return cell.callZome({
      zome_name: "signing_keys",
      fn_name: "search_keys",
      payload: {
        agent_pub_key,
      },
    });
}

export const searchKeysLocal = async (cell: CallableCell, agent_pub_key: AgentPubKey): Promise<VerificationKeyResponse[]> => {
  return cell.callZome({
    zome_name: "signing_keys",
    fn_name: "search_keys_local",
    payload: {
      agent_pub_key,
    },
  });
}

export const markVerificationKeyRotated = async (cell: CallableCell, verification_key_dist_address: ActionHash, mark: VfKeyDistMark): Promise<ActionHash> => {
  return cell.callZome({
      zome_name: "signing_keys",
      fn_name: "mark_verification_key_dist",
      payload: {
        verification_key_dist_address,
        mark,
      },
    })
}


