/**
 * MemoryDriver + compile/fingerprint conformance (0.0.1).
 */
import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { createDatabase, createMemoryDriver, fingerprintSql, type Generated } from "../src/index.ts";

type DemoDb = {
    users: {
        id: Generated<number>;
        email: string;
    };
};

describe("MemoryDriver", () => {
    it("records CompiledQuery history and returns onExecute rows", async () => {
        const driver = createMemoryDriver({
            dialect: "postgres",
            onExecute: (query) => ({
                rows: query.operation === "select" ? [{ id: 1, email: "a@b.c" }] : [],
                rowCount: 1,
                columns: ["id", "email"],
            }),
        });

        const db = createDatabase<DemoDb>({ driver });
        const rows = await db.selectFrom("users").select(["id", "email"]).where("email", "=", "a@b.c").limit(5).execute();

        assert.equal(rows.length, 1);
        assert.equal(rows[0]!.email, "a@b.c");
        assert.equal(driver.history.length, 1);
        assert.equal(driver.history[0]!.fingerprint, fingerprintSql(driver.history[0]!.sql));
    });

    it("compile() fingerprint matches fingerprintSql(sql)", () => {
        const driver = createMemoryDriver({ dialect: "postgres" });
        const db = createDatabase<DemoDb>({ driver });
        const compiled = db.selectFrom("users").select(["id"]).where("id", "=", 1).compile();
        assert.equal(compiled.fingerprint, fingerprintSql(compiled.sql));
        assert.match(compiled.sql, /select/i);
        assert.deepEqual(compiled.parameters, [1]);
    });
});
