# 🧭 SQL Studio

SQL Studio is an agent-first database workbench for the YYDB ecosystem. It gives an AI agent a disciplined path from
datasource discovery to catalog inspection, query compilation, explain evidence, bounded execution, and structured
failure reporting. The browser client talks to a SQL Studio Server endpoint; it does not open PostgreSQL, MySQL, SQLite,
Redis, or MongoDB sockets.

## Install The Agent Workflow

Install the official skills with the standard Skills CLI:

```bash
npx skills add @yydb/sql-studio-skills
```

SQL Studio is not assumed to be part of the agent's general knowledge. Name the
installed skills and tell the agent how to discover capabilities in the user's
application and agent environment:

```text
Use the installed SQL Studio skills. Start with `sql-studio-connect`, then use
`sql-studio-inspect` and `sql-studio-query` only when their prerequisites are
available. Read the installed skill instructions first. Then check the current
application's declared dependencies and scripts for `@yydb/sql-studio*`, check
whether the `sql` CLI is available, and use only a SQL Studio Server endpoint
or tool connection explicitly configured by the application or provided by the
user. List the concrete capabilities you can actually call before doing any
database work.

Do not assume that the SQL Studio source repository is present. If the
application has no SQL Studio package, CLI, Server endpoint, or connected tool,
stop and report the exact missing prerequisite and the package or configuration
the user needs. Do not search for an unrelated product, connect directly to a
database, or invent a tool. If an integration is available, inspect the
registered datasource, identify the relevant tables and columns, compile a
read-only query, show its plan and bound parameters, execute it with a 20-row
limit, and summarize the result. Do not mutate data, expose secrets, or bypass
Server policy. Preserve structured errors and unsupported capability reports.
```

The skills describe the workflow and its safety contract. An agent must verify that a requested live capability exists
before calling it; documentation must never be treated as proof that a driver or transport is implemented.

## What This Repository Contains

| Package                     | Responsibility                                       | Boundary                                             |
|-----------------------------|------------------------------------------------------|------------------------------------------------------|
| `@yydb/sql-studio`          | Browser client and `sql` CLI                         | Protocol endpoint only; no database TCP              |
| `@yydb/sql-studio-server`   | Protocol dispatch, datasource registry, cancellation | Server process; policy and credentials stay here     |
| `@yydb/sql-studio-protocol` | Versioned messages, DTOs, guards                     | Experimental wire contract; no transport or I/O      |
| `@yydb/sql-studio-orm`      | Typed relational builders and compilation            | Produces `CompiledQuery`; does not connect           |
| `@yydb/sql-studio-skills`   | Agent Skills and workflow catalog                    | Skills are documentation-led until tools are wired   |
| `@yydb/mysql`               | MySQL driver, Node wire implementation               | Trusted Node/server boundary                         |
| `@yydb/postgres`            | PostgreSQL registration and ORM driver shape         | Wire execution remains incomplete                    |
| `@yydb/sqlite`              | Node and WASM SQLite entry points                    | Backend execution remains incomplete                 |
| `@yydb/redis`               | Redis datasource registration                        | Browser and live command surfaces are future work    |
| `@yydb/mongodb`             | MongoDB datasource registration                      | Browser and live collection surfaces are future work |

## Architecture

```text
Agent -> Skills -> Studio Protocol -> SQL Studio Server
                                      -> policy / limits / cancellation
                                      -> registered driver -> database
```

The intended browser integration is:

```ts
import {createSqlStudio} from "@yydb/sql-studio";

const studio = createSqlStudio({endpoint: "/api/sql-studio"});
const datasources = await studio.listDatasources();
```

Register drivers on the server, never in browser code. Database URLs, passwords, native modules, and Node socket imports
must remain outside the client bundle and outside skill instructions.

## ORM Compilation Model

The ORM deliberately separates authoring from execution:

```text
typed builder -> immutable AST -> dialect compiler -> CompiledQuery -> SqlDriver
```

`CompiledQuery` carries SQL, bound parameters, dialect, operation metadata, and a fingerprint suitable for inspection
and evidence. The package is relational and type-first: no class entities, decorators, identity map, implicit
connection, or browser driver. `createMemoryDriver()` is the deterministic fixture path.

## CLI

The command is named `sql`:

```bash
sql --help
sql doctor
sql serve
sql orm pull
sql orm generate
sql orm migrate
```

Command names are stable preview entry points. An unfinished command must return an explicit, machine-readable
unsupported error; it must not claim that a query, migration, introspection, or server was completed.

## Agent Safety Contract

- Restrict every operation to the named datasource and requested scope.
- Prefer read-only inspection and bounded result sets.
- Preserve query text, parameters, dialect, fingerprint, timing, and error code as evidence when available.
- Require an explicit plan and approval before mutation or destructive migration.
- Never ask the agent to print credentials or place secrets in prompts, skills, URLs, or browser code.
- Reject unknown protocol versions before dispatch.
- Do not invent MCP methods, HTTP routes, protocol messages, or driver capabilities.
- Treat unsupported errors as final capability information, not as permission to bypass the server.

## Installation For Applications

```bash
pnpm add @yydb/sql-studio@latest @yydb/sql-studio-server@latest
pnpm add @yydb/sql-studio-orm@latest @yydb/mysql@latest
```

Pin package versions in applications and review the changelog before upgrading. The skills package is installed through
`npx skills add`, because skills are agent assets rather than an application runtime dependency.

## Verification

```bash
pnpm install --frozen-lockfile
pnpm run typecheck
pnpm run fmt:check
pnpm sql --help
```

These checks validate package shape and command wiring. They do not prove production authentication, authorization,
audit storage, database availability, browser deployment, or a live agent tool integration.

## Release And Compatibility

The published surface is intended for package-shape validation, protocol experiments, fixture-driven ORM work, and agent
workflow design. Packages may contain explicit stubs and may change their contract; consumers must check protocol
versions and handle structured unsupported errors. Do not describe the repository as production-ready unless a real
database path, secured server boundary, driver and ORM conformance evidence, and reproducible agent-read evidence have
been established.
