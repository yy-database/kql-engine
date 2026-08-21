/**
 * Built-in MemoryDriver + compile smoke used by `sql orm compile`.
 * Keep this free of network I/O.
 */
import { createDatabase, createMemoryDriver, fingerprintSql, type Generated } from "@yydb/sql-studio-orm";

type DemoDb = {
    users: {
        id: Generated<number>;
        email: string;
    };
};

export type CompilePrintout = {
    dialect: string;
    sql: string;
    parameters: unknown[];
    fingerprint: string;
    memoryRowCount: number;
    historyFingerprints: string[];
};

/** Compile a sample select and execute it on MemoryDriver; return printable evidence. */
export async function runMemoryCompileFixture(): Promise<CompilePrintout> {
    const driver = createMemoryDriver({
        dialect: "postgres",
        onExecute: (query) => ({
            rows: query.operation === "select" ? [{ id: 1, email: "demo@example.com" }] : [],
            rowCount: 1,
            columns: ["id", "email"],
        }),
    });

    const db = createDatabase<DemoDb>({ driver });
    const compiled = db.selectFrom("users").select(["id", "email"]).where("email", "=", "demo@example.com").limit(10).compile();

    const rows = await db.selectFrom("users").select(["id", "email"]).where("email", "=", "demo@example.com").limit(10).execute();

    if (compiled.fingerprint !== fingerprintSql(compiled.sql)) {
        throw new Error("@yydb/sql-studio: compile fingerprint mismatch");
    }
    if (driver.history.length < 1) {
        throw new Error("@yydb/sql-studio: MemoryDriver history empty after execute");
    }
    if (rows.length !== 1) {
        throw new Error("@yydb/sql-studio: MemoryDriver expected 1 row");
    }

    return {
        dialect: driver.dialect,
        sql: compiled.sql,
        parameters: [...compiled.parameters],
        fingerprint: compiled.fingerprint,
        memoryRowCount: rows.length,
        historyFingerprints: driver.history.map((q) => q.fingerprint),
    };
}
