import { CallableCell } from '@holochain/tryorama';
import { NewEntryAction, ActionHash, Record, AppBundleSource, fakeActionHash, fakeAgentPubKey, fakeEntryHash, fakeDnaHash } from '@holochain/client';

export function sampleGpgKey() {
    return `
-----BEGIN PGP PUBLIC KEY BLOCK-----

mDMEZdDQBhYJKwYBBAHaRw8BAQdAApOuMlysoJ3/W7g+fssQKD1/o7humgtOWOsu
JSeENb60BnRlc3RlcoiZBBMWCgBBFiEEoAChxfSpzEZptZ7+eYTTRhdnoqgFAmXQ
0AYCGwMFCQWjmoAFCwkIBwICIgIGFQoJCAsCBBYCAwECHgcCF4AACgkQeYTTRhdn
oqiT7QEA72fxTyF+f6yOaf7o6ZghVuYbIZ7pU4VFEVscy1+AZHwA/3iK0zGl7+cq
0QCgTTO0O4T2C7hFQ1h8x22e1B29Z+4IuDgEZdDQBhIKKwYBBAGXVQEFAQEHQPlf
bsGs/A3pEfwmUF+j+vBJZLdOjibxF8Kp5hPBgGoJAwEIB4h4BBgWCgAgFiEEoACh
xfSpzEZptZ7+eYTTRhdnoqgFAmXQ0AYCGwwACgkQeYTTRhdnoqi5hQD8DpDa2yJj
e88dJFTa/r0J8FbR5z9dMvanT3sZ8pLj6sMBALeZsy67vRw9namwb9ZGzL7nHXdm
qCEcfne2pByJ8t4L
=DpDU
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
