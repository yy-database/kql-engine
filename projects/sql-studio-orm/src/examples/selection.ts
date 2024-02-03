/**
 * Compile-time selection / join nullability samples + MemoryDriver smoke.
 */

import { createDatabase, createMemoryDriver, defineRelations, sql, type Generated, type Timestamp } from "../index.ts";

interface Database {
    users: UsersTable;
    posts: PostsTable;
}

interface UsersTable {
    id: Generated<number>;
    email: string;
    displayName: string | null;
    createdAt: Timestamp;
}

interface PostsTable {
    id: Generated<number>;
    authorId: number;
    title: string;
    publishedAt: Timestamp | null;
}

export async function sample() {
    const driver = createMemoryDriver({
        dialect: "postgres",
        onExecute: (query) => ({
            rows: query.operation === "select" ? [{ id: 1, email: "a@b.c" }] : [],
            rowCount: 1,
            columns: ["id", "email"],
        }),
    });

    const db = createDatabase<Database>({ driver });

    const users = await db.selectFrom("users").select(["id", "email"]).where("email", "=", "a@b.c").limit(20).execute();

    const _email: string = users[0]!.email;
    const _id: number = users[0]!.id;

    const joined = await db.selectFrom("users").leftJoin("posts", "posts.authorId", "users.id").select(["users.id", "posts.title"]).execute();

    // leftJoin → posts.title is string | null
    const _title: string | null = joined[0]!.title;

    const compiled = db
        .selectFrom("posts")
        .select(["posts.title"])
        .where("publishedAt", "is not", null)
        .orderBy("publishedAt", "desc")
        .compile();

    const raw = await db.execute(sql`select * from users where email = ${"a@b.c"}`);

    const relations = defineRelations<Database>((r) => ({
        users: {
            posts: r.hasMany("posts", { from: "users.id", to: "posts.authorId" }),
        },
    }));

    await db.transaction().execute(async (tx) => {
        await tx.selectFrom("users").select(["id"]).execute();
    });

    const insertSql = db
        .insertInto("users")
        .values({ email: "n@e.w", displayName: null, createdAt: new Date() as Timestamp })
        .compile();

    const updateSql = db.updateTable("users").set({ displayName: "Ada" }).where("id", "=", 1).compile();

    const deleteSql = db.deleteFrom("users").where("id", "=", 1).compile();

    return {
        users,
        joined,
        email: _email,
        id: _id,
        title: _title,
        sql: compiled.sql,
        fingerprint: compiled.fingerprint,
        raw,
        relations,
        insertSql: insertSql.sql,
        updateSql: updateSql.sql,
        deleteSql: deleteSql.sql,
        history: driver.history,
    };
}
