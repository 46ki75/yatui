# OpenTUI Research Report

## Evidence Header

```text
Research date: 2026-07-16
Project version: OpenTUI v0.4.3 (@opentui/core, @opentui/react, @opentui/solid)
Project revision: 5803b2cfa2942c45a3aedbb3601754e27f2cdc68
Repository: https://github.com/anomalyco/opentui
Documentation version: main docs at 4d5b7c1ad9d444531d0e0907f3b3926e12be318d
Primary platform examined: Source inspection on Linux; no physical terminal reproduction
Report depth: Deep dive
```

The default baseline is the [`v0.4.3` release](https://github.com/anomalyco/opentui/releases/tag/v0.4.3), published on 2026-07-03. The repository was also inspected at the later `main` revision named
above when current documentation or post-release behavior was relevant. Issue and pull-request evidence is labeled separately rather than silently treated as behavior shipped in the release.

## Executive Assessment

OpenTUI is an application-oriented TUI framework built around a native Zig renderer and TypeScript APIs. It combines a retained renderable tree, Yoga layout, a cell compositor, Unicode-aware text
storage, terminal capability detection, input protocol parsing, lifecycle control, and framework bindings for React and Solid. That makes it substantially more complete than a rendering-only library
and explains why it is a good fit for OpenCode's production terminal application.

Its central proposition is not a particular state architecture. The core owns the difficult terminal-facing work while allowing applications to choose imperative renderables, lightweight declarative
constructs, React, or Solid. The core renderer is retained and imperative; the React and Solid packages provide the declarative state model. Application state, network effects, command routing, and
most asynchronous coordination remain outside OpenTUI.

The strongest parts are the native cell pipeline, the explicit separation of screen modes, a serious byte-level input parser, and a headless test renderer that drives the real `CliRenderer` with
native memory output. The project also demonstrates practical recovery work: feed backpressure is modeled, failed native frame assembly forces a later repaint, resize invalidates hit grids, and resume
forces a full repaint.

The important qualification is that OpenTUI's physical-screen contract is weaker than ArborUI's proposed transactional contract. The native diff renderer maintains an intended cell shadow state and
updates it as output is generated. A direct stdout writer does not report whether the terminal accepted a complete sequence, and the public API does not expose a normal operation for declaring the
physical screen unknown and requesting a full repaint. Real reports describe stale cells and cursor/width desynchronization in terminal-specific conditions. OpenTUI has useful mitigations, but they do
not establish commit-after-acceptance semantics.

OpenTUI is therefore a strong choice for Bun-first TypeScript applications that want a full-stack terminal renderer and are willing to track a fast-moving native dependency. It is not evidence that
ArborUI's stronger recovery, backend independence, or deterministic application-runtime guarantees are unnecessary. It is evidence that those guarantees must remain easy to use, and that they need to
coexist with a practical component ecosystem and a low-friction headless harness.

## Project Snapshot

OpenTUI is MIT-licensed TypeScript with a native Zig core. The core package describes itself as a TypeScript library on a native Zig core, ships platform-specific optional packages, and requires Bun
`>=1.3.0` for its ordinary package contract. The release pins Zig `0.15.2`; the native build compiles Zig, C, and C++ sources, including Yoga, and links platform-specific system libraries for audio
and terminal support. The build file lists Linux glibc and musl, macOS Intel and Apple Silicon, and Windows Intel and ARM targets. See the
[release package metadata](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/package.json#L1-L99),
[Zig version](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/.zig-version#L1), and
[native build targets and dependencies](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/zig/build.zig#L10-L30).

The repository presents `@opentui/core` as the imperative/native API, with React and Solid bindings plus packages for keymaps, SSH, QR codes, and Three.js. The native core exposes a C ABI, but the
supported high-level experience is TypeScript and Bun-oriented. Node can import portable entry points without FFI; creating a native renderer requires Node 26.3.0 with experimental FFI.

OpenTUI powers OpenCode in production and is intended for terminal.shop. OpenCode's current TUI depends on `@opentui/core`, `@opentui/solid`, and `@opentui/keymap`, starts the UI in a worker, and uses
split-footer and scrollback-surface concepts. Its own coalescing queues, lifecycle cleanup, and terminal workarounds show that application semantics remain outside OpenTUI.

## Core Proposition

OpenTUI makes a terminal application look more like a small native UI system than a sequence of ANSI writes. An application creates a `CliRenderer`, adds a root renderable tree, mutates renderables or
reconciles a framework tree, and lets the renderer measure, paint, diff, and emit the result. The
[renderer documentation](https://github.com/anomalyco/opentui/blob/4d5b7c1ad9d444531d0e0907f3b3926e12be318d/packages/web/src/content/docs/core-concepts/renderer.mdx#L10-L35) describes the renderer as
responsible for terminal output, input, the render loop, and the context used to construct renderables.

There are deliberately several application models:

- Imperative `Renderable` objects are created with a context, added to parents, and mutated through setters and methods.
- Core constructs build a lightweight VNode graph and replay queued method calls when instantiated. The source README explicitly calls this an exploration, not React or a reactive replacement.
- The Solid binding uses a Solid universal renderer and is the model used by OpenCode's current TUI.
- The React binding uses `react-reconciler`, allowing React application state and reconciliation to drive OpenTUI host objects.

OpenTUI is therefore an application framework at the full-stack boundary, but not an Elm-style application runtime. It supplies renderer scheduling and input delivery, not a required reducer, effect
executor, cancellation model, or external-event queue.

## Architecture

### Retained Tree And Layout

`Renderable` is a retained object with a parent, children, numeric identity, string ID, visibility, dirty state, focus state, event handlers, and a Yoga node. Its base contract includes `add`,
`remove`, `insertBefore`, descendant lookup, and `requestRender`; the concrete class maintains separate layout and z-index child orders. The
[release source](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/Renderable.ts#L136-L179) and
[constructor](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/Renderable.ts#L225-L323) show that this is a real retained tree, not merely a
draw-command list.

Layout is Flexbox-like Yoga. Width, height, flex direction, growth, shrinking, alignment, padding, margin, absolute positioning, overflow, and percentages are exposed as renderable options. The native
build compiles Yoga C++ sources into the native library. This provides familiar composition and responsive geometry, but it also makes layout part of the native build and ABI boundary rather than a
pure TypeScript subsystem.

The core tree is retained without being a general reactive graph. Setters commonly mark a renderable dirty and call `requestRender`; application code can mutate object references directly. Declarative
constructs reduce construction boilerplate, while framework bindings add reconciliation and reactivity at a higher boundary.

### Render Pipeline And Compositor

The renderer owns `nextRenderBuffer` and `currentRenderBuffer`. Each loop runs frame callbacks, renders the root tree into the next buffer, applies post-process functions and the console overlay, then
calls the native renderer. The native side keeps two cell buffers, walks the viewport, and emits only cells whose character, attributes, foreground, or background differ unless a full repaint is
requested. It uses absolute cursor positioning for changed runs and suppresses completely empty frames. The
[native diff path](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/zig/renderer.zig#L1317-L1460) is the most important implementation source for
this behavior.

The cell representation is richer than a string grid. A grapheme pool stores multi-byte clusters, start cells encode grapheme extent, and continuation cells preserve wide-cell structure. Scissors,
framebuffers, hyperlinks, styles, z-order, and a double-buffered hit grid support overlays and mouse interaction while increasing cross-subsystem coherence requirements.

The output layer has two materially different paths. `BufferedBackend` stages ANSI bytes in A/B buffers, optionally hands one buffer to a render thread, and writes committed bytes to stdout or memory.
`FeedBackend` writes spans into a `NativeSpanFeed` for a custom `Writable`, such as an SSH channel. Feed pressure causes a frame to be skipped before diffing, and queued bytes drain in order. The
[output backend](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/zig/renderer-output.zig#L163-L192) and
[frame commit logic](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/zig/renderer-output.zig#L400-L457) show that OpenTUI treats its own frame
assembly and feed queue as transactions. That is valuable, but it is not the same as knowing that the terminal physically consumed the final byte.

### Input And Interaction

The release parser is explicitly byte-oriented. It turns stdin into exactly one typed event among key, mouse, paste, or terminal response, and leaves dispatch to `KeyHandler` and the renderer. It
recognizes UTF-8 boundaries, SGR and X10 mouse sequences, bracketed paste, Kitty keyboard input, capability replies, cursor reports, OSC, DCS, and APC sequences. The parser keeps protocol context so
startup replies and application input share one owner rather than being independently decoded. See the
[parser contract](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/lib/stdin-parser.ts#L1-L114).

`KeyEvent` and `PasteEvent` expose `preventDefault()` and `stopPropagation()`. Internal key dispatch runs global listeners first, then focused renderable listeners, checking those flags between
handlers. A focused renderable installs its key and paste handlers into the internal handler and removes them on blur. Mouse dispatch begins with the native hit grid, calls the target's handlers, and
bubbles through parents until propagation is stopped. The
[event implementation](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/lib/KeyHandler.ts#L5-L222) and
[mouse bubbling path](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/Renderable.ts#L1597-L1660) make the routing rules inspectable.

The renderer also supports click focus, mouse capture for drag and selection, terminal focus events, bracketed paste, Kitty keyboard options, and custom input handlers. Command policy and focus
traversal remain application or binding concerns.

### Scheduling And Lifecycle

The renderer has automatic, continuous, and live modes. In automatic mode, mutations schedule a render; `start()` runs a target-FPS loop; `requestLive()` and `dropLive()` maintain a reference-counted
animation mode. `requestRender()` coalesces invalidations, while `idle()` resolves after scheduled and in-flight rendering has settled. A configurable `Clock` is used for timers, which is important
for tests. The [scheduler code](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/renderer.ts#L1460-L1587) and
[loop](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/renderer.ts#L4373-L4504) are explicit enough to diagnose scheduling state.

The renderer supports `pause`, `suspend`, `resume`, `stop`, and `destroy`. Suspend disables mouse, detaches input, restores raw mode, and suspends the native renderer. Resume drains buffered stdin
before attaching the listener, restores the native terminal, re-enables mouse, and requests a full repaint. Destroy can defer final native teardown when called during a frame. The public lifecycle
documentation requires applications to call `destroy()` and states that OpenTUI does not automatically clean up on `process.exit` or unhandled errors; signal handlers cover configured common signals
but are not a substitute for application cleanup. See the
[release lifecycle implementation](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/renderer.ts#L3954-L4209) and
[current lifecycle documentation](https://github.com/anomalyco/opentui/blob/4d5b7c1ad9d444531d0e0907f3b3926e12be318d/packages/web/src/content/docs/core-concepts/lifecycle.mdx#L7-L17).

### Terminal Ownership And Modes

OpenTUI has three explicit screen modes:

| Mode               | Contract at the examined revision                                                            |
| ------------------ | -------------------------------------------------------------------------------------------- |
| `alternate-screen` | Own the alternate screen for a full-screen application; restore the prior screen on teardown |
| `main-screen`      | Render on the main screen with a reserved region; not a true native-scrollback backend       |
| `split-footer`     | Keep a mutable footer while captured stdout or scrollback snapshots are replayed above it    |

The current documentation is unusually clear that these are not interchangeable. `split-footer` is the closest supported direct-render mode, but it still uses the buffered main-screen renderer. Custom
stdin/stdout streams are supported for SSH, PTY, or WebSocket-backed transports, with explicit dimensions and a remote-mode hint. Only process stdout receives automatic `SIGWINCH`; external transports
must call `renderer.resize()`.

The split-footer implementation is useful for immutable transcript history and a mutable prompt. `writeToScrollback` renders snapshots, while `ScrollbackSurface` supports repeated rendering and
row-range commits after asynchronous highlighting settles. OpenCode builds an application-level stream around it.

## Core Strengths

### Full-Stack Native Rendering Boundary

OpenTUI puts the high-cost and correctness-sensitive terminal work in one native core. Layout measurement, cell storage, diff output, grapheme pools, hit grids, capability-dependent cursor handling,
and feed backpressure are not independently reimplemented by every widget or application. The C ABI and platform packages also leave room for non-TypeScript bindings, even though the supported
high-level experience is TypeScript.

This is consequential for text-heavy applications: widgets can render into an `OptimizedBuffer` while native code preserves wide-cell metadata and output policy. Native tests cover drawing, emoji,
feed pressure, failed assembly, threaded output, and cleanup. The cost is a larger build and release surface.

### Explicit Compositor And Interaction State

Z-order, clipping, framebuffer surfaces, cursor state, and the double-buffered hit grid are first-class. This makes overlays, selections, mouse hover, and modal composition more direct than in a
library where the application must reconstruct hit targets from a fresh draw call. The hit grid's frame consistency rule is particularly good: queries observe a complete previous grid rather than a
partially painted one.

The native state also provides diagnostics: `getNativeStats()` exposes frame and cell-update data, and the debug overlay shows timing and memory information.

### Protocol-Aware Input

OpenTUI's parser avoids the common mistake of decoding stdin to Unicode text before recognizing terminal protocols. It preserves raw framing, handles fragmented escape sequences, recognizes paste as
bytes, and routes terminal responses separately from application keys. Kitty keyboard support provides richer modifier and event information when available, with legacy parsing as a fallback.

Global shortcuts, default prevention, and focused renderable delivery are practical consequences of the handler priority rules. This is a good pattern for ArborUI's single-owner input boundary.

### Useful Screen-Mode Separation

OpenTUI does not pretend that alternate screen, main-screen reservation, and split-footer output have identical semantics. It exposes the modes, validates incompatible external-output settings, and
keeps split-footer bookkeeping in a native model. The scrollback surface is a concrete answer to streaming transcript output that must not be repainted like a mutable viewport.

OpenCode uses Solid for the mutable footer and a separate append-oriented stream for completed rows, keeping the mode boundary visible in application code.

### A Real Headless Renderer Harness

The test renderer constructs `CliRenderer` with native memory output, not a separate mock widget renderer. It exposes keyboard and mouse injection, one-shot rendering, flush and wait helpers,
visual-idle detection, character frames, styled spans, cursor state, resize, external output, and native statistics. `ManualClock` controls timer ordering and time advancement. This is a strong
testing boundary for users who want to test a complete core-based application without opening a terminal.

The harness is not physical-terminal validation, but it exercises production layout, render traversal, native buffer conversion, hit-grid construction, and scheduler state.

## Limitations And Frustrations

### Incremental Diffing Does Not Establish Physical-Screen Recovery

```text
Classification: Bug and extension failure relative to ArborUI's recovery requirement
Requirement: After a partial, failed, or uncertain output, the next accepted frame must resynchronize the physical terminal
Library assumption: The native current buffer is a sufficient record of what the terminal displays
Observable failure or friction: Stale cells or cross-region smearing can persist until a resize or forced full repaint
Root architectural cause: The diff cache records intended cell state; direct stdout output has no applied/unknown outcome
Available workaround: Force a repaint through internal state, use a resize/resume path, or avoid incremental output
Cost of workaround: Private API access or loss of the main performance benefit; terminal-specific behavior remains
Upstream response: Resize and resume fixes exist in parts of the history; the reported alternate-screen stale-column issue remains open
Current status and version: Source behavior verified in v0.4.3; reports concern v0.2.14 and v0.4.1/current consumers
Evidence: Native diff source; output backend; issues #1110 and #1187
Confidence: High for the state-machine limitation, medium for each terminal-specific reproduction
```

The native diff loop compares `currentRenderBuffer` and `nextRenderBuffer`, emits a changed run, and calls `syncCell` so the next frame can skip that cell. It does not query the terminal after the
write. The direct `StdoutOutput` catches write and flush errors in a void callback, so the renderer cannot distinguish complete acceptance from a short or failed physical write. The feed path is
stronger: it preserves pending spans, reports backpressure, and turns failed frame assembly into a full-repaint request. That distinction should not be erased when evaluating the project.

Issue [#1110](https://github.com/anomalyco/opentui/issues/1110), still open when accessed, reports stale right-edge content in OpenCode on macOS Terminal.app after horizontal resize. The issue
identifies that v0.2.14 alternate-screen resize resized buffers and requested a render without forcing a full repaint, while split-footer had broader clearing behavior. The v0.4.3 source still clears
and resizes the native buffers during `processResize` but does not set the full-repaint latch for ordinary alternate-screen resize. Issue [#1187](https://github.com/anomalyco/opentui/issues/1187)
describes a separate v0.4.1 reproduction where autowrap or ambiguous-width disagreement desynchronized the host terminal; it was closed by the author, but the issue page does not establish a merged
fix.

This is not a criticism of using a diff renderer. It is a contract mismatch for ArborUI's failure model. OpenTUI has a private one-shot repaint latch and uses it after resume, capability changes, and
selected split-footer transitions, but a consumer has no ordinary public `invalidatePhysicalScreen()` operation. A consumer workaround that sets the private latch every frame is explicitly described
in #1187 as a diagnostic band-aid. ArborUI should keep commit-after-acceptance and physical-state invalidation as a public backend contract, while still preserving OpenTUI's useful staged output and
no-op suppression.

### Split-Footer Is A Specialized Scrollback Mode, Not A General Native-Scrollback Backend

```text
Classification: Tradeoff and limitation
Requirement: Preserve native terminal scrollback while maintaining an interactive mutable viewport
Library assumption: A reserved main-screen footer plus replayed snapshots is sufficient
Observable failure or friction: Applications must model immutable history, footer geometry, replay, ordering, and surface settlement themselves
Root architectural cause: Main-screen and split-footer reuse a buffered renderer instead of exposing a separate native-scrollback ownership contract
Available workaround: Use split-footer, write snapshots/surfaces, and build an application-level append queue
Cost of workaround: More application state and mode-specific lifecycle code; direct full-screen and transcript semantics remain different
Upstream response: Current documentation explicitly describes split-footer as the closest direct-render mode and adds scrollback surfaces
Current status and version: Supported in v0.4.3/current main
Evidence: Renderer docs; `ScrollbackSurface`; OpenCode `RunFooter`
Confidence: High
```

The feature is useful, but the boundary matters. The current docs state that `main-screen` is not true native scrollback and that split-footer remains a buffered main-screen renderer. A scrollback
snapshot is rendered into an off-screen buffer and then committed above the footer; it is not an immutable terminal history owned independently of the live renderer. Width changes, footer height
changes, resize, suspend, and replay therefore require coordination with the renderer's bookkeeping.

OpenCode demonstrates the intended workaround well. Its `RunFooter` keeps an append-only queue, coalesces adjacent streaming commits in a microtask, flushes into a `RunScrollbackStream`, and treats
the footer as the only mutable region. It subscribes to renderer destruction, waits for renderer idle, and destroys scrollback surfaces and syntax resources. That is a successful application
architecture, but it is also evidence that the mode's correctness contract crosses the library/application boundary. ArborUI should not make native scrollback a boolean variant of its full-screen
viewport; it should define a separate ownership and recovery mode.

### The Renderer Schedules Frames, Not Application Effects

```text
Classification: Tradeoff and maturity problem
Requirement: Serialize model updates, asynchronous effects, cancellation, and rendering settlement for a complete application
Library assumption: Applications or framework bindings own state and call `requestRender()` when visible state changes
Observable failure or friction: Streaming, idle rendering, and frame scheduling races can leave application state updated while the screen remains stale
Root architectural cause: Core scheduling is an invalidation loop, not an application event/effect runtime
Available workaround: Use React/Solid, call `requestRender()`, coalesce external events, and await `renderer.idle()`
Cost of workaround: Every application needs conventions for effect ownership, stale results, backpressure, and settlement
Upstream response: Several concrete scheduling bugs have received tests or pull requests; one render-loop issue remains open
Current status and version: Intentional boundary; reported race is not independently reproduced here
Evidence: Scheduler source; issues #789 and #963; PRs #965 and #1086
Confidence: High for the boundary, medium for the race risk
```

The core loop is disciplined about coalescing requests and exposing `idle()`, but visible state changes still depend on the right setter or callback reaching `requestRender()`.
[Issue #963](https://github.com/anomalyco/opentui/issues/963) documented a real downstream OpenCode streaming path in which `CodeRenderable` stored content and marked highlighting dirty without
scheduling a render when the renderer was idle. [PR #965](https://github.com/anomalyco/opentui/pull/965) merged a fix and a test before the examined release. This is a positive maintenance example,
but it shows how a component optimization can violate the application-visible invalidation contract.

[Issue #789](https://github.com/anomalyco/opentui/issues/789) reported a blank screen after an event burst when a render request raced the loop's scheduling decision. The proposed
[PR #1086](https://github.com/anomalyco/opentui/pull/1086) is closed and unmerged as of the research date. The v0.4.3 source still has `immediateRerenderRequested` checked in the main loop schedule
and resolves `rendering` in a later `finally` block without the proposed second check. This does not prove that the old environment reproduces on v0.4.3, but it is enough to classify the failure mode
as an unverified maturity risk rather than a solved guarantee.

Framework bindings reduce ordinary application boilerplate, and OpenCode's own microtask queues and `idle()` calls make the intended coordination explicit. They do not make external effects
transactional. ArborUI should provide a first-class application harness for serialized updates and effect completion instead of requiring every consumer to invent one around `requestRender()`.

### Runtime And Native Build Assumptions Increase Adoption Cost

```text
Classification: Tradeoff and maturity problem
Requirement: Install, build, test, and distribute a stable TUI across supported platforms and runtimes
Library assumption: Bun plus pinned Zig and prebuilt native packages are acceptable for the target ecosystem
Observable failure or friction: Node native use needs experimental FFI; source builds need exact Zig, C/C++, Yoga, and platform SDK/toolchain support
Root architectural cause: A native core and FFI boundary are part of the ordinary application path
Available workaround: Use Bun and published platform packages, or explicitly provision Node 26.3.0, experimental FFI, Zig, and SDKs
Cost of workaround: More CI/release matrix, native debugging, and runtime policy than a pure library
Upstream response: Prebuilt platform packages and documented cross-target builds exist; compatibility remains fast-moving
Current status and version: Supported but pre-1.0 and changing in v0.4.3/main
Evidence: Package metadata, getting-started docs, and build.zig
Confidence: High
```

The native boundary produces real performance and correctness benefits, but it is not free. The release package ships optional native packages for eight target families. Source builds reject
unsupported Zig versions, compile Yoga C++, include `uucode`, and use platform SDKs and system libraries. The current docs make the runtime distinction explicit: portable imports can load in Node, but
`createCliRenderer()` requires experimental FFI and the documented Node version.

The C ABI is a useful native extension boundary, but it does not make the TypeScript API runtime-independent. Other-language consumers still need bindings and distribution. ArborUI should preserve
runtime independence where it is a product requirement.

### Unicode Logic Is Stronger Than Terminal-Wide Compatibility

```text
Classification: Ecosystem tradeoff
Requirement: Stable grapheme placement and cursor behavior across terminals, multiplexers, fonts, and width policies
Library assumption: Unicode properties plus capability detection can define a useful logical cell model
Observable failure or friction: Ambiguous-width characters, autowrap, CJK wrapping, and emulator behavior can disagree with the logical buffer
Root architectural cause: The terminal ultimately decides physical cursor advancement and display width
Available workaround: Select width methods, use explicit-width capabilities when available, test target terminals, and force repaint after uncertainty
Cost of workaround: Compatibility policy and terminal/emulator coverage remain application concerns
Upstream response: Native grapheme and width tests plus repeated fixes for CJK/wrapping cases
Current status and version: Logical support verified; universal physical correctness unknown
Evidence: UTF-8 implementation/tests and issues #255 and #1187
Confidence: High for logical support, medium for compatibility conclusion
```

OpenTUI does substantially more than count JavaScript string length. The native code supports `unicode` and `wcwidth` models, uses Unicode grapheme breaks, stores wide clusters and continuation cells,
and includes extensive UTF-8, grapheme, CJK, emoji, and editor tests. Its East Asian width policy also contains explicit emoji ranges, while ambiguous characters ultimately depend on terminal
behavior.

The remaining problem is systemic. A logical test cannot confirm how Terminal.app, tmux, ConPTY, or a font advances a cursor at the final column. CJK and wrapping fixes have shipped, but current
reports show that a native Unicode-aware renderer can still encounter physical width disagreement. ArborUI's grapheme invariants need emulator/PTY compatibility tests.

## Testing Strategy

OpenTUI's testing model has three useful layers.

### Headless TypeScript Application Harness

`@opentui/core/testing` creates a `CliRenderer` with `screenMode: "main-screen"`, disabled console capture, passthrough external output, and native memory output. It returns the real renderer plus
`renderOnce`, `flush`, `waitFor`, `waitForFrame`, `waitForVisualIdle`, `captureCharFrame`, `captureSpans`, `resize`, external-output capture, native statistics, mock keyboard input, and mock mouse
input. The [public test documentation](https://github.com/anomalyco/opentui/blob/4d5b7c1ad9d444531d0e0907f3b3926e12be318d/packages/web/src/content/docs/core-concepts/testing.mdx#L9-L41) and
[release implementation](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/testing/test-renderer.ts#L15-L75) describe the boundary.

The helper uses production rendering rather than a fake tree. `renderOnce()` calls the renderer loop, captures read from the native current buffer, and span capture includes styles and cursor
coordinates. `waitForVisualIdle()` observes native cell updates and scheduler state, while `ManualClock` supplies deterministic timers. For core-based applications, this supports terminal-free
rendering, input injection, resize, settling, and visual/semantic assertions.

The harness also exercises external output and feed behavior. Custom stdout tests use a delayed `Writable` to hold the feed in backpressure, verify coalescing and retry, simulate native failure
statuses, and check shutdown output. Those are meaningful failure-injection tests at the library's transport boundary.

### Native Unit And Reference Tests

The Zig test renderer uses memory output and an injected environment map, avoiding the host terminal's `TERM`, multiplexer, and capability state. Native tests cover simple and multi-line rendering,
emoji, current/next buffer behavior, resize and hit-grid dimensions, capability-dependent output, feed queue pressure, failed frame writes, output-thread handoff, allocation failures, and cleanup. The
[native test renderer](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/zig/tests/test-renderer.zig#L47-L126) and
[output failure tests](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/zig/tests/renderer_test.zig#L2320-L2554) show good isolation and failure
intent.

The TypeScript suite separately tests parser fragments, Kitty input, paste, mouse integration, focus, lifecycle state transitions, suspend/resume repaint behavior, custom stdout, idle resolution,
render statistics, scrollback surfaces, and destroy-during-render. The release contains a particularly useful regression test for resume: it asserts that the next main-screen and alternate-screen
native render receives `force = true`.

### Important Gaps

The searched v0.4.3 core workspace does not contain a PTY or terminal-emulator suite. Memory output cannot represent raw-mode restoration, real alternate-screen behavior, signals, cursor-query races,
emulator autowrap, multiplexer width policy, or a physical short write. Custom stdout tests verify stream backpressure, not terminal acceptance; direct stdout errors are swallowed at the Zig writer
boundary.

There is no core-level effect driver for external events, arbitrary promise completion, cancellation, or declared application settlement. `ManualClock` controls renderer timers; application effects
need their own doubles. Visual quiet is not equivalent to model settlement, and the timestamped test recorder is better for debugging frame sequences than as the sole deterministic application
contract.

A complete testing strategy for ArborUI should adopt OpenTUI's production-path memory renderer and manual clock, then add the missing layers: semantic state assertions, external-event queues,
controlled effect completion, failed-write injection, PTY/emulator checks, and a physical-screen invalidation oracle.

### Capability Summary

| Capability                                       | OpenTUI v0.4.3 assessment                                        |
| ------------------------------------------------ | ---------------------------------------------------------------- |
| Complete core application without a terminal     | Strong through `createTestRenderer`                              |
| Production render traversal and native cell path | Strong; memory output uses `CliRenderer` and native core         |
| Character and styled assertions                  | Strong through frames and spans                                  |
| Keyboard, paste, mouse, resize injection         | Strong in core harness                                           |
| Deterministic renderer timers                    | Strong through `ManualClock`                                     |
| Wait for renderer settlement                     | Strong through `idle`, `flush`, and visual-idle helpers          |
| Inject feed backpressure/native statuses         | Partial to strong; custom stdout and native tests cover it       |
| Simulate uncertain physical terminal state       | Not provided as a normal public contract                         |
| PTY or terminal emulator validation              | Not found in the v0.4.3 core workspace                           |
| Full application effect settlement               | Application-defined                                              |
| Unicode and layout boundary tests                | Strong logical coverage; physical compatibility unknown          |
| Cross-platform native CI                         | Build targets are explicit; this research did not run the matrix |

## Common Scenario Assessment

| Scenario                                    | Assessment                                                                                                                                          |
| ------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| Form with focus, validation, and modal      | Supported primitives: Yoga, focus, key routing, z-order, and mouse; validation and modal/application commands remain app-owned                      |
| Large scrollable collection                 | Scrollbox and text-buffer primitives exist; stable collection identity, virtualization policy, and data loading remain application/binding concerns |
| Streaming external updates                  | Supported with `requestRender`, `CodeRenderable`, scrollback surfaces, and feed queues; effect ordering and cancellation are application-owned      |
| Unicode-heavy editing                       | Strong logical grapheme/editor support; terminal and multiplexer width differences remain a compatibility risk                                      |
| Overlay with clipping and mouse interaction | Strong hit-grid, z-index, scissor, and bubbling primitives; physical diff recovery is partial                                                       |
| Resize during active updates                | Debounced resize, Yoga reflow, hit-grid invalidation, and tests exist; alternate-screen stale-edge reports remain relevant                          |
| Deferred or failed output                   | Feed backpressure and native assembly failure are modeled; direct physical acceptance and short writes are not                                      |
| Suspension to a child process               | Explicit suspend/resume, input draining, and resume repaint are supported; external job-control behavior needs PTY validation                       |
| Long idle periods                           | Strong renderer idle and live-request model; application effects can still update state without a framework-level settlement contract               |
| Conversation preserving native scrollback   | Split-footer and scrollback surfaces are useful, but this is a specialized replay mode rather than a general native-scrollback backend              |

## Lessons For ArborUI

### Adopt

- Keep the low-level render buffer, output transport, hit grid, and terminal protocol implementation behind a small public facade.
- Use one byte-preserving input owner that recognizes protocol responses before exposing application events.
- Maintain an explicit parallel hit-test representation and make it frame-consistent rather than querying a half-built render tree.
- Treat screen ownership modes as separate contracts. OpenTUI's distinction between alternate-screen and split-footer is better than one renderer with undocumented combinations.
- Provide a production-path headless renderer with native or backend memory output, explicit dimensions, input injection, resize, styled capture, and a deterministic clock.
- Expose `idle` or an equivalent settlement primitive, but document that visual idle is not application model settlement.
- Invest in a common widget and example surface. OpenTUI's architecture is credible partly because OpenCode and the repository exercise real editors, scrollback, syntax highlighting, selection, and
  input controls.

### Avoid

- Do not treat a cell shadow buffer as proof of physical terminal state after an uncertain write.
- Do not swallow backend write failures behind a void output callback. Model accepted, deferred, failed, and unknown outcomes explicitly.
- Do not make the only full-repaint escape hatch a private flag or a resize side effect. Provide a public physical-state invalidation operation and make the next successful frame full.
- Do not call split-footer replay equivalent to native terminal scrollback. Define immutable history, cursor ownership, resize, and recovery semantics separately.
- Do not require every application to rediscover effect ordering, external-event coalescing, cancellation, and run-until-idle behavior around `requestRender()`.
- Do not claim Unicode correctness from logical buffer tests alone. Include PTY or emulator evidence for final-column, autowrap, ambiguous-width, and multiplexer cases.

### Problems ArborUI Already Approaches Differently

ArborUI's prepared-frame transaction and physical-state invalidation are stronger than OpenTUI's direct stdout contract. ArborUI also deliberately separates the public facade from terminal-specific
and layout implementation crates, and its retained state rules prevent borrowed application data from being held by long-lived UI state. Those boundaries should remain explicit rather than being
weakened to match OpenTUI's TypeScript convenience.

The comparison supports ArborUI's public full-application harness. OpenTUI proves that a real renderer can be driven headlessly; ArborUI should add serialized updates, controlled effects, output fault
injection, and PTY lifecycle tests without reproducing the native core.

### Claims Not Yet Proven

- ArborUI has not yet shown that prepared-frame transactions reduce failures users experience often enough to offset their API and implementation cost.
- ArborUI has not yet demonstrated an end-to-end performance advantage over a native incremental renderer for large text-heavy applications.
- Explicit grapheme and continuation invariants do not by themselves establish better compatibility across terminal emulators.
- A smaller widget ecosystem can outweigh stronger lifecycle and recovery semantics for adopters.
- Alternate-screen correctness does not prove that future inline, remote, or native-scrollback modes can share one renderer contract.

### Follow-Up Work

1. Build the same streaming conversation and modal form in OpenTUI/Solid and ArborUI, recording application-owned code, effect plumbing, tests, emitted bytes, and recovery behavior.
2. Add a PTY or virtual-terminal reproduction matrix for resize shrink, final-column wide/ambiguous glyphs, suspend/resume, raw mode, external output, and short writes.
3. Add a fault-injection test that makes output acceptance unknown after a partial ANSI sequence and verifies that the next successful frame is a full repaint.
4. Benchmark idle, one-cell updates, large text changes, overlays, resize bursts, feed backpressure, and streaming scrollback at fixed terminal sizes.
5. Prototype separate alternate-screen, inline-region, and native-scrollback ownership contracts, then document semantic harness assertions alongside visual snapshots.

## Evidence Appendix

All sources were accessed on 2026-07-16 unless otherwise noted.

| Claim                                           | Source                                                                                                                                                                                                                                                                                                                                                      | Version or revision                       | Source date            | Accessed   | Status                                   | Notes                                                                                                      |
| ----------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------- | ---------------------- | ---------- | ---------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| Stable release baseline                         | [v0.4.3 release](https://github.com/anomalyco/opentui/releases/tag/v0.4.3)                                                                                                                                                                                                                                                                                  | `v0.4.3`, peeled commit `5803b2c`         | 2026-07-03             | 2026-07-16 | Verified                                 | Release API confirms publication and platform archives                                                     |
| Core package and runtime contract               | [`packages/core/package.json`](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/package.json#L1-L99)                                                                                                                                                                                                        | `5803b2c`                                 | 2026-07-03             | 2026-07-16 | Verified                                 | Bun engine, optional native packages, native/test scripts                                                  |
| Native build and target matrix                  | [`build.zig`](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/zig/build.zig#L10-L30)                                                                                                                                                                                                                   | `5803b2c`                                 | 2026-07-03             | 2026-07-16 | Verified                                 | Zig 0.15.2, Yoga C++, platform targets                                                                     |
| Retained renderable tree                        | [`Renderable.ts`](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/Renderable.ts#L136-L179)                                                                                                                                                                                                             | `5803b2c`                                 | 2026-07-03             | 2026-07-16 | Verified                                 | Parent/child, identity, dirty state, render invalidation                                                   |
| Layout and z-order model                        | [`Renderable.ts`](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/Renderable.ts#L225-L323)                                                                                                                                                                                                             | `5803b2c`                                 | 2026-07-03             | 2026-07-16 | Verified                                 | Yoga nodes, buffered rendering, z-index, handlers                                                          |
| Declarative constructs are exploratory          | [Composition README](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/renderables/composition/README.md#L1-L8)                                                                                                                                                                                          | `5803b2c`                                 | 2026-07-03             | 2026-07-16 | Verified                                 | Explicitly not React or reactive                                                                           |
| Native diff and hit-grid behavior               | [`renderer.zig`](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/zig/renderer.zig#L172-L212), [diff loop](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/zig/renderer.zig#L1317-L1460)                                                           | `5803b2c`                                 | 2026-07-03             | 2026-07-16 | Verified                                 | Double-buffered hit grid and intended-state diff cache                                                     |
| Output staging and feed backpressure            | [`renderer-output.zig`](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/zig/renderer-output.zig#L163-L192), [feed backend](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/zig/renderer-output.zig#L548-L649)                                     | `5803b2c`                                 | 2026-07-03             | 2026-07-16 | Verified                                 | Internal frame failure and feed pressure are modeled                                                       |
| Byte-level input ownership                      | [`stdin-parser.ts`](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/lib/stdin-parser.ts#L1-L114)                                                                                                                                                                                                       | `5803b2c`                                 | 2026-07-03             | 2026-07-16 | Verified                                 | Typed key, mouse, paste, response events                                                                   |
| Key priority and propagation                    | [`KeyHandler.ts`](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/lib/KeyHandler.ts#L5-L222)                                                                                                                                                                                                           | `5803b2c`                                 | 2026-07-03             | 2026-07-16 | Verified                                 | `preventDefault`, `stopPropagation`, global then focused handlers                                          |
| Scheduler and lifecycle                         | [`renderer.ts`](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/renderer.ts#L1460-L1587), [control methods](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/renderer.ts#L3954-L4080)                                                              | `5803b2c`                                 | 2026-07-03             | 2026-07-16 | Verified                                 | Coalesced requests, idle, pause/suspend/resume                                                             |
| Screen modes and scrollback contract            | [Current renderer docs](https://github.com/anomalyco/opentui/blob/4d5b7c1ad9d444531d0e0907f3b3926e12be318d/packages/web/src/content/docs/core-concepts/renderer.mdx#L65-L150), [scrollback surfaces](https://github.com/anomalyco/opentui/blob/4d5b7c1ad9d444531d0e0907f3b3926e12be318d/packages/web/src/content/docs/core-concepts/renderer.mdx#L172-L265) | docs `4d5b7c1`                            | 2026-07-15             | 2026-07-16 | Supported                                | Current docs, not silently treated as v0.4.3 docs                                                          |
| Explicit cleanup requirement                    | [Lifecycle docs](https://github.com/anomalyco/opentui/blob/4d5b7c1ad9d444531d0e0907f3b3926e12be318d/packages/web/src/content/docs/core-concepts/lifecycle.mdx#L7-L17)                                                                                                                                                                                       | docs `4d5b7c1`                            | 2026-07-15             | 2026-07-16 | Supported                                | Docs require `destroy()` and explain non-automatic cleanup                                                 |
| Headless test renderer                          | [Testing docs](https://github.com/anomalyco/opentui/blob/4d5b7c1ad9d444531d0e0907f3b3926e12be318d/packages/web/src/content/docs/core-concepts/testing.mdx#L9-L41), [`test-renderer.ts`](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/testing/test-renderer.ts#L15-L75)                              | release `5803b2c`; docs `4d5b7c1`         | 2026-07-03/15          | 2026-07-16 | Verified                                 | Production `CliRenderer` with memory output and input mocks                                                |
| Deterministic clock                             | [`manual-clock.ts`](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/testing/manual-clock.ts#L20-L117)                                                                                                                                                                                                  | `5803b2c`                                 | 2026-07-03             | 2026-07-16 | Verified                                 | Controlled timeout/interval ordering                                                                       |
| Native failure and backpressure tests           | [`renderer.custom-stdout.test.ts`](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/tests/renderer.custom-stdout.test.ts#L302-L368), [native tests](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/zig/tests/renderer_test.zig#L2320-L2554)       | `5803b2c`                                 | 2026-07-03             | 2026-07-16 | Verified                                 | Covers internal failure statuses and feed pressure, not physical terminal acceptance                       |
| Resume full repaint                             | [`renderer.control.test.ts`](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/tests/renderer.control.test.ts#L26-L145)                                                                                                                                                                                  | `5803b2c`                                 | 2026-07-03             | 2026-07-16 | Verified                                 | Main and alternate screen resume assertions                                                                |
| Stale cells after alternate-screen resize       | [Issue #1110](https://github.com/anomalyco/opentui/issues/1110)                                                                                                                                                                                                                                                                                             | Open; report on 0.2.14                    | 2026-05-26             | 2026-07-16 | Reported                                 | OpenCode consumer reproduction; v0.4.3 resize source checked; not reproduced here                          |
| Diff/terminal cursor desynchronization          | [Issue #1187](https://github.com/anomalyco/opentui/issues/1187)                                                                                                                                                                                                                                                                                             | Report on 0.4.1                           | 2026-06-19             | 2026-07-16 | Reported                                 | Closed by author; no merged fix established from issue page                                                |
| Render-loop scheduling race                     | [Issue #789](https://github.com/anomalyco/opentui/issues/789), [PR #1086](https://github.com/anomalyco/opentui/pull/1086)                                                                                                                                                                                                                                   | Issue open; PR closed/unmerged            | 2026-03-07/2026-05-18  | 2026-07-16 | Reported and inferred                    | Historical environment; v0.4.3 source retains related scheduling structure                                 |
| Streaming invalidation bug and fix              | [Issue #963](https://github.com/anomalyco/opentui/issues/963), [PR #965](https://github.com/anomalyco/opentui/pull/965)                                                                                                                                                                                                                                     | PR merged 5e20a2e; included before v0.4.3 | 2026-04-22/23          | 2026-07-16 | Reported then verified as fixed upstream | Downstream OpenCode scenario; release source contains the added request                                    |
| OpenCode consumer and split-footer architecture | [OpenCode TUI command](https://github.com/anomalyco/opencode/blob/dev/packages/opencode/src/cli/cmd/tui.ts), [OpenCode package dependencies](https://github.com/anomalyco/opencode/blob/dev/packages/tui/package.json), [OpenCode RunFooter](https://github.com/anomalyco/opencode/blob/dev/packages/opencode/src/cli/cmd/run/footer.ts)                    | `dev`, accessed current                   | Current at access date | 2026-07-16 | Supported                                | Mutable links; used for current consumer structure, not release pinning                                    |
| Logical Unicode and width handling              | [`utf8.zig`](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/zig/utf8.zig#L628-L815), [`grapheme.zig`](https://github.com/anomalyco/opentui/blob/5803b2cfa2942c45a3aedbb3601754e27f2cdc68/packages/core/src/zig/grapheme.zig#L11-L47)                                                                  | `5803b2c`                                 | 2026-07-03             | 2026-07-16 | Verified                                 | Logical representation and width methods; physical emulator compatibility remains open                     |
| PTY/emulator suite not found                    | Pinned v0.4.3 core workspace and scripts                                                                                                                                                                                                                                                                                                                    | `5803b2c`                                 | 2026-07-03             | 2026-07-16 | Inferred                                 | Searched core tests, native tests, package scripts, and docs; absence is scoped to that revision/workspace |
