/**
 * Driver / connection contracts. ORM never opens TCP/WebSocket itself.
 */

import type { SqlDialect } from "./capabilities.ts";
import type { SqlType } from "./types.ts";

/** Portable abort handle (DOM AbortSignal-compatible subset). */
export type AbortLike = {
    readonly aborted: boolean;
};

export type QueryOperation = "select" | "insert" | "update" | "delete" | "ddl" | "raw";

export type TableReference = {
    readonly schema?: string;
    readonly name: string;
    readonly alias?: string;
};

export type CompiledQuery = {
    readonly sql: string;
    readonly parameters: readonly unknown[];
    readonly parameterTypes?: readonly SqlType[];
    readonly operation: QueryOperation;
    readonly tables: readonly TableReference[];
    /** Stable hash of sql + shape for audit / cache keys. */
    readonly fingerprint: string;
};

export type ExecuteOptions = {
    readonly signal?: AbortLike;
};

export type StreamOptions = {
    readonly signal?: AbortLike;
    readonly highWaterMark?: number;
};

export type QueryResult<R> = {
    readonly rows: R[];
    readonly rowCount: number;
    readonly columns?: readonly string[];
};

export type QueryChunk<R> = {
    readonly rows: R[];
    readonly done: boolean;
};

export type TransactionOptions = {
    readonly isolationLevel?: "read uncommitted" | "read committed" | "repeatable read" | "serializable";
    readonly readOnly?: boolean;
    readonly signal?: AbortLike;
};

export type SqlTransaction = SqlConnection & {
    commit(): Promise<void>;
    rollback(): Promise<void>;
};

export type SqlConnection = {
    execute<R = Record<string, unknown>>(query: CompiledQuery, options?: ExecuteOptions): Promise<QueryResult<R>>;

    stream<R = Record<string, unknown>>(query: CompiledQuery, options?: StreamOptions): AsyncIterable<QueryChunk<R>>;

    begin(options?: TransactionOptions): Promise<SqlTransaction>;
};

export type SqlDriver = {
    readonly dialect: SqlDialect;
    acquire(): Promise<SqlConnection>;
    destroy(): Promise<void>;
};
