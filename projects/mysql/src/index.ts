/**
 * `@yydb/mysql` — MySQL SqlDriver + Studio registration + wire helpers.
 */

import type { SqlConnection, SqlDriver } from "@yydb/sql-studio-orm";
import type { StudioDriverRegistration } from "@yydb/sql-studio-protocol";
import type { ByteDuplex } from "./duplex.ts";
import { parseHandshakeV10, type MysqlHandshakeV10 } from "./handshake.ts";
import { comQuery } from "./query.ts";
import { MysqlSession } from "./session.ts";

export type { MysqlHandshakeV10 } from "./handshake.ts";
export type { MysqlPacket } from "./packet.ts";
export { encodePacket, PacketReader } from "./packet.ts";
export { parseHandshakeV10 } from "./handshake.ts";
export { MysqlSession } from "./session.ts";
export type { ByteDuplex, ByteHandlers } from "./duplex.ts";
export { comQuery, bindMysqlParameters } from "./query.ts";
// Auth / TCP / wire connection live on `@yydb/mysql/node` (node:crypto + node:net).

export const DRIVER_ID = "mysql" as const;
export const DRIVER_STATUS = "experimental" as const;

export type MysqlPoolOptions = {
    min?: number;
    max?: number;
};

export type MysqlDriverOptions = {
    url: string;
    pool?: MysqlPoolOptions;
    tls?: boolean;
};

export type MysqlStudioOptions = MysqlDriverOptions & {
    id: string;
};

export type MysqlClient = {
    readonly driverId: typeof DRIVER_ID;
    readonly duplex: ByteDuplex;
    readonly session: MysqlSession;
    readonly handshake: MysqlHandshakeV10 | null;
    readonly authenticated: boolean;
    query(sql: string, params?: unknown[]): Promise<unknown>;
    close(): void;
};

export type MysqlClientInit = {
    duplex: ByteDuplex;
    session: MysqlSession;
    handshake?: MysqlHandshakeV10 | null;
    authenticated?: boolean;
};

export function createMysqlClient(init: MysqlClientInit): MysqlClient {
    const handshake = init.handshake ?? null;
    const authenticated = init.authenticated ?? false;
    return {
        driverId: DRIVER_ID,
        duplex: init.duplex,
        session: init.session,
        handshake,
        authenticated,
        async query(sqlText: string, params?: unknown[]): Promise<unknown> {
            if (!authenticated) {
                throw new Error("@yydb/mysql: not authenticated — call authenticate / connectTcp first");
            }
            return comQuery(init.session, sqlText, params ?? []);
        },
        close(): void {
            init.session.close();
        },
    };
}

export async function readHandshake(session: MysqlSession): Promise<MysqlHandshakeV10> {
    const packet = await session.readPacket();
    session.setSequenceId((packet.sequenceId + 1) & 0xff);
    return parseHandshakeV10(packet.payload);
}

class MysqlDriver implements SqlDriver {
    readonly dialect = "mysql" as const;
    readonly url: string;
    readonly pool?: MysqlPoolOptions;
    readonly tls?: boolean;
    #live: SqlConnection | null = null;

    constructor(options: MysqlDriverOptions) {
        this.url = options.url;
        this.pool = options.pool;
        this.tls = options.tls;
    }

    async acquire(): Promise<SqlConnection> {
        if (this.#live) return this.#live;
        throw new Error("@yydb/mysql: TCP acquire is Node-only — import openMysqlConnection from @yydb/mysql/node");
    }

    async destroy(): Promise<void> {
        if (this.#live && "session" in this.#live) {
            (this.#live as { session: MysqlSession }).session.close();
        }
        this.#live = null;
    }
}

export function mysql(options: MysqlDriverOptions): SqlDriver;
export function mysql(options: MysqlStudioOptions): StudioDriverRegistration;
export function mysql(options: MysqlDriverOptions | MysqlStudioOptions): SqlDriver | StudioDriverRegistration {
    if ("id" in options && options.id) {
        return {
            id: options.id,
            kind: "mysql",
            driver: new MysqlDriver(options),
        };
    }
    return new MysqlDriver(options);
}
