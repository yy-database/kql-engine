/**
 * Dialect capability flags — static constraints, not runtime surprises.
 */

export type SqlCapabilities = {
    readonly returning: boolean;
    readonly upsert: boolean;
    readonly jsonPath: boolean;
    readonly arrays: boolean;
    readonly fullText: boolean;
    readonly forUpdate: boolean;
    readonly savepoints: boolean;
    readonly advisoryLocks: boolean;
};

export type PostgresCapabilities = {
    readonly returning: true;
    readonly upsert: true;
    readonly jsonPath: true;
    readonly arrays: true;
    readonly fullText: true;
    readonly forUpdate: true;
    readonly savepoints: true;
    readonly advisoryLocks: true;
};

export type MysqlCapabilities = {
    readonly returning: false;
    readonly upsert: true;
    readonly jsonPath: true;
    readonly arrays: false;
    readonly fullText: true;
    readonly forUpdate: true;
    readonly savepoints: true;
    readonly advisoryLocks: false;
};

export type SqliteCapabilities = {
    readonly returning: true;
    readonly upsert: true;
    readonly jsonPath: true;
    readonly arrays: false;
    readonly fullText: true;
    readonly forUpdate: false;
    readonly savepoints: true;
    readonly advisoryLocks: false;
};

export type DefaultCapabilities = {
    readonly returning: false;
    readonly upsert: false;
    readonly jsonPath: false;
    readonly arrays: false;
    readonly fullText: false;
    readonly forUpdate: false;
    readonly savepoints: false;
    readonly advisoryLocks: false;
};

export type SqlDialect = "postgres" | "mysql" | "sqlite";
