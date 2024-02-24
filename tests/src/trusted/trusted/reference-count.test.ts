import { assert, test } from "vitest";

import { runScenario, dhtSync } from '@holochain/tryorama';
import { Record } from '@holochain/client';

import { GpgKeyDist, GpgKeyResponse, KeyCollectionWithKeys, createKeyCollection, decodeRecord, distributeGpgKey, sampleGpgKey } from './common.js';

test('Get my keys for a key which is in another agent\'s collection', async () => {
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
                gpg_key_fingerprint: decodeRecord<GpgKeyDist>(record).fingerprint,
                key_collection_name: "a test",
            },
        });

        await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

        // Alice searches for their own key
        const responses: GpgKeyResponse[] = await alice.cells[0].callZome({
            zome_name: "trusted",
            fn_name: "get_my_gpg_key_dists",
            payload: null,
        });
        assert.equal(responses.length, 1);
        assert.equal(responses[0].gpg_key_dist.name, "Alice");
        assert.equal(responses[0].reference_count, 1);
    });
});

test('Search for a key which is in another agent\'s collection', async () => {
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
                gpg_key_fingerprint: decodeRecord<GpgKeyDist>(record).fingerprint,
                key_collection_name: "a test",
            },
        });

        await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

        // Alice searches for their own key
        const responses: GpgKeyResponse[] = await alice.cells[0].callZome({
            zome_name: "trusted",
            fn_name: "search_keys",
            payload: {
                query: "0B1D4843CA2F198CAC2F5C6A449D7AE5D2532CEF"
            },
        });
        assert.equal(responses.length, 1);
        assert.equal(responses[0].gpg_key_dist.name, "Alice");
        assert.equal(responses[0].reference_count, 1);
    });
});

test('Get my key collections for a key which is in another agent\'s collection', async () => {
    await runScenario(async scenario => {
        const testAppPath = process.cwd() + '/../workdir/hWOT.happ';
        const appSource = { appBundleSource: { path: testAppPath } };

        const [alice, bob, carol] = await scenario.addPlayersWithApps([appSource, appSource, appSource]);

        await scenario.shareAllAgents();

        // Alice distributes a GPG key
        const record: Record = await distributeGpgKey(alice.cells[0], sampleGpgKey());
        assert.ok(record);

        // All need to be able to see Alice's GPG key
        await dhtSync([alice, bob, carol], alice.cells[0].cell_id[0]);

        // Bob creates a collection
        await createKeyCollection(bob.cells[0], "bob test");

        // Bob links Alice's GPG key to their key collection
        await bob.cells[0].callZome({
            zome_name: "trusted",
            fn_name: "link_gpg_key_to_key_collection",
            payload: {
                gpg_key_fingerprint: decodeRecord<GpgKeyDist>(record).fingerprint,
                key_collection_name: "bob test",
            },
        });

        // Carol creates a collection
        await createKeyCollection(carol.cells[0], "carol test");

        // Carol links Alice's GPG key to their key collection
        await carol.cells[0].callZome({
            zome_name: "trusted",
            fn_name: "link_gpg_key_to_key_collection",
            payload: {
                gpg_key_fingerprint: decodeRecord<GpgKeyDist>(record).fingerprint,
                key_collection_name: "carol test",
            },
        });

        await dhtSync([alice, bob, carol], alice.cells[0].cell_id[0]);

        // Bob checks their key collections
        const responses: KeyCollectionWithKeys[] = await bob.cells[0].callZome({
            zome_name: "trusted",
            fn_name: "get_my_key_collections",
            payload: null,
        });
        assert.equal(responses.length, 1);
        assert.equal(responses[0].name, "bob test");
        assert.equal(responses[0].gpg_keys.length, 1);
        // It's in Bob's collection and Carol's collections
        assert.equal(responses[0].gpg_keys[0].reference_count, 2);
    });
});
