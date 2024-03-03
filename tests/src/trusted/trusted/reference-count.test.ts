import { assert, test } from "vitest";

import { runScenario, dhtSync } from "@holochain/tryorama";
import {Create, HoloHashed, Record} from "@holochain/client";

import {
  VerificationKeyResponse,
  KeyCollectionWithKeys,
  createKeyCollection,
  distributeVerificationKey,
  sampleMiniSignKey, testAppPath, sampleMiniSignProof, sampleMiniSignProofSignature,
} from "./common.js";

test("Get my keys for a key which is in another agent's collection", async () => {
  await runScenario(async (scenario) => {
    const appSource = { appBundleSource: { path: testAppPath } };

    const [alice, bob] = await scenario.addPlayersWithApps([
      appSource,
      appSource,
    ]);

    await scenario.shareAllAgents();

    // Alice distributes a MiniSign verification key
    const record: Record = await distributeVerificationKey(
      alice.cells[0],
      sampleMiniSignKey(),
      sampleMiniSignProof(),
      sampleMiniSignProofSignature(),
    );
    assert.ok(record);

    const vf_key_dist_address = (record.signed_action.hashed as HoloHashed<Create>).content.entry_hash;

    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

    // Bob creates a collection
    await createKeyCollection(bob.cells[0], "a test");

    // Bob links Alice's verification key to their key collection
    await bob.cells[0].callZome({
      zome_name: "trusted",
      fn_name: "link_verification_key_to_key_collection",
      payload: {
        verification_key_dist_address: vf_key_dist_address,
        key_collection_name: "a test",
      },
    });

    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

    // Alice searches for their own key
    const responses: VerificationKeyResponse[] = await alice.cells[0].callZome({
      zome_name: "trusted",
      fn_name: "get_my_verification_key_distributions",
      payload: null,
    });
    assert.equal(responses.length, 1);
    assert.equal(responses[0].reference_count, 1);
  });
});

test("Search for a key which is in another agent's collection", async () => {
  await runScenario(async (scenario) => {
    const appSource = { appBundleSource: { path: testAppPath } };

    const [alice, bob] = await scenario.addPlayersWithApps([
      appSource,
      appSource,
    ]);

    await scenario.shareAllAgents();

    // Alice distributes a MiniSign verification key
    const record: Record = await distributeVerificationKey(
      alice.cells[0],
      sampleMiniSignKey(),
      sampleMiniSignProof(),
      sampleMiniSignProofSignature(),
    );
    assert.ok(record);

    const vf_key_dist_address = (record.signed_action.hashed as HoloHashed<Create>).content.entry_hash;

    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

    // Bob creates a collection
    await createKeyCollection(bob.cells[0], "a test");

    // Bob links Alice's verification key to their key collection
    await bob.cells[0].callZome({
      zome_name: "trusted",
      fn_name: "link_verification_key_to_key_collection",
      payload: {
        verification_key_dist_address: vf_key_dist_address,
        key_collection_name: "a test",
      },
    });

    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

    // Alice searches for their own key
    const responses: VerificationKeyResponse[] = await alice.cells[0].callZome({
      zome_name: "trusted",
      fn_name: "search_keys",
      payload: {
        agent_pub_key: alice.agentPubKey,
      },
    });
    assert.equal(responses.length, 1);
    assert.equal(responses[0].reference_count, 1);
  });
});

test("Get my key collections for a key which is in another agent's collection", async () => {
  await runScenario(async (scenario) => {
    const appSource = { appBundleSource: { path: testAppPath } };

    const [alice, bob, carol] = await scenario.addPlayersWithApps([
      appSource,
      appSource,
      appSource,
    ]);

    await scenario.shareAllAgents();

    // Alice distributes a GPG key
    const record: Record = await distributeVerificationKey(
      alice.cells[0],
      sampleMiniSignKey(),
      sampleMiniSignProof(),
      sampleMiniSignProofSignature(),
    );
    assert.ok(record);

    const vf_key_dist_address = (record.signed_action.hashed as HoloHashed<Create>).content.entry_hash;

    // All need to be able to see Alice's GPG key
    await dhtSync([alice, bob, carol], alice.cells[0].cell_id[0]);

    // Bob creates a collection
    await createKeyCollection(bob.cells[0], "bob test");

    // Bob links Alice's verification key to their key collection
    await bob.cells[0].callZome({
      zome_name: "trusted",
      fn_name: "link_verification_key_to_key_collection",
      payload: {
        verification_key_dist_address: vf_key_dist_address,
        key_collection_name: "bob test",
      },
    });

    // Carol creates a collection
    await createKeyCollection(carol.cells[0], "carol test");

    // Carol links Alice's verification key to their key collection
    await carol.cells[0].callZome({
      zome_name: "trusted",
      fn_name: "link_verification_key_to_key_collection",
      payload: {
        verification_key_dist_address: vf_key_dist_address,
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
    assert.equal(responses[0].verification_keys.length, 1);
    // It's in Bob's collection and Carol's collections
    assert.equal(responses[0].verification_keys[0].reference_count, 2);
  });
});
