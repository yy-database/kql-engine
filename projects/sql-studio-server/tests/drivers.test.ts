/**
 * Driver default-entry honesty (0.0.1 surface).
 */
import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { mongodb, QUERY_MODEL as MONGO_MODEL } from "@yydb/mongodb";
import { mysql } from "@yydb/mysql";
import { postgres } from "@yydb/postgres";
import { redis, QUERY_MODEL as REDIS_MODEL } from "@yydb/redis";
import { createSqlStudioServer } from "@yydb/sql-studio-server";
import { PROTOCOL_VERSION } from "@yydb/sql-studio-protocol";
import { sqlite } from "@yydb/sqlite";

describe("relational driver default acquire", () => {
    it("postgres throws toward /node", async () => {
        const d = postgres({ url: "postgres://localhost/db" });
        await assert.rejects(() => d.acquire(), /postgres\/node/);
    });

    it("mysql throws toward /node", async () => {
        const d = mysql({ url: "mysql://localhost/db" });
        await assert.rejects(() => d.acquire(), /mysql\/node/);
    });

    it("sqlite throws toward wasm|node", async () => {
        const d = sqlite({ path: ":memory:" });
        await assert.rejects(() => d.acquire(), /sqlite\/(wasm|node)/);
    });
});

describe("native datasource registration", () => {
    it("redis/mongodb are not SqlDriver — server returns driver_unavailable", async () => {
        assert.equal(REDIS_MODEL, "native");
        assert.equal(MONGO_MODEL, "native");
        const server = createSqlStudioServer({
            databases: [redis({ id: "r1", url: "redis://localhost" }), mongodb({ id: "m1", url: "mongodb://localhost" })],
        });
        const opened = await server.handleMessage({
            v: PROTOCOL_VERSION,
            type: "openConnection",
            connectionId: "c1",
            datasourceId: "r1",
        });
        assert.equal(opened[0]?.type, "error");
        if (opened[0]?.type === "error") {
            assert.equal(opened[0].code, "driver_unavailable");
        }
        await server.destroy();
    });

    it("postgres studio registration lists as datasource", async () => {
        const server = createSqlStudioServer({
            databases: [postgres({ id: "pg", url: "postgres://localhost/db" })],
        });
        const listed = await server.handleMessage({ v: PROTOCOL_VERSION, type: "listDatasources" });
        assert.equal(listed[0]?.type, "datasources");
        if (listed[0]?.type === "datasources") {
            assert.deepEqual(listed[0].items, [{ id: "pg", kind: "postgres" }]);
        }
        await server.destroy();
    });
});
