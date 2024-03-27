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
  sampleMiniSignProofSignature2,
  sampleMiniSignProof2,
  sampleMiniSignKey2, getMyVerificationKeyDistributions, searchKeys, searchKeysLocal, markVerificationKeyRotated,
} from "./common.js";

test("Distribute a key", async () => {
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

test("Distribute a key with invalid proof", async () => {
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
    const keys = await getMyVerificationKeyDistributions(alice.cells[0]);

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
    const responses = await searchKeys(bob.cells[0], alice.agentPubKey);

    assert.equal(responses.length, 1);
    assert.equal(responses[0].verification_key_dist.name, "test");
    assert.equal(
      responses[0].verification_key_dist.verification_key,
      sampleMiniSignKey().trim(),
    );

    // Note: This is more of a PoC than anything at this point, but it's a start
    // Bob searches for Alice's key while offline
    const offline_responses = await searchKeysLocal(bob.cells[0], alice.agentPubKey);

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

test("Mark a key as compromised", async () => {
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

    // Sync here so Bob knows about Alice's key
    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

    // Alice marks her own key as compromised
    const compromisedSince = new Date().getUTCMilliseconds() * 1000;
    await markVerificationKeyRotated(alice.cells[0], vf_key_dist_address, { Compromised: { note: "I think someone is using my private key!", since: compromisedSince } })

    // TODO should not need to DHT sync here. What I actually want is an 'Alice synced' to be serving up the links
    //      that Bob will need in the next step. For now, the test is flaky without this sync...
    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

    // No DHT sync, but Bob does an online search for Alice's keys which *should* do a get_links to the network and see Alice's mark.
    const responses = await searchKeys(bob.cells[0], alice.agentPubKey);

    assert.equal(responses.length, 1);
    assert.equal(responses[0].verification_key_dist.marks.length, 1);

    const mark = responses[0].verification_key_dist.marks[0];
    assert.ok(mark["Compromised"]);
    assert.equal(
      mark["Compromised"].note,
      "I think someone is using my private key!",
    );
    assert.equal(mark["Compromised"].since, compromisedSince);

    // Sync at the end to check validation worked
    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);
  });
});

test("Mark a key as rotated", async () => {
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

    // Alice distributes a new MiniSign verification key
    const new_record: Record = await distributeVerificationKey(
      alice.cells[0],
      sampleMiniSignKey2(),
      sampleMiniSignProof2(),
      sampleMiniSignProofSignature2(),
    );
    assert.ok(record);

    // Sync here so Bob knows about both of Alice's keys
    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

    const new_vf_key_dist_address = new_record.signed_action.hashed.hash;

    // Alice marks her own key as compromised
    await markVerificationKeyRotated(alice.cells[0], vf_key_dist_address, { Rotated: { new_verification_key_dist_address: new_vf_key_dist_address } })

    // TODO should not need to DHT sync here. What I actually want is an 'Alice synced' to be serving up the links
    //      that Bob will need in the next step. For now, the test is flaky without this sync...
    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

    // No DHT sync, but Bob does an online search for Alice's keys which *should* do a get_links to the network and see Alice's mark.
    const responses = await searchKeys(bob.cells[0], alice.agentPubKey);

    // Sort in place by created_at. The ordering is not guaranteed from the search!
    responses.sort((a, b) => a.created_at - b.created_at);

    assert.equal(responses.length, 2);
    assert.equal(responses[0].verification_key_dist.marks.length, 1);

    const mark = responses[0].verification_key_dist.marks[0];
    assert.ok(mark["Rotated"]);
    assert.deepEqual(
      mark["Rotated"].new_verification_key_dist_address,
      new_vf_key_dist_address,
    );

    assert.equal(responses[1].verification_key_dist.marks.length, 0);
    assert.equal(
      responses[1].verification_key_dist.verification_key,
      sampleMiniSignKey2().trim(),
    );

    // Sync at the end to check validation worked
    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);
  });
});
