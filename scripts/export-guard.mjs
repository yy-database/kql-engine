/**
 * export-guard — browser-safe default entries must not pull node:* or CLI.
 *
 * Usage: node scripts/export-guard.mjs
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

/** Default entry files that must stay free of node: / process.exit / cac. */
const BROWSER_SAFE_ENTRY = [
    "projects/sql-studio/src/index.ts",
    "projects/sql-studio-protocol/src/index.ts",
    "projects/sql-studio-orm/src/index.ts",
    "projects/mysql/src/index.ts",
    "projects/postgres/src/index.ts",
    "projects/sqlite/src/index.ts",
    "projects/sqlite/src/wasm.ts",
    "projects/redis/src/index.ts",
    "projects/mongodb/src/index.ts",
];

const FORBIDDEN = [
    { re: /from\s+["']node:/, msg: "static node: import" },
    { re: /import\s*\(\s*["']node:/, msg: "dynamic node: import" },
    { re: /from\s+["']cac["']/, msg: "cac (CLI) import" },
    { re: /process\.exit(?:Code)?\b/, msg: "process.exit / exitCode" },
];

/** Packages whose default `.` must not re-export Node CLI or TCP acquire helpers. */
const MUST_NOT_REEXPORT = [
    {
        file: "projects/sql-studio/src/index.ts",
        patterns: [/createSqlCli/, /create-cli/],
        msg: "CLI must live on @yydb/sql-studio/cli",
    },
    {
        file: "projects/mysql/src/index.ts",
        patterns: [/from\s+["']\.\/auth\.ts["']/, /from\s+["']\.\/node\.ts["']/, /import\s*\(\s*["']\.\/node/],
        msg: "auth/TCP must live on @yydb/mysql/node",
    },
];

function fail(msg) {
    console.error(`export-guard: ${msg}`);
    process.exit(1);
}

let problems = 0;

for (const rel of BROWSER_SAFE_ENTRY) {
    const abs = path.join(REPO_ROOT, rel);
    const text = fs.readFileSync(abs, "utf8");
    for (const { re, msg } of FORBIDDEN) {
        if (re.test(text)) {
            console.error(`  ✗ ${rel}: ${msg}`);
            problems += 1;
        }
    }
}

for (const rule of MUST_NOT_REEXPORT) {
    const abs = path.join(REPO_ROOT, rule.file);
    const text = fs.readFileSync(abs, "utf8");
    for (const re of rule.patterns) {
        if (re.test(text)) {
            console.error(`  ✗ ${rule.file}: ${rule.msg} (matched ${re})`);
            problems += 1;
        }
    }
}

const studioPkg = JSON.parse(fs.readFileSync(path.join(REPO_ROOT, "projects/sql-studio/package.json"), "utf8"));
if (!studioPkg.exports?.["./cli"]) {
    console.error("  ✗ @yydb/sql-studio missing exports['./cli']");
    problems += 1;
}

const mysqlPkg = JSON.parse(fs.readFileSync(path.join(REPO_ROOT, "projects/mysql/package.json"), "utf8"));
if (!mysqlPkg.exports?.["./node"]) {
    console.error("  ✗ @yydb/mysql missing exports['./node']");
    problems += 1;
}

const postgresPkg = JSON.parse(fs.readFileSync(path.join(REPO_ROOT, "projects/postgres/package.json"), "utf8"));
if (!postgresPkg.exports?.["./node"]) {
    console.error("  ✗ @yydb/postgres missing exports['./node']");
    problems += 1;
}

const sqlitePkg = JSON.parse(fs.readFileSync(path.join(REPO_ROOT, "projects/sqlite/package.json"), "utf8"));
if (!sqlitePkg.exports?.["./node"] || !sqlitePkg.exports?.["./wasm"]) {
    console.error("  ✗ @yydb/sqlite missing exports['./node'] or exports['./wasm']");
    problems += 1;
}

if (problems) fail(`${problems} problem(s)`);
console.log(`export-guard: ok (${BROWSER_SAFE_ENTRY.length} entries)`);
