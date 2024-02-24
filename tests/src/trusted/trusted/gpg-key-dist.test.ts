import { assert, test } from "vitest";

import { runScenario, dhtSync } from "@holochain/tryorama";
import { Record } from "@holochain/client";

import { GpgKeyResponse, distributeGpgKey, sampleGpgKey } from "./common.js";

test("Distribute GPG Key", async () => {
  await runScenario(async (scenario) => {
    const testAppPath = process.cwd() + "/../workdir/hWOT.happ";
    const appSource = { appBundleSource: { path: testAppPath } };

    const [alice] = await scenario.addPlayersWithApps([appSource]);

    // Alice distributes a GpgKey
    const record: Record = await distributeGpgKey(
      alice.cells[0],
      sampleGpgKey(),
    );
    assert.ok(record);
  });
});

test("Get my keys", async () => {
  await runScenario(async (scenario) => {
    const testAppPath = process.cwd() + "/../workdir/hWOT.happ";
    const appSource = { appBundleSource: { path: testAppPath } };

    const [alice] = await scenario.addPlayersWithApps([appSource]);

    // Alice distributes a GpgKey
    const record: Record = await distributeGpgKey(
      alice.cells[0],
      sampleGpgKey(),
    );
    assert.ok(record);

    // Bob gets the created GpgKey
    const keys: GpgKeyResponse[] = await alice.cells[0].callZome({
      zome_name: "trusted",
      fn_name: "get_my_gpg_key_dists",
      payload: null,
    });
    assert.equal(keys.length, 1);
    assert.deepEqual(keys[0].gpg_key_dist.public_key, sampleGpgKey().trim());
    assert.deepEqual(keys[0].reference_count, 0);
  });
});

test("Search for a key", async () => {
  await runScenario(async (scenario) => {
    const testAppPath = process.cwd() + "/../workdir/hWOT.happ";
    const appSource = { appBundleSource: { path: testAppPath } };

    const [alice, bob] = await scenario.addPlayersWithApps([
      appSource,
      appSource,
    ]);

    await scenario.shareAllAgents();

    // Alice distributes a GPG key
    const record: Record = await distributeGpgKey(
      alice.cells[0],
      sampleGpgKey(),
    );
    assert.ok(record);

    await dhtSync([alice, bob], alice.cells[0].cell_id[0]);

    // Bob searches for Alice's GPG key
    const responses: GpgKeyResponse[] = await bob.cells[0].callZome({
      zome_name: "trusted",
      fn_name: "search_keys",
      payload: {
        // Assume Alice has told Bob the fingerprint
        query: "0B1D4843CA2F198CAC2F5C6A449D7AE5D2532CEF",
      },
    });
    assert.equal(responses.length, 1);
    assert.equal(responses[0].gpg_key_dist.name, "Alice");
    assert.equal(responses[0].gpg_key_dist.public_key, sampleGpgKey().trim());
  });
});
