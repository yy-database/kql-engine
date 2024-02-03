/**
 * Immutable DML AST.
 */

import type { AstTableRef, AstWhere } from "./ast.ts";

export type InsertAst = {
    readonly kind: "insert";
    readonly into: AstTableRef;
    readonly columns: readonly string[];
    readonly values: readonly unknown[];
};

export type UpdateAst = {
    readonly kind: "update";
    readonly table: AstTableRef;
    readonly set: readonly { column: string; value: unknown }[];
    readonly where: readonly AstWhere[];
};

export type DeleteAst = {
    readonly kind: "delete";
    readonly from: AstTableRef;
    readonly where: readonly AstWhere[];
};
