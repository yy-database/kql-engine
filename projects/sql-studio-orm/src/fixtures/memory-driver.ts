/**
 * Minimal MemoryDriver conformance fixture (0.0.1).
 * Canonical runner is `sql orm compile` in @yydb/sql-studio.
 */
export const MEMORY_DRIVER_FIXTURE = {
    id: "memory-driver-compile-execute",
    version: "0.0.1",
    asserts: [
        "compile() returns sql + parameters + fingerprint",
        "fingerprint matches fingerprintSql(sql)",
        "execute() records CompiledQuery in driver.history",
        "onExecute can return bounded rows without network I/O",
    ],
};
