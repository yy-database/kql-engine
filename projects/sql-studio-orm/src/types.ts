/**
 * Column / value type helpers for schema-first Database interfaces.
 * No runtime decorators — compile-time structure only.
 */

declare const generatedBrand: unique symbol;
declare const timestampBrand: unique symbol;

/** Column populated by the database (identity / default); omit on insert. */
export type Generated<T> = T & { readonly [generatedBrand]?: true };

/** Opaque timestamp column (Date or dialect-native). */
export type Timestamp = Date & { readonly [timestampBrand]?: true };

export type JsonPrimitive = string | number | boolean | null;
export type JsonValue = JsonPrimitive | JsonObject | JsonArray;
export type JsonObject = { readonly [key: string]: JsonValue };
export type JsonArray = readonly JsonValue[];

/** Map a table row type to selectable (Generated unwraps). */
export type Selectable<R> = {
    [K in keyof R]: R[K] extends Generated<infer S> ? S : R[K];
};

/** Insertable: Generated columns optional. */
export type Insertable<R> = {
    [K in keyof R as R[K] extends Generated<unknown> ? K : never]?: R[K] extends Generated<infer S> ? S : never;
} & {
    [K in keyof R as R[K] extends Generated<unknown> ? never : K]: R[K];
};

/** Updateable: all columns optional except never Generated-required. */
export type Updateable<R> = {
    [K in keyof R]?: R[K] extends Generated<infer S> ? S : R[K];
};

/** Logical SQL type tags for CompiledQuery metadata. */
export type SqlType =
    | "boolean"
    | "integer"
    | "bigint"
    | "float"
    | "decimal"
    | "text"
    | "bytes"
    | "json"
    | "timestamp"
    | "date"
    | "uuid"
    | "unknown";

export type ColumnType<Select, Insert = Select, Update = Select> = {
    readonly __select: Select;
    readonly __insert: Insert;
    readonly __update: Update;
};
