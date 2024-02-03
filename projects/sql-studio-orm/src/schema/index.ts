/**
 * Runtime schema DSL — declaration source for migration/codegen.
 * Do not hand-maintain parallel interface + defineSchema + migration triples.
 */

export type ColumnBuilder = {
    notNull(): ColumnBuilder;
    unique(): ColumnBuilder;
    primaryKey(): ColumnBuilder;
    generated(): ColumnBuilder;
    default(value: unknown): ColumnBuilder;
};

export type TableBuilder = {
    addColumn(name: string, type: string, build?: (col: ColumnBuilder) => ColumnBuilder): TableBuilder;
};

export type SchemaDefinition = {
    readonly tables: Readonly<Record<string, TableDefinition>>;
};

export type TableDefinition = {
    readonly columns: Readonly<Record<string, ColumnDefinition>>;
};

export type ColumnDefinition = {
    type: string;
    notNull: boolean;
    unique: boolean;
    primaryKey: boolean;
    generated: boolean;
    defaultValue?: unknown;
};

function column(type: string): ColumnBuilder & { __def: ColumnDefinition } {
    const def: ColumnDefinition = {
        type,
        notNull: false,
        unique: false,
        primaryKey: false,
        generated: false,
    };
    const builder: ColumnBuilder & { __def: ColumnDefinition } = {
        __def: def,
        notNull() {
            def.notNull = true;
            return builder;
        },
        unique() {
            def.unique = true;
            return builder;
        },
        primaryKey() {
            def.primaryKey = true;
            return builder;
        },
        generated() {
            def.generated = true;
            return builder;
        },
        default(value: unknown) {
            def.defaultValue = value;
            return builder;
        },
    };
    return builder;
}

export function integer(): ColumnBuilder {
    return column("integer");
}

export function varchar(length: number): ColumnBuilder {
    return column(`varchar(${length})`);
}

export function text(): ColumnBuilder {
    return column("text");
}

export function table(columns: Record<string, ColumnBuilder>): TableDefinition {
    const out: Record<string, ColumnDefinition> = {};
    for (const [name, builder] of Object.entries(columns)) {
        out[name] = (builder as ColumnBuilder & { __def: ColumnDefinition }).__def;
    }
    return { columns: out };
}

export function defineSchema(tables: Record<string, TableDefinition>): SchemaDefinition {
    return { tables };
}
