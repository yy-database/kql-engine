---
name: sql-studio-connect
description: >-
  Discover datasources and diagnose connection/TLS/auth/network for SQL Studio
  without echoing secrets. Use when an agent must connect or verify a datasource.
---

# sql-studio-connect

## Path

```text
Agent -> sql-studio-skills -> Studio tool protocol -> sql-studio-server -> driver
```

Agents never hold raw DB credentials or open database TCP themselves.

## Planned tools (not live until Server DTO freeze)

- `datasource.list`
- `datasource.describe`

## Available today

- Product docs / architecture only
- CLI stub: `sql doctor` (not implemented yet — exit non-zero)

## Rules

1. Never print passwords, connection strings with secrets, or TLS private keys.
2. Natural-language “确认” does not grant server policy.
3. Do not invent MCP/HTTP tools that are not in the frozen DTO list.
4. Do not route through `@yydb/iris` or translate VOS ↔ SQL.

Authority: Spark `决策和进度表/sql-studio-architecture.md` §3–§4.
