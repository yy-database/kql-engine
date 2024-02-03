---
name: sql-studio-performance
description: >-
  Read explain/metrics and propose index or SQL changes. Suggestions are separate
  from apply. Use for slow-query investigation in SQL Studio.
---

# sql-studio-performance

## Planned tools

- `query.explain`
- `query.describe`
- metrics hooks (Server — TBD)

## Rules

1. Separate **suggest** from **apply** (apply needs migrate/mutate policy).
2. Base advice on explain evidence, not folklore.
3. Do not auto-create indexes without migration.plan/review/apply.
4. Stay in SQL Studio lane — Iris `plan.explain` is a different product.

## Available today

Docs-only.
