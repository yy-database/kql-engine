/**
 * Publish real `@yydb/*` packages for SQL Studio.
 *
 * Trusted Publisher (OIDC) only in CI — no NPM_TOKEN.
 * Contract: file=publish-npm.yml env=NPM_PUBLISH repo=yy-database/sql-studio
 *
 * Usage:
 *   node scripts/ci/publish-npm.mjs --version=0.0.0
 *   GITHUB_REF=refs/tags/v0.0.0 node scripts/ci/publish-npm.mjs
 *
 * Dist-tag:
 *   0.0.x → dev
 *   0.1.x → beta
 *   else  → latest
 */

import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

/** Publish order: protocol first, then leaves, then facades. */
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
    console.error(`ci-publish-npm: ${msg}`);
    process.exit(1);
}

/** @param {string[]} args */
function takeFlag(args, flag) {
    const i = args.indexOf(flag);
    if (i >= 0 && args[i + 1] && !args[i + 1].startsWith("-")) return args[i + 1];
    const eq = args.find((a) => a.startsWith(`${flag}=`));
    if (eq) return eq.slice(flag.length + 1);
    return undefined;
}

function resolveVersion(argv) {
    const fromFlag = takeFlag(argv, "--version");
    if (fromFlag) return fromFlag.replace(/^v/, "");
    const ref = process.env.GITHUB_REF ?? "";
    const m = ref.match(/^refs\/tags\/(?:placeholder-)?v?(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)$/);
    if (m) return m[1];
    fail("pass --version=X.Y.Z or set GITHUB_REF=refs/tags/vX.Y.Z");
}

/** @param {string} version */
function distTag(version) {
    if (version.startsWith("0.0.")) return "dev";
    if (version.startsWith("0.1.")) return "beta";
    return "latest";
}

/**
 * @param {string[]} args
 * @param {{ cwd?: string }} [opts]
 */
function run(cmd, args, opts = {}) {
    const r = spawnSync(cmd, args, {
        cwd: opts.cwd ?? REPO_ROOT,
        encoding: "utf8",
        shell: process.platform === "win32",
        stdio: "inherit",
        env: {
            ...process.env,
            NODE_AUTH_TOKEN: undefined,
            NPM_TOKEN: undefined,
        },
    });
    return r.status ?? 1;
}

/** Rewrite workspace:* deps to concrete versions for the release set. */
function rewriteWorkspaceDeps(pkgDir, version) {
    const pkgPath = path.join(REPO_ROOT, pkgDir, "package.json");
    const pkg = JSON.parse(fs.readFileSync(pkgPath, "utf8"));
    pkg.version = version;
    for (const field of ["dependencies", "optionalDependencies", "peerDependencies"]) {
        const block = pkg[field];
        if (!block || typeof block !== "object") continue;
        for (const [name, range] of Object.entries(block)) {
            if (typeof range === "string" && range.startsWith("workspace:")) {
                if (name.startsWith("@yydb/")) {
                    block[name] = version;
                }
            }
        }
    }
    pkg.publishConfig = {
        ...(pkg.publishConfig ?? {}),
        access: "public",
        tag: distTag(version),
    };
    fs.writeFileSync(pkgPath, `${JSON.stringify(pkg, null, 2)}\n`);
    return pkg.name;
}

function isAlreadyPublished(stderr) {
    return /cannot publish over existing|EPUBLISHCONFLICT|previously published|version already exists|cannot publish.*same version/i.test(
        stderr,
    );
}

const argv = process.argv.slice(2);
const version = resolveVersion(argv);
const tag = distTag(version);

console.log(`ci-publish-npm: version=${version} tag=${tag}`);
console.log(" Trusted Publisher contract: publish-npm.yml + env NPM_PUBLISH + repo yy-database/sql-studio\n");

delete process.env.NPM_TOKEN;
delete process.env.NODE_AUTH_TOKEN;

let published = 0;
let skipped = 0;

for (const dir of PACKAGES) {
    const name = rewriteWorkspaceDeps(dir, version);
    console.log(`\n→ ${name}@${version}`);
    const r = spawnSync("npm", ["publish", "--access", "public", "--tag", tag, "--provenance"], {
        cwd: path.join(REPO_ROOT, dir),
        encoding: "utf8",
        shell: process.platform === "win32",
        env: {
            ...process.env,
            NODE_AUTH_TOKEN: undefined,
            NPM_TOKEN: undefined,
        },
    });
    const blob = `${r.stdout ?? ""}\n${r.stderr ?? ""}`;
    if (r.status === 0) {
        published += 1;
        console.log(`  ✓ published ${name}@${version}`);
        continue;
    }
    if (isAlreadyPublished(blob)) {
        skipped += 1;
        console.log(`  · already published — skip`);
        continue;
    }
    console.error(blob);
    fail(`publish failed for ${name}`);
}

console.log(`\nci-publish-npm: done (published=${published} skipped=${skipped})`);
