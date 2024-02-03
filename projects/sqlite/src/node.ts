/**
 * Node / Bun / Deno file SQLite surface.
 */

export function openSqliteFile(_path: string): Promise<never> {
    return Promise.reject(new Error("@yydb/sqlite/node: file backend not implemented yet"));
}
