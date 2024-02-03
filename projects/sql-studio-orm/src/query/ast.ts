/**
 * Immutable query AST (builder → compile → driver).
 */

export type AstTableRef = {
    readonly kind: "table";
    readonly name: string;
    readonly alias?: string;
};

export type AstJoin = {
    readonly kind: "inner" | "left";
    readonly table: AstTableRef;
    readonly onLeft: string;
    readonly onRight: string;
};

export type AstOrder = {
    readonly column: string;
    readonly direction: "asc" | "desc";
};

export type AstWhere =
    | {
          readonly kind: "cmp";
          readonly column: string;
          readonly op: "=" | "!=" | ">" | ">=" | "<" | "<=" | "is" | "is not";
          readonly value: unknown;
      }
    | { readonly kind: "raw"; readonly sql: string; readonly params: readonly unknown[] };

export type SelectAst = {
    readonly kind: "select";
    readonly from: AstTableRef;
    readonly joins: readonly AstJoin[];
    readonly columns: readonly string[];
    readonly where: readonly AstWhere[];
    readonly orderBy: readonly AstOrder[];
    readonly limit?: number;
    readonly offset?: number;
};
