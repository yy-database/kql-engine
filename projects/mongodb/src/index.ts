/**
 * `@yydb/mongodb` — MongoDB driver for SQL Studio
 * (Collection Browser / Query Editor).
 */

import type { StudioDriverRegistration } from "@yydb/sql-studio-protocol";

export const DRIVER_ID = "mongodb" as const;
export const DRIVER_STATUS = "experimental" as const;
/** Native Mongo query model — never a SqlDriver. */
export const QUERY_MODEL = "native" as const;

export type MongodbFactoryOptions = {
    id: string;
    url: string;
};

export type MongodbNativeHandle = {
    readonly kind: typeof DRIVER_ID;
    readonly queryModel: typeof QUERY_MODEL;
    readonly url: string;
};

export function mongodb(options: MongodbFactoryOptions): StudioDriverRegistration {
    const handle: MongodbNativeHandle = {
        kind: DRIVER_ID,
        queryModel: QUERY_MODEL,
        url: options.url,
    };
    return {
        id: options.id,
        kind: "mongodb",
        driver: handle,
    };
}
