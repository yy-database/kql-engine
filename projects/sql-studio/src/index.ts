/**
 * `@yydb/sql-studio` — browser workbench client.
 *
 * Talks SQL Studio Protocol to `@yydb/sql-studio-server`.
 * Does not open database TCP or interpret PG/MySQL wire protocols.
 */

import {
    PROTOCOL_VERSION,
    type StudioClientMessage,
    type StudioDatasourceInfo,
    type StudioQueryChunk,
    type StudioQueryRequest,
    type StudioServerMessage,
} from "@yydb/sql-studio-protocol";

export type CreateSqlStudioOptions = {
    /** HTTP(S) API prefix, e.g. `/api/sql-studio`. */
    endpoint: string;
    fetchImpl?: typeof fetch;
};

export type SqlStudio = {
    readonly endpoint: string;
    readonly protocolVersion: typeof PROTOCOL_VERSION;
    hello(): Promise<void>;
    listDatasources(): Promise<StudioDatasourceInfo[]>;
    openConnection(connectionId: string, datasourceId: string): Promise<void>;
    closeConnection(connectionId: string): Promise<void>;
    runQuery(request: StudioQueryRequest): AsyncIterable<StudioQueryChunk>;
    cancel(requestId: string): Promise<void>;
};

async function postMessages(endpoint: string, fetchImpl: typeof fetch, message: StudioClientMessage): Promise<StudioServerMessage[]> {
    const res = await fetchImpl(endpoint, {
        method: "POST",
        headers: { "content-type": "application/json", accept: "application/json" },
        body: JSON.stringify(message),
    });
    if (!res.ok) {
        throw new Error(`@yydb/sql-studio: HTTP ${res.status} from ${endpoint}`);
    }
    const data: unknown = await res.json();
    return Array.isArray(data) ? (data as StudioServerMessage[]) : [data as StudioServerMessage];
}

function firstError(messages: StudioServerMessage[]): never | void {
    const err = messages.find((m) => m.type === "error");
    if (err && err.type === "error") {
        throw new Error(`@yydb/sql-studio: ${err.code ?? "error"}: ${err.message}`);
    }
}

/** Create a browser Studio client bound to a server endpoint. */
export function createSqlStudio(options: CreateSqlStudioOptions): SqlStudio {
    const fetchImpl = options.fetchImpl ?? fetch;
    const endpoint = options.endpoint;

    return {
        endpoint,
        protocolVersion: PROTOCOL_VERSION,

        async hello() {
            const messages = await postMessages(endpoint, fetchImpl, {
                v: PROTOCOL_VERSION,
                type: "hello",
            });
            firstError(messages);
            if (!messages.some((m) => m.type === "helloOk")) {
                throw new Error("@yydb/sql-studio: missing helloOk");
            }
        },

        async listDatasources() {
            const messages = await postMessages(endpoint, fetchImpl, {
                v: PROTOCOL_VERSION,
                type: "listDatasources",
            });
            firstError(messages);
            const found = messages.find((m) => m.type === "datasources");
            if (!found || found.type !== "datasources") {
                throw new Error("@yydb/sql-studio: missing datasources");
            }
            return found.items;
        },

        async openConnection(connectionId, datasourceId) {
            const messages = await postMessages(endpoint, fetchImpl, {
                v: PROTOCOL_VERSION,
                type: "openConnection",
                connectionId,
                datasourceId,
            });
            firstError(messages);
            if (!messages.some((m) => m.type === "connectionOpened")) {
                throw new Error("@yydb/sql-studio: missing connectionOpened");
            }
        },

        async closeConnection(connectionId) {
            const messages = await postMessages(endpoint, fetchImpl, {
                v: PROTOCOL_VERSION,
                type: "closeConnection",
                connectionId,
            });
            firstError(messages);
            if (!messages.some((m) => m.type === "connectionClosed")) {
                throw new Error("@yydb/sql-studio: missing connectionClosed");
            }
        },

        async *runQuery(request: StudioQueryRequest): AsyncIterable<StudioQueryChunk> {
            const messages = await postMessages(endpoint, fetchImpl, {
                v: PROTOCOL_VERSION,
                type: "query",
                ...request,
            });
            for (const m of messages) {
                if (m.type === "queryChunk") {
                    yield {
                        requestId: m.requestId,
                        columns: m.columns,
                        rows: m.rows,
                        done: m.done,
                        error: m.error,
                    };
                } else if (m.type === "error") {
                    yield {
                        requestId: request.requestId,
                        done: true,
                        error: { message: m.message, code: m.code },
                    };
                }
            }
        },

        async cancel(requestId) {
            await postMessages(endpoint, fetchImpl, {
                v: PROTOCOL_VERSION,
                type: "cancel",
                requestId,
            });
        },
    };
}

export { PROTOCOL_VERSION };
export type { StudioQueryChunk, StudioQueryRequest, StudioDatasourceInfo };
// Node CLI builder is exported only via package exports path "./cli".
