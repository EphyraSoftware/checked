import { assert, test } from "vitest";

import { runScenario } from "@holochain/tryorama";

import {testAppPath} from "../common";
import {prepareFetch} from "./common";

test("Fetch with no existing signatures", async () => {
    await runScenario(async (scenario) => {
        const appSource = { appBundleSource: { path: testAppPath } };

        const [alice] = await scenario.addPlayersWithApps([appSource]);

        const check_signatures = await prepareFetch(alice.cells[0], { fetch_url: "https://example.com/sample.csv" });

        assert.equal(check_signatures.length, 0);
    });
});
