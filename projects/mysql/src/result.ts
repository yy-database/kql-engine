/**
 * MySQL OK / ERR / result-set parsing (text protocol).
 */

export type MysqlOkPacket = {
    kind: "ok";
    affectedRows: number;
    lastInsertId: number;
    statusFlags: number;
    warningCount: number;
    info: string;
};

export type MysqlErrPacket = {
    kind: "err";
    code: number;
    sqlState: string;
    message: string;
};

export type MysqlResultSet = {
    kind: "resultset";
    columns: string[];
    rows: Record<string, unknown>[];
};

function readLenencInt(buf: Uint8Array, offset: number): { value: number; next: number } {
    const fb = buf[offset]!;
    if (fb < 0xfb) return { value: fb, next: offset + 1 };
    if (fb === 0xfc) {
        return { value: buf[offset + 1]! | (buf[offset + 2]! << 8), next: offset + 3 };
    }
    if (fb === 0xfd) {
        return {
            value: buf[offset + 1]! | (buf[offset + 2]! << 8) | (buf[offset + 3]! << 16),
            next: offset + 4,
        };
    }
    if (fb === 0xfe) {
        // 8-byte — clamp to Number for v0
        let v = 0;
        for (let i = 0; i < 8; i += 1) v += buf[offset + 1 + i]! * 256 ** i;
        return { value: v, next: offset + 9 };
    }
    throw new Error("@yydb/mysql: invalid length-encoded integer");
}

function readLenencString(buf: Uint8Array, offset: number): { value: string | null; next: number } {
    if (buf[offset] === 0xfb) return { value: null, next: offset + 1 };
    const len = readLenencInt(buf, offset);
    const start = len.next;
    const end = start + len.value;
    const value = new TextDecoder("utf-8").decode(buf.subarray(start, end));
    return { value, next: end };
}

export function parseOkPacket(payload: Uint8Array): MysqlOkPacket {
    let o = 1; // skip 0x00
    const affected = readLenencInt(payload, o);
    o = affected.next;
    const insertId = readLenencInt(payload, o);
    o = insertId.next;
    const statusFlags = payload[o]! | (payload[o + 1]! << 8);
    o += 2;
    const warningCount = payload[o]! | (payload[o + 1]! << 8);
    o += 2;
    const info = o < payload.byteLength ? new TextDecoder("utf-8").decode(payload.subarray(o)) : "";
    return {
        kind: "ok",
        affectedRows: affected.value,
        lastInsertId: insertId.value,
        statusFlags,
        warningCount,
        info,
    };
}

export function parseErrPacket(payload: Uint8Array): MysqlErrPacket {
    const code = payload[1]! | (payload[2]! << 8);
    let o = 3;
    let sqlState = "";
    if (payload[o] === 0x23) {
        sqlState = new TextDecoder("utf-8").decode(payload.subarray(o + 1, o + 6));
        o += 6;
    }
    const message = new TextDecoder("utf-8").decode(payload.subarray(o));
    return { kind: "err", code, sqlState, message };
}

/** Column definition packet — return name (org or alias). */
export function parseColumnDefinitionName(payload: Uint8Array): string {
    let o = 0;
    // catalog, schema, table, org_table, name, org_name — all lenenc strings
    for (let i = 0; i < 4; i += 1) {
        const s = readLenencString(payload, o);
        o = s.next;
    }
    const name = readLenencString(payload, o);
    return name.value ?? "";
}

export function parseTextRow(payload: Uint8Array, columns: string[]): Record<string, unknown> {
    const row: Record<string, unknown> = {};
    let o = 0;
    for (const col of columns) {
        const cell = readLenencString(payload, o);
        o = cell.next;
        row[col] = cell.value;
    }
    return row;
}

export { readLenencInt, readLenencString };
