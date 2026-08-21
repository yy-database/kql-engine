/**
 * CLI surface smoke — cac registration + orm compile wiring.
 */
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { describe, it } from "node:test";
import { fileURLToPath } from "node:url";
import { createSqlCli } from "../src/create-cli.ts";

const pkg = JSON.parse(readFileSync(join(dirname(fileURLToPath(import.meta.url)), "../package.json"), "utf8")) as {
    version: string;
};

describe("createSqlCli", () => {
    it("registers orm / serve / doctor commands", () => {
        const cli = createSqlCli();
        const names = cli.commands.map((c) => c.name);
        assert.deepEqual(names, ["serve", "probe", "doctor", "orm"]);
    });

    it("package version matches lockstep 0.0.1", () => {
        assert.equal(pkg.version, "0.0.1");
    });
});
