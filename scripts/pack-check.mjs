/**
 * pack:check — ensure each public package is safe to publish.
 * Temporarily rewrites workspace:* → package version (same as CI publish-npm),
 * runs npm pack --dry-run, then restores package.json.
 *
 * Usage: node scripts/pack-check.mjs
 */
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

const PACKAGES = [
    "projects/sql-studio-protocol",
    "projects/postgres",
    "projects/mysql",
    "projects/sqlite",
    "projects/redis",
    "projects/mongodb",
    "projects/sql-studio-orm",
    "projects/sql-studio-skills",
    "projects/sql-studio-server",
    "projects/sql-studio",
];

function fail(msg) {
    console.error(`pack-check: ${msg}`);
    process.exit(1);
}

/**
 * @param {string} pkgDir
 * @returns {{ name: string, version: string, backup: string, restore: () => void }}
 */
function prepare(pkgDir) {
    const pkgPath = path.join(REPO_ROOT, pkgDir, "package.json");
    const backup = fs.readFileSync(pkgPath, "utf8");
    const pkg = JSON.parse(backup);
    const version = pkg.version;
    for (const field of ["dependencies", "optionalDependencies", "peerDependencies"]) {
        const block = pkg[field];
        if (!block || typeof block !== "object") continue;
        for (const [name, range] of Object.entries(block)) {
            if (typeof range === "string" && range.startsWith("workspace:")) {
                if (!name.startsWith("@yydb/")) {
                    fail(`${pkg.name} has non-@yydb workspace dep ${name}`);
                }
                block[name] = version;
            }
        }
    }
    pkg.publishConfig = {
        ...(pkg.publishConfig ?? {}),
        access: "public",
        tag: "dev",
    };
    fs.writeFileSync(pkgPath, `${JSON.stringify(pkg, null, 2)}\n`);
    return {
        name: pkg.name,
        version,
        backup,
        restore() {
            fs.writeFileSync(pkgPath, backup);
        },
    };
}

let failed = 0;
/** @type {{ restore: () => void }[]} */
const prepared = [];

try {
    for (const dir of PACKAGES) {
        const prep = prepare(dir);
        prepared.push(prep);
        console.log(`\n→ ${prep.name}@${prep.version}`);

        const r = spawnSync("npm", ["pack", "--dry-run", "--json"], {
            cwd: path.join(REPO_ROOT, dir),
            encoding: "utf8",
            shell: process.platform === "win32",
        });
        if ((r.status ?? 1) !== 0) {
            console.error(r.stderr || r.stdout);
            console.error(`  ✗ npm pack --dry-run failed`);
            failed += 1;
            continue;
        }

        let files = [];
        try {
            const parsed = JSON.parse((r.stdout || "").trim() || "[]");
            const entry = Array.isArray(parsed) ? parsed[0] : parsed;
            files = (entry?.files ?? []).map((f) => f.path ?? f);
        } catch {
            files = `${r.stderr}\n${r.stdout}`
                .split(/\r?\n/)
                .map((l) => l.match(/npm notice\s+\d+B\s+(.+)$/i)?.[1]?.trim())
                .filter(Boolean);
        }

        const banned = [/node_modules/, /^\.env/, /secret/i, /(^|\/)tests?\//, /(^|\/)__tests__\//];
        for (const f of files) {
            const norm = String(f).replace(/\\/g, "/");
            if (banned.some((re) => re.test(norm))) {
                console.error(`  ✗ banned path in tarball: ${norm}`);
                failed += 1;
            }
        }

        const packedPkg = JSON.parse(fs.readFileSync(path.join(REPO_ROOT, dir, "package.json"), "utf8"));
        for (const field of ["dependencies", "optionalDependencies", "peerDependencies"]) {
            const block = packedPkg[field];
            if (!block) continue;
            for (const [name, range] of Object.entries(block)) {
                if (typeof range === "string" && range.startsWith("workspace:")) {
                    console.error(`  ✗ packed ${field} still workspace:* → ${name}`);
                    failed += 1;
                }
            }
        }

        console.log(`  ✓ pack dry-run (${files.length || "?"} files)`);
    }
} finally {
    for (const p of prepared) p.restore();
}

if (failed) fail(`${failed} problem(s)`);
console.log(`\npack-check: ok (${PACKAGES.length} packages)`);
