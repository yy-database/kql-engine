#!/usr/bin/env node
/**
 * SQL Studio CLI entry — registered command name is `sql`.
 * Uses `cac`; not hand-rolled argv/help.
 */
import { createSqlCli } from "./create-cli.ts";

createSqlCli().parse();
