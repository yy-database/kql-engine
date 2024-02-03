/**
 * Browser / WASM SQLite surface (sql.js or native WASM later).
 */

export function openSqliteWasm(_options: { wasmUrl?: string }): Promise<never> {
    return Promise.reject(new Error("@yydb/sqlite/wasm: not implemented yet"));
}
