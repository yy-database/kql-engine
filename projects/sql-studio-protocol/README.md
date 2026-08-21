# `@yydb/sql-studio-protocol`

Experimental public DTOs for SQL Studio clients, servers, drivers, and agent tools.

This package is intentionally public so early SQL Studio clients, Server hosts, drivers, Agent tools, and third-party
experiments can share the same typed message shapes. It is an evolving contract; callers must check
`PROTOCOL_VERSION` before exchanging messages.

The package contains protocol types and lightweight guards only. It does not open database connections, hold
credentials, enforce authorization, or replace
`@yydb/sql-studio-server` policy.
