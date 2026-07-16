# PTerm Research Report

## Evidence Header

```text
Research date: 2026-07-16
Project version: github.com/pterm/pterm v0.12.83
Project revision: ed296ee75a70b5bb062438995e147aa114c2d539
Repository: https://github.com/pterm/pterm
Documentation version: pkg.go.dev v0.12.83 and docs/index.html at the pinned revision; accessed 2026-07-16
Primary platform examined: Linux source and test inspection; platform claims and CI checked; no physical terminal reproduction
Report depth: Brief survey
```

## Project And Version

PTerm is a Go 1.24 module for making CLI output attractive and informative. The
Go proxy identifies `v0.12.83` as the latest tagged release available on the
research date, published 2026-02-25, and maps the tag to the revision above.
The tagged `go.mod` requires Go 1.24.0 and a Go 1.24.3 toolchain. The README
advertises Windows CMD, macOS iTerm2, Linux, and CI environments; the pinned
workflow builds and tests on Windows and runs lint on Ubuntu, but has no macOS
test job.

## Category And Scope

PTerm belongs in the presentation or CLI toolkit category, not among retained
full-screen application frameworks. Its public model is a set of composable
printers: `TextPrinter` formats and writes text, `RenderPrinter` can render or
return a string with `Srender`, and `LivePrinter` starts and stops an updating
display. The catalog covers boxes, panels, charts, trees, logs, progress bars,
spinners, and tables with headers, boxes, alignment, and multiline cells.
Interactive printers provide confirmation, continue, select, multiselect, and
text-input prompts, including filtering and multiline input. `AreaPrinter` can
refresh a cursor region and has a fullscreen-sized option, but this is bounded
live output rather than a retained scene or alternate-screen application model.

## Distinctive Strength

PTerm makes common CLI presentation tasks concise while retaining useful
composition seams. Render-to-string APIs allow a table, panel, or chart to be
embedded in another printer or captured for an assertion; many printers accept
an `io.Writer`, and `SetDefaultOutput` redirects ordinary output. Spinners and
progress bars handle live status without requiring an application framework.
Its width-aware formatting strips terminal escapes and uses `go-runewidth` for
table and prompt sizing, so common wide characters are considered. This is
useful display-width handling, not evidence of a grapheme-level cell model or
universal physical-terminal behavior.

## Important Limitation Or Tradeoff

**Classification:** Tradeoff and intentional scope boundary. **Requirement:**
ArborUI targets long-running stateful applications with one terminal owner,
recoverable frame commits, and deterministic complete-application tests.
**Library assumption:** PTerm components can update bounded output and delegate
input to the terminal. Default text output is `os.Stdout`, live defaults use
writers such as `os.Stderr`, terminal size is queried from the stdout file
descriptor, and live components use cursor control, goroutines or scheduling,
and `keyboard.Listen`. `Stop` clears or resolves a component; the package does
not expose a prepared-frame backend, applied/deferred/unknown write outcome, or
physical-screen invalidation and recovery contract. **Workaround and cost:**
Use `Srender` with an application-owned renderer and event loop, or adopt a
full-screen framework. That keeps PTerm useful for bounded prompts and progress
displays, but shifts focus, retained identity, scheduling, resize, terminal
restoration, uncertain-write recovery, and their tests to the application.

## Testing Observation

The pinned tree has focused tests for nearly every printer, width helpers,
output/styling flags, forced terminal dimensions, and interface conformance.
`Srender`, writer injection, and forced dimensions support deterministic
component-level assertions without a real terminal. No public complete-app
harness, virtual terminal, PTY lifecycle suite, controlled clock, event driver,
or partial-write fault injection was found in the pinned tree or workflows.
Testing is therefore useful for bounded presentation logic but does not exercise
the full production path of a retained application or physical recovery.

## Relevance To ArborUI

PTerm is an adjacent reference, not a direct ArborUI competitor. ArborUI can
borrow its printer composition, render-to-string escape hatch, writer seams,
width-aware formatting, and ergonomic progress/prompt components for a future
presentation mode. It should not treat PTerm's cursor areas or `WithFullscreen`
as evidence for ArborUI's alternate-screen ownership, Unicode invariants,
transactional output, lifecycle recovery, or complete-app testing boundary.

## Best Sources

All sources below were accessed on 2026-07-16.

| Claim | Primary immutable source | Source date |
| --- | --- | --- |
| Release, tag, and exact commit | [Go proxy version metadata](https://proxy.golang.org/github.com/pterm/pterm/@v/v0.12.83.info); [pinned release tree](https://github.com/pterm/pterm/tree/ed296ee75a70b5bb062438995e147aa114c2d539) | 2026-02-25 |
| Intent, platforms, and component catalog | [README](https://github.com/pterm/pterm/blob/ed296ee75a70b5bb062438995e147aa114c2d539/README.md); [pinned documentation](https://github.com/pterm/pterm/blob/ed296ee75a70b5bb062438995e147aa114c2d539/docs/index.html) | 2026-02-25 |
| Printer composition and live behavior | [`RenderPrinter`](https://github.com/pterm/pterm/blob/ed296ee75a70b5bb062438995e147aa114c2d539/interface_renderable_printer.go), [`LivePrinter`](https://github.com/pterm/pterm/blob/ed296ee75a70b5bb062438995e147aa114c2d539/interface_live_printer.go), [`AreaPrinter`](https://github.com/pterm/pterm/blob/ed296ee75a70b5bb062438995e147aa114c2d539/area_printer.go) | 2026-02-25 |
| Terminal, Unicode, and testing boundaries | [`print.go`](https://github.com/pterm/pterm/blob/ed296ee75a70b5bb062438995e147aa114c2d539/print.go), [`terminal.go`](https://github.com/pterm/pterm/blob/ed296ee75a70b5bb062438995e147aa114c2d539/terminal.go), [`width helper`](https://github.com/pterm/pterm/blob/ed296ee75a70b5bb062438995e147aa114c2d539/internal/max_text_width.go), [`tests`](https://github.com/pterm/pterm/blob/ed296ee75a70b5bb062438995e147aa114c2d539/pterm_test.go), [CI workflow](https://github.com/pterm/pterm/blob/ed296ee75a70b5bb062438995e147aa114c2d539/.github/workflows/go.yml) | 2026-02-25 |
| Progress, spinner, table, and interactive APIs | [`progressbar_printer.go`](https://github.com/pterm/pterm/blob/ed296ee75a70b5bb062438995e147aa114c2d539/progressbar_printer.go), [`spinner_printer.go`](https://github.com/pterm/pterm/blob/ed296ee75a70b5bb062438995e147aa114c2d539/spinner_printer.go), [`table_printer.go`](https://github.com/pterm/pterm/blob/ed296ee75a70b5bb062438995e147aa114c2d539/table_printer.go), [`interactive_select_printer.go`](https://github.com/pterm/pterm/blob/ed296ee75a70b5bb062438995e147aa114c2d539/interactive_select_printer.go) | 2026-02-25 |
