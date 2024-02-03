# `@yydb/sql-studio-orm`

Schema-first, type-first SQL toolkit for **SQL Studio**.

- Builder → AST → `CompiledQuery` → `SqlDriver` (no TCP in this package)
- No class entities, decorators, or identity map
- Dialect capabilities are explicit; placeholders follow the driver dialect
- Relational only: PostgreSQL / MySQL / SQLite
- `createMemoryDriver()` for tests and Studio fixtures

```ts
import { createDatabase, type Generated } from "@yydb/sql-studio-orm";
import { postgres } from "@yydb/postgres";

interface Database {
  users: {
    id: Generated<number>;
    email: string;
  };
  posts: {
    id: Generated<number>;
    authorId: number;
    title: string;
  };
}

const db = createDatabase<Database>({
  driver: postgres({ url: process.env.DATABASE_URL! }),
});

const users = await db
  .selectFrom("users")
  .leftJoin("posts", "posts.authorId", "users.id")
  .select(["users.id", "users.email", "posts.title"])
  .execute();
// posts.title is string | null
```

Subpaths: `@yydb/sql-studio-orm/schema`, `/migrations`, `/codegen`.

See [`../../documentation/orm.md`](../../documentation/orm.md).
