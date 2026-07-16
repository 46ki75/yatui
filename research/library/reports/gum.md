# Gum Research Survey

## Evidence Header

```text
Research date: 2026-07-16
Project version: gum v0.17.0
Project revision: 6045525ab92f75c169d3c69596844d8748437e37 (tag v0.17.0)
Repository: https://github.com/charmbracelet/gum
Documentation version: tagged README and package comments at v0.17.0; accessed 2026-07-16
Primary platform examined: Tagged source and tests on Linux; CI and release platform claims checked
Report depth: Brief survey
```

## Project And Version

The latest stable release available on the research date is Gum `v0.17.0`,
released 2025-09-05. The Go module declares Go 1.23.0 and a Go 1.24.1 toolchain.
Its CI builds and tests on Ubuntu, macOS, and Windows; the tagged README lists
release binaries for Linux, macOS, Windows, FreeBSD, OpenBSD, and NetBSD. Gum is
implemented as a Go command, not a general-purpose application library.

## Category And Scope

Gum is a **shell-facing CLI helper built from Charm components**, in the
presentation or CLI toolkit category. It exposes commands for prompts and
selectors (`input`, `write`, `confirm`, `choose`, `filter`, `file`, and `table`),
bounded display (`pager`, `spin`), and Lip Gloss-based styling, joining, and
formatting. Each command creates a short-lived Bubble Tea program when it needs
interactive input. Shell pipelines, stdout, exit status, and environment flags
are the composition API; Gum is not a retained application framework.

## Distinctive Strength

Gum makes polished shell UX composable without Go: selectors and prompts return
values on stdout, confirmation maps naturally to exit status, and most UI is
rendered on stderr. `gum spin -- command` is the strongest integration seam. It
passes stdin to the child, uses PTYs and concurrent copies when a Unix terminal
is available, can display captured stdout/stderr while the command runs, and can
emit the selected output after success. When not attached to a TTY it passes the
child's stdout and stderr through instead. This is practical subprocess UX,
rather than an application event-stream abstraction.

The implementation also pays attention to terminal width: `table` uses
`lipgloss.Width`, while `filter` uses `rivo/uniseg` to translate fuzzy-match byte
ranges into visible grapheme positions. This supports common Unicode composition
without claiming physical-terminal or universal cell-width correctness.

## Important Limitation Or Tradeoff

**Classification:** Intentional scope limitation. **Requirement:** ArborUI needs
one long-running terminal owner, retained interaction identity, serialized
external effects, recoverable frame commits, and a deterministic complete-app
harness. **Library assumption:** Gum owns one bounded command invocation at a
time and lets the shell compose multiple invocations. Most commands render to
stderr and return a final value on stdout; `pager` always requests the alternate
screen, while `filter` requests it only for automatic-height use. The pinned Gum
source exposes no shared retained tree, prepared-frame commit, physical-screen
invalidation, or application-level event/test driver. The workaround is to build
a Bubble Tea application using the underlying Charm components or coordinate Gum
as child processes through a PTY. The cost is repeated terminal lifecycle ownership,
process-boundary state transfer, and application-owned recovery and testing.

## Testing Observation

At the recorded tag, the only repository `_test.go` file is
`filter/filter_test.go`; it tests pure matching-range and Unicode-position
helpers. CI runs `go test -v -cover -timeout=30s ./...` on three operating
systems, but a complete-application harness, injected key/mouse/resize or
external events, controlled clocks, virtual-terminal assertions, PTY lifecycle
tests, and partial-write fault injection were not found at that revision. The
manual `examples/test.sh` script is smoke coverage, not a production test
boundary.

## Relevance To ArborUI

Gum is an adjacent reference, not a direct full-screen competitor. ArborUI
should borrow its stdout/stderr separation, explicit exit semantics, timeout and
abort behavior, non-TTY degradation, subprocess wrapper, and width-aware text
handling for a possible presentation mode. It should not treat Gum's prompts,
spinners, alternate-screen pager, or subprocess streaming as evidence for a
single retained runtime, transactional output recovery, or complete application
testing.

## Best Sources

All sources below were accessed on 2026-07-16.

| Claim | Primary immutable source | Source date |
| --- | --- | --- |
| Stable release, tag, and commit | [Go module version metadata](https://proxy.golang.org/github.com/charmbracelet/gum/@v/v0.17.0.info); [tagged commit](https://github.com/charmbracelet/gum/commit/6045525ab92f75c169d3c69596844d8748437e37) | 2025-09-05 |
| Intent, commands, Go/platform scope | [Tagged README](https://github.com/charmbracelet/gum/blob/6045525ab92f75c169d3c69596844d8748437e37/README.md); [`go.mod`](https://github.com/charmbracelet/gum/blob/6045525ab92f75c169d3c69596844d8748437e37/go.mod) | 2025-09-05 |
| Component boundary and command lifecycle | [`gum.go`](https://github.com/charmbracelet/gum/blob/6045525ab92f75c169d3c69596844d8748437e37/gum.go#L24-L228); [`filter` command](https://github.com/charmbracelet/gum/blob/6045525ab92f75c169d3c69596844d8748437e37/filter/command.go#L49-L174); [`pager` command](https://github.com/charmbracelet/gum/blob/6045525ab92f75c169d3c69596844d8748437e37/pager/command.go#L47-L60) | 2025-09-05 |
| Subprocess UX and external output | [`spin` implementation](https://github.com/charmbracelet/gum/blob/6045525ab92f75c169d3c69596844d8748437e37/spin/spin.go#L68-L131); [`spin` command](https://github.com/charmbracelet/gum/blob/6045525ab92f75c169d3c69596844d8748437e37/spin/command.go#L17-L81) | 2025-09-05 |
| Unicode handling and tests | [`filter` Unicode path](https://github.com/charmbracelet/gum/blob/6045525ab92f75c169d3c69596844d8748437e37/filter/filter.go#L216-L225); [`filter` tests](https://github.com/charmbracelet/gum/blob/6045525ab92f75c169d3c69596844d8748437e37/filter/filter_test.go); [pinned CI](https://github.com/charmbracelet/gum/blob/6045525ab92f75c169d3c69596844d8748437e37/.github/workflows/build.yml#L5-L29) | 2025-09-05 |
