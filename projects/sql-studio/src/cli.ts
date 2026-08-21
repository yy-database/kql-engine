#!/usr/bin/env node
/**
 * SQL Studio CLI entry — registered command name is `sql`.
 * Uses `cac`; not hand-rolled argv/help.
 */
import { createSqlCli } from "./create-cli.ts";

const cli = createSqlCli();
cli.parse(process.argv, { run: false });
await cli.runMatchedCommand();
