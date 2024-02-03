/**
 * `@yydb/sql-studio-orm` — schema-first, type-first SQL toolkit.
 *
 * Relational only (PostgreSQL / MySQL / SQLite). Redis & MongoDB stay in their
 * own packages with native query models.
 */

export type {
    ColumnType,
    Generated,
    Insertable,
    JsonArray,
    JsonObject,
    JsonPrimitive,
    JsonValue,
    Selectable,
    SqlType,
    Timestamp,
    Updateable,
} from "./types.ts";

export type {
    DefaultCapabilities,
    MysqlCapabilities,
    PostgresCapabilities,
    SqliteCapabilities,
    SqlCapabilities,
    SqlDialect,
} from "./capabilities.ts";

export type {
    AbortLike,
    CompiledQuery,
    ExecuteOptions,
    QueryChunk,
    QueryOperation,
    QueryResult,
    SqlConnection,
    SqlDriver,
    SqlTransaction,
    StreamOptions,
    TableReference,
    TransactionOptions,
} from "./driver.ts";

export {
    createDatabase,
    sql,
    type CreateDatabaseOptions,
    type DatabaseApi,
    type TransactionApi,
} from "./database.ts";
export { compileFragment, compileFragmentMysql } from "./sql.ts";
export type { Sql, SqlFragment } from "./sql.ts";
export { fingerprintSql } from "./fingerprint.ts";
export {
    compileSelectAst,
    compileInsertAst,
    compileUpdateAst,
    compileDeleteAst,
} from "./query/compile.ts";
export type { SelectAst } from "./query/ast.ts";
export type { InsertAst, UpdateAst, DeleteAst } from "./query/dml-ast.ts";
export type { AnyColumnRef, ComparisonOp, SelectQueryBuilder } from "./query/select.ts";
export type {
    DeleteQueryBuilder,
    InsertQueryBuilder,
    UpdateQueryBuilder,
} from "./query/dml.ts";
export { createMemoryDriver, type CreateMemoryDriverOptions, type MemoryDriver } from "./memory.ts";
export {
    defineRelations,
    type RelationEdge,
    type RelationHelpers,
    type RelationsDefinition,
} from "./relations.ts";
