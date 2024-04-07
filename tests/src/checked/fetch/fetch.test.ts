import { assert, test } from "vitest";

import { dhtSync, runScenario } from "@holochain/tryorama";

import { testAppPath } from "../common";
import {
  createAssetSignature,
  prepareFetch,
  sampleFetchAssetSignature,
  sampleFetchKey,
  sampleFetchKeyOther,
  sampleFetchKeyOtherProofSignature,
  sampleFetchKeyProof,
  sampleFetchKeyProofSignature,
  sampleFetchOtherAssetSignature,
} from "./common";
import {
  createKeyCollection,
  distributeVerificationKey,
  linkVerificationKeyToKeyCollection,
} from "../signing_keys/common";

test("Fetch with no existing signatures", async () => {
  await runScenario(async (scenario) => {
    const appSource = { appBundleSource: { path: testAppPath } };

    const [alice] = await scenario.addPlayersWithApps([appSource]);

    const check_signatures = await prepareFetch(alice.cells[0], {
      fetch_url: "https://example.com/sample.csv",
    });

    assert.equal(check_signatures.length, 0);
  });
});

test("Create asset signature", async () => {
  await runScenario(async (scenario) => {
    const appSource = { appBundleSource: { path: testAppPath } };

    const [alice] = await scenario.addPlayersWithApps([appSource]);

    await distributeVerificationKey(
      alice.cells[0],
      sampleFetchKey(),
      sampleFetchKeyProof(),
      sampleFetchKeyProofSignature(),
    );

    await createAssetSignature(alice.cells[0], {
      fetch_url: "https://example.com/sample.csv",
      signature: sampleFetchAssetSignature(),
      key_type: { MiniSignEd25519: null },
      verification_key: sampleFetchKey(),
    });

    const check_signatures = await prepareFetch(alice.cells[0], {
      fetch_url: "https://example.com/sample.csv",
    });

    assert.equal(check_signatures.length, 1);
    assert.deepEqual(check_signatures[0].reason, { RandomRecent: null });
  });
});

test("Signatures from multiple selection strategies", async () => {
  await runScenario(async (scenario) => {
    const appSource = { appBundleSource: { path: testAppPath } };

    const [alice, bob] = await scenario.addPlayersWithApps([
      appSource,
      appSource,
    ]);

    const alice_record = await distributeVerificationKey(
      alice.cells[0],
      sampleFetchKey(),
      sampleFetchKeyProof(),
      sampleFetchKeyProofSignature(),
    );
    const vf_key_dist_address_alice = alice_record.signed_action.hashed.hash;

    // Bob needs to be able to see Alice's key
    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

    await distributeVerificationKey(
      bob.cells[0],
      sampleFetchKeyOther(),
      sampleFetchKeyProof(),
      sampleFetchKeyOtherProofSignature(),
    );

    await createKeyCollection(bob.cells[0], "bob collection");

    await linkVerificationKeyToKeyCollection(
      bob.cells[0],
      vf_key_dist_address_alice,
      "bob collection",
    );

    await createAssetSignature(alice.cells[0], {
      fetch_url: "https://example.com/sample.csv",
      signature: sampleFetchAssetSignature(),
      key_type: { MiniSignEd25519: null },
      verification_key: sampleFetchKey(),
    });

    await createAssetSignature(bob.cells[0], {
      fetch_url: "https://example.com/sample.csv",
      signature: sampleFetchOtherAssetSignature(),
      key_type: { MiniSignEd25519: null },
      verification_key: sampleFetchKeyOther(),
    });

    // Make sure Alice and Bob can see each other's asset signatures
    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

    const check_signatures_alice = await prepareFetch(alice.cells[0], {
      fetch_url: "https://example.com/sample.csv",
    });

    assert.equal(check_signatures_alice.length, 2);
    assert.deepEqual(check_signatures_alice[0].reason, {
      RandomRecent: null,
    });
    assert.deepEqual(check_signatures_alice[1].reason, {
      RandomRecent: null,
    });

    const check_signatures_bob = await prepareFetch(bob.cells[0], {
      fetch_url: "https://example.com/sample.csv",
    });

    assert.equal(check_signatures_bob.length, 2);
    assert.deepEqual(check_signatures_bob[0].author, alice.agentPubKey);
    assert.deepEqual(check_signatures_bob[0].reason, {
      Pinned: {
        key_name: "test",
        key_collection: "bob collection",
      },
    });
    // TODO should self signatures be returned? Probably yes but with its own reason
    assert.deepEqual(check_signatures_bob[1].author, bob.agentPubKey);
    assert.deepEqual(check_signatures_bob[1].reason, {
      RandomRecent: null,
    });
  });
});
