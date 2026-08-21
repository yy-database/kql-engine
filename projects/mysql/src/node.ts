/**
 * `@yydb/mysql/node` — Node TCP/TLS MySQL wire client + authenticated SqlConnection.
 */

import net from "node:net";
import tls from "node:tls";
import { authenticate } from "./authenticate.ts";
import { MysqlWireConnection } from "./connection.ts";
import type { ByteDuplex, ByteHandlers } from "./duplex.ts";
import { createMysqlClient, readHandshake, type MysqlClient, MysqlSession } from "./index.ts";
import { comQuery } from "./query.ts";

export {
    nativePasswordToken,
    cachingSha2Token,
    buildHandshakeResponse41,
} from "./auth.ts";
export { authenticate } from "./authenticate.ts";
export { MysqlWireConnection } from "./connection.ts";

export type MysqlTcpOptions = {
    host?: string;
    port?: number;
    tls?: boolean | tls.ConnectionOptions;
    connectTimeoutMs?: number;
    url?: string;
    user?: string;
    password?: string;
    database?: string;
};

export type ParsedMysqlUrl = {
    host: string;
    port: number;
    user?: string;
    password?: string;
    database?: string;
};

export function parseMysqlUrl(url: string): ParsedMysqlUrl {
    const u = new URL(url);
    if (u.protocol !== "mysql:" && u.protocol !== "mysql2:") {
        throw new Error(`@yydb/mysql/node: expected mysql:// URL, got ${u.protocol}`);
    }
    const database = u.pathname.replace(/^\//, "") || undefined;
    return {
        host: u.hostname || "127.0.0.1",
        port: u.port ? Number(u.port) : 3306,
        user: u.username ? decodeURIComponent(u.username) : undefined,
        password: u.password ? decodeURIComponent(u.password) : undefined,
        database,
    };
}

export class NodeTcpDuplex implements ByteDuplex {
    #socket: net.Socket;
    #open: boolean;
    #handlers: ByteHandlers | null = null;
    #early: Uint8Array[] = [];

    private constructor(socket: net.Socket) {
        this.#socket = socket;
        this.#open = !socket.destroyed;
        socket.on("data", (buf: Buffer) => {
            const bytes = new Uint8Array(buf);
            if (this.#handlers) this.#handlers.onMessage(bytes);
            else this.#early.push(bytes);
        });
        socket.on("close", () => {
            this.#open = false;
            this.#handlers?.onClose?.({ code: 1000, reason: "tcp-close" });
        });
        socket.on("error", (err) => {
            this.#handlers?.onError?.(err);
        });
    }

    setHandlers(handlers: ByteHandlers): void {
        this.#handlers = handlers;
        if (this.#early.length === 0) return;
        const pending = this.#early;
        this.#early = [];
        for (const chunk of pending) handlers.onMessage(chunk);
    }

    get connected(): boolean {
        return this.#open && !this.#socket.destroyed;
    }

    send(bytes: Uint8Array): void {
        if (!this.connected) {
            throw new Error("@yydb/mysql/node: TCP socket is not open");
        }
        const buf = Buffer.from(bytes);
        const ok = this.#socket.write(buf);
        if (!ok) {
            this.#socket.pause();
            this.#socket.once("drain", () => this.#socket.resume());
        }
    }

    close(): void {
        this.#open = false;
        this.#socket.destroy();
    }

    static connect(options: {
        host: string;
        port: number;
        tls?: boolean | tls.ConnectionOptions;
        connectTimeoutMs?: number;
    }): Promise<NodeTcpDuplex> {
        const timeoutMs = options.connectTimeoutMs ?? 10_000;

        return new Promise<NodeTcpDuplex>((resolve, reject) => {
            let settled = false;
            const fail = (err: unknown) => {
                if (settled) return;
                settled = true;
                clearTimeout(timer);
                reject(err);
            };
            const ok = (socket: net.Socket) => {
                if (settled) {
                    socket.destroy();
                    return;
                }
                settled = true;
                clearTimeout(timer);
                resolve(new NodeTcpDuplex(socket));
            };

            const timer = setTimeout(() => {
                fail(new Error(`@yydb/mysql/node: connect timed out after ${timeoutMs}ms`));
            }, timeoutMs);

            const onErr = (err: Error) => fail(err);

            if (options.tls) {
                const tlsOpts: tls.ConnectionOptions =
                    typeof options.tls === "object"
                        ? { ...options.tls, host: options.host, port: options.port }
                        : { host: options.host, port: options.port };
                const s = tls.connect(tlsOpts, () => {
                    s.off("error", onErr);
                    ok(s);
                });
                s.once("error", onErr);
                return;
            }

            const s = net.connect({ host: options.host, port: options.port }, () => {
                s.off("error", onErr);
                ok(s);
            });
            s.once("error", onErr);
        });
    }
}

function resolveTcpOptions(options: MysqlTcpOptions) {
    if (options.url) {
        const parsed = parseMysqlUrl(options.url);
        return {
            host: options.host ?? parsed.host,
            port: options.port ?? parsed.port,
            tls: options.tls,
            connectTimeoutMs: options.connectTimeoutMs,
            user: options.user ?? parsed.user ?? "root",
            password: options.password ?? parsed.password ?? "",
            database: options.database ?? parsed.database,
        };
    }
    if (!options.host) {
        throw new Error("@yydb/mysql/node: host or url is required");
    }
    return {
        host: options.host,
        port: options.port ?? 3306,
        tls: options.tls,
        connectTimeoutMs: options.connectTimeoutMs,
        user: options.user ?? "root",
        password: options.password ?? "",
        database: options.database,
    };
}

export type MysqlTcpClient = MysqlClient & {
    readonly credentials: {
        user: string;
        password: string;
        database?: string;
    };
    readonly connection: MysqlWireConnection;
};

/** Open TCP, complete handshake + auth, return a query-capable client. */
export async function connectTcp(options: MysqlTcpOptions): Promise<MysqlTcpClient> {
    const resolved = resolveTcpOptions(options);
    const duplex = await NodeTcpDuplex.connect({
        host: resolved.host,
        port: resolved.port,
        tls: resolved.tls,
        connectTimeoutMs: resolved.connectTimeoutMs,
    });

    const session = MysqlSession.attach(duplex, (handlers) => {
        duplex.setHandlers(handlers);
    });

    const handshake = await readHandshake(session);
    await authenticate(session, {
        handshake,
        user: resolved.user,
        password: resolved.password,
        database: resolved.database,
        secureTransport: Boolean(resolved.tls),
    });

    const connection = new MysqlWireConnection(session);
    const client = createMysqlClient({
        duplex,
        session,
        handshake,
        authenticated: true,
    });

    return {
        ...client,
        connection,
        credentials: {
            user: resolved.user,
            password: resolved.password,
            database: resolved.database,
        },
    };
}

/** SqlDriver.acquire helper — authenticated wire connection. */
export async function openMysqlConnection(options: MysqlTcpOptions): Promise<MysqlWireConnection> {
    const client = await connectTcp(options);
    return client.connection;
}

export type { MysqlClient } from "./index.ts";
export type { MysqlHandshakeV10 } from "./handshake.ts";
export { comQuery };
