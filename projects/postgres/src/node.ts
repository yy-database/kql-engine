/**
 * Node TCP / wire surface for PostgreSQL (stub).
 */

export function openPostgresConnection(_url: string): Promise<never> {
    return Promise.reject(new Error("@yydb/postgres/node: wire connection not implemented yet"));
}
