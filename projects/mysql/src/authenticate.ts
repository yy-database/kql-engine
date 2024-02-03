/**
 * Authenticate a session after HandshakeV10 (sequence continues from handshake).
 */

import { authTokenForPlugin, buildHandshakeResponse41, CLIENT_CAPABILITIES } from "./auth.ts";
import type { MysqlHandshakeV10 } from "./handshake.ts";
import { parseErrPacket, parseOkPacket } from "./result.ts";
import type { MysqlSession } from "./session.ts";

export type AuthenticateOptions = {
    handshake: MysqlHandshakeV10;
    user: string;
    password: string;
    database?: string;
    /** True when the socket is already TLS-wrapped. */
    secureTransport?: boolean;
};

/**
 * Send HandshakeResponse41 and finish auth (native_password / caching_sha2 fast path).
 */
export async function authenticate(session: MysqlSession, options: AuthenticateOptions): Promise<void> {
    const response = buildHandshakeResponse41({
        handshake: options.handshake,
        user: options.user,
        password: options.password,
        database: options.database,
        clientCapabilities: CLIENT_CAPABILITIES,
    });
    // Handshake was seq 0; client response is seq 1
    session.setSequenceId(1);
    session.sendPayload(response, 1);

    let packet = await session.readPacket();

    for (;;) {
        const b0 = packet.payload[0];
        if (b0 === 0x00) {
            parseOkPacket(packet.payload);
            return;
        }
        if (b0 === 0xff) {
            const err = parseErrPacket(packet.payload);
            throw new Error(`@yydb/mysql auth: [${err.code}] ${err.message}`);
        }
        if (b0 === 0xfe) {
            // AuthSwitchRequest: 0xfe + plugin\0 + scramble
            const payload = packet.payload;
            let o = 1;
            let end = o;
            while (end < payload.byteLength && payload[end] !== 0) end += 1;
            const plugin = new TextDecoder().decode(payload.subarray(o, end));
            o = end + 1;
            const scramble = payload.subarray(o, payload.byteLength - (payload[payload.byteLength - 1] === 0 ? 1 : 0));
            const token = authTokenForPlugin(plugin, options.password, scramble);
            session.sendPayload(token, (packet.sequenceId + 1) & 0xff);
            packet = await session.readPacket();
            continue;
        }
        if (b0 === 0x01) {
            // AuthMoreData (caching_sha2)
            const status = packet.payload[1];
            if (status === 0x03) {
                // fast auth success — wait for OK
                packet = await session.readPacket();
                continue;
            }
            if (status === 0x04) {
                if (!options.secureTransport) {
                    throw new Error("@yydb/mysql: caching_sha2_password full auth requires TLS (or use mysql_native_password)");
                }
                // Cleartext password over secure channel: 0x01? Actually send password + NUL
                const pwd = new TextEncoder().encode(options.password);
                const clear = new Uint8Array(pwd.byteLength + 1);
                clear.set(pwd, 0);
                session.sendPayload(clear, (packet.sequenceId + 1) & 0xff);
                packet = await session.readPacket();
                continue;
            }
            throw new Error(`@yydb/mysql: unexpected AuthMoreData status 0x${status?.toString(16)}`);
        }
        throw new Error(`@yydb/mysql: unexpected auth packet 0x${b0?.toString(16)}`);
    }
}
