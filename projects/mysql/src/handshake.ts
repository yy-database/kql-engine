/**
 * Parse MySQL HandshakeV10 (server greeting) — wire payload only.
 */

export type MysqlHandshakeV10 = {
    protocolVersion: number;
    serverVersion: string;
    connectionId: number;
    /** auth plugin data part 1 (8 bytes) + filler skipped; full scramble assembled when part 2 present */
    authPluginData: Uint8Array;
    capabilityFlags: number;
    characterSet: number;
    statusFlags: number;
    authPluginName: string;
};

function readCString(view: Uint8Array, offset: number): { value: string; next: number } {
    let end = offset;
    while (end < view.byteLength && view[end] !== 0) end += 1;
    const value = new TextDecoder("utf-8").decode(view.subarray(offset, end));
    return { value, next: Math.min(end + 1, view.byteLength) };
}

/** Parse HandshakeV10 payload (without the 4-byte packet header). */
export function parseHandshakeV10(payload: Uint8Array): MysqlHandshakeV10 {
    if (payload.byteLength < 20) {
        throw new Error("@yydb/mysql: handshake payload too short");
    }
    const protocolVersion = payload[0]!;
    if (protocolVersion !== 10) {
        throw new Error(`@yydb/mysql: unsupported protocol version ${protocolVersion}`);
    }

    const ver = readCString(payload, 1);
    let o = ver.next;
    if (o + 13 > payload.byteLength) {
        throw new Error("@yydb/mysql: truncated handshake after server version");
    }

    const connectionId = payload[o]! | (payload[o + 1]! << 8) | (payload[o + 2]! << 16) | (payload[o + 3]! << 24);
    o += 4;

    const auth1 = payload.subarray(o, o + 8);
    o += 8;
    o += 1; // filler

    let capabilityFlags = payload[o]! | (payload[o + 1]! << 8);
    o += 2;

    let characterSet = 0;
    let statusFlags = 0;
    let authPluginDataLen = 0;
    if (o < payload.byteLength) {
        characterSet = payload[o]!;
        o += 1;
    }
    if (o + 2 <= payload.byteLength) {
        statusFlags = payload[o]! | (payload[o + 1]! << 8);
        o += 2;
    }
    if (o + 2 <= payload.byteLength) {
        capabilityFlags |= (payload[o]! | (payload[o + 1]! << 8)) << 16;
        o += 2;
    }
    if (o < payload.byteLength) {
        authPluginDataLen = payload[o]!;
        o += 1;
    }
    // reserved 10 bytes
    o = Math.min(o + 10, payload.byteLength);

    const auth2Len = Math.max(13, authPluginDataLen - 8);
    const auth2 = payload.subarray(o, Math.min(o + auth2Len, payload.byteLength));
    o += auth2.length;

    // trim trailing NUL from auth plugin data part 2 if present
    let auth2Trim = auth2;
    if (auth2Trim.byteLength > 0 && auth2Trim[auth2Trim.byteLength - 1] === 0) {
        auth2Trim = auth2Trim.subarray(0, auth2Trim.byteLength - 1);
    }

    const authPluginData = new Uint8Array(auth1.byteLength + auth2Trim.byteLength);
    authPluginData.set(auth1, 0);
    authPluginData.set(auth2Trim, auth1.byteLength);

    let authPluginName = "mysql_native_password";
    if (o < payload.byteLength) {
        authPluginName = readCString(payload, o).value || authPluginName;
    }

    return {
        protocolVersion,
        serverVersion: ver.value,
        connectionId: connectionId >>> 0,
        authPluginData,
        capabilityFlags: capabilityFlags >>> 0,
        characterSet,
        statusFlags,
        authPluginName,
    };
}
