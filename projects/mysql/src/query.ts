/**
 * COM_QUERY + result reading over MysqlSession.
 */

import type { MysqlSession } from "./session.ts";
import {
    parseColumnDefinitionName,
    parseErrPacket,
    parseOkPacket,
    parseTextRow,
    readLenencInt,
    type MysqlOkPacket,
    type MysqlResultSet,
} from "./result.ts";

const COM_QUERY = 0x03;

/** Escape and bind `?` placeholders for text-protocol COM_QUERY. */
export function bindMysqlParameters(sql: string, parameters: readonly unknown[]): string {
    if (parameters.length === 0) return sql;
    let i = 0;
    return sql.replace(/\?/g, () => {
        if (i >= parameters.length) {
            throw new Error("@yydb/mysql: not enough parameters for placeholders");
        }
        const v = parameters[i++];
        return literal(v);
    });
}

function literal(value: unknown): string {
    if (value === null || value === undefined) return "NULL";
    if (typeof value === "number") {
        if (!Number.isFinite(value)) throw new Error("@yydb/mysql: non-finite number parameter");
        return String(value);
    }
    if (typeof value === "bigint") return value.toString();
    if (typeof value === "boolean") return value ? "1" : "0";
    if (value instanceof Date) return `'${value.toISOString().slice(0, 19).replace("T", " ")}'`;
    if (value instanceof Uint8Array) {
        let hex = "";
        for (let i = 0; i < value.byteLength; i += 1) {
            hex += value[i]!.toString(16).padStart(2, "0");
        }
        return `x'${hex}'`;
    }
    const s = String(value);
    return `'${s.replace(/\\/g, "\\\\").replace(/'/g, "''")}'`;
}

export type QueryOutcome = MysqlOkPacket | MysqlResultSet;

function isTerminator(payload: Uint8Array): boolean {
    const b = payload[0];
    if (b === 0x00) return true;
    // EOF_Packet is 0xfe with small payload (< 9 bytes in protocol 4.1)
    if (b === 0xfe && payload.byteLength < 9) return true;
    return false;
}

export async function comQuery(session: MysqlSession, sql: string, parameters: readonly unknown[] = []): Promise<QueryOutcome> {
    const text = bindMysqlParameters(sql, parameters);
    const encoded = new TextEncoder().encode(text);
    const payload = new Uint8Array(1 + encoded.byteLength);
    payload[0] = COM_QUERY;
    payload.set(encoded, 1);
    session.setSequenceId(0);
    session.sendPayload(payload, 0);

    const first = await session.readPacket();
    const head = first.payload[0];

    if (head === 0xff) {
        const err = parseErrPacket(first.payload);
        throw new Error(`@yydb/mysql: [${err.code}] ${err.message}`);
    }
    if (head === 0x00) {
        return parseOkPacket(first.payload);
    }

    const colCount = readLenencInt(first.payload, 0).value;
    const columns: string[] = [];
    for (let i = 0; i < colCount; i += 1) {
        const def = await session.readPacket();
        columns.push(parseColumnDefinitionName(def.payload));
    }

    // EOF or OK after column definitions
    let packet = await session.readPacket();
    if (!isTerminator(packet.payload)) {
        // Unexpected — treat as first row (some servers)
    } else {
        packet = await session.readPacket();
    }

    const rows: Record<string, unknown>[] = [];
    for (;;) {
        if (isTerminator(packet.payload)) break;
        if (packet.payload[0] === 0xff) {
            const err = parseErrPacket(packet.payload);
            throw new Error(`@yydb/mysql: [${err.code}] ${err.message}`);
        }
        rows.push(parseTextRow(packet.payload, columns));
        packet = await session.readPacket();
    }

    return { kind: "resultset", columns, rows };
}
