# Tool protocol (planned)

Authority: Spark `决策和进度表/sql-studio-architecture.md` §3.3–§4.

Until Server DTOs freeze, **no MCP/HTTP tool in this list is live**.
Skills must not pretend otherwise.

```text
datasource.list / datasource.describe
catalog.inspect / catalog.search
query.compile / query.describe / query.explain
query.execute / query.stream / query.cancel
migration.plan / migration.review / migration.apply
audit.list / evidence.export
```

Risk classes: Observe | Read | Mutate | Destructive | Administrative.

Destructive/Administrative require plan hash, fingerprints, short-lived grant,
human approval, and audit — not chat “确认执行”.
