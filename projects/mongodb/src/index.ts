/**
 * `@yydb/mongodb` — MongoDB driver for SQL Studio
 * (Collection Browser / Query Editor).
 */

import type { StudioDriverRegistration } from "@yydb/sql-studio-protocol";

export const DRIVER_ID = "mongodb" as const;

export type MongodbFactoryOptions = {
    id: string;
    url: string;
};

export function mongodb(options: MongodbFactoryOptions): StudioDriverRegistration {
    return {
        id: options.id,
        kind: "mongodb",
        driver: { url: options.url, kind: DRIVER_ID },
    };
}
