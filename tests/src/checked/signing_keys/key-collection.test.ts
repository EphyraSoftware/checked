import { assert, test } from "vitest";

import { runScenario, dhtSync } from "@holochain/tryorama";
import { Record } from "@holochain/client";

import {
  createKeyCollection,
  distributeVerificationKey,
  sampleMiniSignKey,
  sampleMiniSignProof,
  sampleMiniSignProofSignature,
  getMyKeyCollections,
  linkVerificationKeyToKeyCollection,
  unlinkVerificationKeyToKeyCollection,
} from "./common.js";
import { testAppPath } from "../common";

test("Create key collection", async () => {
  await runScenario(async (scenario) => {
    const appSource = { appBundleSource: { path: testAppPath } };

    const [alice] = await scenario.addPlayersWithApps([appSource]);

    // Alice creates a key collection
    const record: Record = await createKeyCollection(alice.cells[0], "a test");
    assert.ok(record);
  });
});

test("Create key collection limit", async () => {
  await runScenario(async (scenario) => {
    const appSource = { appBundleSource: { path: testAppPath } };

    const [alice] = await scenario.addPlayersWithApps([appSource]);

    // Alice creates the allowed number of key collections
    for (let i = 0; i < 10; i++) {
      const record: Record = await createKeyCollection(
        alice.cells[0],
        `a test ${i}`,
      );
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

test("Get my key collections", async () => {
  await runScenario(async (scenario) => {
    const appSource = { appBundleSource: { path: testAppPath } };

    const [alice] = await scenario.addPlayersWithApps([appSource]);

    // Alice creates some key collections
    for (let i = 0; i < 2; i++) {
      const record: Record = await createKeyCollection(
        alice.cells[0],
        `a test ${i}`,
      );
      assert.ok(record);
    }

    const key_collections = await getMyKeyCollections(alice.cells[0]);

    assert.equal(key_collections.length, 2);
  });
});

test("Link verification key distribution to collection", async () => {
  await runScenario(async (scenario) => {
    const appSource = { appBundleSource: { path: testAppPath } };

    const [alice, bob] = await scenario.addPlayersWithApps([
      appSource,
      appSource,
    ]);

    // Alice distributes a MiniSign verification key
    const verification_key_record: Record = await distributeVerificationKey(
      alice.cells[0],
      sampleMiniSignKey(),
      sampleMiniSignProof(),
      sampleMiniSignProofSignature(),
    );
    assert.ok(verification_key_record);

    const vf_key_dist_address =
      verification_key_record.signed_action.hashed.hash;

    // Bob creates a key collection
    const key_collection_record: Record = await createKeyCollection(
      bob.cells[0],
      "a test",
    );
    assert.ok(key_collection_record);

    // Bob links Alice's verification key to the key collection
    await linkVerificationKeyToKeyCollection(
      bob.cells[0],
      vf_key_dist_address,
      "a test",
    );

    const key_collections = await getMyKeyCollections(bob.cells[0]);

    assert.equal(key_collections.length, 1);
    assert.equal(key_collections[0].verification_keys.length, 1);
  });
});

test("Unlink verification key from collection", async () => {
  await runScenario(async (scenario) => {
    const appSource = { appBundleSource: { path: testAppPath } };

    const [alice, bob] = await scenario.addPlayersWithApps([
      appSource,
      appSource,
    ]);

    // Alice distributes a MiniSign verification key
    const verification_key_record: Record = await distributeVerificationKey(
      alice.cells[0],
      sampleMiniSignKey(),
      sampleMiniSignProof(),
      sampleMiniSignProofSignature(),
    );
    assert.ok(verification_key_record);

    const vf_key_dist_address =
      verification_key_record.signed_action.hashed.hash;

    // Bob creates a key collection
    const key_collection_record: Record = await createKeyCollection(
      bob.cells[0],
      "a test",
    );
    assert.ok(key_collection_record);

    // Bob links Alice's verification key to the key collection
    await linkVerificationKeyToKeyCollection(
      bob.cells[0],
      vf_key_dist_address,
      "a test",
    );

    // Bob unlinks Alice's verification key from the key collection
    await unlinkVerificationKeyToKeyCollection(
      bob.cells[0],
      vf_key_dist_address,
      "a test",
    );

    // Now getting key collections should return a single, empty key collection
    const key_collections = await getMyKeyCollections(bob.cells[0]);

    assert.equal(key_collections.length, 1);
    assert.equal(key_collections[0].verification_keys.length, 0);
  });
});

test("Remote validation", async () => {
  await runScenario(async (scenario) => {
    const appSource = { appBundleSource: { path: testAppPath } };

    const [alice, bob] = await scenario.addPlayersWithApps([
      appSource,
      appSource,
    ]);

    await scenario.shareAllAgents();

    // Bob creates some keys collections
    for (let i = 0; i < 3; i++) {
      const record: Record = await createKeyCollection(
        bob.cells[0],
        `a test ${i}`,
      );
      assert.ok(record);
    }

    // The DHT shouldn't sync if the remote validation fails
    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

    // Alice distributes a MiniSign verification key
    const verification_key_record: Record = await distributeVerificationKey(
      alice.cells[0],
      sampleMiniSignKey(),
      sampleMiniSignProof(),
      sampleMiniSignProofSignature(),
    );
    assert.ok(verification_key_record);

    const vf_key_dist_address =
      verification_key_record.signed_action.hashed.hash;

    // Bob must know about Alice's verification key before they can add it to a key collection
    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

    // Bob links Alice's verification key to the key collection
    await linkVerificationKeyToKeyCollection(
      bob.cells[0],
      vf_key_dist_address,
      "a test 1",
    );

    // The DHT shouldn't sync if the remote validation fails
    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

    // Bob unlinks Alice's verification key from the key collection
    await unlinkVerificationKeyToKeyCollection(
      bob.cells[0],
      vf_key_dist_address,
      "a test 1",
    );

    // The DHT shouldn't sync if the remote validation fails
    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);
  });
});
