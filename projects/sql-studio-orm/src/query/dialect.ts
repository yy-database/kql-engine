/**
 * Shared SQL identity / placeholder helpers for dialect compilers.
 */

import type { SqlDialect } from "../capabilities.ts";
import type { AstWhere } from "./ast.ts";

export function quoteIdent(name: string, dialect: SqlDialect): string {
    const q = dialect === "mysql" ? "`" : '"';
    if (name.includes(".")) {
        return name
            .split(".")
            .map((p) => `${q}${p}${q}`)
            .join(".");
    }
    return `${q}${name}${q}`;
}

export function placeholder(dialect: SqlDialect, index: number): string {
    if (dialect === "postgres") return `$${index}`;
    return "?";
}

export function compileWhereClause(where: readonly AstWhere[], dialect: SqlDialect, params: unknown[]): string {
    if (where.length === 0) return "";
    const parts: string[] = [];
    for (const w of where) {
        if (w.kind === "raw") {
            parts.push(w.sql);
            params.push(...w.params);
        } else if ((w.op === "is" || w.op === "is not") && w.value === null) {
            parts.push(`${quoteIdent(w.column, dialect)} ${w.op} null`);
        } else {
            params.push(w.value);
            parts.push(`${quoteIdent(w.column, dialect)} ${w.op} ${placeholder(dialect, params.length)}`);
        }
    }
    return ` where ${parts.join(" and ")}`;
}
