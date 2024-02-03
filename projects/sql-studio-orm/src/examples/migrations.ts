/**
 * Migration runner smoke against MemoryDriver.
 */

import { createMemoryDriver } from "../memory.ts";
import { migration, runMigrations } from "../migrations/index.ts";

export async function sampleMigrations() {
    const appliedNames = new Set<string>();
    const driver = createMemoryDriver({
        dialect: "sqlite",
        onExecute: (query) => {
            if (query.sql.startsWith("select name from")) {
                return {
                    rows: [...appliedNames].map((name) => ({ name })),
                    rowCount: appliedNames.size,
                    columns: ["name"],
                };
            }
            if (query.sql.startsWith("insert into") && query.parameters[0]) {
                appliedNames.add(String(query.parameters[0]));
            }
            if (query.sql.startsWith("delete from") && query.parameters[0]) {
                appliedNames.delete(String(query.parameters[0]));
            }
            return { rows: [], rowCount: 0, columns: [] };
        },
    });

    const m1 = migration("20260821_create_users", {
        async up(db) {
            await db.schema
                .createTable("users")
                .addColumn("id", "integer", (c) => c.primaryKey().generated())
                .addColumn("email", "varchar(320)", (c) => c.notNull().unique())
                .execute();
        },
        async down(db) {
            await db.schema.dropTable("users").execute();
        },
    });

    const first = await runMigrations({ driver, migrations: [m1], direction: "up" });
    const second = await runMigrations({ driver, migrations: [m1], direction: "up" });
    return { first, second, history: driver.history };
}
