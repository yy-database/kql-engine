/**
 * Protocol DTO guards + error code table presence.
 */
import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { PROTOCOL_VERSION, STUDIO_ERROR_CODES, isClientMessage, isStudioErrorCode } from "../src/index.ts";

describe("sql-studio-protocol", () => {
    it("PROTOCOL_VERSION is frozen at 1 for 0.0.x wire", () => {
        assert.equal(PROTOCOL_VERSION, 1);
    });

    it("isClientMessage accepts hello and rejects garbage", () => {
        assert.equal(isClientMessage({ v: PROTOCOL_VERSION, type: "hello" }), true);
        assert.equal(isClientMessage({ v: 999, type: "hello" }), false);
        assert.equal(isClientMessage(null), false);
        assert.equal(isClientMessage({ v: PROTOCOL_VERSION, type: "notARealType" }), false);
    });

    it("error code table is non-empty and closed", () => {
        assert.ok(STUDIO_ERROR_CODES.length >= 6);
        assert.equal(isStudioErrorCode("bad_message"), true);
        assert.equal(isStudioErrorCode("nope"), false);
    });
});
