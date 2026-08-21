/**
 * `@yydb/postgres` — PostgreSQL SqlDriver + Studio registration.
 *
 * Default entry is browser-safe stub. TCP wire lives on `@yydb/postgres/node`.
 */

import type { SqlConnection, SqlDriver } from "@yydb/sql-studio-orm";
import type { StudioDriverRegistration } from "@yydb/sql-studio-protocol";

export const DRIVER_ID = "postgres" as const;
export const DRIVER_STATUS = "experimental" as const;

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

class PostgresDriver implements SqlDriver {
    readonly dialect = "postgres" as const;
    readonly url: string;
    readonly pool?: PostgresPoolOptions;

    constructor(options: PostgresDriverOptions) {
        this.url = options.url;
        this.pool = options.pool;
    }

    async acquire(): Promise<SqlConnection> {
        throw new Error("@yydb/postgres: TCP acquire is Node-only — import from @yydb/postgres/node (not implemented yet)");
    }

    async destroy(): Promise<void> {}
}

/** ORM / app entry — returns a real `SqlDriver` shape (TCP later via `/node`). */
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
