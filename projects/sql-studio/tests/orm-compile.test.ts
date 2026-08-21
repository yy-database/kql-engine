/**
 * `sql orm compile` fixture — MemoryDriver sample evidence.
 */
import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { runMemoryCompileFixture } from "../src/orm-compile.ts";

describe("orm compile fixture", () => {
    it("prints sql + fingerprint and exercises MemoryDriver", async () => {
        const out = await runMemoryCompileFixture();
        assert.equal(out.dialect, "postgres");
        assert.match(out.sql, /"users"/);
        assert.equal(out.parameters[0], "demo@example.com");
        assert.equal(out.memoryRowCount, 1);
        assert.ok(out.fingerprint.length >= 8);
        assert.deepEqual(out.historyFingerprints, [out.fingerprint]);
    });
});
