# SQL Studio

**AI-agent-first** database workbench for the YYDB brand family.

```text
@yydb/sql-studio           workbench + `sql` CLI
@yydb/sql-studio-server    authenticated query host
@yydb/sql-studio-orm       schema-first typed SQL
@yydb/sql-studio-skills    Agent Skills catalog
@yydb/postgres|mysql|sqlite|redis|mongodb
```

```bash
pnpm add @yydb/sql-studio @yydb/sql-studio-server
pnpm add @yydb/sql-studio-orm @yydb/postgres
pnpm add -D @yydb/sql-studio-skills
```

```bash
pnpm install
pnpm run typecheck
pnpm sql --help
```

Copy skills into Cursor:

```bash
cp -r projects/sql-studio-skills/skills/* .cursor/skills/
```

See [`documentation/index.md`](./documentation/index.md) and
[`documentation/agents.md`](./documentation/agents.md).
