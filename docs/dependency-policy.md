# Dependency And Security Policy

CI audits every Cargo dependency graph with `cargo-deny`. The shared
[`deny.toml`](../deny.toml) policy applies to the main workspace, the separate
fuzz workspace, and the Collection Lab comparison workspace.

## Enforced Checks

- RustSec advisories and soundness notices fail the check. Unmaintained-package
  notices apply to workspace crates so an abandoned direct foundation cannot
  become an accidental release dependency.
- Runtime, build, and development dependency licenses must be in the explicit
  permissive-license allowlist. Additions require review of both the SPDX
  expression and the package's role. The shared allowlist is the union needed
  by all three dependency graphs, so a license need not appear in every
  individual workspace.
- Wildcard registry dependency requirements are denied. Path-only dependencies
  between repository packages are allowed. Duplicate versions are reported as
  warnings because transitive graphs can require them; each warning should be
  reviewed rather than bypassed globally.
- Unknown registries and Git sources are denied. Current dependencies must come
  from crates.io or local workspace paths.
- Every check uses `--locked`; stale or missing lockfiles fail instead of
  silently resolving a different graph in CI.

The configured target graph covers the Linux, macOS, and Windows environments
in the compatibility matrix, including target-specific dependencies that are
not active on the developer's host.

## Local Verification

Install the pinned policy tool and run the repository recipe:

```console
cargo +1.90.0 install cargo-deny --version 0.20.2 --locked
just deny
```

`cargo-deny` 0.20.2 requires Rust 1.88 or newer to install. This does not change
ArborUI's Rust 1.85.0 MSRV: the audit tool inspects Cargo metadata and lockfiles,
while library compilation and normal CI continue to use the pinned MSRV.

## Exceptions

Do not ignore an advisory or add a license/source exception without documenting
the affected crate, scope, rationale, compensating controls, and review date in
this file. Prefer updating or replacing the dependency. Temporary exceptions
must identify an owner and an expiration condition.

Current exception:

- `libfuzzer-sys` 0.4.13 requires NCSA in addition to MIT and Apache-2.0. NCSA
  is a permissive license and this crate is confined to the unpublished fuzz
  workspace. Project maintainers own the exception; review it when the pinned
  crate version changes. Last reviewed: 2026-07-17.
