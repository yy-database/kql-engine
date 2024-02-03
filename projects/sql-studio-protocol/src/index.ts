/**
 * SQL Studio Protocol — experimental public DTOs shared by workbench + server.
 *
 * 0.0.x developer-preview contract: breaking changes are allowed and the
 * protocol version/fingerprint must be checked by every caller.
 */

export const PROTOCOL_VERSION = 1 as const;

export type StudioDatasourceKind = "postgres" | "mysql" | "sqlite" | "redis" | "mongodb";

export type StudioErrorBody = {
    message: string;
    code?: string;
};

/** Opaque driver registration consumed by `@yydb/sql-studio-server`. */
export type StudioDriverRegistration = {
    id: string;
    kind: StudioDatasourceKind;
    /** Opaque handle from `@yydb/{postgres,mysql,...}` factories. */
    driver: unknown;
};

export type StudioDatasourceInfo = {
    id: string;
    kind: StudioDatasourceKind;
};

export type StudioQueryRequest = {
    connectionId: string;
    /** SQL text for relational engines; opaque command for Redis/Mongo. */
    text: string;
    params?: unknown[];
    requestId: string;
};

export type StudioQueryChunk = {
    requestId: string;
    columns?: string[];
    rows?: unknown[][];
    done?: boolean;
    error?: StudioErrorBody;
};

export type StudioCancelRequest = {
    requestId: string;
};

export type StudioProgressEvent = {
    requestId: string;
    message: string;
    percent?: number;
};

/** One row-batch without requestId (driver → server). */
export type StudioResultBatch = {
    columns?: string[];
    rows?: unknown[][];
    done?: boolean;
    error?: StudioErrorBody;
};

/**
 * Portable abort handle (DOM `AbortSignal`-compatible subset).
 * Avoids requiring DOM libs in every Node package tsconfig.
 */
export type StudioAbortSignal = {
    readonly aborted: boolean;
};

/**
 * Live connection owned by the server after `openConnection`.
 * Drivers implement this; browsers never see it.
 */
export type StudioLiveConnection = {
    query(text: string, params: unknown[] | undefined, signal: StudioAbortSignal): AsyncIterable<StudioResultBatch>;
    close(): Promise<void>;
};

/**
 * Driver package binding: turn registration.driver config into a live connection.
 * Registered on the server (or supplied by the driver factory).
 */
export type StudioDriverOpener = {
    kind: StudioDatasourceKind;
    open(config: unknown, signal: StudioAbortSignal): Promise<StudioLiveConnection>;
};

// ── Wire messages (HTTPS JSON or WebSocket frames) ──────────────────────────

export type StudioClientMessage =
    | { v: typeof PROTOCOL_VERSION; type: "hello" }
    | { v: typeof PROTOCOL_VERSION; type: "listDatasources" }
    | {
          v: typeof PROTOCOL_VERSION;
          type: "openConnection";
          connectionId: string;
          datasourceId: string;
      }
    | {
          v: typeof PROTOCOL_VERSION;
          type: "closeConnection";
          connectionId: string;
      }
    | ({ v: typeof PROTOCOL_VERSION; type: "query" } & StudioQueryRequest)
    | ({ v: typeof PROTOCOL_VERSION; type: "cancel" } & StudioCancelRequest);

export type StudioServerMessage =
    | {
          v: typeof PROTOCOL_VERSION;
          type: "helloOk";
          protocolVersion: typeof PROTOCOL_VERSION;
      }
    | {
          v: typeof PROTOCOL_VERSION;
          type: "datasources";
          items: StudioDatasourceInfo[];
      }
    | {
          v: typeof PROTOCOL_VERSION;
          type: "connectionOpened";
          connectionId: string;
          datasourceId: string;
      }
    | {
          v: typeof PROTOCOL_VERSION;
          type: "connectionClosed";
          connectionId: string;
      }
    | ({ v: typeof PROTOCOL_VERSION; type: "queryChunk" } & StudioQueryChunk)
    | ({ v: typeof PROTOCOL_VERSION; type: "progress" } & StudioProgressEvent)
    | {
          v: typeof PROTOCOL_VERSION;
          type: "error";
          message: string;
          code?: string;
          requestId?: string;
      };

export function isClientMessage(value: unknown): value is StudioClientMessage {
    if (!value || typeof value !== "object") return false;
    const msg = value as { v?: unknown; type?: unknown };
    return msg.v === PROTOCOL_VERSION && typeof msg.type === "string";
}
