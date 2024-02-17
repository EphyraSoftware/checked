import { assert, test } from "vitest";

import { runScenario, dhtSync, CallableCell } from '@holochain/tryorama';
import { NewEntryAction, ActionHash, Record, AppBundleSource, fakeDnaHash, fakeActionHash, fakeAgentPubKey, fakeEntryHash } from '@holochain/client';
import { decode } from '@msgpack/msgpack';

import { distributeGpgKey, sampleGpgKey } from './common.js';

test('Distribute GPG Key', async () => {
  await runScenario(async scenario => {
    const testAppPath = process.cwd() + '/../workdir/hWOT.happ';
    const appSource = { appBundleSource: { path: testAppPath } };

    const [alice] = await scenario.addPlayersWithApps([appSource]);

    // Alice distributes a GpgKey
    const record: Record = await distributeGpgKey(alice.cells[0], sampleGpgKey());
    assert.ok(record);
  });
});

test('Get my keys', async () => {
  await runScenario(async scenario => {
    const testAppPath = process.cwd() + '/../workdir/hWOT.happ';
    const appSource = { appBundleSource: { path: testAppPath } };

    const [alice] = await scenario.addPlayersWithApps([appSource]);

    // Alice distributes a GpgKey
    const record: Record = await distributeGpgKey(alice.cells[0], sampleGpgKey());
    assert.ok(record);

    // Bob gets the created GpgKey
    const keys: Record[] = await alice.cells[0].callZome({
      zome_name: "trusted",
      fn_name: "get_my_keys",
      payload: null,
    });
    assert.equal(1, keys.length);
    assert.deepEqual(sampleGpgKey().trim(), (decode((keys[0].entry as any).Present.entry) as any).public_key);
  });
});

test('Search for a key', async () => {
  await runScenario(async scenario => {
    const testAppPath = process.cwd() + '/../workdir/hWOT.happ';
    const appSource = { appBundleSource: { path: testAppPath } };

    const [alice, bob] = await scenario.addPlayersWithApps([appSource, appSource]);

    await scenario.shareAllAgents();

    // Alice distributes a GPG key
    const record: Record = await distributeGpgKey(alice.cells[0], sampleGpgKey());
    assert.ok(record);

    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

    // Bob searches for Alice's GPG key
    const keys: Record[] = await bob.cells[0].callZome({
      zome_name: "trusted",
      fn_name: "search_keys",
      payload: {
        // Assume Alice has told Bob the fingerprint
        query: "A000A1C5F4A9CC4669B59EFE7984D3461767A2A8"
      },
    });
    assert.equal(1, keys.length);
    assert.deepEqual(sampleGpgKey().trim(), (decode((keys[0].entry as any).Present.entry) as any).public_key);
  });
});
