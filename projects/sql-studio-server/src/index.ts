/**
 * `@yydb/sql-studio-server` — authenticated data-access host for SQL Studio.
 */

import type { SqlDriver } from "@yydb/sql-studio-orm";
import {
    PROTOCOL_VERSION,
    isClientMessage,
    type StudioClientMessage,
    type StudioDatasourceInfo,
    type StudioDatasourceKind,
    type StudioDriverRegistration,
    type StudioErrorCode,
    type StudioServerMessage,
} from "@yydb/sql-studio-protocol";

function errorMessage(code: StudioErrorCode, message: string, requestId?: string): StudioServerMessage {
    return requestId === undefined
        ? { v: PROTOCOL_VERSION, type: "error", code, message }
        : { v: PROTOCOL_VERSION, type: "error", code, message, requestId };
}

export type CreateSqlStudioServerOptions = {
    databases: StudioDriverRegistration[];
};

export type SqlStudioServer = {
    readonly protocolVersion: typeof PROTOCOL_VERSION;
    readonly databases: readonly StudioDriverRegistration[];
    listDatasources(): StudioDatasourceInfo[];
    handleMessage(message: StudioClientMessage): Promise<StudioServerMessage[]>;
    destroy(): Promise<void>;
};

/** Minimal cancel token — avoids DOM `AbortController` in Node tsconfigs. */
class CancelToken {
    #aborted = false;
    readonly signal: { readonly aborted: boolean };

    constructor() {
        const self = this;
        this.signal = {
            get aborted() {
                return self.#aborted;
            },
        };
    }

    abort(): void {
        this.#aborted = true;
    }
}

function isSqlDriver(value: unknown): value is SqlDriver {
    return (
        !!value && typeof value === "object" && "dialect" in value && "acquire" in value && typeof (value as SqlDriver).acquire === "function"
    );
}

/**
 * Create a Studio server runtime (in-memory connections + protocol dispatch).
 * HTTP/WS adapters mount `handleMessage` later.
 */
export function createSqlStudioServer(options: CreateSqlStudioServerOptions): SqlStudioServer {
    const databases = Object.freeze([...options.databases]);
    const byId = new Map(databases.map((d) => [d.id, d]));
    const live = new Map<string, { datasourceId: string; driver: SqlDriver; abort: CancelToken }>();
    const inflight = new Map<string, CancelToken>();

    const listDatasources = (): StudioDatasourceInfo[] => databases.map((d) => ({ id: d.id, kind: d.kind }));

    const handleMessage = async (message: StudioClientMessage): Promise<StudioServerMessage[]> => {
        if (!isClientMessage(message)) {
            return [errorMessage("bad_message", "invalid Studio Protocol message")];
        }

        switch (message.type) {
            case "hello":
                return [
                    {
                        v: PROTOCOL_VERSION,
                        type: "helloOk",
                        protocolVersion: PROTOCOL_VERSION,
                    },
                ];

            case "listDatasources":
                return [
                    {
                        v: PROTOCOL_VERSION,
                        type: "datasources",
                        items: listDatasources(),
                    },
                ];

            case "openConnection": {
                const reg = byId.get(message.datasourceId);
                if (!reg) {
                    return [errorMessage("unknown_datasource", `unknown datasource ${message.datasourceId}`)];
                }
                if (!isSqlDriver(reg.driver)) {
                    return [errorMessage("driver_unavailable", `datasource ${reg.id} has no SqlDriver (Redis/Mongo use native openers)`)];
                }
                if (live.has(message.connectionId)) {
                    return [errorMessage("connection_exists", `connection ${message.connectionId} already open`)];
                }
                live.set(message.connectionId, {
                    datasourceId: reg.id,
                    driver: reg.driver,
                    abort: new CancelToken(),
                });
                return [
                    {
                        v: PROTOCOL_VERSION,
                        type: "connectionOpened",
                        connectionId: message.connectionId,
                        datasourceId: reg.id,
                    },
                ];
            }

            case "closeConnection": {
                const conn = live.get(message.connectionId);
                if (conn) {
                    conn.abort.abort();
                    live.delete(message.connectionId);
                }
                return [
                    {
                        v: PROTOCOL_VERSION,
                        type: "connectionClosed",
                        connectionId: message.connectionId,
                    },
                ];
            }

            case "cancel": {
                inflight.get(message.requestId)?.abort();
                inflight.delete(message.requestId);
                return [];
            }

            case "query": {
                const conn = live.get(message.connectionId);
                if (!conn) {
                    return [errorMessage("no_connection", `connection ${message.connectionId} is not open`, message.requestId)];
                }
                const ac = new CancelToken();
                inflight.set(message.requestId, ac);
                try {
                    const sqlConn = await conn.driver.acquire();
                    const compiled = {
                        sql: message.text,
                        parameters: message.params ?? [],
                        operation: "raw" as const,
                        tables: [],
                        fingerprint: message.requestId,
                    };
                    const result = await sqlConn.execute(compiled, { signal: ac.signal });
                    const columns = result.columns ? [...result.columns] : undefined;
                    const rows = result.rows.map((row) =>
                        columns ? columns.map((c) => (row as Record<string, unknown>)[c]) : Object.values(row as Record<string, unknown>),
                    );
                    return [
                        {
                            v: PROTOCOL_VERSION,
                            type: "queryChunk",
                            requestId: message.requestId,
                            columns,
                            rows,
                            done: true,
                        },
                    ];
                } catch (err) {
                    return [
                        {
                            v: PROTOCOL_VERSION,
                            type: "queryChunk",
                            requestId: message.requestId,
                            done: true,
                            error: {
                                message: err instanceof Error ? err.message : String(err),
                                code: "query_failed",
                            },
                        },
                    ];
                } finally {
                    inflight.delete(message.requestId);
                }
            }

            default:
                return [errorMessage("unsupported", "unsupported message type")];
        }
    };

    return {
        protocolVersion: PROTOCOL_VERSION,
        databases,
        listDatasources,
        handleMessage,
        async destroy() {
            for (const c of live.values()) c.abort.abort();
            live.clear();
            for (const d of databases) {
                if (isSqlDriver(d.driver)) await d.driver.destroy();
            }
        },
    };
}

export { PROTOCOL_VERSION };
export type { StudioDatasourceKind, StudioDriverRegistration };
