/**
 * MySQL client/server packet framing (wire protocol — not "SQL text").
 *
 * Packet = 3-byte little-endian length + 1-byte sequence id + payload.
 */

export type MysqlPacket = {
    sequenceId: number;
    payload: Uint8Array;
};

/** Encode one MySQL packet. */
export function encodePacket(sequenceId: number, payload: Uint8Array): Uint8Array {
    if (payload.byteLength > 0xffffff) {
        throw new Error("@yydb/mysql: packet payload exceeds 16MiB-1");
    }
    const out = new Uint8Array(4 + payload.byteLength);
    out[0] = payload.byteLength & 0xff;
    out[1] = (payload.byteLength >> 8) & 0xff;
    out[2] = (payload.byteLength >> 16) & 0xff;
    out[3] = sequenceId & 0xff;
    out.set(payload, 4);
    return out;
}

/**
 * Incremental packet reader over a byte stream.
 */
export class PacketReader {
    #buffer = new Uint8Array(0);

    push(chunk: Uint8Array): void {
        if (chunk.byteLength === 0) return;
        const next = new Uint8Array(this.#buffer.byteLength + chunk.byteLength);
        next.set(this.#buffer, 0);
        next.set(chunk, this.#buffer.byteLength);
        this.#buffer = next;
    }

    /** Try to take one full packet; returns null if more bytes are needed. */
    tryRead(): MysqlPacket | null {
        if (this.#buffer.byteLength < 4) return null;
        const len = this.#buffer[0]! | (this.#buffer[1]! << 8) | (this.#buffer[2]! << 16);
        const sequenceId = this.#buffer[3]!;
        if (this.#buffer.byteLength < 4 + len) return null;
        const payload = this.#buffer.subarray(4, 4 + len);
        this.#buffer = this.#buffer.subarray(4 + len);
        return { sequenceId, payload: payload.slice() };
    }
}
