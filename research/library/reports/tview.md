# tview Research Report

## Evidence Header

```text
Research date: 2026-07-16
Project version: tview v0.42.0
Project revision: 5ce6a2b588145610060000a4f75d7e2af081a794
Repository: https://github.com/rivo/tview
Documentation version: pkg.go.dev github.com/rivo/tview@v0.42.0; source README at pinned revision
Primary platform examined: Source inspection on Linux; no physical terminal reproduction
Report depth: Standard profile
```

The latest stable release at the start of this research was
[tview v0.42.0](https://github.com/rivo/tview/releases/tag/v0.42.0), released on
2025-08-27. Source conclusions refer to that tag unless a later development
revision is named. The current `master` snapshot examined separately was
`63ee97f9e01448f58772e29a005388f8c3b2e622` from 2026-03-16. It retained the
same module dependencies and did not add a test suite, so it does not change the
release-baseline conclusions.

## Executive Assessment

tview is a Go retained-widget and terminal application toolkit built on
[tcell](https://github.com/gdamore/tcell). Its central abstraction is the
mutable `Primitive`: an object that owns a rectangle, draws itself into a
`tcell.Screen`, reports focus, and supplies keyboard, mouse, and paste handlers.
`Application` adds a full-screen-oriented event loop, focus management, redraws,
serialized updates from goroutines, and terminal suspension. This puts tview
above a terminal substrate, but below a framework that owns application state,
effects, or deterministic settlement.

tview is a strong fit for Go dashboards, administrative tools, forms, file
browsers, log viewers, and interactive CLI clients. It is productive when the
application can keep durable widget objects and express behavior through
callbacks. Its standard widgets cover forms, text input, tables, lists, trees,
text views, text areas, grids, flex layouts, pages, and modals. The README lists
applications including K9s, `gh`, Podman TUI, and IRCCloud.

The same simplicity creates the relevant ArborUI boundary. tview clears and
redraws the root primitive tree for each requested frame; tcell may optimize
physical output, but tview has no prepared-frame transaction or dirty-subtree
contract. `QueueUpdate` serializes callbacks but does not own tasks, clocks,
cancellation, backpressure, or run-until-idle behavior. Headless testing is
possible through tcell's `SimulationScreen`, but tview itself provides no
application harness and no repository test suite at the examined revision.

## Project Snapshot

tview is a Go package with a Go 1.18 module. Direct dependencies are tcell
v2.8.1, `rivo/uniseg` v0.4.7, and `go-colorful` v1.2.0.
The package presents itself as a widget library with an optional application
wrapper: the other classes do not depend on `Application`. Its public surface
is intentionally fluent and mutable, and the README promises backward
compatibility while acknowledging that `Primitive` is a public interface with
internal-interface tradeoffs.

The project is an ecosystem component: K9s maintains a fork, while `gh` and
Podman TUI use upstream tview.

## Core Proposition

tview makes a terminal application look like a tree of durable controls. An
application creates widgets, puts them into layout containers, attaches
callbacks, and gives the root to `Application`. Retained objects preserve focus,
selection, scroll offsets, text buffers, and callbacks between draws. This is
different from a renderer that constructs a new visual description every frame,
but it is not a state-management framework: application state and goroutine
lifetimes remain in user code.

The distinction from raw tcell is composition and interaction. tcell supplies
cells, protocols, and `Screen`; tview supplies layout, widgets, focus, mouse
capture, forms, modal composition, and text editing. The application still
accepts tcell's screen and event contracts and tview's redraw choices.

## Architecture

### Primitives And Composition

The [`Primitive` interface](https://github.com/rivo/tview/blob/5ce6a2b588145610060000a4f75d7e2af081a794/primitive.go#L5-L69)
contains drawing, geometry, focus, keyboard, mouse, and paste methods. `Box` is
the common embeddable base: it owns the rectangle, background, border, title,
focus state, and input or mouse capture hooks. Custom controls usually embed
`*Box`, call `DrawForSubclass`, and implement the remaining behavior.

`Flex` and `Grid` retain children and assign rectangles during `Draw`. `Pages`
retains named primitives, draws visible pages back to front, and routes input to
the focused or topmost page. This makes overlays direct, but there is no keyed
reconciliation or disposal model. `Modal` illustrates the composition style: it
contains a frame and form rather than being a separate runtime session.

### Events, Focus, And Updates

`Application.Run` creates or accepts a tcell screen, draws once, polls terminal
events in a separate goroutine, and forwards them to the main loop. The loop
handles keys, paste, resize, mouse actions, and tcell errors. Keyboard input
starts at the root and follows the primitive hierarchy toward focus. Mouse
handlers can return a capturing primitive, and application or box-level capture
callbacks can intercept input.

`QueueUpdate` places a callback on a bounded queue of 100 entries and waits for
the application loop; `QueueUpdateDraw` adds a redraw. This is a useful
single-threaded mutation boundary, not an effect system. Tasks, cancellation,
clocks, and backpressure remain application policy.

### Rendering And Text

The normal draw path resizes a fullscreen root, calls `screen.Clear()`, invokes
`root.Draw(screen)`, and calls `screen.Show()`. Containers recursively draw
children; tcell owns the concrete screen and physical optimization, while tview
owns logical traversal and cell painting. `Application.Sync` exposes explicit
resynchronization for a known-corrupt screen.

Text processing is stronger than rune-counting. tview uses `uniseg.StepString`
for grapheme clusters and cell widths. `TextView` indexes wrapped lines lazily,
can cap retained lines, and implements `io.Writer`; `TextArea` uses spans for
editing, selection, undo, and cursor movement. Terminal and font behavior still
exceeds logical width tests.

### Terminal Ownership And Extension

Applications can inject any `tcell.Screen` with `SetScreen`, the main testing
escape hatch. Panic cleanup calls `Fini`; `Suspend` calls `screen.Suspend`, runs
a callback, and resumes. Resume errors are not represented in application state.

The extension boundary is explicit but low-level: implement a `Primitive`,
embed `Box`, provide `TableContent`, or supply a tcell screen. Replacing the
event loop, output transaction, or terminal mode usually bypasses `Application`.

## Core Strengths

### Productive Widget Composition

The built-in catalog addresses applications rather than only painting. `Form`
supplies fields, buttons, focus traversal, and finish callbacks. `Pages`,
`Modal`, `Table`, `List`, `TreeView`, `TextView`, `TextArea`, `InputField`,
`Grid`, and `Flex` cover common screens before custom cells are needed.

### Coherent Retained Interaction

Durable objects keep focus and editing state local. Focus delegation is part of
the primitive contract, and mouse capture supports drag-like interactions. This
is more complete than a renderer-only library without adopting an Elm or
React-style runtime.

### Useful Data And Rendering Escape Hatches

`TableContent` can expose a table backed by application data rather than the
default cell matrix. The draw path selects visible rows and columns unless
`SetEvaluateAllRows(true)` is requested, and the virtual-table demo demonstrates
the pattern. Custom primitives can write directly to tcell.

### Strong Logical Text Facilities

Grapheme segmentation, word wrapping, streaming text, bounded log buffers,
Unicode-aware cursor movement, and a multiline editor are practical advantages.
They do not prove universal terminal compatibility: [issue #1121](https://github.com/rivo/tview/issues/1121)
shows that edge cases remain, not a general Unicode failure.

## Limitations And Frustrations

### Output Is Not A Recoverable Transaction

```text
Classification: Limitation relative to ArborUI's recovery requirement
Requirement: Commit logical state only after complete output acceptance and repaint after uncertain output
Library assumption: tcell Screen.Show is sufficient for the current frame
Observable failure or friction: Show has no applied/deferred/unknown result; a partial write can leave screen state uncertain
Root architectural cause: tview clears and paints directly into a tcell screen, which exposes void Show and Sync operations
Available workaround: Call Application.Sync after known corruption, replace the screen, or terminate and reinitialize
Cost of workaround: The application cannot reliably distinguish a complete frame from an interrupted one
Upstream response: Sync is provided for user-requested repair; no automatic transaction or invalidation contract was found
Current status and version: Verified in v0.42.0
Evidence: Verified source boundary; physical partial-write behavior not reproduced
Confidence: High for the API boundary, medium for terminal impact
```

`Application.draw` calls `Clear`, draws the root, and calls `Show` without a
commit result. `Application.Sync` is an explicit recovery command, not a
response to a failed write. tcell's lifecycle discussions, including
[suspend issue #677](https://github.com/gdamore/tcell/issues/677) and
[suspend issue #779](https://github.com/gdamore/tcell/issues/779), reinforce that
terminal state transitions are an ecosystem boundary. This is acceptable for
applications that exit on output failure, but it does not meet ArborUI's
in-session recovery contract.

### Full Primitive Traversal Limits Large Updates

```text
Classification: Performance tradeoff
Requirement: Update very large trees or high-rate streams without repeating all logical paint work
Library assumption: Applications draw a complete visible primitive hierarchy when an update is needed
Observable failure or friction: Every requested draw clears and traverses the root; unchanged subtrees have no public dirty contract
Root architectural cause: Retained widget identity is separate from retained render invalidation
Available workaround: Avoid unnecessary draws, cap TextView lines, use TableContent, or write a custom primitive
Cost of workaround: Caching, virtualization, scroll anchoring, and synchronization move into application code
Upstream response: Table's custom content and virtual-table example provide local mitigations; no general dirty tree was found
Current status and version: Intentional behavior in v0.42.0
Evidence: Verified in Application, Flex, Pages, Table, and TextView source
Confidence: High for traversal; medium for workload cost
```

tcell can reduce physical output, so this is not a claim that every frame emits
the entire terminal. The logical layout and painting work still repeats. tview
mitigates common cases with lazy text indexing and visible table rows, but a
large dynamic tree or a rapid stream remains application-managed.

### QueueUpdate Is Not An Effect Or Settlement Model

```text
Classification: Tradeoff
Requirement: Serialize asynchronous effects with cancellation, backpressure, and deterministic settlement
Library assumption: Application goroutines own effects and enqueue mutations
Observable failure or friction: Producers, task lifetimes, redraw coalescing, and completion ordering are user policy
Root architectural cause: Application owns a callback queue, not an effect runtime
Available workaround: Add contexts, channels, worker ownership, explicit completion signals, and a test harness
Cost of workaround: Every application recreates lifecycle and run-until-idle conventions
Upstream response: QueueUpdate and the concurrency guidance define safe mutation, but no broader effect contract was found
Current status and version: Supported boundary in v0.42.0
Evidence: Verified Application queue and redraw API; total application cost inferred
Confidence: High for the boundary, medium for cost
```

The fixed queue and synchronous `QueueUpdate` call are useful primitives, not a
scheduler. A producer can block when the queue is full, and a callback can
outlive the widget it intends to update unless the application coordinates
cancellation. This is a deliberate small-runtime tradeoff rather than a bug.

## Testing Strategy

No `*_test.go` files, fuzz targets, benchmarks, test workflow, or application
harness were found in the v0.42.0 tree or inspected 2026 `master` snapshot.

tcell's versioned [`SimulationScreen`](https://pkg.go.dev/github.com/gdamore/tcell/v2@v2.8.1#SimulationScreen)
implements the production `Screen` boundary in memory. It injects key and mouse
events and exposes contents, making primitive rendering and basic event paths
testable without a terminal. `Application.SetScreen` uses the same tview draw
path, but the screen does not validate ANSI bytes, raw mode, cursor queries,
wrapping, suspend/resume, or partial writes.

Full application tests must supply a goroutine, stop signal, event injection,
synchronization, and `QueueUpdate` assertions. There is no clock control,
run-until-idle operation, settlement API, or fault-injecting screen. The testing
discussions in [#894](https://github.com/rivo/tview/issues/894) and
[#1060](https://github.com/rivo/tview/issues/1060) are consistent with this
boundary: a practical widget seam, but a weak framework-level contract.

## Common Scenario Assessment

| Scenario | tview assessment |
| --- | --- |
| Form with focus, validation, and modal | Strong widgets and focus callbacks; application owns validation and modal lifetime |
| Large scrollable collection | Table supports offsets and custom content; identity and virtualization policy remain application-owned |
| Streaming external updates | TextView and QueueUpdate help; producer lifecycle and redraw coalescing are user-owned |
| Unicode text input | Strong logical grapheme/editor support; physical terminal compatibility remains unproven |
| Overlay with clipping and mouse | Pages, Modal, Box, and mouse capture are supported; z-order is container policy |
| Resize during updates | Fullscreen roots resize on draw; fixed or custom regions need application coordination |
| Deferred or failed output | No outcome-aware transaction; explicit Sync is the repair escape hatch |
| Suspend to a child process | `Application.Suspend` provides a callback, with limited error/recovery reporting |
| Long idle periods | Efficient if the application blocks; tview does not schedule idle work |
| Native scrollback conversation | No standard high-level mode |

## Lessons For ArborUI

### Adopt Or Preserve

- Keep a small, composable widget contract and make custom controls practical.
- Provide a test screen that uses the production drawing path, with key, mouse,
  resize, and external-event injection.
- Offer data-source escape hatches for large collections and bounded streaming
  buffers instead of forcing all content into the default retained model.
- Invest in common form, table, text, modal, and layout widgets; architecture
  alone does not replace application-ready controls.
- Treat full-screen, inline, and native-scrollback modes as separate ownership
  contracts.

### Avoid Or Make Explicit

- Do not treat a void screen flush as successful frame acceptance.
- Do not clear logical dirty state until output acceptance is known, and force a
  full repaint after any uncertain write.
- Do not present a goroutine update queue as cancellation, backpressure, or
  deterministic effect settlement.
- Do not imply that a simulation screen proves PTY lifecycle, final-column
  behavior, or physical Unicode compatibility.

### ArborUI's Different Position

ArborUI already approaches output recovery, retained identity, borrowed element
lifetimes, runtime independence, and deterministic application testing more
explicitly than tview. Those guarantees have costs that tview avoids: more
framework policy, more state machinery, and a smaller mature widget ecosystem.
The comparison does not establish that ArborUI is faster, more Unicode-compatible
on real terminals, or cheaper for ordinary Go applications.

The next useful experiment is a matched application with a form, virtual
collection, streaming log, resize storm, and injected output failure. Compare
application code, tests, logical work, and recovery behavior against tview.

## Evidence Appendix

All sources were accessed on 2026-07-16. Source links are pinned to
`5ce6a2b588145610060000a4f75d7e2af081a794` unless noted.

| Claim | Source | Version or revision | Status | Notes |
| --- | --- | --- | --- | --- |
| Stable release baseline | [v0.42.0 release](https://github.com/rivo/tview/releases/tag/v0.42.0) | v0.42.0, 2025-08-27 | Verified | Latest stable release used for this report |
| Module and direct dependencies | [`go.mod`](https://github.com/rivo/tview/blob/5ce6a2b588145610060000a4f75d7e2af081a794/go.mod) | `5ce6a2b` | Verified | Go 1.18; tcell, uniseg, go-colorful |
| Project scope and applications | [README](https://github.com/rivo/tview/blob/5ce6a2b588145610060000a4f75d7e2af081a794/README.md#L1-L23) and [project list](https://github.com/rivo/tview/blob/5ce6a2b588145610060000a4f75d7e2af081a794/README.md#L56-L115) | `5ce6a2b` | Supported | README intent and listed users; adoption not measured |
| Primitive extension contract | [`Primitive`](https://github.com/rivo/tview/blob/5ce6a2b588145610060000a4f75d7e2af081a794/primitive.go#L5-L69) and [`Box`](https://github.com/rivo/tview/blob/5ce6a2b588145610060000a4f75d7e2af081a794/box.go#L7-L15) | `5ce6a2b` | Verified | Drawing and interaction are tcell-facing |
| Event loop and queue | [`Application.Run`](https://github.com/rivo/tview/blob/5ce6a2b588145610060000a4f75d7e2af081a794/application.go#L271-L380) and [`QueueUpdate`](https://github.com/rivo/tview/blob/5ce6a2b588145610060000a4f75d7e2af081a794/application.go#L865-L898) | `5ce6a2b` | Verified | Polling goroutine, bounded updates, callback serialization |
| Full redraw and explicit sync | [`Application.draw`](https://github.com/rivo/tview/blob/5ce6a2b588145610060000a4f75d7e2af081a794/application.go#L678-L763) | `5ce6a2b` | Verified | Clear, root draw, Show; Sync is explicit |
| Large-data escape hatches | [`TableContent`](https://github.com/rivo/tview/blob/5ce6a2b588145610060000a4f75d7e2af081a794/table.go#L223-L282), [`Table.Draw`](https://github.com/rivo/tview/blob/5ce6a2b588145610060000a4f75d7e2af081a794/table.go#L961-L1017), and [virtual-table demo](https://github.com/rivo/tview/blob/5ce6a2b588145610060000a4f75d7e2af081a794/demos/table/virtualtable/main.go) | `5ce6a2b` | Verified | Visible-row access unless all-row evaluation is enabled |
| Unicode and text editing | [`strings.step`](https://github.com/rivo/tview/blob/5ce6a2b588145610060000a4f75d7e2af081a794/strings.go#L71-L128), [`TextView`](https://github.com/rivo/tview/blob/5ce6a2b588145610060000a4f75d7e2af081a794/textview.go#L54-L88), and [`TextArea`](https://github.com/rivo/tview/blob/5ce6a2b588145610060000a4f75d7e2af081a794/textarea.go#L192-L250) | `5ce6a2b` | Verified | Logical grapheme processing; physical compatibility not tested |
| Headless screen boundary | [tcell `Screen`](https://pkg.go.dev/github.com/gdamore/tcell/v2@v2.8.1#Screen) and [tcell `SimulationScreen`](https://pkg.go.dev/github.com/gdamore/tcell/v2@v2.8.1#SimulationScreen) | tcell v2.8.1 | Supported | Same screen interface, logical inspection and event injection |
| Testing gaps | [Pinned tview tree](https://github.com/rivo/tview/tree/5ce6a2b588145610060000a4f75d7e2af081a794), [issue #894](https://github.com/rivo/tview/issues/894), and [issue #1060](https://github.com/rivo/tview/issues/1060) | Release tag; issues accessed 2026-07-16 | Inferred and reported | No tview test files or harness found; issue discussions are not a test result |
