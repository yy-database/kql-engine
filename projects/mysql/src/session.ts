/**
 * Byte → MySQL packet session over a ByteDuplex.
 */

import type { ByteDuplex, ByteHandlers } from "./duplex.ts";
import { encodePacket, PacketReader, type MysqlPacket } from "./packet.ts";

export class MysqlSession {
    readonly duplex: ByteDuplex;
    #reader = new PacketReader();
    #pending: Array<{
        resolve: (p: MysqlPacket) => void;
        reject: (e: unknown) => void;
    }> = [];
    #queue: MysqlPacket[] = [];
    #closed = false;
    #sequenceId = 0;

    private constructor(duplex: ByteDuplex) {
        this.duplex = duplex;
    }

    static attach(duplex: ByteDuplex, bindHandlers: (handlers: ByteHandlers) => void): MysqlSession {
        const session = new MysqlSession(duplex);
        bindHandlers({
            onMessage: (bytes) => session.#onBytes(bytes),
            onClose: () => session.#failAll(new Error("@yydb/mysql: duplex closed")),
            onError: (err) => session.#failAll(err),
        });
        return session;
    }

    get sequenceId(): number {
        return this.#sequenceId;
    }

    setSequenceId(id: number): void {
        this.#sequenceId = id & 0xff;
    }

    readPacket(): Promise<MysqlPacket> {
        if (this.#closed) {
            return Promise.reject(new Error("@yydb/mysql: session closed"));
        }
        const queued = this.#queue.shift();
        if (queued) return Promise.resolve(queued);
        return new Promise((resolve, reject) => {
            this.#pending.push({ resolve, reject });
        });
    }

    sendPayload(payload: Uint8Array, sequenceId = this.#sequenceId): void {
        this.duplex.send(encodePacket(sequenceId, payload));
        this.#sequenceId = (sequenceId + 1) & 0xff;
    }

    close(): void {
        this.#closed = true;
        this.duplex.close();
        this.#failAll(new Error("@yydb/mysql: session closed"));
    }

    #onBytes(bytes: Uint8Array): void {
        this.#reader.push(bytes);
        for (;;) {
            const packet = this.#reader.tryRead();
            if (!packet) break;
            const waiter = this.#pending.shift();
            if (waiter) waiter.resolve(packet);
            else this.#queue.push(packet);
        }
    }

    #failAll(err: unknown): void {
        this.#closed = true;
        const pending = this.#pending.splice(0);
        for (const w of pending) w.reject(err);
    }
}
