import { CallableCell } from '@holochain/tryorama';
import { NewEntryAction, ActionHash, Record, AppBundleSource, fakeActionHash, fakeAgentPubKey, fakeEntryHash, fakeDnaHash } from '@holochain/client';



export async function sampleGpgKey(cell: CallableCell, partialGpgKey = {}) {
    return {
        ...{
	  public_key: [10],
	  fingerprint: "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
        },
        ...partialGpgKey
    };
}

export async function createGpgKey(cell: CallableCell, gpgKey = undefined): Promise<Record> {
    return cell.callZome({
      zome_name: "trusted",
      fn_name: "create_gpg_key",
      payload: gpgKey || await sampleGpgKey(cell),
    });
}

