import { assert, test } from "vitest";

import { runScenario, dhtSync } from "@holochain/tryorama";
import { Record } from "@holochain/client";

import {
  VerificationKeyResponse,
  distributeVerificationKey,
  sampleMiniSignKey,
  testAppPath,
  sampleMiniSignProof,
  sampleMiniSignProofSignature,
} from "./common.js";

test("Distribute a MiniSign key", async () => {
  await runScenario(async (scenario) => {
    const appSource = { appBundleSource: { path: testAppPath } };

    const [alice] = await scenario.addPlayersWithApps([appSource]);

    // Alice distributes a MiniSign key
    const record: Record = await distributeVerificationKey(
      alice.cells[0],
      sampleMiniSignKey(),
      sampleMiniSignProof(),
      sampleMiniSignProofSignature(),
    );
    assert.ok(record);
  });
});

test("Distribute a MiniSign key with invalid proof", async () => {
  await runScenario(async (scenario) => {
    const appSource = { appBundleSource: { path: testAppPath } };

    const [alice] = await scenario.addPlayersWithApps([appSource]);

    // Alice distributes a MiniSign key with an invalid proof
    let errMsg = "";
    try {
      await distributeVerificationKey(
        alice.cells[0],
        sampleMiniSignKey(),
        sampleMiniSignProof() + "invalid", // Changing the proof makes the signature invalid
        sampleMiniSignProofSignature(),
      );
    } catch (e) {
      errMsg = e.message;
    }

    assert.isTrue(errMsg.includes("Failed to verify proof signature"));
  });
});

test("Get my keys", async () => {
  await runScenario(async (scenario) => {
    const appSource = { appBundleSource: { path: testAppPath } };

    const [alice] = await scenario.addPlayersWithApps([appSource]);

    // Alice distributes a MiniSign verification key
    const record: Record = await distributeVerificationKey(
      alice.cells[0],
      sampleMiniSignKey(),
      sampleMiniSignProof(),
      sampleMiniSignProofSignature(),
    );
    assert.ok(record);

    // Alice gets the created key in her list of keys
    const keys: VerificationKeyResponse[] = await alice.cells[0].callZome({
      zome_name: "trusted",
      fn_name: "get_my_verification_key_distributions",
      payload: null,
    });
    assert.equal(keys.length, 1);
    assert.deepEqual(
      keys[0].verification_key_dist.verification_key,
      sampleMiniSignKey().trim(),
    );
    assert.deepEqual(keys[0].reference_count, 0);
  });
});

test("Search for a key", async () => {
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

    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

    // Bob searches for Alice's key
    const responses: VerificationKeyResponse[] = await bob.cells[0].callZome({
      zome_name: "trusted",
      fn_name: "search_keys",
      payload: {
        agent_pub_key: alice.agentPubKey,
      },
    });
    assert.equal(responses.length, 1);
    assert.equal(responses[0].verification_key_dist.name, "test");
    assert.equal(
      responses[0].verification_key_dist.verification_key,
      sampleMiniSignKey().trim(),
    );

    // Note: This is more of a PoC than anything at this point, but it's a start
    // Bob searches for Alice's key while offline
    const offline_responses: VerificationKeyResponse[] =
      await bob.cells[0].callZome({
        zome_name: "trusted",
        fn_name: "search_keys_local",
        payload: {
          agent_pub_key: alice.agentPubKey,
        },
      });
    assert.equal(offline_responses.length, 1);
    assert.equal(offline_responses[0].verification_key_dist.name, "test");
    assert.equal(
      offline_responses[0].verification_key_dist.verification_key,
      sampleMiniSignKey().trim(),
    );

    // Should be an identical result
    assert.deepEqual(responses, offline_responses);
  });
});
