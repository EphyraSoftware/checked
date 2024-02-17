import { CallableCell } from '@holochain/tryorama';
import { NewEntryAction, ActionHash, Record, AppBundleSource, fakeActionHash, fakeAgentPubKey, fakeEntryHash, fakeDnaHash } from '@holochain/client';

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
