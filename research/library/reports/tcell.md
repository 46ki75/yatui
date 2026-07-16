# tcell Research Report

## Evidence Header

```text
Research date: 2026-07-16
Project version: github.com/gdamore/tcell/v3 v3.4.0
Project revision: c67165c6c22b6758eb43209aaee45303f5b08b5b
Repository: https://github.com/gdamore/tcell
Documentation version: pkg.go.dev tcell/v3@v3.4.0; README and TUTORIAL at the pinned revision
Primary platform examined: Linux amd64 source and test inspection; no physical terminal reproduction
Report depth: Standard profile
```

The latest stable v3 module at the start of research was
[v3.4.0](https://github.com/gdamore/tcell/releases/tag/v3.4.0), published
2026-05-17. The Go module proxy records the exact tag-to-commit mapping in
[version metadata](https://proxy.golang.org/github.com/gdamore/tcell/v3/@v/v3.4.0.info).
The v2 line also has a parallel v2.13.9 release, but this report examines
v3.4.0; v1 is explicitly unmaintained.

## Snapshot And Core Proposition

tcell is a pure-Go terminal and console substrate. Its package documentation calls
the API lower-level and portable, while its [tutorial](https://github.com/gdamore/tcell/blob/c67165c6c22b6758eb43209aaee45303f5b08b5b/TUTORIAL.md#L1-L13)
says that users who need more structure should use or create a higher-level
framework. It owns terminal capability translation, a physical cell surface,
input decoding, raw-mode setup, and repainting. It does not own application
state, layout, widgets, focus routing, effects, or a scheduler. It should
therefore be compared with ArborUI's backend and terminal layers, not with
ArborUI as a complete application framework.

The strongest use case is a Go application or framework that wants direct control
over a full-screen terminal while retaining its own event loop and state model.
tcell supports POSIX systems, modern Windows, browser-backed WASM, and best-effort
Plan 9. The default screen enters the alternate screen, but
`OptAltScreen(false)` is available. Its scope is terminal integration, not a
retained UI tree or document/scrollback model.

## Architecture

### Screen, Cells, And Output

The public [`Screen` interface](https://github.com/gdamore/tcell/blob/c67165c6c22b6758eb43209aaee45303f5b08b5b/screen.go#L23-L260)
is central. Applications write content with `Put`, `PutStr`, `SetContent`, and
styles, then call `Show`; `Sync` invalidates every visible cell for repair.
`CellBuffer` keeps current and last strings/styles, width, and an optional lock
per cell. The screen is clipped to the physical size, not an unbounded document.

`Put` consumes one grapheme cluster, measures it with the current display-width
policy, and returns the remainder and width. v3.4.0 uses UAX 29 segmentation and
`displaywidth`; `RUNEWIDTH_EASTASIAN` selects East Asian width. Wide cells are
dirtied together, but writing into a continuation cell is documented as
undefined. A wide glyph in the final column becomes a single-width space. This
is strong logical behavior, not a promise that terminals and fonts agree about
emoji or ambiguous width.

`Show` locks the screen, checks size, and scans the grid. `drawCell` skips clean
cells, emits cursor/style transitions, writes the grapheme, and marks it clean.
The draw is bracketed by DEC synchronized-output mode when supported. `Sync`
clears and invalidates the buffer. `LockRegion` is an escape hatch for direct TTY
content such as sixel. There is no public damage tree, widget layout, native
history, or scrollback abstraction.

### Input, Lifecycle, And Extension

The TTY reader runs separately from the parser. Input is decoded as UTF-8 or
UTF-16 plus ECMA-48 sequences, with a 100 ms delay for a lone escape. `EventQ`
carries key, mouse, resize, paste, focus, clipboard, interrupt, and error events.
Applications can select on it and inject events by writing to the channel. Mouse,
focus, and paste are opt-in and capability-dependent. Legacy keyboards provide
press-style reports; advanced Kitty, XTerm, or Win32 protocols can add modifiers,
physical keys, repeats, and releases. Resize comes from SIGWINCH, Windows input,
or in-band reports.

`Init` starts the injected [`Tty` contract](https://github.com/gdamore/tcell/blob/c67165c6c22b6758eb43209aaee45303f5b08b5b/tty/tty.go#L19-L68),
enters raw mode, negotiates capabilities, enables the alternate screen, and
starts goroutines. `Fini` restores modes; `Suspend` and `Resume` disengage and
re-engage the TTY for a child. Panic cleanup is an application responsibility.
Extension points are a custom TTY or Screen, options and environment overrides,
`LockRegion`, `ShimScreen`, and the public-but-unstable `vt` interfaces. Layout,
widgets, propagation, and scheduling remain above this boundary.

## Core Strengths

### 1. Portable Terminal Capability Substrate

tcell handles variation between terminfo POSIX terminals, Windows console
input/output, and browser-backed WASM without CGO. Its TTY interface separates
terminal state from screen logic, and options provide escape hatches for keyboard
protocol, negotiation, mouse, color, alternate screen, and sanitization. The
[platform documentation](https://github.com/gdamore/tcell/blob/c67165c6c22b6758eb43209aaee45303f5b08b5b/README.md#L20-L37)
is candid about terminal and SSH degradation. This lets applications own their
loop without reimplementing raw mode, negotiation, and restoration.

### 2. Cell-Aware Rendering With Repair And Flicker Controls

The per-cell current/last baseline gives `Show` incremental output while
preserving a simple drawing API. `Sync` is an explicit recovery repaint, and
v3.4.0 brackets draws with synchronized output. Recent releases moved to faster
grapheme segmentation and fixed wide-character cases. This remains a
physical-cell model, but is materially stronger than rune-by-rune output or
untracked ANSI writes.

### 3. Production-Path Emulation And Useful Seams

The old `SimulationScreen` was removed in v3.0.6 in favor of `vt.NewMockTerm`,
which supplies a mock TTY, VT emulator, input injection, resize, focus, raw input,
and a cell-observable backend. Tests can construct the real
`NewTerminfoScreenFromTty` around it and inspect cells, styles, cursor, clipboard,
and protocol behavior. `ShimScreen` lets demos use that screen without changing
production construction. The `vt` package still labels its direct interfaces
work in progress.

## Limitations And Tradeoffs

### 1. Application Runtime Is Deliberately Outside The Contract

```text
Classification: Tradeoff
Requirement: Retained identity, focus routing, serialized model updates, effects, and deterministic full-application tests
Library assumption: The caller owns state, event policy, redraw policy, goroutines, and component architecture
Observable failure or friction: Forms, overlays, validation, hit testing, timers, cancellation, and backpressure require another layer
Root architectural cause: Screen is a physical canvas and EventQ is an event transport, not an application runtime
Workaround: Build a framework above Screen or adopt tview/another tcell consumer
Cost of workaround: Repeated conventions for focus, scheduling, settlement, and test drivers; cost was not measured
Upstream status: Intentional and current in v3.4.0
Evidence status: Verified; confidence high
```

The event channel makes asynchronous integration possible, but tcell does not
schedule tasks or serialize application mutations. `EventTime` timestamps events;
it does not provide controllable clocks, effects, cancellation, or run-until-idle.
The fixed event queue also makes consumption and backpressure the caller's
concern. This is not a defect in a substrate. It is why tcell fits synchronous,
goroutine-based, or framework-owned state models. It becomes an extension failure
only when an adopter expects tcell alone to provide complete application semantics.

### 2. Output And Suspension Are Not Transactional Recovery

```text
Classification: Limitation relative to ArborUI's recoverable-output requirement
Requirement: Commit physical-frame state only after complete output acceptance and recover from partial or unknown writes
Library assumption: A terminal write is best effort; applications can call Sync or end the session after corruption
Observable failure or friction: Show returns no error; a short or failed TTY write is not surfaced, while dirty cells are marked clean before WriteTo completes
Root architectural cause: Screen exposes void Show/Sync and tScreen ignores Write errors and byte counts
Workaround: Terminate and restore after suspected failure, call Sync when corruption is known, or implement a replacement Screen with transactional state
Cost of workaround: No automatic detection, no standard fault injection, and a custom Screen rather than merely a custom Tty
Upstream status: Current behavior in v3.4.0; synchronized output is present, but it is not write acknowledgement
Evidence status: Verified from source; confidence high
```

The current implementation is better than the historical v2 discussion of resize
flicker: [issue #797](https://github.com/gdamore/tcell/issues/797) is closed, and
v3.4.0 emits DEC 2026 synchronized-output boundaries. That protects presentation
on capable terminals, not the application from a broken pipe, short write, or
uncertain physical cursor. `Suspend` and `Resume` restore TTY modes, but
[issue #779](https://github.com/gdamore/tcell/issues/779) records nondeterministic
SIGSTOP/foreground handoff symptoms and was closed as application/signal
coordination rather than a tcell bug. The application must coordinate SIGCONT,
child ownership, and repaint policy.

### 3. Physical Canvas Semantics Do Not Provide Native Scrollback Or Universal Widths

```text
Classification: Limitation and compatibility tradeoff
Requirement: Native-scrollback conversations and stable grapheme placement across terminals, fonts, and width policies
Library assumption: Applications render the visible physical grid and accept terminal capability and width differences
Observable failure or friction: No logical history or inline-region contract; wide-cell overlap is undefined and ambiguous width remains environment/terminal dependent
Root architectural cause: CellBuffer is bounded to the current Screen size and the output model positions cells directly on the terminal
Workaround: Retain history in the application, render a viewport, use OptAltScreen(false) only when its main-screen behavior is acceptable, and test target terminals
Cost of workaround: Duplicate scrolling/history ownership; it cannot turn the Screen API into a native scrollback protocol
Upstream status: Intentional scope; concrete Unicode bugs continue to be fixed
Evidence status: Supported by source, docs, and historical discussions; confidence high for scrollback absence and medium for cross-terminal width behavior
```

The old [scrolling discussion](https://github.com/gdamore/tcell/issues/77) describes
the screen as a canvas whose scrolling belongs in a higher layer. The current API
still has no history or viewport operation. [Issue #976](https://github.com/gdamore/tcell/issues/976)
demonstrates why Unicode remains an ecosystem boundary: v3.1.1 fixed the reported
wide overwrite case, but terminal emulators still disagree about grapheme and
wide-cell behavior. tcell has strong logical segmentation and width handling,
but physical compatibility needs a terminal matrix.

## Testing Strategy

tcell has unusually good substrate-level testing. [`cell_test.go`](https://github.com/gdamore/tcell/blob/c67165c6c22b6758eb43209aaee45303f5b08b5b/cell_test.go#L19-L145)
checks grapheme placement, wide-cell dirtying, style inheritance, locks, resize,
fill, and sanitization. [`input_test.go`](https://github.com/gdamore/tcell/blob/c67165c6c22b6758eb43209aaee45303f5b08b5b/input_test.go#L29-L136)
exercises control bytes, UTF-8, fragmented parser states, special keys,
modifiers, and concurrent parser access. Mouse, focus, resize, cursor, style,
and lifecycle tests exercise the public screen surface. The `vt/tests` package
drives the emulator with exact escape sequences and checks
cursor movement, colors, scroll regions, insert/delete, mouse encodings,
keyboard layouts, repeats, and focus. These are parser and emulator tests, not
just fake `Screen` tests.

The recommended complete-screen seam is `vt.NewMockTerm` plus
`NewTerminfoScreenFromTty`. Demo tests run production `main` functions through
that screen and assert cells and styles; clipboard tests inject keys into the
real input path. `SendRaw` supports malformed-input testing. I ran
`go test ./...` at v3.4.0 on Linux with Go 1.25.6; all packages passed. CI tests
stable and oldstable Go on Linux, macOS, and Windows, builds WASM, and uploads
coverage.

The gaps matter for ArborUI. No dedicated PTY or external terminal-emulator
compatibility suite was found at the pinned revision. `TestInitScreen` requires a
working TTY and skips otherwise; the Unicode demo test says correctness still
needs manual real-terminal validation. Mock writes succeed, so partial writes,
backpressure, disconnect, and recovery after unknown physical state are not
exercised. No Go fuzz target or deterministic full-application harness was found.
Demo tests use goroutines and sleeps rather than controlled clocks or settlement.
A complete application can run headlessly through the production screen and
input parser, but its model, scheduler, focus policy, and effects remain
application test infrastructure. PTY tests are still required for raw mode,
alternate-screen restoration, suspension, signals, and platform behavior.

## Common Scenario Relevance

| Scenario | tcell assessment |
| --- | --- |
| Form with focus and modal | Cell composition and cursor are supported; focus, validation, routing, and modal identity are application-owned. |
| Large scrollable collection | Render a visible slice into the grid; stable identity, virtualization, and history are outside tcell. |
| Streaming external updates | `EventQ` and Go channels integrate well; serialization, coalescing, clocks, and backpressure are caller policy. |
| Unicode text input | Grapheme-aware output is strong; editing, selection, cursor movement, and terminal width compatibility need higher layers. |
| Overlay with mouse | Mouse and focus events are available when supported; hit testing, capture, clipping policy, and dispatch are not. |
| Resize during updates | Resize events and `Sync` are supported; the app must settle concurrent state and redraw. |
| Deferred or failed output | Synchronized output reduces flicker, but `Show` has no accepted/partial/unknown result. |
| Suspension to a child | `Suspend`/`Resume` exists; signal timing, child TTY ownership, and post-resume repaint remain application work. |
| Long idle periods | Blocking on `EventQ` is cheap; timers and redraw scheduling are not supplied. |
| Native scrollback conversation | Not a base Screen mode; retain history and define a separate inline/native-scrollback contract. |

## Lessons For ArborUI

ArborUI should adopt tcell's substrate boundary, terminal injection seam,
capability overrides, grapheme-aware cells, synchronized output, and
emulator-backed production-path tests. A single input owner should route protocol
responses, resize, focus, paste, and application events before higher-level
dispatch. A compact `Tty` contract can support real and virtual terminals without
forcing a runtime.

ArborUI should preserve guarantees tcell does not promise: prepared-frame commit
only after backend acceptance, physical-state invalidation after uncertain write,
retained identity for focus and hit targets, deterministic event/time/effect
settlement, and explicit alternate-screen versus inline/native-scrollback modes.
It should not criticize tcell for lacking widgets or scheduling; those choices
make it embeddable. It should prove that stronger application semantics reduce
total code without making simple integrations harder.

Follow-up work should compare ArborUI against a tcell-plus-runtime application,
add a failing-writer and PTY matrix, test wide-cell and final-column cases across
emulators, and measure event-queue behavior under large paste and streaming load.
The tcell emulator is a good oracle for protocol and cell behavior, but cannot
establish physical recovery or universal Unicode rendering.

## Evidence Appendix

All sources were accessed on 2026-07-16. Source links use the full v3.4.0 commit wherever the source is version-sensitive.

| Claim | Source | Version or revision | Source date | Accessed | Status | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| Stable module and exact revision | [Go proxy metadata](https://proxy.golang.org/github.com/gdamore/tcell/v3/@v/v3.4.0.info) and [v3.4.0 release](https://github.com/gdamore/tcell/releases/tag/v3.4.0) | v3.4.0, `c67165c6c22b6758eb43209aaee45303f5b08b5b` | 2026-05-17 | 2026-07-16 | Verified | Latest stable v3 baseline; v2.13.9 is a parallel line. |
| Substrate intent and platform scope | [doc.go](https://github.com/gdamore/tcell/blob/c67165c6c22b6758eb43209aaee45303f5b08b5b/doc.go#L15-L23), [README](https://github.com/gdamore/tcell/blob/c67165c6c22b6758eb43209aaee45303f5b08b5b/README.md#L1-L13), [Windows](https://github.com/gdamore/tcell/blob/c67165c6c22b6758eb43209aaee45303f5b08b5b/README-windows.md#L1-L35), [WASM](https://github.com/gdamore/tcell/blob/c67165c6c22b6758eb43209aaee45303f5b08b5b/README-wasm.md#L1-L28), [Plan 9](https://github.com/gdamore/tcell/blob/c67165c6c22b6758eb43209aaee45303f5b08b5b/README-plan9.md#L1-L23) | `c67165c` | 2026-05-17 | 2026-07-16 | Verified | Lower-level API; POSIX, Windows, WASM, and best-effort Plan 9. |
| Screen, cell, and lifecycle contracts | [screen.go](https://github.com/gdamore/tcell/blob/c67165c6c22b6758eb43209aaee45303f5b08b5b/screen.go#L23-L260), [cell.go](https://github.com/gdamore/tcell/blob/c67165c6c22b6758eb43209aaee45303f5b08b5b/cell.go#L46-L221), [tscreen.go](https://github.com/gdamore/tcell/blob/c67165c6c22b6758eb43209aaee45303f5b08b5b/tscreen.go#L1459-L1663) | `c67165c` | 2026-05-17 | 2026-07-16 | Verified | Physical cells, Show/Sync, Init/Fini, Suspend/Resume, alternate screen. |
| Unicode, output, and input behavior | [width policy](https://github.com/gdamore/tcell/blob/c67165c6c22b6758eb43209aaee45303f5b08b5b/internal/widthutil/widthutil.go#L15-L31), [draw pipeline](https://github.com/gdamore/tcell/blob/c67165c6c22b6758eb43209aaee45303f5b08b5b/tscreen.go#L803-L1031), [input options](https://github.com/gdamore/tcell/blob/c67165c6c22b6758eb43209aaee45303f5b08b5b/tscreen.go#L77-L131) | `c67165c` | 2026-05-17 | 2026-07-16 | Verified | Graphemes, dirty diff, DEC 2026, keyboard/mouse/focus/paste options; write results are ignored. |
| TTY replacement and current test seam | [Tty](https://github.com/gdamore/tcell/blob/c67165c6c22b6758eb43209aaee45303f5b08b5b/tty/tty.go#L19-L68), [MockTerm](https://github.com/gdamore/tcell/blob/c67165c6c22b6758eb43209aaee45303f5b08b5b/vt/mock.go#L98-L238), [demo test](https://github.com/gdamore/tcell/blob/c67165c6c22b6758eb43209aaee45303f5b08b5b/demos/hello/hello_test.go#L27-L109) | `c67165c` | 2026-05-17 | 2026-07-16 | Verified | Real screen path over a virtual terminal with event and cell assertions. |
| Emulator, CI, and compatibility evidence | [Linux CI](https://github.com/gdamore/tcell/blob/c67165c6c22b6758eb43209aaee45303f5b08b5b/.github/workflows/linux.yml#L9-L42), [VT tests](https://github.com/gdamore/tcell/blob/c67165c6c22b6758eb43209aaee45303f5b08b5b/vt/tests/event_test.go#L23-L76), [Unicode test](https://github.com/gdamore/tcell/blob/c67165c6c22b6758eb43209aaee45303f5b08b5b/demos/unicode/unicode_test.go#L26-L68) | `c67165c` | 2026-05-17 | 2026-07-16 | Verified | Cross-platform CI and protocol tests; Unicode physical accuracy remains manual. |
| Release history and shipped fixes | [v3.0.6](https://github.com/gdamore/tcell/releases/tag/v3.0.6), [v3.1.1](https://github.com/gdamore/tcell/releases/tag/v3.1.1), [v3.3.0](https://github.com/gdamore/tcell/releases/tag/v3.3.0), [v3.4.0](https://github.com/gdamore/tcell/releases/tag/v3.4.0) | `61b0903`, `c7a03e2`, `29b8586`, `c67165c` | 2025-12-31 through 2026-05-17 | 2026-07-16 | Verified | SimScreen removal, wide-character fix, faster graphemes, advanced keys and WASM updates. |
| Maintainer discussions and scope boundaries | [#797](https://github.com/gdamore/tcell/issues/797), [#779](https://github.com/gdamore/tcell/issues/779), [#976](https://github.com/gdamore/tcell/issues/976), [#77](https://github.com/gdamore/tcell/issues/77), [#262](https://github.com/gdamore/tcell/issues/262) | Historical and current discussions; all closed | 2015-11-11 through 2025-04-23 | 2026-07-16 | Reported/supported | Sync work shipped; suspend was treated as signal coordination; scrolling remains higher-layer; Unicode issue received a release fix. |
