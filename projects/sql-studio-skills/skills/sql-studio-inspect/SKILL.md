---
name: sql-studio-inspect
description: >-
  On-demand catalog inspect and search (tables, columns, indexes, constraints,
  small stats). Use before querying an unfamiliar SQL Studio datasource.
---

# sql-studio-inspect

## Planned tools (not live until Server DTO freeze)

- `catalog.inspect`
- `catalog.search`

Risk class: **Observe** (field redaction + scope still apply).

## Rules

1. Prefer targeted inspect over dumping whole catalogs into context.
2. Cap result size; summarize large schemas.
3. Do not run SELECT under this skill — use `sql-studio-query`.
4. Do not claim Iris/VOS catalog semantics here.

## Available today

Docs-only. Wait for `catalog.*` Server tools.
