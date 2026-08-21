/**
 * doctor + serve HTTP (MemoryDriver loopback) tests.
 */
import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { PROTOCOL_VERSION } from "@yydb/sql-studio-protocol";
import { runDoctor } from "../src/doctor.ts";
import { startMemoryServeHttp } from "../src/serve-http.ts";

describe("sql doctor", () => {
    it("reports ok with protocol + memory + hello", async () => {
        const report = await runDoctor("0.0.1");
        assert.equal(report.ok, true);
        assert.equal(report.protocolVersion, PROTOCOL_VERSION);
        assert.equal(report.memoryDriver.acquireOk, true);
        assert.equal(report.serverHello, true);
    });
});

describe("sql serve http", () => {
    it("self-test probes helloOk", async () => {
        const handle = await startMemoryServeHttp({ host: "127.0.0.1", port: 0, selfTest: true });
        // selfTest closes the server
        assert.ok(handle.url.startsWith("http://"));
    });

    it("POST hello returns helloOk", async () => {
        const handle = await startMemoryServeHttp({ host: "127.0.0.1", port: 0 });
        try {
            const res = await fetch(handle.url, {
                method: "POST",
                headers: { "content-type": "application/json" },
                body: JSON.stringify({ v: PROTOCOL_VERSION, type: "hello" }),
            });
            assert.equal(res.status, 200);
            const body = (await res.json()) as Array<{ type: string }>;
            assert.ok(body.some((m) => m.type === "helloOk"));
        } finally {
            await handle.close();
        }
    });
});
