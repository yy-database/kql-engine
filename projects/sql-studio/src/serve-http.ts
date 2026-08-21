/**
 * Minimal Node HTTP adapter: POST JSON StudioClientMessage → StudioServerMessage[].
 * Used by `sql serve` (CLI only — not the browser entry).
 */
import http from "node:http";
import type { AddressInfo } from "node:net";
import { createMemoryDriver } from "@yydb/sql-studio-orm";
import { PROTOCOL_VERSION, isClientMessage } from "@yydb/sql-studio-protocol";
import { createSqlStudioServer, type SqlStudioServer } from "@yydb/sql-studio-server";

export type ServeHttpOptions = {
    host: string;
    port: number;
    /** When true, run hello probe then close (CI / self-test). */
    selfTest?: boolean;
};

export type ServeHttpHandle = {
    readonly url: string;
    readonly server: SqlStudioServer;
    close(): Promise<void>;
};

function readBody(req: http.IncomingMessage): Promise<string> {
    return new Promise((resolve, reject) => {
        const chunks: Buffer[] = [];
        req.on("data", (c) => chunks.push(Buffer.isBuffer(c) ? c : Buffer.from(c)));
        req.on("end", () => resolve(Buffer.concat(chunks).toString("utf8")));
        req.on("error", reject);
    });
}

export async function startMemoryServeHttp(options: ServeHttpOptions): Promise<ServeHttpHandle> {
    const driver = createMemoryDriver({
        dialect: "postgres",
        onExecute: (query) => ({
            rows: query.operation === "select" || query.operation === "raw" ? [{ ok: true }] : [],
            rowCount: 1,
            columns: ["ok"],
        }),
    });
    const studio = createSqlStudioServer({
        databases: [{ id: "memory", kind: "postgres", driver }],
    });

    const httpServer = http.createServer(async (req, res) => {
        if (req.method === "GET" && (req.url === "/" || req.url === "/health")) {
            res.writeHead(200, { "content-type": "application/json" });
            res.end(JSON.stringify({ ok: true, protocolVersion: PROTOCOL_VERSION, datasources: studio.listDatasources() }));
            return;
        }
        if (req.method !== "POST") {
            res.writeHead(405, { "content-type": "application/json" });
            res.end(JSON.stringify({ error: "method_not_allowed" }));
            return;
        }
        try {
            const raw = await readBody(req);
            const parsed: unknown = raw ? JSON.parse(raw) : null;
            if (!isClientMessage(parsed)) {
                res.writeHead(400, { "content-type": "application/json" });
                res.end(
                    JSON.stringify([
                        {
                            v: PROTOCOL_VERSION,
                            type: "error",
                            code: "bad_message",
                            message: "invalid Studio Protocol message",
                        },
                    ]),
                );
                return;
            }
            const out = await studio.handleMessage(parsed);
            res.writeHead(200, { "content-type": "application/json" });
            res.end(JSON.stringify(out));
        } catch (err) {
            res.writeHead(500, { "content-type": "application/json" });
            res.end(JSON.stringify({ error: err instanceof Error ? err.message : String(err) }));
        }
    });

    await new Promise<void>((resolve, reject) => {
        httpServer.once("error", reject);
        httpServer.listen(options.port, options.host, () => resolve());
    });

    const addr = httpServer.address() as AddressInfo;
    const url = `http://${options.host === "0.0.0.0" ? "127.0.0.1" : options.host}:${addr.port}`;

    const handle: ServeHttpHandle = {
        url,
        server: studio,
        async close() {
            await new Promise<void>((resolve, reject) => {
                httpServer.close((err) => (err ? reject(err) : resolve()));
            });
            await studio.destroy();
        },
    };

    if (options.selfTest) {
        const res = await fetch(url, {
            method: "POST",
            headers: { "content-type": "application/json" },
            body: JSON.stringify({ v: PROTOCOL_VERSION, type: "hello" }),
        });
        const body = (await res.json()) as unknown;
        const ok = Array.isArray(body) && body.some((m) => (m as { type?: string }).type === "helloOk");
        if (!ok) {
            await handle.close();
            throw new Error("@yydb/sql-studio: serve --self-test failed (no helloOk)");
        }
        await handle.close();
    }

    return handle;
}
