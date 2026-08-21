/**
 * SQL Studio CLI builder (`cac`) — command name is `sql`.
 * No side effects on import.
 */
import cac from "cac";
import { runDoctor } from "./doctor.ts";
import { runMemoryCompileFixture } from "./orm-compile.ts";
import { runMemoryProbe } from "./probe.ts";
import { startMemoryServeHttp } from "./serve-http.ts";

const VERSION = "0.0.1";

function notImplemented(name: string): void {
    console.error(`sql ${name}: not implemented yet in @yydb/sql-studio@${VERSION} (Developer Preview)`);
    process.exitCode = 1;
}

/** Build the `sql` CLI (exported for tests / embedding). */
export function createSqlCli() {
    const cli = cac("sql");

    cli.version(VERSION);
    cli.help();

    cli.command("serve", "Start SQL Studio server host (MemoryDriver DP loopback)")
        .option("--config <path>", "Path to sql-studio config")
        .option("--host <host>", "Bind host", { default: "127.0.0.1" })
        .option("--port <port>", "Bind port (0 = ephemeral)", { default: "8787" })
        .option("--self-test", "Probe hello then exit (CI)")
        .action(async (opts: { host?: string; port?: string; selfTest?: boolean }) => {
            try {
                const port = Number(opts.port ?? 8787);
                const handle = await startMemoryServeHttp({
                    host: opts.host ?? "127.0.0.1",
                    port: Number.isFinite(port) ? port : 8787,
                    selfTest: Boolean(opts.selfTest),
                });
                if (opts.selfTest) {
                    console.log(JSON.stringify({ ok: true, mode: "self-test", version: VERSION }, null, 2));
                    return;
                }
                console.log(
                    JSON.stringify(
                        {
                            ok: true,
                            url: handle.url,
                            protocolVersion: handle.server.protocolVersion,
                            datasources: handle.server.listDatasources(),
                            version: VERSION,
                        },
                        null,
                        2,
                    ),
                );
                const shutdown = async () => {
                    await handle.close();
                    process.exit(0);
                };
                process.on("SIGINT", () => void shutdown());
                process.on("SIGTERM", () => void shutdown());
            } catch (err) {
                console.error(err instanceof Error ? err.message : String(err));
                process.exitCode = 1;
            }
        });

    cli.command("probe", "Client↔server MemoryDriver round-trip (hello/open/query/close)").action(async () => {
        try {
            const report = await runMemoryProbe(VERSION);
            console.log(JSON.stringify(report, null, 2));
            if (!report.ok) process.exitCode = 1;
        } catch (err) {
            console.error(err instanceof Error ? err.message : String(err));
            process.exitCode = 1;
        }
    });

    cli.command("doctor", "Local diagnostics (protocol / MemoryDriver / server hello)")
        .option("--config <path>", "Path to sql-studio config")
        .action(async () => {
            try {
                const report = await runDoctor(VERSION);
                console.log(JSON.stringify(report, null, 2));
                if (!report.ok) process.exitCode = 1;
            } catch (err) {
                console.error(err instanceof Error ? err.message : String(err));
                process.exitCode = 1;
            }
        });

    // cac does not match multi-word command names like "orm compile"; use one `orm <action>`.
    cli.command("orm <action>", "ORM tools: compile | pull | generate | migrate")
        .option("--dialect <name>", "postgres | mysql | sqlite")
        .option("--url <url>", "Database URL")
        .option("--out <path>", "Output path")
        .option("--config <path>", "Path to sql-studio / ORM config")
        .option("--direction <dir>", "up | down", { default: "up" })
        .action(async (action: string) => {
            if (action === "compile") {
                try {
                    const out = await runMemoryCompileFixture();
                    console.log(JSON.stringify(out, null, 2));
                } catch (err) {
                    console.error(err instanceof Error ? err.message : String(err));
                    process.exitCode = 1;
                }
                return;
            }
            if (action === "pull" || action === "generate" || action === "migrate") {
                notImplemented(`orm ${action}`);
                return;
            }
            console.error(`sql orm: unknown action \`${action}\` (compile | pull | generate | migrate)`);
            process.exitCode = 1;
        });

    return cli;
}
