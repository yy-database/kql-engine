/**
 * Server handleMessage ↔ protocol DTO conformance.
 */
import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { createMemoryDriver } from "@yydb/sql-studio-orm";
import { PROTOCOL_VERSION, isStudioErrorCode } from "@yydb/sql-studio-protocol";
import { createSqlStudioServer } from "../src/index.ts";

describe("createSqlStudioServer.handleMessage", () => {
    it("hello → helloOk", async () => {
        const server = createSqlStudioServer({ databases: [] });
        const out = await server.handleMessage({ v: PROTOCOL_VERSION, type: "hello" });
        assert.deepEqual(out, [{ v: PROTOCOL_VERSION, type: "helloOk", protocolVersion: PROTOCOL_VERSION }]);
        await server.destroy();
    });

    it("open/query/close round-trip on MemoryDriver", async () => {
        const driver = createMemoryDriver({
            dialect: "postgres",
            onExecute: () => ({
                rows: [{ id: 1 }],
                rowCount: 1,
                columns: ["id"],
            }),
        });
        const server = createSqlStudioServer({
            databases: [{ id: "mem", kind: "postgres", driver }],
        });

        const opened = await server.handleMessage({
            v: PROTOCOL_VERSION,
            type: "openConnection",
            connectionId: "c1",
            datasourceId: "mem",
        });
        assert.equal(opened[0]?.type, "connectionOpened");

        const chunk = await server.handleMessage({
            v: PROTOCOL_VERSION,
            type: "query",
            connectionId: "c1",
            text: "select 1",
            requestId: "r1",
        });
        assert.equal(chunk[0]?.type, "queryChunk");
        if (chunk[0]?.type === "queryChunk") {
            assert.equal(chunk[0].done, true);
            assert.deepEqual(chunk[0].rows, [[1]]);
        }

        const closed = await server.handleMessage({
            v: PROTOCOL_VERSION,
            type: "closeConnection",
            connectionId: "c1",
        });
        assert.equal(closed[0]?.type, "connectionClosed");
        await server.destroy();
    });

    it("unknown datasource → typed error code", async () => {
        const server = createSqlStudioServer({ databases: [] });
        const out = await server.handleMessage({
            v: PROTOCOL_VERSION,
            type: "openConnection",
            connectionId: "c1",
            datasourceId: "missing",
        });
        assert.equal(out[0]?.type, "error");
        if (out[0]?.type === "error") {
            assert.ok(out[0].code);
            assert.equal(isStudioErrorCode(out[0].code), true);
            assert.equal(out[0].code, "unknown_datasource");
        }
        await server.destroy();
    });

    it("rejects non-protocol payloads with bad_message", async () => {
        const server = createSqlStudioServer({ databases: [] });
        const out = await server.handleMessage({ v: 1, type: "notARealType" } as never);
        assert.equal(out[0]?.type, "error");
        if (out[0]?.type === "error") {
            assert.equal(out[0].code, "bad_message");
        }
        await server.destroy();
    });
});
