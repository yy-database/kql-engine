/**
 * Parameterized SQL fragments. `${value}` is always a bind parameter.
 * Identifiers and unsafe raw fragments require explicit APIs.
 */

import type { CompiledQuery } from "./driver.ts";
import { fingerprintSql } from "./fingerprint.ts";

export type SqlFragment = {
    readonly strings: readonly string[];
    readonly values: readonly unknown[];
};

export type Sql = SqlFragment & {
    compile(): CompiledQuery;
};

function isFragment(value: unknown): value is SqlFragment {
    return !!value && typeof value === "object" && "strings" in value && "values" in value && Array.isArray((value as SqlFragment).strings);
}

/** Build a parameterized SQL fragment (values become bind params). */
export function sql(strings: TemplateStringsArray, ...values: unknown[]): Sql {
    const flatStrings: string[] = [strings[0] ?? ""];
    const flatValues: unknown[] = [];

    for (let i = 0; i < values.length; i += 1) {
        const value = values[i];
        if (isFragment(value)) {
            flatStrings[flatStrings.length - 1] += value.strings[0] ?? "";
            for (let j = 0; j < value.values.length; j += 1) {
                flatValues.push(value.values[j]);
                flatStrings.push(value.strings[j + 1] ?? "");
            }
            flatStrings[flatStrings.length - 1] += strings[i + 1] ?? "";
        } else {
            flatValues.push(value);
            flatStrings.push(strings[i + 1] ?? "");
        }
    }

    const fragment: Sql = {
        strings: flatStrings,
        values: flatValues,
        compile(): CompiledQuery {
            return compileFragment(fragment);
        },
    };
    return fragment;
}

sql.identifier = (parts: string | readonly string[]): SqlFragment => {
    const list = typeof parts === "string" ? [parts] : [...parts];
    for (const part of list) {
        if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(part)) {
            throw new Error(`@yydb/sql-studio-orm: invalid SQL identifier ${JSON.stringify(part)}`);
        }
    }
    const quoted = list.map((p) => `"${p}"`).join(".");
    return { strings: [quoted], values: [] };
};

/** Explicit escape hatch — auditable raw SQL text, never from user input. */
sql.unsafe = (raw: string): SqlFragment => {
    return { strings: [raw], values: [] };
};

export function compileFragment(fragment: SqlFragment, placeholder: (index: number) => string = (i) => `$${i + 1}`): CompiledQuery {
    let sqlText = fragment.strings[0] ?? "";
    for (let i = 0; i < fragment.values.length; i += 1) {
        sqlText += placeholder(i);
        sqlText += fragment.strings[i + 1] ?? "";
    }
    return {
        sql: sqlText,
        parameters: [...fragment.values],
        operation: "raw",
        tables: [],
        fingerprint: fingerprintSql(sqlText),
    };
}

/** MySQL / SQLite style `?` placeholders. */
export function compileFragmentMysql(fragment: SqlFragment): CompiledQuery {
    return compileFragment(fragment, () => "?");
}
