/**
 * SQL Studio CLI builder (`cac`) — command name is `sql`.
 * No side effects on import.
 */
import cac from "cac";

const VERSION = "0.1.0";

function notImplemented(name: string): void {
    console.error(`sql ${name}: not implemented yet in @yydb/sql-studio`);
    process.exitCode = 1;
}

/** Build the `sql` CLI (exported for tests / embedding). */
export function createSqlCli() {
    const cli = cac("sql");

    cli.version(VERSION);
    cli.help();

    cli.command("serve", "Start SQL Studio server host")
        .option("--config <path>", "Path to sql-studio config")
        .option("--host <host>", "Bind host", { default: "127.0.0.1" })
        .option("--port <port>", "Bind port", { default: "8787" })
        .action(() => {
            notImplemented("serve");
        });

    cli.command("doctor", "Local diagnostics (config / drivers / environment)")
        .option("--config <path>", "Path to sql-studio config")
        .action(() => {
            notImplemented("doctor");
        });

    cli.command("orm pull", "Introspect a database into an ORM type snapshot")
        .option("--dialect <name>", "postgres | mysql | sqlite")
        .option("--url <url>", "Database URL")
        .option("--out <path>", "Output path for generated types")
        .action(() => {
            notImplemented("orm pull");
        });

    cli.command("orm generate", "Generate Database types / bindings from schema or snapshot")
        .option("--config <path>", "Path to sql-studio / ORM config")
        .option("--out <dir>", "Output directory")
        .action(() => {
            notImplemented("orm generate");
        });

    cli.command("orm migrate", "Run typed migrations")
        .option("--config <path>", "Path to migrations config")
        .option("--direction <dir>", "up | down", { default: "up" })
        .action(() => {
            notImplemented("orm migrate");
        });

    return cli;
}
