# `@yydb/sql-studio`

SQL Studio workbench client + **`sql` CLI** (agent-first).

## CLI

```bash
sql --help
sql doctor
sql serve
sql orm pull
sql orm generate
sql orm migrate
```

The registered binary name is **`sql`**, not `sql-studio`.

## Library

```ts
import { createSqlStudio, createSqlCli } from "@yydb/sql-studio";

const studio = createSqlStudio({
  endpoint: "/api/sql-studio",
});
```

Connects only to `@yydb/sql-studio-server` via the Studio Protocol. Never opens PostgreSQL / MySQL / Redis / MongoDB TCP
from the browser.

Agent workflows: see `@yydb/sql-studio-skills`.
