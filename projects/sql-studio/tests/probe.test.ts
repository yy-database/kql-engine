/**
 * createSqlStudio ↔ MemoryDriver HTTP serve end-to-end.
 */
import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { runMemoryProbe } from "../src/probe.ts";

describe("sql probe (client round-trip)", () => {
    it("hello → list → open → query → close", async () => {
        const report = await runMemoryProbe("0.0.1");
        assert.equal(report.ok, true);
        assert.deepEqual(report.datasources, ["memory"]);
        assert.ok(report.queryRows.length >= 1);
        assert.deepEqual(report.queryRows[0], [true]);
    });
});
