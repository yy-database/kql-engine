/**
 * Local diagnostics for `sql doctor` (Node CLI only).
 */
import { PROTOCOL_VERSION, STUDIO_ERROR_CODES } from "@yydb/sql-studio-protocol";
import { createMemoryDriver } from "@yydb/sql-studio-orm";
import { createSqlStudioServer } from "@yydb/sql-studio-server";

export type DoctorReport = {
    ok: boolean;
    version: string;
    node: string;
    protocolVersion: typeof PROTOCOL_VERSION;
    errorCodeCount: number;
    memoryDriver: { dialect: string; acquireOk: boolean };
    serverHello: boolean;
};

export async function runDoctor(version: string): Promise<DoctorReport> {
    const driver = createMemoryDriver({ dialect: "postgres" });
    let acquireOk = false;
    try {
        const conn = await driver.acquire();
        acquireOk = typeof conn.execute === "function";
    } catch {
        acquireOk = false;
    }

    const server = createSqlStudioServer({
        databases: [{ id: "memory", kind: "postgres", driver }],
    });
    const hello = await server.handleMessage({ v: PROTOCOL_VERSION, type: "hello" });
    const serverHello = hello.some((m) => m.type === "helloOk");
    await server.destroy();

    const report: DoctorReport = {
        ok: acquireOk && serverHello,
        version,
        node: process.versions.node,
        protocolVersion: PROTOCOL_VERSION,
        errorCodeCount: STUDIO_ERROR_CODES.length,
        memoryDriver: { dialect: driver.dialect, acquireOk },
        serverHello,
    };
    return report;
}
