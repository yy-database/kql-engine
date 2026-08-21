/**
 * `@yydb/redis` — Redis driver for SQL Studio (Key Browser / Command Console).
 * Product brand is SQL Studio; Redis is a Data Source, not a SQL engine.
 */

import type { StudioDriverRegistration } from "@yydb/sql-studio-protocol";

export const DRIVER_ID = "redis" as const;
export const DRIVER_STATUS = "experimental" as const;
/** Native Redis command model — never a SqlDriver. */
export const QUERY_MODEL = "native" as const;

export type RedisFactoryOptions = {
    id: string;
    url: string;
};

export type RedisNativeHandle = {
    readonly kind: typeof DRIVER_ID;
    readonly queryModel: typeof QUERY_MODEL;
    readonly url: string;
};

export function redis(options: RedisFactoryOptions): StudioDriverRegistration {
    const handle: RedisNativeHandle = {
        kind: DRIVER_ID,
        queryModel: QUERY_MODEL,
        url: options.url,
    };
    return {
        id: options.id,
        kind: "redis",
        driver: handle,
    };
}
