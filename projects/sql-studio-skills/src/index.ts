/**
 * Official skill catalog for `@yydb/sql-studio-skills`.
 * Authority: Spark `决策和进度表/sql-studio-architecture.md` §3.
 *
 * Skills teach workflows. They do NOT implement Server tools. Until Server DTOs
 * are frozen, SKILL.md files mark tools as planned vs available (CLI stubs only).
 */

export type SkillDelivery = "docs-only" | "cli-stub" | "tool-live";

export type SqlStudioSkillMeta = {
    readonly id: string;
    readonly name: string;
    readonly description: string;
    readonly skillMd: string;
    /** Honest delivery state — never claim tool-live without Server/MCP wiring. */
    readonly delivery: SkillDelivery;
};

export const SQL_STUDIO_SKILLS: readonly SqlStudioSkillMeta[] = [
    {
        id: "sql-studio-connect",
        name: "sql-studio-connect",
        description:
            "Discover datasources, validate connection config, diagnose TLS/auth/network without echoing secrets. Use when connecting SQL Studio to a database.",
        skillMd: "skills/sql-studio-connect/SKILL.md",
        delivery: "docs-only",
    },
    {
        id: "sql-studio-inspect",
        name: "sql-studio-inspect",
        description:
            "On-demand catalog inspect/search for tables, columns, indexes, constraints, and small stats. Use before querying unfamiliar schemas.",
        skillMd: "skills/sql-studio-inspect/SKILL.md",
        delivery: "docs-only",
    },
    {
        id: "sql-studio-query",
        name: "sql-studio-query",
        description:
            "Bounded SQL workflow: compile/describe/explain then execute/stream/cancel via Studio Server policy. Default read-only with row/time budgets.",
        skillMd: "skills/sql-studio-query/SKILL.md",
        delivery: "docs-only",
    },
    {
        id: "sql-studio-migrate",
        name: "sql-studio-migrate",
        description:
            "Migration plan/review/apply with plan hash binding. Destructive steps require human approval. Use for schema changes through SQL Studio.",
        skillMd: "skills/sql-studio-migrate/SKILL.md",
        delivery: "docs-only",
    },
    {
        id: "sql-studio-diagnose",
        name: "sql-studio-diagnose",
        description:
            "Evidence-based diagnosis from structured errors, connection state, and slow-query signals. Use when Studio or driver calls fail.",
        skillMd: "skills/sql-studio-diagnose/SKILL.md",
        delivery: "docs-only",
    },
    {
        id: "sql-studio-performance",
        name: "sql-studio-performance",
        description:
            "Read explain/metrics and propose index or query changes. Suggestions are separate from apply. Use for slow SQL investigation.",
        skillMd: "skills/sql-studio-performance/SKILL.md",
        delivery: "docs-only",
    },
] as const;

/** Planned Server tool surface (not live until DTO freeze). */
export const SQL_STUDIO_PLANNED_TOOLS = [
    "datasource.list",
    "datasource.describe",
    "catalog.inspect",
    "catalog.search",
    "query.compile",
    "query.describe",
    "query.explain",
    "query.execute",
    "query.stream",
    "query.cancel",
    "migration.plan",
    "migration.review",
    "migration.apply",
    "audit.list",
    "evidence.export",
] as const;

export type RiskClass = "Observe" | "Read" | "Mutate" | "Destructive" | "Administrative";

export function listSqlStudioSkills(): readonly SqlStudioSkillMeta[] {
    return SQL_STUDIO_SKILLS;
}

export function getSqlStudioSkill(id: string): SqlStudioSkillMeta | undefined {
    return SQL_STUDIO_SKILLS.find((s) => s.id === id);
}
