#!/usr/bin/env node
import { execFileSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const rootDir = join(dirname(fileURLToPath(import.meta.url)), "..");
const checkOnly = process.argv.includes("--check");

/**
 * @param {string} file
 * @param {string[]} args
 */
function run(file, args) {
    console.log(`$ ${file} ${args.join(" ")}`);
    execFileSync(file, args, {
        cwd: rootDir,
        stdio: "inherit",
        env: process.env,
        shell: process.platform === "win32",
    });
}

if (checkOnly) {
    run("pnpm", ["exec", "biome", "ci", "--formatter-enabled=true", "--linter-enabled=false", "--assist-enabled=false", "."]);
} else {
    run("pnpm", ["exec", "biome", "format", "--write", "."]);
}
