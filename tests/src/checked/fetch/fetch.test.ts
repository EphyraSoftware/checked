import { assert, test } from "vitest";

import { dhtSync, runScenario } from "@holochain/tryorama";

import { testAppPath } from "../common";
import {
  createAssetSignature,
  deleteAssetSignature,
  getMyAssetSignatures,
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

test("Prepare fetch with no existing signatures", async () => {
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

    const record = await distributeVerificationKey(
      alice.cells[0],
      sampleFetchKey(),
      sampleFetchKeyProof(),
      sampleFetchKeyProofSignature(),
    );
    const vf_key_dist_address = record.signed_action.hashed.hash;

    await createAssetSignature(alice.cells[0], {
      fetch_url: "https://example.com/sample.csv",
      signature: sampleFetchAssetSignature(),
      key_type: { MiniSignEd25519: null },
      verification_key: sampleFetchKey(),
    });

    const my_asset_signatures = await getMyAssetSignatures(alice.cells[0]);

    assert.equal(my_asset_signatures.length, 1);
    assert.deepEqual(my_asset_signatures[0].key_dist_address, vf_key_dist_address);
  });
});

test("Get my asset signatures", async () => {
  await runScenario(async (scenario) => {
    const appSource = { appBundleSource: { path: testAppPath } };

    const [alice, bob] = await scenario.addPlayersWithApps([appSource, appSource]);

    await distributeVerificationKey(
      alice.cells[0],
      sampleFetchKey(),
      sampleFetchKeyProof(),
      sampleFetchKeyProofSignature(),
    );

    await distributeVerificationKey(
        bob.cells[0],
        sampleFetchKeyOther(),
        sampleFetchKeyProof(),
        sampleFetchKeyOtherProofSignature(),
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

    const aliceSignatures = await getMyAssetSignatures(alice.cells[0]);

    assert.equal(aliceSignatures.length, 1);
    assert.equal(aliceSignatures[0].fetch_url, "https://example.com/sample.csv");

    const bobSignatures = await getMyAssetSignatures(bob.cells[0]);

    assert.equal(bobSignatures.length, 1);
    assert.equal(bobSignatures[0].fetch_url, "https://example.com/sample.csv");

    assert.notEqual(aliceSignatures[0].key_dist_address, bobSignatures[0].key_dist_address);
  });
});

test("Delete an asset signature", async () => {
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

    await deleteAssetSignature(alice.cells[0], {
      fetch_url: "https://example.com/sample.csv",
    });

    const mySignatures = await getMyAssetSignatures(alice.cells[0]);

    assert.equal(mySignatures.length, 0);
  });
});

test("Cannot resign an asset", async () => {
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

    const mySignatures = await getMyAssetSignatures(alice.cells[0]);
    assert.equal(mySignatures.length, 1);

    let err_msg = "";
    try {
      await createAssetSignature(alice.cells[0], {
        fetch_url: "https://example.com/sample.csv",
        signature: sampleFetchAssetSignature(),
        key_type: { MiniSignEd25519: null },
        verification_key: sampleFetchKey(),
      });
    } catch (e) {
      err_msg = e.message;
    }
    assert.isTrue(err_msg.includes("An asset signature with the same fetch URL already exists"));

    const mySignaturesAfter = await getMyAssetSignatures(alice.cells[0]);
    assert.equal(mySignaturesAfter.length, 1);
  });
});

test("Cannot resign an asset after deleting the original signature", async () => {
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

    const mySignatures = await getMyAssetSignatures(alice.cells[0]);
    assert.equal(mySignatures.length, 1);

    await deleteAssetSignature(alice.cells[0], {
        fetch_url: "https://example.com/sample.csv",
    });

    let err_msg = "";
    try {
      await createAssetSignature(alice.cells[0], {
        fetch_url: "https://example.com/sample.csv",
        signature: sampleFetchAssetSignature(),
        key_type: { MiniSignEd25519: null },
        verification_key: sampleFetchKey(),
      });
    } catch (e) {
      err_msg = e.message;
    }
    assert.isTrue(err_msg.includes("An asset signature with the same fetch URL already exists"));

    const mySignaturesAfter = await getMyAssetSignatures(alice.cells[0]);
    assert.equal(mySignaturesAfter.length, 0);
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
      Mine: null,
    });
    assert.deepEqual(check_signatures_alice[1].reason, {
      RandomRecent: null,
    });

    const check_signatures_bob = await prepareFetch(bob.cells[0], {
      fetch_url: "https://example.com/sample.csv",
    });

    assert.equal(check_signatures_bob.length, 2);
    assert.deepEqual(check_signatures_bob[0].author, bob.agentPubKey);
    assert.deepEqual(check_signatures_bob[0].reason, {
      Mine: null,
    });
    assert.deepEqual(check_signatures_bob[1].author, alice.agentPubKey);
    assert.deepEqual(check_signatures_bob[1].reason, {
      Pinned: {
        key_name: "test",
        key_collection: "bob collection",
      },
    });
  });
});

test("Remote validation", async () => {
  await runScenario(async (scenario) => {
    const appSource = { appBundleSource: { path: testAppPath } };

    const [alice, bob] = await scenario.addPlayersWithApps([
      appSource,
      appSource,
    ]);

    await distributeVerificationKey(
      alice.cells[0],
      sampleFetchKey(),
      sampleFetchKeyProof(),
      sampleFetchKeyProofSignature(),
    );

    await distributeVerificationKey(
      bob.cells[0],
      sampleFetchKeyOther(),
      sampleFetchKeyProof(),
      sampleFetchKeyOtherProofSignature(),
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

    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

    await deleteAssetSignature(alice.cells[0], {
      fetch_url: "https://example.com/sample.csv",
    });

    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

    const bobAssetSignatures = await getMyAssetSignatures(bob.cells[0]);
    assert.equal(bobAssetSignatures.length, 1);

    await deleteAssetSignature(bob.cells[0], {
      fetch_url: "https://example.com/sample.csv",
    });

    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);
  });
});
