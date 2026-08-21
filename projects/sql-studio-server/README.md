# `@yydb/sql-studio-server`

Server host for **SQL Studio**: protocol dispatch, connection registry, query execution via `SqlDriver`, cancel, and
(later) auth/audit.

```ts
import { createSqlStudioServer } from "@yydb/sql-studio-server";
import { postgres } from "@yydb/postgres";
import { mysql } from "@yydb/mysql";

export const server = createSqlStudioServer({
  databases: [
    postgres({ id: "production", url: process.env.POSTGRES_URL! }),
    mysql({ id: "legacy", url: process.env.MYSQL_URL! }),
  ],
});

// HTTP adapter later: POST body → server.handleMessage(msg)
```
