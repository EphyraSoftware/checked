import { assert, test } from "vitest";

import { runScenario, dhtSync, CallableCell } from '@holochain/tryorama';
import { NewEntryAction, ActionHash, Record, AppBundleSource, fakeDnaHash, fakeActionHash, fakeAgentPubKey, fakeEntryHash } from '@holochain/client';
import { decode } from '@msgpack/msgpack';

import { createKeyCollection, distributeGpgKey, sampleGpgKey } from './common.js';

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
      fn_name: "get_my_gpg_key_dists",
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
    const responses: any[] = await bob.cells[0].callZome({
      zome_name: "trusted",
      fn_name: "search_keys",
      payload: {
        // Assume Alice has told Bob the fingerprint
        query: "0B1D4843CA2F198CAC2F5C6A449D7AE5D2532CEF"
      },
    });
    assert.equal(1, responses.length);
    const decoded = (decode((responses[0].key.entry as any).Present.entry) as any);
    assert.equal("Alice", decoded.name);
    assert.equal(sampleGpgKey().trim(), decoded.public_key);
  });
});

test('Search for a key which is in another agent collection', async () => {
  await runScenario(async scenario => {
    const testAppPath = process.cwd() + '/../workdir/hWOT.happ';
    const appSource = { appBundleSource: { path: testAppPath } };

    const [alice, bob] = await scenario.addPlayersWithApps([appSource, appSource]);

    await scenario.shareAllAgents();

    // Alice distributes a GPG key
    const record: Record = await distributeGpgKey(alice.cells[0], sampleGpgKey());
    assert.ok(record);

    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

    // Bob creates a collection
    await createKeyCollection(bob.cells[0], "a test");

    // Bob links the GPG key to their key collection
    await bob.cells[0].callZome({
      zome_name: "trusted",
      fn_name: "link_gpg_key_to_key_collection",
      payload: {
        gpg_key_fingerprint: (decode((record.entry as any).Present.entry) as any).fingerprint,
        key_collection_name: "a test",
      },
    });

    // Alice searches for their own key
    const responses: any[] = await bob.cells[0].callZome({
      zome_name: "trusted",
      fn_name: "search_keys",
      payload: {
        query: "0B1D4843CA2F198CAC2F5C6A449D7AE5D2532CEF"
      },
    });
    assert.equal(1, responses.length);
    const decoded = (decode((responses[0].key.entry as any).Present.entry) as any);
    assert.equal("Alice", decoded.name);
    assert.equal(1, responses[0].key_collection_count);
  });
});
