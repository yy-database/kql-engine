/**
 * `@yydb/sqlite` — SQLite SqlDriver + Studio registration.
 * Use `./wasm` or `./node` for concrete backends.
 */

import type {
    CompiledQuery,
    ExecuteOptions,
    QueryChunk,
    QueryResult,
    SqlConnection,
    SqlDriver,
    SqlTransaction,
    StreamOptions,
    TransactionOptions,
} from "@yydb/sql-studio-orm";
import type { StudioDriverRegistration } from "@yydb/sql-studio-protocol";

export const DRIVER_ID = "sqlite" as const;

export type SqliteDriverOptions = {
    path: string;
};

export type SqliteStudioOptions = SqliteDriverOptions & {
    id: string;
};

class NotImplementedConnection implements SqlConnection {
    async execute<R>(_query: CompiledQuery, _options?: ExecuteOptions): Promise<QueryResult<R>> {
        throw new Error("@yydb/sqlite: execute not implemented yet");
    }

    stream<R>(_query: CompiledQuery, _options?: StreamOptions): AsyncIterable<QueryChunk<R>> {
        throw new Error("@yydb/sqlite: stream not implemented yet");
    }

    async begin(_options?: TransactionOptions): Promise<SqlTransaction> {
        throw new Error("@yydb/sqlite: transactions not implemented yet");
    }
}

class SqliteDriver implements SqlDriver {
    readonly dialect = "sqlite" as const;
    readonly path: string;

    constructor(options: SqliteDriverOptions) {
        this.path = options.path;
    }

    async acquire(): Promise<SqlConnection> {
        return new NotImplementedConnection();
    }

    async destroy(): Promise<void> {}
}

export function sqlite(options: SqliteDriverOptions): SqlDriver;
export function sqlite(options: SqliteStudioOptions): StudioDriverRegistration;
export function sqlite(options: SqliteDriverOptions | SqliteStudioOptions): SqlDriver | StudioDriverRegistration {
    if ("id" in options && options.id) {
        return {
            id: options.id,
            kind: "sqlite",
            driver: new SqliteDriver(options),
        };
    }
    return new SqliteDriver(options);
}
