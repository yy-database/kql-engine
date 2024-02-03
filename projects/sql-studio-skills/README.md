# `@yydb/sql-studio-skills`

Official Agent Skills for **SQL Studio** (architecture §3).

```text
Agent -> skills -> structured tool protocol -> sql-studio-server -> policy -> driver
```

## First-batch skills

| Skill | Role | Delivery |
| --- | --- | --- |
| `sql-studio-connect` | datasource discover / connect diagnose | docs-only |
| `sql-studio-inspect` | catalog inspect/search | docs-only |
| `sql-studio-query` | compile → explain → bounded execute | docs-only |
| `sql-studio-migrate` | plan/review/apply | docs-only |
| `sql-studio-diagnose` | evidence-based failure diagnosis | docs-only |
| `sql-studio-performance` | explain/metrics suggestions | docs-only |

**Gate:** Server connection/catalog/query/policy/audit DTOs must freeze before
any skill is marked `tool-live`. Skills must not invent tools.

## Parallel to Iris

`@yydb/iris-skills` speaks VOS/Iris only. No VOS↔SQL translation either way.

## Install

```bash
cp -r skills/* .cursor/skills/
```

See `skills/references/tool-protocol.md`.
