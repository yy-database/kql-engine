# Known limitations — 0.0.1 Developer Preview

Install with the `dev` tag: `pnpm add @yydb/sql-studio@dev`.

npm `latest` may still resolve to the empty `0.0.0` placeholder stub. Prefer `@dev`.

## Honest status

| Surface                                           | Status                                                             |
|---------------------------------------------------|--------------------------------------------------------------------|
| Package names / TypeScript entrypoints            | Installable                                                        |
| `@yydb/sql-studio-protocol`                       | Experimental DTOs; breaking changes allowed                        |
| `@yydb/sql-studio-orm`                            | Explore builders + `createMemoryDriver`; not Kysely/Drizzle parity |
| `@yydb/mysql`                                     | Wire / handshake / COM_QUERY trial; not production-ready           |
| `@yydb/postgres` / `sqlite` / `redis` / `mongodb` | Skeleton or unsupported execute paths                              |
| `@yydb/sql-studio-server`                         | In-memory `handleMessage` skeleton; no HTTP/WS/auth/audit host     |
| `@yydb/sql-studio` CLI (`sql`)                    | Command names stable; actions are stubs                            |
| `@yydb/sql-studio-skills`                         | `docs-only` — no live MCP/HTTP tools                               |

## Do not claim

- Browser direct TCP to PostgreSQL / MySQL / Redis
- Production SQL Studio Server
- Completed query / transaction / stream / cancel for arbitrary databases
- Live Agent tools (`tool-live`)
- Migration apply / ACL / audit ready for production

## Exit to 0.1.0

Requires one real database end-to-end chain (CLI → server → protocol → ORM → driver → DB). See workspace release plan.
