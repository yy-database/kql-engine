/**
 * Dialect-aware select / insert / update / delete AST → CompiledQuery.
 */

import type { SqlDialect } from "../capabilities.ts";
import type { CompiledQuery } from "../driver.ts";
import { fingerprintSql } from "../fingerprint.ts";
import type { SelectAst } from "./ast.ts";
import { compileWhereClause, placeholder, quoteIdent } from "./dialect.ts";
import type { DeleteAst, InsertAst, UpdateAst } from "./dml-ast.ts";

export function compileSelectAst(ast: SelectAst, dialect: SqlDialect = "postgres"): CompiledQuery {
    const params: unknown[] = [];
    const tables = [{ name: ast.from.name, alias: ast.from.alias }];

    let sql = "select ";
    sql += ast.columns.length === 0 ? "*" : ast.columns.map((c) => quoteIdent(c, dialect)).join(", ");
    sql += ` from ${quoteIdent(ast.from.name, dialect)}`;
    if (ast.from.alias) sql += ` as ${quoteIdent(ast.from.alias, dialect)}`;

    for (const join of ast.joins) {
        tables.push({ name: join.table.name, alias: join.table.alias });
        sql += ` ${join.kind} join ${quoteIdent(join.table.name, dialect)}`;
        if (join.table.alias) sql += ` as ${quoteIdent(join.table.alias, dialect)}`;
        sql += ` on ${quoteIdent(join.onLeft, dialect)} = ${quoteIdent(join.onRight, dialect)}`;
    }

    sql += compileWhereClause(ast.where, dialect, params);

    if (ast.orderBy.length > 0) {
        sql += " order by " + ast.orderBy.map((o) => `${quoteIdent(o.column, dialect)} ${o.direction}`).join(", ");
    }

    if (ast.limit !== undefined) {
        params.push(ast.limit);
        sql += ` limit ${placeholder(dialect, params.length)}`;
    }
    if (ast.offset !== undefined) {
        params.push(ast.offset);
        sql += ` offset ${placeholder(dialect, params.length)}`;
    }

    return {
        sql,
        parameters: params,
        operation: "select",
        tables,
        fingerprint: fingerprintSql(sql),
    };
}

export function compileInsertAst(ast: InsertAst, dialect: SqlDialect = "postgres"): CompiledQuery {
    const params = [...ast.values];
    const cols = ast.columns.map((c) => quoteIdent(c, dialect)).join(", ");
    const ph = ast.columns.map((_, i) => placeholder(dialect, i + 1)).join(", ");
    const sql = `insert into ${quoteIdent(ast.into.name, dialect)} (${cols}) values (${ph})`;
    return {
        sql,
        parameters: params,
        operation: "insert",
        tables: [{ name: ast.into.name }],
        fingerprint: fingerprintSql(sql),
    };
}

export function compileUpdateAst(ast: UpdateAst, dialect: SqlDialect = "postgres"): CompiledQuery {
    const params: unknown[] = [];
    const sets = ast.set.map((s) => {
        params.push(s.value);
        return `${quoteIdent(s.column, dialect)} = ${placeholder(dialect, params.length)}`;
    });
    let sql = `update ${quoteIdent(ast.table.name, dialect)} set ${sets.join(", ")}`;
    sql += compileWhereClause(ast.where, dialect, params);
    return {
        sql,
        parameters: params,
        operation: "update",
        tables: [{ name: ast.table.name }],
        fingerprint: fingerprintSql(sql),
    };
}

export function compileDeleteAst(ast: DeleteAst, dialect: SqlDialect = "postgres"): CompiledQuery {
    const params: unknown[] = [];
    let sql = `delete from ${quoteIdent(ast.from.name, dialect)}`;
    sql += compileWhereClause(ast.where, dialect, params);
    return {
        sql,
        parameters: params,
        operation: "delete",
        tables: [{ name: ast.from.name }],
        fingerprint: fingerprintSql(sql),
    };
}
