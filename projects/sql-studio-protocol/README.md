# `@yydb/sql-studio-protocol`

Experimental public DTOs for the SQL Studio `0.0.x` Developer Preview.

This package is intentionally public so early SQL Studio clients, Server hosts,
drivers, Agent tools, and third-party experiments can share the same typed
message shapes. It is **not stable**: breaking changes may land in any `0.0.x`
release, and callers must check `PROTOCOL_VERSION` before exchanging messages.

The package contains protocol types and lightweight guards only. It does not
open database connections, hold credentials, enforce authorization, or replace
`@yydb/sql-studio-server` policy.
