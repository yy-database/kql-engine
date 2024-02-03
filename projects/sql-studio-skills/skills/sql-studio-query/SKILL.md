---
name: sql-studio-query
description: >-
  Bounded SQL via Studio Server: compile/describe/explain then execute/stream/cancel.
  Default read-only with row and time budgets. Use for SQL Studio queries.
---

# sql-studio-query

## Planned tools (not live until Server DTO freeze)

- `query.compile` / `query.describe` / `query.explain` — Observe/Read
- `query.execute` / `query.stream` / `query.cancel` — Read (or Mutate if DML)

## Workflow (when tools exist)

1. compile / describe / explain first
2. show sql + parameters + fingerprint + tables
3. execute only within budget; stream large results
4. cancel on timeout or user abort

## Rules

1. Parameterize all values; no string-concat SQL from model output.
2. Default read-only; Mutate/Destructive need Server policy + approval.
3. Never bypass `@yydb/sql-studio-server`.
4. Never use `@yydb/sql-studio-orm` as a secret Iris execution path.
5. Do not load unbounded result sets into the chat context.

## Available today

- ORM local `compile()` via app code (not Agent tool protocol)
- CLI stubs: `sql orm *` (not implemented)

Do not pretend Server `query.*` tools exist until DTO freeze.
