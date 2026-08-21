/**
 * `@yydb/sqlite` — SQLite SqlDriver + Studio registration.
 * Use `./wasm` or `./node` for concrete backends.
 */

import type { SqlConnection, SqlDriver } from "@yydb/sql-studio-orm";
import type { StudioDriverRegistration } from "@yydb/sql-studio-protocol";

export const DRIVER_ID = "sqlite" as const;
export const DRIVER_STATUS = "experimental" as const;

export type SqliteDriverOptions = {
    path: string;
};

export type SqliteStudioOptions = SqliteDriverOptions & {
    id: string;
};

class SqliteDriver implements SqlDriver {
    readonly dialect = "sqlite" as const;
    readonly path: string;

    constructor(options: SqliteDriverOptions) {
        this.path = options.path;
    }

    async acquire(): Promise<SqlConnection> {
        throw new Error("@yydb/sqlite: use @yydb/sqlite/wasm or @yydb/sqlite/node for a concrete backend");
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
