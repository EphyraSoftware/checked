import { assert, test } from "vitest";

import { runScenario } from "@holochain/tryorama";

import {testAppPath} from "../common";
import {
    createAssetSignature,
    prepareFetch, sampleFetchAssetSignature,
    sampleFetchKey,
    sampleFetchKeyProof,
    sampleFetchKeyProofSignature
} from "./common";
import {
    distributeVerificationKey,
} from "../signing_keys/common";

test("Fetch with no existing signatures", async () => {
    await runScenario(async (scenario) => {
        const appSource = { appBundleSource: { path: testAppPath } };

        const [alice] = await scenario.addPlayersWithApps([appSource]);

        const check_signatures = await prepareFetch(alice.cells[0], { fetch_url: "https://example.com/sample.csv" });

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
        )

        await createAssetSignature(alice.cells[0], {
            fetch_url: "https://example.com/sample.csv",
            signature: sampleFetchAssetSignature(),
            key_type: { MiniSignEd25519: null },
            verification_key: sampleFetchKey(),
        });

        const check_signatures = await prepareFetch(alice.cells[0], { fetch_url: "https://example.com/sample.csv" });

        assert.equal(check_signatures.length, 1);
    });
});
