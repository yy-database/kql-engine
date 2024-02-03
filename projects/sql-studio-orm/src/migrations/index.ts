/**
 * Typed migration helpers + SqlDriver-backed runner.
 */

import type { CompiledQuery, SqlDriver } from "../driver.ts";
import { fingerprintSql } from "../fingerprint.ts";

export type MigrationContext = {
    schema: {
        createTable(name: string): CreateTableBuilder;
        dropTable(name: string): DropTableBuilder;
    };
    /** Execute arbitrary DDL/DML via the migration driver. */
    execute(sqlText: string, parameters?: readonly unknown[]): Promise<void>;
};

export type CreateTableBuilder = {
    addColumn(name: string, type: string, build?: (col: MigrationColumnBuilder) => MigrationColumnBuilder): CreateTableBuilder;
    execute(): Promise<void>;
};

export type DropTableBuilder = {
    execute(): Promise<void>;
};

export type MigrationColumnBuilder = {
    primaryKey(): MigrationColumnBuilder;
    generated(): MigrationColumnBuilder;
    notNull(): MigrationColumnBuilder;
    unique(): MigrationColumnBuilder;
};

export type MigrationDefinition = {
    readonly name: string;
    up(db: MigrationContext): Promise<void>;
    down(db: MigrationContext): Promise<void>;
};

export type RunMigrationsOptions = {
    readonly driver: SqlDriver;
    readonly migrations: readonly MigrationDefinition[];
    readonly direction?: "up" | "down";
    /** Journal table name (default sql_studio_migrations). */
    readonly table?: string;
};

export type MigrationRunResult = {
    readonly applied: readonly string[];
};

export function migration(
    name: string,
    body: {
        up(db: MigrationContext): Promise<void>;
        down(db: MigrationContext): Promise<void>;
    },
): MigrationDefinition {
    return { name, up: body.up, down: body.down };
}

type ColState = {
    name: string;
    type: string;
    primaryKey: boolean;
    generated: boolean;
    notNull: boolean;
    unique: boolean;
};

function quoteIdent(name: string, dialect: SqlDriver["dialect"]): string {
    const q = dialect === "mysql" ? "`" : '"';
    return `${q}${name}${q}`;
}

function ddlQuery(sqlText: string): CompiledQuery {
    return {
        sql: sqlText,
        parameters: [],
        operation: "ddl",
        tables: [],
        fingerprint: fingerprintSql(sqlText),
    };
}

function createMigrationContext(driver: SqlDriver): MigrationContext {
    const dialect = driver.dialect;

    async function execSql(sqlText: string, parameters: readonly unknown[] = []): Promise<void> {
        const conn = await driver.acquire();
        await conn.execute({
            sql: sqlText,
            parameters,
            operation: "ddl",
            tables: [],
            fingerprint: fingerprintSql(sqlText),
        });
    }

    return {
        async execute(sqlText, parameters) {
            await execSql(sqlText, parameters ?? []);
        },
        schema: {
            createTable(name: string) {
                const columns: ColState[] = [];
                const builder: CreateTableBuilder = {
                    addColumn(colName, type, build) {
                        const state: ColState = {
                            name: colName,
                            type,
                            primaryKey: false,
                            generated: false,
                            notNull: false,
                            unique: false,
                        };
                        const col: MigrationColumnBuilder = {
                            primaryKey() {
                                state.primaryKey = true;
                                return col;
                            },
                            generated() {
                                state.generated = true;
                                return col;
                            },
                            notNull() {
                                state.notNull = true;
                                return col;
                            },
                            unique() {
                                state.unique = true;
                                return col;
                            },
                        };
                        if (build) build(col);
                        columns.push(state);
                        return builder;
                    },
                    async execute() {
                        const parts = columns.map((c) => {
                            let frag = `${quoteIdent(c.name, dialect)} ${c.type}`;
                            if (c.primaryKey) {
                                if (dialect === "sqlite" && c.generated) {
                                    frag += " primary key autoincrement";
                                } else if (dialect === "mysql" && c.generated) {
                                    frag += " primary key auto_increment";
                                } else if (dialect === "postgres" && c.generated) {
                                    frag = `${quoteIdent(c.name, dialect)} serial primary key`;
                                } else {
                                    frag += " primary key";
                                }
                            } else {
                                if (c.notNull) frag += " not null";
                                if (c.unique) frag += " unique";
                            }
                            return frag;
                        });
                        const sqlText = `create table ${quoteIdent(name, dialect)} (${parts.join(", ")})`;
                        await execSql(sqlText);
                    },
                };
                return builder;
            },
            dropTable(name: string) {
                return {
                    async execute() {
                        await execSql(`drop table ${quoteIdent(name, dialect)}`);
                    },
                };
            },
        },
    };
}

async function ensureJournal(driver: SqlDriver, table: string): Promise<void> {
    const dialect = driver.dialect;
    const q = quoteIdent(table, dialect);
    let sqlText: string;
    if (dialect === "mysql") {
        sqlText = `create table if not exists ${q} (name varchar(255) primary key, applied_at timestamp not null)`;
    } else if (dialect === "sqlite") {
        sqlText = `create table if not exists ${q} (name text primary key, applied_at text not null)`;
    } else {
        sqlText = `create table if not exists ${q} (name text primary key, applied_at timestamptz not null default now())`;
    }
    const conn = await driver.acquire();
    await conn.execute(ddlQuery(sqlText));
}

async function listApplied(driver: SqlDriver, table: string): Promise<Set<string>> {
    const conn = await driver.acquire();
    const result = await conn.execute<{ name: string }>({
        sql: `select name from ${quoteIdent(table, driver.dialect)} order by name`,
        parameters: [],
        operation: "select",
        tables: [{ name: table }],
        fingerprint: fingerprintSql(`select name from ${table}`),
    });
    return new Set(result.rows.map((r) => String((r as { name: string }).name)));
}

async function markApplied(driver: SqlDriver, table: string, name: string): Promise<void> {
    const dialect = driver.dialect;
    const conn = await driver.acquire();
    if (dialect === "postgres") {
        await conn.execute({
            sql: `insert into ${quoteIdent(table, dialect)} (name, applied_at) values ($1, now())`,
            parameters: [name],
            operation: "insert",
            tables: [{ name: table }],
            fingerprint: fingerprintSql("mig-insert"),
        });
    } else {
        const now = new Date().toISOString();
        await conn.execute({
            sql: `insert into ${quoteIdent(table, dialect)} (name, applied_at) values (?, ?)`,
            parameters: [name, now],
            operation: "insert",
            tables: [{ name: table }],
            fingerprint: fingerprintSql("mig-insert"),
        });
    }
}

async function unmarkApplied(driver: SqlDriver, table: string, name: string): Promise<void> {
    const dialect = driver.dialect;
    const conn = await driver.acquire();
    const ph = dialect === "postgres" ? "$1" : "?";
    await conn.execute({
        sql: `delete from ${quoteIdent(table, dialect)} where name = ${ph}`,
        parameters: [name],
        operation: "delete",
        tables: [{ name: table }],
        fingerprint: fingerprintSql("mig-delete"),
    });
}

/** Apply migrations in order (up) or reverse (down) against a SqlDriver. */
export async function runMigrations(options: RunMigrationsOptions): Promise<MigrationRunResult> {
    const direction = options.direction ?? "up";
    const table = options.table ?? "sql_studio_migrations";
    const { driver, migrations } = options;
    const ctx = createMigrationContext(driver);

    await ensureJournal(driver, table);
    const applied = await listApplied(driver, table);
    const done: string[] = [];

    if (direction === "up") {
        for (const m of migrations) {
            if (applied.has(m.name)) continue;
            await m.up(ctx);
            await markApplied(driver, table, m.name);
            done.push(m.name);
        }
    } else {
        for (const m of [...migrations].reverse()) {
            if (!applied.has(m.name)) continue;
            await m.down(ctx);
            await unmarkApplied(driver, table, m.name);
            done.push(m.name);
        }
    }

    return { applied: done };
}
