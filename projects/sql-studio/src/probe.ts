/**
 * Full client ↔ MemoryDriver HTTP serve round-trip for DP / CI.
 */
import { createSqlStudio } from "./index.ts";
import { startMemoryServeHttp } from "./serve-http.ts";

export type ProbeReport = {
    ok: boolean;
    version: string;
    url: string;
    datasources: string[];
    queryRows: unknown[][];
};

export async function runMemoryProbe(version: string): Promise<ProbeReport> {
    const handle = await startMemoryServeHttp({ host: "127.0.0.1", port: 0 });
    try {
        const client = createSqlStudio({ endpoint: handle.url });
        await client.hello();
        const items = await client.listDatasources();
        const memory = items.find((d) => d.id === "memory");
        if (!memory) {
            throw new Error("@yydb/sql-studio: probe missing memory datasource");
        }
        await client.openConnection("probe-conn", memory.id);
        const rows: unknown[][] = [];
        for await (const chunk of client.runQuery({
            connectionId: "probe-conn",
            text: "select 1 as ok",
            requestId: "probe-q1",
        })) {
            if (chunk.error) {
                throw new Error(`@yydb/sql-studio: probe query failed: ${chunk.error.message}`);
            }
            if (chunk.rows) rows.push(...chunk.rows);
        }
        await client.closeConnection("probe-conn");
        return {
            ok: rows.length > 0,
            version,
            url: handle.url,
            datasources: items.map((d) => d.id),
            queryRows: rows,
        };
    } finally {
        await handle.close();
    }
}
