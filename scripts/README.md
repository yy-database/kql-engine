# Scripts

- `format.mjs` / `typecheck.mjs` — local quality gates (also used by CI)
- `pack-check.mjs` — `npm pack --dry-run` + workspace rewrite guard (`pnpm pack:check`)
- `ci/publish-placeholder.mjs` — local-only npm helpers for reserving names and testing package publication; never use this flow as a production release.
- `ci/publish-npm.mjs` — the real tagged-release publisher using OIDC Trusted Publisher credentials.

Secrets: gitignored `.env.placeholder.local` (`NPM_TOTP_SECRET`, optional `NPM_TOKEN`).

Trusted Publisher expect:

```text
repo = yy-database/sql-studio
file = publish-npm.yml
env  = NPM_PUBLISH
```
