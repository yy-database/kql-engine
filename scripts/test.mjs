/**
 * Workspace test runner — Node built-in test runner + TS strip-types.
 *
 * Usage: node scripts/test.mjs
 */
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

/** @param {string} dir */
function collectTests(dir, out = []) {
    if (!fs.existsSync(dir)) return out;
    for (const ent of fs.readdirSync(dir, { withFileTypes: true })) {
        const p = path.join(dir, ent.name);
        if (ent.isDirectory()) {
            if (ent.name === "node_modules" || ent.name === "dist") continue;
            collectTests(p, out);
        } else if (ent.isFile() && ent.name.endsWith(".test.ts")) {
            out.push(p);
        }
    }
    return out;
}

const tests = collectTests(path.join(ROOT, "projects"));
if (tests.length === 0) {
    console.error("test: no *.test.ts files under projects/");
    process.exit(1);
}

const relative = tests.map((t) => path.relative(ROOT, t).split(path.sep).join("/"));
console.log(`test: ${relative.length} file(s)`);

const r = spawnSync(process.execPath, ["--experimental-strip-types", "--test", "--test-reporter=spec", ...relative], {
    cwd: ROOT,
    stdio: "inherit",
    env: {
        ...process.env,
        NODE_OPTIONS: [process.env.NODE_OPTIONS, "--experimental-strip-types"].filter(Boolean).join(" "),
    },
});

process.exit(r.status ?? 1);
