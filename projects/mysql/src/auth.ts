/**
 * MySQL authentication plugins + Handshake Response41.
 */

import { createHash } from "node:crypto";
import type { MysqlHandshakeV10 } from "./handshake.ts";

export const CLIENT_LONG_PASSWORD = 0x0000_0001;
export const CLIENT_FOUND_ROWS = 0x0000_0002;
export const CLIENT_LONG_FLAG = 0x0000_0004;
export const CLIENT_CONNECT_WITH_DB = 0x0000_0008;
export const CLIENT_PROTOCOL_41 = 0x0000_0200;
export const CLIENT_TRANSACTIONS = 0x0000_2000;
export const CLIENT_SECURE_CONNECTION = 0x0000_8000;
export const CLIENT_PLUGIN_AUTH = 0x0008_0000;
export const CLIENT_PLUGIN_AUTH_LENENC_CLIENT_DATA = 0x0020_0000;
export const CLIENT_DEPRECATE_EOF = 0x0100_0000;
export const CLIENT_SESSION_TRACK = 0x0080_0000;
export const CLIENT_MULTI_RESULTS = 0x0002_0000;

export const CLIENT_CAPABILITIES =
    CLIENT_LONG_PASSWORD |
    CLIENT_FOUND_ROWS |
    CLIENT_LONG_FLAG |
    CLIENT_CONNECT_WITH_DB |
    CLIENT_PROTOCOL_41 |
    CLIENT_TRANSACTIONS |
    CLIENT_SECURE_CONNECTION |
    CLIENT_PLUGIN_AUTH |
    CLIENT_PLUGIN_AUTH_LENENC_CLIENT_DATA |
    CLIENT_DEPRECATE_EOF |
    CLIENT_SESSION_TRACK |
    CLIENT_MULTI_RESULTS;

function sha1(data: Uint8Array | string): Buffer {
    return createHash("sha1").update(data).digest();
}

function sha256(data: Uint8Array | string): Buffer {
    return createHash("sha256").update(data).digest();
}

function xorBuffers(a: Buffer, b: Buffer): Buffer {
    const out = Buffer.allocUnsafe(a.length);
    for (let i = 0; i < a.length; i += 1) out[i] = a[i]! ^ b[i]!;
    return out;
}

/** mysql_native_password scramble. */
export function nativePasswordToken(password: string, scramble: Uint8Array): Uint8Array {
    if (!password) return new Uint8Array(0);
    const stage1 = sha1(password);
    const stage2 = sha1(stage1);
    const stage3 = sha1(Buffer.concat([Buffer.from(scramble), stage2]));
    return xorBuffers(stage1, stage3);
}

/** caching_sha2_password scramble (fast auth path). */
export function cachingSha2Token(password: string, scramble: Uint8Array): Uint8Array {
    if (!password) return new Uint8Array(0);
    const stage1 = sha256(password);
    const stage2 = sha256(stage1);
    const stage3 = sha256(Buffer.concat([stage2, Buffer.from(scramble)]));
    return xorBuffers(stage1, stage3);
}

export function authTokenForPlugin(plugin: string, password: string, scramble: Uint8Array): Uint8Array {
    if (plugin === "mysql_native_password") {
        return nativePasswordToken(password, scramble);
    }
    if (plugin === "caching_sha2_password") {
        return cachingSha2Token(password, scramble);
    }
    throw new Error(`@yydb/mysql: unsupported auth plugin ${plugin}`);
}

function writeUint32LE(view: Uint8Array, offset: number, value: number): void {
    view[offset] = value & 0xff;
    view[offset + 1] = (value >>> 8) & 0xff;
    view[offset + 2] = (value >>> 16) & 0xff;
    view[offset + 3] = (value >>> 24) & 0xff;
}

export type HandshakeResponseOptions = {
    handshake: MysqlHandshakeV10;
    user: string;
    password: string;
    database?: string;
    /** Override negotiated client caps (default CLIENT_CAPABILITIES ∩ server). */
    clientCapabilities?: number;
};

/** Build HandshakeResponse41 payload (no packet header). */
export function buildHandshakeResponse41(options: HandshakeResponseOptions): Uint8Array {
    const { handshake, user, password, database } = options;
    let caps = options.clientCapabilities ?? (CLIENT_CAPABILITIES & handshake.capabilityFlags) | CLIENT_PROTOCOL_41;
    if (!database) caps &= ~CLIENT_CONNECT_WITH_DB;
    else caps |= CLIENT_CONNECT_WITH_DB;

    const plugin = handshake.authPluginName || "mysql_native_password";
    const token = authTokenForPlugin(plugin, password, handshake.authPluginData);
    const userBytes = new TextEncoder().encode(user);
    const dbBytes = database ? new TextEncoder().encode(database) : null;
    const pluginBytes = new TextEncoder().encode(plugin);

    // caps(4) + maxPacket(4) + charset(1) + reserved(23) + user\0 + lenenc-auth + [db\0] + plugin\0
    const authLen = token.byteLength;
    const lenencSize = authLen < 251 ? 1 : 3;
    let size =
        4 + 4 + 1 + 23 + userBytes.byteLength + 1 + lenencSize + authLen + (dbBytes ? dbBytes.byteLength + 1 : 0) + pluginBytes.byteLength + 1;

    const out = new Uint8Array(size);
    let o = 0;
    writeUint32LE(out, o, caps);
    o += 4;
    writeUint32LE(out, o, 0xffffff);
    o += 4;
    out[o++] = handshake.characterSet || 45; // utf8mb4
    o += 23;
    out.set(userBytes, o);
    o += userBytes.byteLength;
    out[o++] = 0;

    if (authLen < 251) {
        out[o++] = authLen;
    } else {
        out[o++] = 0xfc;
        out[o++] = authLen & 0xff;
        out[o++] = (authLen >> 8) & 0xff;
    }
    out.set(token, o);
    o += authLen;

    if (dbBytes) {
        out.set(dbBytes, o);
        o += dbBytes.byteLength;
        out[o++] = 0;
    }

    out.set(pluginBytes, o);
    o += pluginBytes.byteLength;
    out[o++] = 0;

    return out.subarray(0, o);
}
