---
name: sql-studio-diagnose
description: >-
  Evidence-based diagnosis from structured Studio/driver errors, connection
  state, and slow-query signals. Use when SQL Studio operations fail.
---

# sql-studio-diagnose

## Planned tools

- `datasource.describe`
- `audit.list`
- `evidence.export`
- (plus error payloads from failed `query.*` / `catalog.*`)

## Rules

1. Prefer structured error codes over guessing.
2. Redact secrets from error text before quoting in chat.
3. Export evidence packs via Server — do not scrape raw logs with credentials.
4. Do not invent root causes without evidence.

## Available today

Docs-only. CLI stub: `sql doctor`.
