/**
 * In-memory SqlDriver for tests, Studio fixtures, and ORM demos.
 * Does not speak any database wire protocol.
 */

import type {
    CompiledQuery,
    ExecuteOptions,
    QueryChunk,
    QueryResult,
    SqlConnection,
    SqlDriver,
    SqlTransaction,
    StreamOptions,
    TransactionOptions,
} from "./driver.ts";

export type MemoryExecuteHandler = (query: CompiledQuery, options?: ExecuteOptions) => QueryResult<unknown> | Promise<QueryResult<unknown>>;

export type CreateMemoryDriverOptions = {
    readonly dialect?: SqlDriver["dialect"];
    /** Override execute; default returns empty rows and records the query. */
    readonly onExecute?: MemoryExecuteHandler;
};

export type MemoryDriver = SqlDriver & {
    readonly history: readonly CompiledQuery[];
    clearHistory(): void;
};

class MemoryConnection implements SqlConnection {
    #onExecute: MemoryExecuteHandler;
    #history: CompiledQuery[];

    constructor(onExecute: MemoryExecuteHandler, history: CompiledQuery[]) {
        this.#onExecute = onExecute;
        this.#history = history;
    }

    async execute<R>(query: CompiledQuery, options?: ExecuteOptions): Promise<QueryResult<R>> {
        if (options?.signal?.aborted) {
            throw new Error("@yydb/sql-studio-orm: query aborted");
        }
        this.#history.push(query);
        const result = await this.#onExecute(query, options);
        return result as QueryResult<R>;
    }

    async *stream<R>(query: CompiledQuery, options?: StreamOptions): AsyncIterable<QueryChunk<R>> {
        const result = await this.execute<R>(query, options);
        yield { rows: result.rows, done: true };
    }

    async begin(_options?: TransactionOptions): Promise<SqlTransaction> {
        const self = this;
        let finished = false;
        return {
            execute: (q, o) => {
                if (finished) {
                    return Promise.reject(new Error("@yydb/sql-studio-orm: transaction finished"));
                }
                return self.execute(q, o);
            },
            stream: (q, o) => {
                if (finished) {
                    throw new Error("@yydb/sql-studio-orm: transaction finished");
                }
                return self.stream(q, o);
            },
            begin: () => Promise.reject(new Error("@yydb/sql-studio-orm: nested transactions pending")),
            async commit() {
                finished = true;
            },
            async rollback() {
                finished = true;
            },
        };
    }
}

export function createMemoryDriver(options: CreateMemoryDriverOptions = {}): MemoryDriver {
    const history: CompiledQuery[] = [];
    const onExecute: MemoryExecuteHandler =
        options.onExecute ??
        (() => ({
            rows: [],
            rowCount: 0,
            columns: [],
        }));

    return {
        dialect: options.dialect ?? "postgres",
        history,
        clearHistory() {
            history.length = 0;
        },
        async acquire(): Promise<SqlConnection> {
            return new MemoryConnection(onExecute, history);
        },
        async destroy(): Promise<void> {
            history.length = 0;
        },
    };
}
