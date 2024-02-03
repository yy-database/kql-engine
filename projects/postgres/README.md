# `@yydb/postgres`

PostgreSQL `SqlDriver` for `@yydb/sql-studio-orm`, and Studio registration when
`id` is set.

```ts
import { postgres } from "@yydb/postgres";
import { createDatabase } from "@yydb/sql-studio-orm";
import { createSqlStudioServer } from "@yydb/sql-studio-server";

const driver = postgres({ url: process.env.DATABASE_URL!, pool: { max: 10 } });
const db = createDatabase<{ users: { id: number } }>({ driver });

createSqlStudioServer({
  databases: [postgres({ id: "production", url: process.env.DATABASE_URL! })],
});
```
