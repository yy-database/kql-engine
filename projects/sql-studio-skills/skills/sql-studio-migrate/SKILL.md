---
name: sql-studio-migrate
description: >-
  SQL Studio migration plan/review/apply with plan-hash binding and human approval
  for destructive steps. Use for schema changes through Studio, not Iris/VOS.
---

# sql-studio-migrate

## Planned tools (not live until Server DTO freeze)

- `migration.plan`
- `migration.review`
- `migration.apply`

Risk: **Mutate** / **Destructive** — apply requires:

```text
plan hash + schema fingerprint + capability fingerprint + short-lived grant + human approval
```

Natural-language confirmation is not enough.

## Rules

1. Always plan → review → apply.
2. Input change invalidates the plan.
3. No auto-apply of DROP/TRUNCATE/irreversible migrations.
4. Do not use Iris migration tools here (`iris-skills` / VOS migrations).

## Available today

Docs-only + future `sql orm migrate` CLI stub.
