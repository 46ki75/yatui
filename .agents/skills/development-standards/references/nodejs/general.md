# Node.js Generic Development Standards

Org-wide runtime-level convention for Node projects. Language-level
TypeScript conventions (tsconfig, lint/format tooling) live in
[`references/typescript/general.md`](../typescript/general.md); this file
covers `package.json` shape, scripts, and CI.

Package manager is pnpm — see `references/general/git-repository.md` for
the org-wide pnpm mandate and the Bun exception.

## Script naming: dot-namespaced variants

Name script variants of the same task with a `.` separator
(`verb.variant`), not a `-` or camelCase suffix. This is the single most
consistent convention across every audited package:

| Script | Purpose |
| --- | --- |
| `fmt` / `fmt.check` | Prettier write / check |
| `lint` / `lint.css` | ESLint / Stylelint |
| `build` / `build.client` / `build.server` / `build.types` | split build phases |
| `test.unit` / `test.browser` / `test.coverage` | test tiers (see below) |
| `check` | fast local gate — lint + typecheck + format-check |
| `check.ci` | full gate — `check` plus the full test suite |

Compose the gate scripts with [`concurrently`](https://www.npmjs.com/package/concurrently)
run in parallel-with-grouped-output mode, not a `&&` chain — a failing step
still lets the others finish and report, instead of stopping at the first
failure:

```json
{
  "scripts": {
    "fmt": "prettier --write ./src",
    "fmt.check": "prettier --check ./src",
    "lint": "eslint ./src",
    "lint.css": "stylelint \"src/**/*.{css,scss}\"",
    "build.types": "tsc --emitDeclarationOnly --incremental false",
    "check": "concurrently -g \"pnpm:fmt.check\" \"pnpm:lint\" \"pnpm:lint.css\" \"pnpm:build.types\"",
    "check.ci": "concurrently -g \"pnpm:check\" \"pnpm:test.unit\""
  }
}
```

`check` is what a contributor runs locally before pushing; `check.ci` is
what the CI workflow invokes. Keep both defined even if `check.ci` is
currently just `check` plus tests — it gives CI one stable entry point to
call regardless of how the fast/slow split evolves.

## `package.json` in a pnpm workspace

- **Root `package.json` does not need a `scripts` block.** Cross-package
  invocation goes through pnpm's own workspace flags
  (`pnpm --filter <pkg> <script>`, `pnpm run --recursive check`), not a
  hand-rolled root script that re-lists every package. Add a root script
  only for something that's genuinely repo-wide and not package-shaped
  (e.g. the markdownlint `lint` script from `git-repository.md`).
- **`engines` is declared only on packages that are actually published or
  deployed**, not on internal/private workspace members — an internal
  package inherits whatever Node version CI and contributors already use,
  so pinning it there is noise.
- When an `engines` range exists, **say why** next to it, not just what.
  A range exists because of a real constraint (a native dependency's
  Node-API version, a runtime feature) — document that constraint so the
  next person doesn't loosen the range without knowing what breaks:

  ```json
  {
    "engines": {
      "node": "^18.17.0 || ^20.3.0 || >=21.0.0"
    },
    "engines-annotation": "sharp requires Node-API v9+, available from these Node versions"
  }
  ```

## Type-safe clients from a live OpenAPI backend

When a TypeScript frontend consumes a Rust `axum` + `utoipa` backend (see
`references/rust/web-openapi.md`), generate the client's types directly
from the backend's own generated spec rather than hand-writing request/
response types a second time:

```json
{
  "scripts": {
    "generate:openapi": "openapi-typescript http://localhost:9000/api/v1/openapi.json -o src/openapi/schema.ts"
  },
  "dependencies": {
    "openapi-fetch": "^0.13.0"
  },
  "devDependencies": {
    "openapi-typescript": "^7.0.0"
  }
}
```

Point the URL at a locally running instance of the backend during
development. The generated file is checked in like any other generated
artifact (see the project's `.gitignore` conventions) or regenerated in CI
before the build, depending on how often the backend's schema actually
changes — pick whichever keeps schema drift visible in diffs rather than
silently absorbed.

## CI shape

- Pin a single Node version (`NODE_VERSION: 22.x` or similar) rather than a
  matrix, unless the package is a published library that explicitly
  supports a range of Node runtimes.
- Install pnpm via [`pnpm/action-setup`](https://github.com/pnpm/action-setup),
  then Node via `actions/setup-node` with `cache: "pnpm"`.
- Split into separate jobs per concern (`lint`, `format-check`, `build`,
  `test-unit`, `test-browser`, ...) rather than one job running every
  script in sequence — a failing lint job shouldn't hide whether the build
  still succeeds.

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 22.x
          cache: "pnpm"
      - run: pnpm install --frozen-lockfile
      - run: pnpm run lint

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 22.x
          cache: "pnpm"
      - run: pnpm install --frozen-lockfile
      - run: pnpm run test.unit
```

This is the target shape. It is not universally implemented yet — at least
one audited org repo's only Rust-adjacent CI workflow silently no-ops its
test step, and markdown/format checks in several repos run only through
editor extensions with no CI job at all. Wire the jobs above for real when
setting up a new repo rather than treating an existing repo's CI as proof
the gap is acceptable.
