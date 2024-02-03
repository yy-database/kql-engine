/**
 * Relation declarations for Studio object navigation / multi-query loading.
 * Layered on the query builder — never lazy entity getters / N+1 by default.
 */

export type RelationHelpers = {
    hasMany(table: string, keys: { from: string; to: string }): { kind: "hasMany"; table: string; from: string; to: string };
    hasOne(table: string, keys: { from: string; to: string }): { kind: "hasOne"; table: string; from: string; to: string };
    belongsTo(table: string, keys: { from: string; to: string }): { kind: "belongsTo"; table: string; from: string; to: string };
};

export type RelationEdge = {
    kind: "hasMany" | "hasOne" | "belongsTo";
    table: string;
    from: string;
    to: string;
};

export type RelationsDefinition = Readonly<Record<string, Readonly<Record<string, RelationEdge>>>>;

const helpers: RelationHelpers = {
    hasMany(table, keys) {
        return { kind: "hasMany", table, from: keys.from, to: keys.to };
    },
    hasOne(table, keys) {
        return { kind: "hasOne", table, from: keys.from, to: keys.to };
    },
    belongsTo(table, keys) {
        return { kind: "belongsTo", table, from: keys.from, to: keys.to };
    },
};

/** Declare relations over a `Database` schema interface (type param is documentary). */
export function defineRelations<_DB>(build: (r: RelationHelpers) => RelationsDefinition): RelationsDefinition {
    return build(helpers);
}
