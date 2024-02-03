/**
 * `@yydb/redis` — Redis driver for SQL Studio (Key Browser / Command Console).
 * Product brand is SQL Studio; Redis is a Data Source, not a SQL engine.
 */

import type { StudioDriverRegistration } from "@yydb/sql-studio-protocol";

export const DRIVER_ID = "redis" as const;

export type RedisFactoryOptions = {
    id: string;
    url: string;
};

export function redis(options: RedisFactoryOptions): StudioDriverRegistration {
    return {
        id: options.id,
        kind: "redis",
        driver: { url: options.url, kind: DRIVER_ID },
    };
}
