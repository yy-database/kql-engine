/**
 * Typed select builder — selection / join nullability at compile time.
 */

import type { SqlCapabilities, SqlDialect } from "../capabilities.ts";
import type { CompiledQuery, QueryResult, SqlDriver } from "../driver.ts";
import type { Selectable } from "../types.ts";
import type { AstJoin, AstOrder, AstWhere, SelectAst } from "./ast.ts";
import { compileSelectAst } from "./compile.ts";

export type ComparisonOp = "=" | "!=" | ">" | ">=" | "<" | "<=" | "is" | "is not";

type TableOf<DB, TB extends keyof DB> = DB[TB] extends object ? DB[TB] : never;
type RowOf<DB, TB extends keyof DB> = Selectable<TableOf<DB, TB>>;

/** `users.id` or bare `id` (bare = from-table only). */
export type AnyColumnRef<DB, From extends keyof DB & string, Tables extends keyof DB & string> =
    | {
          [T in Tables]: {
              [C in keyof RowOf<DB, T> & string]: `${T & string}.${C}`;
          }[keyof RowOf<DB, T> & string];
      }[Tables]
    | (keyof RowOf<DB, From> & string);

type OutputKey<C extends string> = C extends `${string}.${infer Col}` ? Col : C;

type CellType<DB, From extends keyof DB & string, LeftJoined extends keyof DB & string, C extends string> = C extends `${infer T}.${infer Col}`
    ? T extends keyof DB
        ? Col extends keyof RowOf<DB, T>
            ? T extends LeftJoined
                ? RowOf<DB, T>[Col] | null
                : RowOf<DB, T>[Col]
            : never
        : never
    : C extends keyof RowOf<DB, From>
      ? RowOf<DB, From>[C]
      : never;

type SelectionOut<DB, From extends keyof DB & string, LeftJoined extends keyof DB & string, Cols extends readonly string[]> = {
    [C in Cols[number] as OutputKey<C & string>]: CellType<DB, From, LeftJoined, C & string>;
};

export type SelectQueryBuilder<
    DB,
    From extends keyof DB & string,
    Tables extends keyof DB & string,
    LeftJoined extends keyof DB & string,
    O,
    Caps extends SqlCapabilities,
> = {
    select<const Cols extends readonly AnyColumnRef<DB, From, Tables>[]>(
        columns: Cols,
    ): SelectQueryBuilder<DB, From, Tables, LeftJoined, SelectionOut<DB, From, LeftJoined, Cols>, Caps>;

    innerJoin<T2 extends keyof DB & string>(
        table: T2,
        left: AnyColumnRef<DB, From, Tables | T2>,
        right: AnyColumnRef<DB, From, Tables | T2>,
    ): SelectQueryBuilder<DB, From, Tables | T2, LeftJoined, O, Caps>;

    leftJoin<T2 extends keyof DB & string>(
        table: T2,
        left: AnyColumnRef<DB, From, Tables | T2>,
        right: AnyColumnRef<DB, From, Tables | T2>,
    ): SelectQueryBuilder<DB, From, Tables | T2, LeftJoined | T2, O, Caps>;

    where(column: AnyColumnRef<DB, From, Tables>, op: ComparisonOp, value: unknown): SelectQueryBuilder<DB, From, Tables, LeftJoined, O, Caps>;

    orderBy(column: AnyColumnRef<DB, From, Tables>, direction?: "asc" | "desc"): SelectQueryBuilder<DB, From, Tables, LeftJoined, O, Caps>;

    limit(n: number): SelectQueryBuilder<DB, From, Tables, LeftJoined, O, Caps>;

    offset(n: number): SelectQueryBuilder<DB, From, Tables, LeftJoined, O, Caps>;

    /** Compile without executing — Studio preview / audit uses this. */
    compile(): CompiledQuery;

    execute(): Promise<O[]>;

    executeTakeFirst(): Promise<O | undefined>;

    executeTakeFirstOrThrow(): Promise<O>;
};

type BuilderState = {
    from: string;
    joins: AstJoin[];
    columns: string[];
    where: AstWhere[];
    orderBy: AstOrder[];
    limit?: number;
    offset?: number;
};

export function createSelectBuilder<DB, From extends keyof DB & string, Caps extends SqlCapabilities>(
    driver: SqlDriver,
    table: From,
    _caps: Caps,
): SelectQueryBuilder<DB, From, From, never, RowOf<DB, From>, Caps> {
    const state: BuilderState = {
        from: table,
        joins: [],
        columns: [],
        where: [],
        orderBy: [],
    };

    const dialect: SqlDialect = driver.dialect;

    const api = {
        select(columns: readonly string[]) {
            state.columns = [...columns];
            return api;
        },
        innerJoin(tableName: string, left: string, right: string) {
            state.joins.push({
                kind: "inner",
                table: { kind: "table", name: tableName },
                onLeft: left,
                onRight: right,
            });
            return api;
        },
        leftJoin(tableName: string, left: string, right: string) {
            state.joins.push({
                kind: "left",
                table: { kind: "table", name: tableName },
                onLeft: left,
                onRight: right,
            });
            return api;
        },
        where(column: string, op: ComparisonOp, value: unknown) {
            state.where.push({ kind: "cmp", column, op, value });
            return api;
        },
        orderBy(column: string, direction: "asc" | "desc" = "asc") {
            state.orderBy.push({ column, direction });
            return api;
        },
        limit(n: number) {
            state.limit = n;
            return api;
        },
        offset(n: number) {
            state.offset = n;
            return api;
        },
        compile(): CompiledQuery {
            const ast: SelectAst = {
                kind: "select",
                from: { kind: "table", name: state.from },
                joins: state.joins,
                columns: state.columns,
                where: state.where,
                orderBy: state.orderBy,
                limit: state.limit,
                offset: state.offset,
            };
            return compileSelectAst(ast, dialect);
        },
        async execute(): Promise<unknown[]> {
            const compiled = api.compile();
            const conn = await driver.acquire();
            const result: QueryResult<unknown> = await conn.execute(compiled);
            return result.rows;
        },
        async executeTakeFirst(): Promise<unknown | undefined> {
            const rows = await api.execute();
            return rows[0];
        },
        async executeTakeFirstOrThrow(): Promise<unknown> {
            const row = await api.executeTakeFirst();
            if (row === undefined) {
                throw new Error("@yydb/sql-studio-orm: no rows returned");
            }
            return row;
        },
    };

    return api as SelectQueryBuilder<DB, From, From, never, RowOf<DB, From>, Caps>;
}
