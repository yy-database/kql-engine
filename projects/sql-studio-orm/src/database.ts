/**
 * createDatabase — typed entry for SQL Studio ORM.
 */

import type { DefaultCapabilities, SqlCapabilities } from "./capabilities.ts";
import type { CompiledQuery, QueryResult, SqlConnection, SqlDriver, SqlTransaction, TransactionOptions } from "./driver.ts";
import {
    createDeleteBuilder,
    createInsertBuilder,
    createUpdateBuilder,
    type DeleteQueryBuilder,
    type InsertQueryBuilder,
    type UpdateQueryBuilder,
} from "./query/dml.ts";
import { createSelectBuilder, type SelectQueryBuilder } from "./query/select.ts";
import { compileFragment, sql, type Sql, type SqlFragment } from "./sql.ts";
import type { Selectable } from "./types.ts";

type TableOf<DB, TB extends keyof DB> = DB[TB] extends object ? DB[TB] : never;

export type CreateDatabaseOptions = {
    readonly driver: SqlDriver;
};

export type TransactionApi<DB, Caps extends SqlCapabilities> = DatabaseApi<DB, Caps> & {
    readonly __transaction: true;
};

export type DatabaseApi<DB, Caps extends SqlCapabilities = DefaultCapabilities> = {
    readonly driver: SqlDriver;
    selectFrom<TB extends keyof DB & string>(table: TB): SelectQueryBuilder<DB, TB, TB, never, Selectable<TableOf<DB, TB>>, Caps>;
    insertInto<TB extends keyof DB & string>(table: TB): InsertQueryBuilder<DB, TB>;
    updateTable<TB extends keyof DB & string>(table: TB): UpdateQueryBuilder<DB, TB>;
    deleteFrom<TB extends keyof DB & string>(table: TB): DeleteQueryBuilder<DB, TB>;
    execute<R = Record<string, unknown>>(query: Sql | CompiledQuery): Promise<QueryResult<R>>;
    transaction(): {
        execute<T>(fn: (tx: TransactionApi<DB, Caps>) => Promise<T>, options?: TransactionOptions): Promise<T>;
    };
    destroy(): Promise<void>;
};

function isSqlFragment(value: unknown): value is SqlFragment {
    return !!value && typeof value === "object" && "strings" in value && "values" in value;
}

function wrapConnectionAsDriver(conn: SqlConnection, dialect: SqlDriver["dialect"]): SqlDriver {
    return {
        dialect,
        async acquire() {
            return conn;
        },
        async destroy() {},
    };
}

function createApi<DB, Caps extends SqlCapabilities>(driver: SqlDriver, caps: Caps): DatabaseApi<DB, Caps> {
    return {
        driver,
        selectFrom<TB extends keyof DB & string>(table: TB) {
            return createSelectBuilder<DB, TB, Caps>(driver, table, caps);
        },
        insertInto<TB extends keyof DB & string>(table: TB) {
            return createInsertBuilder<DB, TB, Caps>(driver, table);
        },
        updateTable<TB extends keyof DB & string>(table: TB) {
            return createUpdateBuilder<DB, TB, Caps>(driver, table);
        },
        deleteFrom<TB extends keyof DB & string>(table: TB) {
            return createDeleteBuilder<DB, TB, Caps>(driver, table);
        },
        async execute<R = Record<string, unknown>>(query: Sql | CompiledQuery): Promise<QueryResult<R>> {
            const compiled = isSqlFragment(query)
                ? compileFragment(query, driver.dialect === "postgres" ? (i) => `$${i + 1}` : () => "?")
                : query;
            const conn = await driver.acquire();
            return conn.execute<R>(compiled);
        },
        transaction() {
            return {
                async execute<T>(fn: (tx: TransactionApi<DB, Caps>) => Promise<T>, options?: TransactionOptions): Promise<T> {
                    const conn = await driver.acquire();
                    const txn: SqlTransaction = await conn.begin(options);
                    let closed = false;
                    const txDriver = wrapConnectionAsDriver(
                        {
                            execute(q, o) {
                                if (closed) {
                                    return Promise.reject(new Error("@yydb/sql-studio-orm: transaction already finished"));
                                }
                                return txn.execute(q, o);
                            },
                            stream(q, o) {
                                if (closed) {
                                    throw new Error("@yydb/sql-studio-orm: transaction already finished");
                                }
                                return txn.stream(q, o);
                            },
                            begin() {
                                return Promise.reject(new Error("@yydb/sql-studio-orm: nested begin via DatabaseApi not supported yet"));
                            },
                        },
                        driver.dialect,
                    );
                    const txApi = createApi<DB, Caps>(txDriver, caps) as TransactionApi<DB, Caps>;
                    Object.defineProperty(txApi, "__transaction", { value: true });
                    try {
                        const result = await fn(txApi);
                        await txn.commit();
                        return result;
                    } catch (err) {
                        await txn.rollback();
                        throw err;
                    } finally {
                        closed = true;
                    }
                },
            };
        },
        destroy(): Promise<void> {
            return driver.destroy();
        },
    };
}

export function createDatabase<DB, Caps extends SqlCapabilities = DefaultCapabilities>(options: CreateDatabaseOptions): DatabaseApi<DB, Caps> {
    return createApi<DB, Caps>(options.driver, {} as Caps);
}

export { sql };
