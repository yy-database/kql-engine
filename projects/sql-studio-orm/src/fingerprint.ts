/** Stable fingerprint for CompiledQuery / Studio audit. */

export function fingerprintSql(sql: string): string {
    // FNV-1a 32-bit — good enough for cache keys until a crypto hash is wired.
    let hash = 0x811c9dc5;
    for (let i = 0; i < sql.length; i += 1) {
        hash ^= sql.charCodeAt(i);
        hash = Math.imul(hash, 0x01000193);
    }
    return (hash >>> 0).toString(16).padStart(8, "0");
}
