import { assert, test } from "vitest";

import { runScenario, dhtSync, CallableCell } from '@holochain/tryorama';
import { NewEntryAction, ActionHash, Record, AppBundleSource, fakeDnaHash, fakeActionHash, fakeAgentPubKey, fakeEntryHash } from '@holochain/client';
import { decode } from '@msgpack/msgpack';

import { createKeyCollection, distributeGpgKey, sampleGpgKey } from './common.js';

test('Create key collection', async () => {
    await runScenario(async scenario => {
        const testAppPath = process.cwd() + '/../workdir/hWOT.happ';
        const appSource = { appBundleSource: { path: testAppPath } };

        const [alice] = await scenario.addPlayersWithApps([appSource]);

        // Alice creates a key collection
        const record: Record = await createKeyCollection(alice.cells[0], "a test");
        assert.ok(record);
    });
});

test('Create key collection limit', async () => {
    await runScenario(async scenario => {
        const testAppPath = process.cwd() + '/../workdir/hWOT.happ';
        const appSource = { appBundleSource: { path: testAppPath } };

        const [alice] = await scenario.addPlayersWithApps([appSource]);

        // Alice creates the allowed number of key collections
        for (let i = 0; i < 10; i++) {
            const record: Record = await createKeyCollection(alice.cells[0], `a test ${i}`);
            assert.ok(record);
        }

        let failed = false;
        try {
            await createKeyCollection(alice.cells[0], `a test too many`);
        } catch {
            failed = true;
        }
        assert.ok(failed);
    });
});

test('Get my key collections', async () => {
    await runScenario(async scenario => {
        const testAppPath = process.cwd() + '/../workdir/hWOT.happ';
        const appSource = { appBundleSource: { path: testAppPath } };

        const [alice] = await scenario.addPlayersWithApps([appSource]);

        // Alice creates some key collections
        for (let i = 0; i < 2; i++) {
            const record: Record = await createKeyCollection(alice.cells[0], `a test ${i}`);
            assert.ok(record);
        }

        const key_collections: object[] = await alice.cells[0].callZome({
            zome_name: "trusted",
            fn_name: "get_my_key_collections",
            payload: null,
        });

        assert.equal(2, key_collections.length);
    });
});

test('Link GPG key to collection', async () => {
    await runScenario(async scenario => {
        const testAppPath = process.cwd() + '/../workdir/hWOT.happ';
        const appSource = { appBundleSource: { path: testAppPath } };

        const [alice] = await scenario.addPlayersWithApps([appSource]);

        // Alice distributes a GPG key
        const gpg_key_record: Record = await distributeGpgKey(alice.cells[0], sampleGpgKey());
        assert.ok(gpg_key_record);

        // Alice creates a key collection
        const key_collection_record: Record = await createKeyCollection(alice.cells[0], "a test");
        assert.ok(key_collection_record);

        // Alice links the GPG key to the key collection
        await alice.cells[0].callZome({
            zome_name: "trusted",
            fn_name: "link_gpg_key_to_key_collection",
            payload: {
                gpg_key_fingerprint: (decode((gpg_key_record.entry as any).Present.entry) as any).fingerprint,
                key_collection_name: "a test",
            },
        });

        const key_collections: object[] = await alice.cells[0].callZome({
            zome_name: "trusted",
            fn_name: "get_my_key_collections",
            payload: null,
        });

        assert.equal(1, key_collections.length);
        assert.equal(1, (key_collections[0] as any).gpg_keys.length);
    });
});

test('Remote validation', async () => {
    await runScenario(async scenario => {
        const testAppPath = process.cwd() + '/../workdir/hWOT.happ';
        const appSource = { appBundleSource: { path: testAppPath } };

        const [alice, bob] = await scenario.addPlayersWithApps([appSource, appSource]);

        await scenario.shareAllAgents();

        // Alice creates some keys collections
        for (let i = 0; i < 3; i++) {
            const record: Record = await createKeyCollection(alice.cells[0], `a test ${i}`);
            assert.ok(record);
        }

        // The DHT shouldn't sync if the remote validation fails
        await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

        // Alice distributes a GPG key
        const gpg_key_record: Record = await distributeGpgKey(alice.cells[0], sampleGpgKey());
        assert.ok(gpg_key_record);

        // Alice links the GPG key to a key collection
        await alice.cells[0].callZome({
            zome_name: "trusted",
            fn_name: "link_gpg_key_to_key_collection",
            payload: {
                gpg_key_fingerprint: (decode((gpg_key_record.entry as any).Present.entry) as any).fingerprint,
                key_collection_name: "a test 1",
            },
        });

        // The DHT shouldn't sync if the remote validation fails
        await dhtSync([alice, bob], alice.cells[0].cell_id[0]);
    });
});
