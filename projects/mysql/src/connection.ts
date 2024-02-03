/**
 * SqlConnection backed by an authenticated MysqlSession (COM_QUERY).
 */

import type {
    CompiledQuery,
    ExecuteOptions,
    QueryChunk,
    QueryResult,
    SqlConnection,
    SqlTransaction,
    StreamOptions,
    TransactionOptions,
} from "@yydb/sql-studio-orm";
import { comQuery } from "./query.ts";
import type { MysqlSession } from "./session.ts";

export class MysqlWireConnection implements SqlConnection {
    readonly session: MysqlSession;

    constructor(session: MysqlSession) {
        this.session = session;
    }

    async execute<R = Record<string, unknown>>(query: CompiledQuery, options?: ExecuteOptions): Promise<QueryResult<R>> {
        if (options?.signal?.aborted) {
            throw new Error("@yydb/mysql: query aborted");
        }
        const outcome = await comQuery(this.session, query.sql, query.parameters);
        if (outcome.kind === "ok") {
            return {
                rows: [] as R[],
                rowCount: outcome.affectedRows,
                columns: [],
            };
        }
        return {
            rows: outcome.rows as R[],
            rowCount: outcome.rows.length,
            columns: outcome.columns,
        };
    }

    async *stream<R = Record<string, unknown>>(query: CompiledQuery, options?: StreamOptions): AsyncIterable<QueryChunk<R>> {
        const result = await this.execute<R>(query, options);
        yield { rows: result.rows, done: true };
    }

    async begin(options?: TransactionOptions): Promise<SqlTransaction> {
        const isolation = options?.isolationLevel;
        if (isolation) {
            await comQuery(this.session, `SET TRANSACTION ISOLATION LEVEL ${isolation.toUpperCase()}`);
        }
        if (options?.readOnly) {
            await comQuery(this.session, "SET TRANSACTION READ ONLY");
        }
        await comQuery(this.session, "BEGIN");
        const self = this;
        let finished = false;
        return {
            execute: (q, o) => {
                if (finished) {
                    return Promise.reject(new Error("@yydb/mysql: transaction finished"));
                }
                return self.execute(q, o);
            },
            stream: (q, o) => {
                if (finished) throw new Error("@yydb/mysql: transaction finished");
                return self.stream(q, o);
            },
            begin: () => Promise.reject(new Error("@yydb/mysql: nested begin not supported yet")),
            async commit() {
                await comQuery(self.session, "COMMIT");
                finished = true;
            },
            async rollback() {
                await comQuery(self.session, "ROLLBACK");
                finished = true;
            },
        };
    }
}
