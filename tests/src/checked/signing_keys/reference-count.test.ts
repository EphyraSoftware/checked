import { assert, test } from "vitest";

import { runScenario, dhtSync } from "@holochain/tryorama";
import { Record } from "@holochain/client";

import {
  createKeyCollection,
  distributeVerificationKey,
  sampleMiniSignKey,
  testAppPath,
  sampleMiniSignProof,
  sampleMiniSignProofSignature,
  linkVerificationKeyToKeyCollection,
  getMyVerificationKeyDistributions,
  searchKeys,
  getMyKeyCollections,
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

    const vf_key_dist_address = record.signed_action.hashed.hash;

    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

    // Bob creates a collection
    await createKeyCollection(bob.cells[0], "a test");

    // Bob links Alice's verification key to their key collection
    await linkVerificationKeyToKeyCollection(
      bob.cells[0],
      vf_key_dist_address,
      "a test",
    );

    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

    // Alice searches for their own key
    const responses = await getMyVerificationKeyDistributions(alice.cells[0]);
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

    const vf_key_dist_address = record.signed_action.hashed.hash;

    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

    // Bob creates a collection
    await createKeyCollection(bob.cells[0], "a test");

    // Bob links Alice's verification key to their key collection
    await linkVerificationKeyToKeyCollection(
      bob.cells[0],
      vf_key_dist_address,
      "a test",
    );

    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

    // Alice searches for their own key
    const responses = await searchKeys(alice.cells[0], alice.agentPubKey);

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

    const vf_key_dist_address = record.signed_action.hashed.hash;

    // All need to be able to see Alice's GPG key
    await dhtSync([alice, bob, carol], alice.cells[0].cell_id[0]);

    // Bob creates a collection
    await createKeyCollection(bob.cells[0], "bob test");

    // Bob links Alice's verification key to their key collection
    await linkVerificationKeyToKeyCollection(
      bob.cells[0],
      vf_key_dist_address,
      "bob test",
    );

    // Carol creates a collection
    await createKeyCollection(carol.cells[0], "carol test");

    // Carol links Alice's verification key to their key collection
    await linkVerificationKeyToKeyCollection(
      carol.cells[0],
      vf_key_dist_address,
      "carol test",
    );

    await dhtSync([alice, bob, carol], alice.cells[0].cell_id[0]);

    // Bob checks their key collections
    const responses = await getMyKeyCollections(bob.cells[0]);

    assert.equal(responses.length, 1);
    assert.equal(responses[0].name, "bob test");
    assert.equal(responses[0].verification_keys.length, 1);
    // It's in Bob's collection and Carol's collections
    assert.equal(responses[0].verification_keys[0].reference_count, 2);
  });
});
