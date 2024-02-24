import { CallableCell } from '@holochain/tryorama';
import { Record } from '@holochain/client';
import { decode } from '@msgpack/msgpack';

export interface GpgKeyDist {
  public_key: string;
  fingerprint: string;
  name: string;
  email?: string;
  expires_at: number;
}

export interface GpgKeyResponse {
  gpg_key_dist: GpgKeyDist;
  reference_count: number;
}

export interface KeyCollectionWithKeys {
  name: string;
  gpg_keys: GpgKeyResponse[];
}

export const decodeRecord = <T> (record: Record): T => {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return decode((record.entry as any).Present.entry) as T
}

export function sampleGpgKey() {
    return `
-----BEGIN PGP PUBLIC KEY BLOCK-----

mDMEZdDsIBYJKwYBBAHaRw8BAQdAed156Mxx8965zeCQwuGxP1IbkyebXlSyY8Ux
bOEgBUu0GUFsaWNlIDxhbGljZUB0ZXN0aW5nLmNvbT6ImQQTFgoAQRYhBAsdSEPK
LxmMrC9cakSdeuXSUyzvBQJl0OwgAhsDBQkFo5qABQsJCAcCAiICBhUKCQgLAgQW
AgMBAh4HAheAAAoJEESdeuXSUyzvBcQA/Rbg+fLGubvhYRsL2PFLwWQjgG8nyWKm
QFeEVnBvaAGOAQChHakNklfdeqQ4G/+Wp60UnLnJi5JfkAZRTLGEFyjuDbg4BGXQ
7CASCisGAQQBl1UBBQEBB0D6qt2YFWWwj/sXAiifVsPBmMZfRWG/CPi7W3MiFpgw
HgMBCAeIfgQYFgoAJhYhBAsdSEPKLxmMrC9cakSdeuXSUyzvBQJl0OwgAhsMBQkF
o5qAAAoJEESdeuXSUyzvh4sA/3XkOHCxs5fAGpSyCMor9JV0psg9aVpWLOFe3Sdc
NTxJAQC4xdYMveSHdOiKO4ZhoojG6r7IqX8B8vAZsod6cF/8CA==
=d0p+
-----END PGP PUBLIC KEY BLOCK-----
    `;
}

export async function distributeGpgKey(cell: CallableCell, gpgKey: string): Promise<Record> {
    return cell.callZome({
      zome_name: "trusted",
      fn_name: "distribute_gpg_key",
      payload: {
        public_key: gpgKey
      },
    });
}

export async function createKeyCollection(cell: CallableCell, name: string): Promise<Record> {
    return cell.callZome({
      zome_name: "trusted",
      fn_name: "create_key_collection",
      payload: { name },
    });
}
