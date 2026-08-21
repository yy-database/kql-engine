# `@yydb/mysql`

MySQL driver for **SQL Studio** / `@yydb/sql-studio-orm`.

Wire path (`@yydb/mysql/node`): TCP/TLS → HandshakeV10 → auth (`mysql_native_password` / `caching_sha2_password` fast
path) →
`SqlConnection.execute` via COM_QUERY.

```ts
import { createDatabase } from "@yydb/sql-studio-orm";
import { mysql } from "@yydb/mysql";
import { connectTcp } from "@yydb/mysql/node";

const driver = mysql({ url: process.env.MYSQL_URL! });
const db = createDatabase<{ users: { id: number; email: string } }>({ driver });
await db.selectFrom("users").select(["id", "email"]).execute();

// Or low-level:
const client = await connectTcp({ url: "mysql://root:secret@127.0.0.1:3306/app" });
await client.connection.execute({
  sql: "select 1 as n",
  parameters: [],
  operation: "select",
  tables: [],
  fingerprint: "x",
});
```

`caching_sha2_password` full auth (status 0x04) requires TLS; otherwise switch the account to `mysql_native_password` or
enable `tls: true`.

Browsers use `@yydb/sql-studio` → server; they do not speak MySQL wire.
