/**
 * Typed insert / update / delete builders.
 */

import type { SqlCapabilities, SqlDialect } from "../capabilities.ts";
import type { CompiledQuery, QueryResult, SqlDriver } from "../driver.ts";
import type { Insertable, Updateable } from "../types.ts";
import type { AstWhere } from "./ast.ts";
import { compileDeleteAst, compileInsertAst, compileUpdateAst } from "./compile.ts";
import type { ComparisonOp } from "./select.ts";

type TableOf<DB, TB extends keyof DB> = DB[TB] extends object ? DB[TB] : never;

export type InsertQueryBuilder<DB, TB extends keyof DB & string> = {
    values(row: Insertable<TableOf<DB, TB>>): InsertQueryBuilder<DB, TB>;
    compile(): CompiledQuery;
    execute(): Promise<QueryResult<Record<string, unknown>>>;
};

export type UpdateQueryBuilder<DB, TB extends keyof DB & string> = {
    set(row: Updateable<TableOf<DB, TB>>): UpdateQueryBuilder<DB, TB>;
    where(column: keyof TableOf<DB, TB> & string, op: ComparisonOp, value: unknown): UpdateQueryBuilder<DB, TB>;
    compile(): CompiledQuery;
    execute(): Promise<QueryResult<Record<string, unknown>>>;
};

export type DeleteQueryBuilder<DB, TB extends keyof DB & string> = {
    where(column: keyof TableOf<DB, TB> & string, op: ComparisonOp, value: unknown): DeleteQueryBuilder<DB, TB>;
    compile(): CompiledQuery;
    execute(): Promise<QueryResult<Record<string, unknown>>>;
};

export function createInsertBuilder<DB, TB extends keyof DB & string, _Caps extends SqlCapabilities>(
    driver: SqlDriver,
    table: TB,
): InsertQueryBuilder<DB, TB> {
    let columns: string[] = [];
    let values: unknown[] = [];
    const dialect: SqlDialect = driver.dialect;

    const api: InsertQueryBuilder<DB, TB> = {
        values(row) {
            columns = Object.keys(row as object);
            values = columns.map((c) => (row as Record<string, unknown>)[c]);
            return api;
        },
        compile() {
            if (columns.length === 0) {
                throw new Error("@yydb/sql-studio-orm: insert.values() required before compile");
            }
            return compileInsertAst(
                {
                    kind: "insert",
                    into: { kind: "table", name: table },
                    columns,
                    values,
                },
                dialect,
            );
        },
        async execute() {
            const conn = await driver.acquire();
            return conn.execute(api.compile());
        },
    };
    return api;
}

export function createUpdateBuilder<DB, TB extends keyof DB & string, _Caps extends SqlCapabilities>(
    driver: SqlDriver,
    table: TB,
): UpdateQueryBuilder<DB, TB> {
    let set: { column: string; value: unknown }[] = [];
    const where: AstWhere[] = [];
    const dialect: SqlDialect = driver.dialect;

    const api: UpdateQueryBuilder<DB, TB> = {
        set(row) {
            set = Object.entries(row as object).map(([column, value]) => ({ column, value }));
            return api;
        },
        where(column, op, value) {
            where.push({ kind: "cmp", column, op, value });
            return api;
        },
        compile() {
            if (set.length === 0) {
                throw new Error("@yydb/sql-studio-orm: update.set() required before compile");
            }
            return compileUpdateAst(
                {
                    kind: "update",
                    table: { kind: "table", name: table },
                    set,
                    where,
                },
                dialect,
            );
        },
        async execute() {
            const conn = await driver.acquire();
            return conn.execute(api.compile());
        },
    };
    return api;
}

export function createDeleteBuilder<DB, TB extends keyof DB & string, _Caps extends SqlCapabilities>(
    driver: SqlDriver,
    table: TB,
): DeleteQueryBuilder<DB, TB> {
    const where: AstWhere[] = [];
    const dialect: SqlDialect = driver.dialect;

    const api: DeleteQueryBuilder<DB, TB> = {
        where(column, op, value) {
            where.push({ kind: "cmp", column, op, value });
            return api;
        },
        compile() {
            return compileDeleteAst(
                {
                    kind: "delete",
                    from: { kind: "table", name: table },
                    where,
                },
                dialect,
            );
        },
        async execute() {
            const conn = await driver.acquire();
            return conn.execute(api.compile());
        },
    };
    return api;
}
