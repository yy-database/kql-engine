/**
 * `@yydb/postgres` — PostgreSQL SqlDriver + Studio registration.
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

export const DRIVER_ID = "postgres" as const;

export type PostgresPoolOptions = {
    min?: number;
    max?: number;
};

export type PostgresDriverOptions = {
    url: string;
    pool?: PostgresPoolOptions;
};

export type PostgresStudioOptions = PostgresDriverOptions & {
    id: string;
};

class NotImplementedConnection implements SqlConnection {
    async execute<R>(_query: CompiledQuery, _options?: ExecuteOptions): Promise<QueryResult<R>> {
        throw new Error("@yydb/postgres: wire execute not implemented yet");
    }

    stream<R>(_query: CompiledQuery, _options?: StreamOptions): AsyncIterable<QueryChunk<R>> {
        throw new Error("@yydb/postgres: wire stream not implemented yet");
    }

    async begin(_options?: TransactionOptions): Promise<SqlTransaction> {
        throw new Error("@yydb/postgres: transactions not implemented yet");
    }
}

class PostgresDriver implements SqlDriver {
    readonly dialect = "postgres" as const;
    readonly url: string;
    readonly pool?: PostgresPoolOptions;

    constructor(options: PostgresDriverOptions) {
        this.url = options.url;
        this.pool = options.pool;
    }

    async acquire(): Promise<SqlConnection> {
        return new NotImplementedConnection();
    }

    async destroy(): Promise<void> {}
}

/** ORM / app entry — returns a real `SqlDriver` (TCP later). */
export function postgres(options: PostgresDriverOptions): SqlDriver;
/** Studio server registration when `id` is present. */
export function postgres(options: PostgresStudioOptions): StudioDriverRegistration;
export function postgres(options: PostgresDriverOptions | PostgresStudioOptions): SqlDriver | StudioDriverRegistration {
    if ("id" in options && options.id) {
        return {
            id: options.id,
            kind: "postgres",
            driver: new PostgresDriver(options),
        };
    }
    return new PostgresDriver(options);
}
