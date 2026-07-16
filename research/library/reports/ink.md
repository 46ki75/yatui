# Ink Research Report

## Evidence Header

```text
Research date: 2026-07-16
Project version: Ink 7.1.0
Project revision: 25766aec618bd62030069f57dd081e5ebdd46add
Repository: https://github.com/vadimdemedes/ink
Documentation version: README and API documentation at 25766aec618bd62030069f57dd081e5ebdd46add
Primary platform examined: Source inspection on Linux; no physical terminal reproduction
Report depth: Deep dive
```

The latest stable release at the start of this research was
[`v7.1.0`](https://github.com/vadimdemedes/ink/releases/tag/v7.1.0), published on
2026-06-17. The tag object is
`844c172b2023c6258c482cd06cfe2cd5ce10d999` and it resolves to the source
revision recorded above. The release adds `suspendTerminal()`; source and test
claims below refer to the tagged revision unless a later issue, pull request, or
downstream commit is named explicitly.

## Executive Assessment

Ink is a TypeScript application framework and renderer that makes React's
declarative component model usable for command-line applications. It is not a
terminal substrate, a widget-only library, or a virtual terminal emulator. The
central abstraction is a React tree whose host nodes are Ink boxes and text
nodes. React reconciliation retains component identity, Yoga calculates flexbox
geometry, and Ink turns the laid-out tree into ANSI output for a Node writable
stream.

Ink is unusually effective when the application already wants React: state can
live in ordinary React components, composition is familiar, and a large npm
ecosystem supplies spinners, prompts, lists, gradients, and application-specific
components. Ink also has a more serious terminal lifecycle than its small JSX
surface suggests. It handles raw input ownership, bracketed paste, resize,
cursor state, alternate-screen mode, console interleaving, synchronized output,
and, in 7.1.0, scoped handoff to a child process.

The same boundary is decisive for ArborUI. Ink retains a React tree, but it does
not retain a terminal-independent committed cell model with a transactional
backend contract. It provides keyboard and focus helpers but deliberately does
not provide mouse hit testing. It provides clipping and measurements but not a
native scroll view or a visible-range collection primitive. Its test suite
exercises many production paths, while the public `ink-testing-library` is a
small frame-capture fake rather than a complete application harness. These are
not defects relative to Ink's stated goal of being React for CLI applications;
they are extension boundaries that matter for ArborUI's target of full-screen,
stateful applications with mouse interaction, large collections, deterministic
headless tests, and recoverable terminal output.

## Project Snapshot

Ink is implemented in TypeScript for Node.js and published as an ESM package.
The package identifies itself as "React for CLI" and depends on
`react-reconciler`, `scheduler`, `yoga-layout`, ANSI tokenization and width
packages, and Node stream utilities. The [release package metadata](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/package.json#L1-L74)
requires Node 22 or newer. Ink 7 also requires React 19.2 or newer and uses
React 19 APIs internally; the [v7 migration notes](https://github.com/vadimdemedes/ink/releases/tag/v7.0.0)
describe the Node and React requirement changes as breaking changes.

The primary category is an actively maintained application framework, with an integrated terminal
renderer and a small set of presentation components. The intended applications
include interactive prompts, spinners, progress displays, command-line tools,
chat interfaces, and other programs whose state can be represented by a React
tree. Ink also supports non-interactive output, where the final live frame is
written at unmount, and append-only output through `<Static>`.

The ecosystem is both a strength and a compatibility caveat. The inspected
[`ink-testing-library` 4.0.0 checkout](https://github.com/vadimdemedes/ink-testing-library/tree/4993171957dff60858bc9a860327f5d305696bc9)
has a Node 18 engine and development dependencies on Ink 5 and React 18. It
may still be useful as a simple frame harness, but its metadata and implementation
should not be assumed to describe the full Ink 7 lifecycle. A substantial
consumer, [Gemini CLI at commit `3ff5ba20fc1ad7d867218bbdb34756eb54d6eccb`](https://github.com/google-gemini/gemini-cli/tree/3ff5ba20fc1ad7d867218bbdb34756eb54d6eccb),
pins a patched Ink 6.6.9 fork and React 19.2.4, which is useful evidence of
application needs but not a clean compatibility test for Ink 7.

## Core Proposition

Ink makes a terminal application look like a React application. The user writes
JSX, stores model state with React state or external state libraries, and uses
hooks such as `useInput`, `useFocus`, `useApp`, and `useAnimation`. A component
does not manually position every ANSI escape sequence. Instead, `<Box>` and
`<Text>` describe a tree, styles express flexbox and terminal presentation, and
Ink reconciles updates before painting the result.

This differs from a lower-level ANSI helper because Ink owns the composition
pipeline and supplies a host renderer. It differs from a lower-level cell
library because application code does not normally receive a mutable buffer or
choose a draw callback. It also differs from an opinionated message-loop
framework because React remains the state and scheduling model rather than a
framework-owned reducer and typed command stream.

The strongest use case is a Node CLI whose developers already value React's
component reuse and ecosystem, and whose visible tree is small enough to lay out
and repaint on each meaningful update. Streaming logs and progress indicators
are especially well served by `<Static>` plus a live region. A dashboard or
chat-like application can also work well, but scrolling, pointer interaction,
large collections, and native scrollback require application or ecosystem
infrastructure rather than only Ink primitives.

## Architecture

### React Host And State Model

Ink supplies a custom `react-reconciler` host configuration. The host supports
mutation but not persistence or hydration. It creates `ink-root`, `ink-box`,
`ink-text`, and virtual text nodes, attaches a Yoga node to layout-bearing
elements, and applies changed props and styles during React commit. The
[reconciler host configuration](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/reconciler.ts#L138-L192)
calls layout listeners and then the root render callback after a commit.

This is retained identity at the React/Fiber and host-tree level. Keys and React
component identity determine whether state and host nodes are reused. Ink does
not retain application model references separately from React's normal state
rules, and it does not expose a second retained UI tree with independent
focus, hit-test, and invalidation metadata. The public API exports React hooks
and components, not a framework-owned model-update-view boundary.

The root `<App>` component provides contexts for stdin, stdout, stderr, focus,
animation, cursor state, and application lifecycle. Each active `useInput` hook
registers a listener on an internal event emitter and contributes to a raw-mode
reference count. The [input hook](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/hooks/use-input.ts#L159-L268)
parses the input event, builds a `Key` value, and wraps the callback in
`discreteUpdates` so keyboard-driven state changes receive high priority in
concurrent mode. The parser carries incomplete escape sequences across chunks,
recognizes bracketed paste, and optionally recognizes Kitty keyboard protocol
events.

Focus is intentionally lightweight. `useFocus` registers components in render
order, tracks an active ID, and supports activation, deactivation, forward and
reverse Tab traversal, and programmatic focus. The implementation is useful for
forms and prompts, but it is not spatial focus navigation or a general event
routing tree. A component without an explicit ID receives a random generated ID
from `useFocus`, so applications that need durable programmatic identity should
provide one.

### Layout And Painting

Ink's [DOM layer](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/dom.ts#L95-L269)
creates a Yoga node for boxes and text containers. Text measurement uses
`widest-line`; wrapping and truncation use `wrap-ansi` and `cli-truncate`.
During a render, Ink sets the root Yoga width to the current terminal width and
calls `calculateLayout`. The layout tree is therefore retained between updates,
but the complete visible output is derived again from the laid-out tree.

The painter walks the tree in layout order. It records write, clip, and unclip
operations into an `Output` object, which initializes a rectangular grid of
spaces and applies operations in order. The [output implementation](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/output.ts#L91-L319)
uses ANSI tokenization and `string-width` caches, clips with ANSI-aware slicing,
and handles multi-column characters by placing a leading value and clearing
covered cells. The [tree painter](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/render-node-to-output.ts#L99-L214)
applies box clipping and coordinates after Yoga has computed positions.

The important distinction is that this is not a committed renderer-owned cell
buffer in the ArborUI sense. Ink creates a current output string and then
`log-update` compares it with its previous string or previous line array. The
standard mode erases the previous line block and writes the new block; the
optional incremental mode compares lines and writes only changed lines, but it
still receives a complete freshly rendered output string. Layout, reconciliation,
tree traversal, and output-grid construction are not dirty-subtree painting.

`<Static>` is the special append-only path. Its [component implementation](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/components/Static.tsx#L21-L58)
renders only items added after the previous index and marks a host node as
static. The reconciler emits new static content before deleting the temporary
children from the live tree. Ink then skips static nodes while rendering the
live region and keeps the append-only text separately. This is a strong solution
for completed tasks and logs, but it is intentionally immutable history, not a
scrollable retained viewport.

### Scheduling And Terminal Ownership

Ink has two distinct scheduling layers. React schedules component updates, and
Ink throttles calls to its root render callback, defaulting to 30 frames per
second. `useAnimation` consolidates animation subscribers onto one timer and
reports frame, elapsed time, and delta values. The `concurrent` render option
selects React's concurrent root and makes Suspense, transitions, and deferred
values available, while the default remains legacy synchronous rendering.

The `render()` options define the terminal contract: interactive detection,
alternate screen, debug output, console patching, maximum FPS, incremental
rendering, Kitty keyboard probing, and injected streams. In non-interactive
mode, Ink writes static output as it becomes available but defers the live frame
until unmount. In interactive mode it hides the cursor, erases or updates the
live region, and restores the cursor on teardown. The [render API documentation](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/render.ts#L8-L135)
explicitly says that alternate-screen mode does not provide the terminal's
scrollback buffer.

Resize is handled by one stdout listener on the Ink instance. A width decrease
clears the current log-update state before recalculating layout; the next frame
is then rendered at the new width. `useWindowSize` exposes the dimensions to
components. `useCursor` provides an imperative cursor position relative to the
Ink output origin, primarily for IME composition, and propagates it during the
commit phase so abandoned concurrent renders do not leak cursor state.

Version 7.1.0 adds `useApp().suspendTerminal()`. The [release discussion for
PR #972](https://github.com/vadimdemedes/ink/issues/972) records the intended
behavior: flush pending output, stop consuming input, restore cursor and modes,
leave the alternate screen, run the child process, re-enter the screen, and
force a full redraw. The implementation follows that outline and resets its
previous output before repainting on resume. This is a meaningful lifecycle
strength, but it is scoped handoff and redraw, not a general physical-screen
transaction for every failed write.

### Public Extension Points

Application authors extend Ink primarily through React components and hooks.
They can inject streams, use `useStdin` to access the input stream, use
`useStdout` and `useStderr` for external output, use `useBoxMetrics` and
`measureElement` for layout information, transform rendered strings with
`<Transform>`, and use `renderToString` for synchronous or headless output.
The [public exports](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/index.ts#L1-L45)
do not include a mouse hook, a hit-map interface, a replaceable layout engine,
or a terminal backend trait. A user can implement a component-level workaround
or fork the renderer, but cannot replace the core rendering transaction through
the same narrow contract as an ArborUI backend.

## Core Strengths

### Declarative Composition With A Large Ecosystem

React is a consequential choice rather than a cosmetic syntax layer. Existing
React developers can reuse state patterns, component boundaries, testing
conventions, and third-party packages. React keys provide identity, effects
provide integration with subprocesses and external services, and the component
tree naturally describes nested layout. This lowers adoption cost for Node teams
that already use React and makes custom application composition inexpensive.

### A Practical Flexbox Renderer

Yoga gives Ink a familiar flexbox model with box dimensions, padding, borders,
alignment, growth, shrinkage, positioning, clipping, and text wrapping. The
virtual output layer handles styles, ANSI tokens, wide characters, and overlays
without requiring each component to know terminal cursor arithmetic. The source
contains explicit handling for wide-character overlap and the test suite covers
last-column, fullscreen, clipping, borders, Unicode, and resize regressions.

This is a productive middle ground: Ink is more structured than concatenating
ANSI strings, while raw stream access remains available when the normal tree is
insufficient.

### Useful Input And Lifecycle Primitives

`useInput`, `usePaste`, focus hooks, `useWindowSize`, `useAnimation`, cursor
positioning, console interception, synchronized updates, and alternate-screen
support cover many needs that otherwise become repeated application boilerplate.
Input parsing handles fragmented reads and ambiguous Escape prefixes with a
short pending-sequence timeout. Raw mode and bracketed paste are reference
counted, so multiple input components can coexist without one unmount disabling
the mode for another. Suspension in 7.1.0 extends this practical lifecycle
surface to editors, pagers, and fuzzy finders.

### Streaming History Is Explicitly Separated

`<Static>` gives Ink a clear answer for unbounded completed output. Instead of
forcing the live tree to retain every completed item, it emits new history above
the live view and removes those items from the dynamic rendering path. This
supports progress tools and test runners that need logs plus a current status
line. The separation also makes the limitation clear: immutable history and a
scrollable viewport are different products.

### Strong Maintainer-Level Regression Coverage

The Ink checkout contains focused tests for input parsing, React reconciliation,
focus, cursor positioning, concurrent rendering, resize, output throttling,
static output, terminal clearing, ended streams, and suspension. It uses fake
streams for deterministic output assertions and `node-pty` for process-isolated
terminal behavior. This is stronger than the public testing utility alone and
shows that terminal edge cases are treated as implementation concerns rather
than left entirely to application authors.

## Limitations And Frustrations

### 1. Pointer Interaction Has No Framework Path

**Classification:** Limitation with an extension failure for spatial interaction.

**Requirement:** A full-screen application should route mouse presses, releases,
movement, drag, and wheel events to the visible control under the pointer, while
respecting clipping, overlays, capture, and focus.

**Library assumption:** Input is primarily a stream of keyboard or text events.
Components subscribe through `useInput`; there is no built-in relationship
between a screen coordinate and a rendered box.

**Observable failure or friction:** Ink 7.1.0 exports `useInput` and
`usePaste`, but no mouse hook or hit-map API. The keyboard hook's `Key` type has
no pointer event shape, and the host DOM nodes do not expose a public spatial
event dispatch contract. An application can read stdin through `useStdin` and
parse mouse escape sequences itself, but it must then maintain coordinate
conversion, hit testing, clipping, z-order, capture, and focus rules outside Ink.

**Root architectural cause:** Ink's renderer produces terminal output strings;
it does not commit a parallel interaction scene or per-cell target map. React
component boundaries are not automatically terminal hit regions.

**Available workaround:** Build an application-level parser and route events to
component state, use explicit measurements or refs, or use a separate terminal
input package. This is feasible for one custom control but becomes a second UI
event system for a composed application.

**Cost of workaround:** High for dashboards and overlays. The application must
duplicate renderer decisions about clipping and layering, and it cannot rely on
Ink to repair capture or hover state when reconciliation changes the tree.

**Upstream response:** In the closed [mouse click hook issue #632](https://github.com/vadimdemedes/ink/issues/632),
the owner stated on 2023-11-11, "I have no plans to add mouse support to Ink.
You should be able to implement this yourself in your app." The issue remains
closed and no mouse API is present in the examined release.

**Current status and version:** Verified unsupported in 7.1.0. The upstream
position is intentional, not evidence of a rendering bug.

**Evidence:** [Ink exports](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/index.ts#L20-L45),
[input hook](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/hooks/use-input.ts#L126-L268),
and [maintainer comment](https://github.com/vadimdemedes/ink/issues/632#issuecomment-1806879782).

**Confidence:** High for the current API; high for the governance statement.

### 2. Scrolling And Large Collections Are Userland Responsibilities

**Classification:** Limitation and maturity problem at the widget extension
boundary.

**Requirement:** A text-heavy application needs a bounded viewport, predictable
scroll offsets, overflow indicators, dynamic item measurement, stable identity,
and visible-range construction for very large collections.

**Library assumption:** Boxes can clip overflow, and application components can
use measurements and ordinary React state to simulate a viewport. Ink should
provide primitives rather than decide scrollbar and indicator policy.

**Observable failure or friction:** The current tree painter only treats hidden
overflow as a clipping operation, and Ink 7.1.0 has no built-in scroll offset or
virtualized collection component. The open [scrolling primitives issue #765](https://github.com/vadimdemedes/ink/issues/765)
asks for scroll overflow and measurements. A maintainer response favors
`contentOffsetX/Y`, client and scroll sizes, and `useBoxMetrics` as primitives
for a userland `ScrollView`, rather than an opinionated built-in widget. Ink 7
does include `useBoxMetrics`, but not the offset or scroll contract.

**Root architectural cause:** Yoga computes the complete child layout and Ink
walks the complete tree. Clipping prevents visible output but does not by itself
avoid constructing, reconciling, measuring, or painting all children.

**Available workaround:** The ecosystem demonstrates two different strategies.
[`ink-virtual-list` 0.2.3](https://github.com/archcorsair/ink-virtual-list/blob/6f6ed6a37943a56e84d5d6b0575e811a93f7c4f4/src/VirtualList.tsx)
slices the data to visible items and exposes imperative index scrolling, but its
default model assumes a fixed positive item height and its package declares an
Ink `^6.6.0` peer. [`ink-scroll-view` 0.3.7](https://github.com/ByteLandTechnology/ink-scroll-view/blob/1d4b4b6cb5602657b5035e6be8257ed415823cb6/src/ControlledScrollView.tsx)
measures every child with `measureElement`, caches item heights, clips a nested
box, and shifts content with a negative margin. It supports variable heights,
but still renders all children and asks applications to call `remeasure()` after
resize. Gemini CLI's [MaxSizedBox workaround](https://github.com/google-gemini/gemini-cli/blob/3ff5ba20fc1ad7d867218bbdb34756eb54d6eccb/packages/cli/src/ui/components/shared/MaxSizedBox.tsx)
uses `ResizeObserver`, `maxHeight`, clipping, negative offsets, and an explicit
hidden-lines indicator instead of native scrolling.

**Cost of workaround:** A virtual list needs its own identity, measurement,
selection, and focus semantics. A measured scroll view pays for all child
construction and must coordinate resize and dynamic-height updates. A local
truncation component is easier but is not a general scroll model.

**Upstream response:** Issue #765 is open as of the research date. The discussion
explicitly prioritizes small primitives and userland battle testing; no shipped
`overflow="scroll"` or scroll offset API appears in v7.1.0.

**Current status and version:** Verified as a current limitation; the issue is
not a historical missing feature that was fixed before the selected release.

**Evidence:** [open issue and design discussion](https://github.com/vadimdemedes/ink/issues/765),
[Ink clipping path](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/render-node-to-output.ts#L160-L209),
[virtual list package metadata](https://github.com/archcorsair/ink-virtual-list/blob/6f6ed6a37943a56e84d5d6b0575e811a93f7c4f4/package.json#L1-L55),
and [scroll-view documentation](https://github.com/ByteLandTechnology/ink-scroll-view/blob/1d4b4b6cb5602657b5035e6be8257ed415823cb6/README.md#how-it-works).

**Confidence:** High for Ink's missing primitive; medium for ecosystem maturity
because third-party packages evolve independently.

### 3. Screen Ownership Trades Correct Repainting Against Scrollback

**Classification:** Tradeoff, with an active upstream behavior issue.

**Requirement:** A long-running application should state whether it owns the
alternate screen, an inline main-screen region, or native scrollback, and its
repaint strategy should not silently destroy user history.

**Library assumption:** Alternate-screen output is isolated and disposable; for
main-screen output, an oversized live frame may require clearing the whole
terminal because ordinary line erasure cannot reach content above the viewport.

**Observable failure or friction:** Ink documents that alternate-screen mode has
no scrollback. In the normal interactive path, the current source calls
`ansiEscapes.clearTerminal` when a frame is fullscreen or overflowing in the
cases covered by `shouldClearTerminalForFrame`. The call is visible in
[`renderInteractiveFrame`](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/ink.tsx#L1112-L1177).
On terminals where `clearTerminal` includes erase-saved-lines, this can remove
native scrollback while repainting. The behavior also exists to avoid stale
frames and fullscreen scrolling, so simply removing the clear is not a free
fix.

**Root architectural cause:** Ink's live output is a mutable cursor-positioned
region on a stream, not a terminal-independent viewport with an explicit history
owner. Alternate-screen isolation and main-screen repaint therefore have
different behavior, but `render()` exposes them mainly as an option rather than
as separate high-level rendering contracts.

**Available workaround:** Use `<Static>` for immutable history, constrain live
output below the viewport, use alternate screen when scrollback preservation is
not required, or patch the renderer to use viewport-only clearing. Applications
can also move scrolling into a userland component, but that does not solve the
terminal ownership question for main-screen output.

**Cost of workaround:** The application must choose between isolated full-screen
UX, append-only history, or a fragile custom repaint policy. A patch must be
revalidated across xterm-like terminals, Windows consoles, multiplexers, and
frames that exactly fill the viewport.

**Upstream response:** [Issue #935](https://github.com/vadimdemedes/ink/issues/935)
is open and describes scrollback loss from `clearTerminal`. [PR #936](https://github.com/vadimdemedes/ink/pull/936)
proposes `eraseScreen` plus cursor-home, but was still open and unmerged on the
research date. The selected v7.1.0 source still uses `clearTerminal`, so the
proposal must not be reported as shipped behavior. Older issue #359 is useful
historical evidence of the stale-frame and flicker tradeoff, not proof that the
current release has the same exact frequency of clears.

**Current status and version:** The alternate-screen limitation is documented
and verified. The main-screen scrollback concern is verified in current source
and reported by an open issue; the proposed change is not part of 7.1.0.

**Evidence:** [alternate-screen API contract](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/render.ts#L122-L135),
[current clear path](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/ink.tsx#L119-L164),
[issue #935](https://github.com/vadimdemedes/ink/issues/935), and [PR #936 status](https://github.com/vadimdemedes/ink/pull/936).

**Confidence:** High for the mode contract and source call; medium for terminal-
specific scrollback effects without a physical emulator matrix.

### 4. Stream Flush Is Not A Transactional Physical-Screen Commit

**Classification:** Extension failure for recoverable output, not a bug against
Ink's documented stream-oriented contract.

**Requirement:** If a backend accepts no bytes, accepts a complete patch, or
leaves output partially applied, the renderer must know which logical frame is
physically believed and force a full repaint after an uncertain write.

**Library assumption:** A Node writable stream is the output boundary. A write
callback or an empty write barrier is sufficient to know that pending stream
work has completed for lifecycle APIs.

**Observable failure or friction:** Ink writes rendered strings and control
sequences directly to `stdout`. `waitUntilRenderFlush()` yields to React and
waits for a writable-stream callback, which is useful for stream ordering but
does not mean a terminal has accepted and displayed a complete semantic frame.
`log-update` updates its previous output bookkeeping around the write operation,
and there is no public `WriteOutcome` or physical-state invalidation API. Some
lifecycle writes use `writeBestEffort`, which intentionally catches write errors
when restoring cursor, Kitty protocol, or alternate-screen state.

**Root architectural cause:** Ink's renderer state is the previous output string
and line count, while the backend boundary is a Node stream rather than a
transactional patch writer. A stream can report queued or completed writes
without modeling terminal parsing, and a write failure can occur after a prefix
has reached the terminal.

**Available workaround:** Use a stream implementation that guarantees ordered
whole-frame delivery, wait for `waitUntilRenderFlush()` before sequencing
application effects, use PTY tests, and recreate or fork the Ink instance after
a known output failure. None of these gives an application a supported way to
tell the existing renderer that its previous frame is physically unknown.

**Cost of workaround:** A custom stream can provide stronger transport
semantics, but it cannot provide terminal acceptance semantics. Recreating the
renderer complicates React state and terminal modes; forking the renderer makes
the application responsible for maintaining output recovery across releases.

**Upstream response:** Ink has improved stream lifecycle behavior over time.
Release 6.7.0 added flushing pending renders and awaiting stdout drain, 6.8.0
fixed ended stdout during unmount, and 7.1.0 forces a redraw after suspension.
These changes address deferred callbacks, closed streams, and stale output after
a child process, but they do not introduce a complete-patch commit protocol.
The recorded test suite contains delayed callbacks and ended-stream cases; a
byte-by-byte partial-write fault injector or terminal emulator was not found in
the recorded checkout.

**Current status and version:** Verified as an absent public transaction
contract in 7.1.0. The testing gap is scoped to the searched source and should
not be read as proof that no downstream user has such a harness.

**Evidence:** [flush implementation](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/ink.tsx#L933-L979),
[direct stream writes and best-effort cleanup](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/ink.tsx#L706-L775),
[log-update bookkeeping](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/log-update.ts#L55-L170),
and [stream/lifecycle tests](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/test/render.tsx#L1054-L1249).

**Confidence:** High for the public and source contract; medium for the
practical frequency of partial terminal writes because no physical failure
reproduction was run.

## Testing Strategy

Ink's own tests are more comprehensive than its public testing package. The
repository uses AVA with serial execution for terminal-sensitive tests, TypeScript
type checking, XO linting, fake timers, fake stdin/stdout helpers, and
`node-pty`. The fake stdout records every write and exposes configurable columns
and TTY state. The PTY helper launches real fixture processes at a fixed width,
injects input, and captures output. It is used for raw input, escape parsing,
resize, child-process behavior, fullscreen boundaries, and exit behavior.

The source suite also includes useful reference-style tests. `renderToString`
helpers render at an explicit width and assert text, borders, padding, wrapping,
and screen-reader output. Concurrent tests wrap updates in React `act()`. Fake
timers verify `maxFps`, animation and pending-render flush behavior. Stream tests
delay write callbacks to prove that `waitUntilRenderFlush` and `waitUntilExit`
do not resolve before queued writes. Fullscreen tests count clear and erase
sequences and distinguish initial overflow from later transitions. These are
good examples of testing the actual Ink renderer rather than only testing a
component's state.

The production tests still have important representation limits. The PTY helper
captures bytes from `node-pty`; it does not apply those bytes to a semantic
virtual terminal and inspect the resulting cells, cursor, scrollback, or hit
targets. Many assertions strip ANSI or count control sequences. The fake stdout
does not model a terminal emulator, and the tests searched at the release
revision did not expose a fuzz target for fragmented rendering or partial
output at every byte boundary. This is a reasonable cost for Ink's scope, but
it leaves protocol and physical-screen behavior dependent on fixture design and
external manual testing.

The public [`ink-testing-library` 4.0.0 implementation](https://github.com/vadimdemedes/ink-testing-library/blob/4993171957dff60858bc9a860327f5d305696bc9/source/index.ts)
is intentionally smaller. `render()` injects fake stdin, stdout, and stderr,
sets `debug: true`, and returns `lastFrame()`, all `frames`, `rerender`,
`unmount`, and the fake streams. Its stdout uses a fixed width of 100; stdin
emits readable and data events and provides no-op raw-mode methods. The package
tests cover initial frames, rerendering, lifecycle effects, stdin, and stderr.
This is convenient for semantic string assertions, but it does not provide
`waitUntilRenderFlush`, manual clock control, resize dimensions, mouse input,
PTY lifecycle, a virtual screen, failure injection, or a settling primitive.

Therefore the answer to the testing framework questions is mixed:

| Question | Ink evidence and result |
| --- | --- |
| Can users test a complete app without a real terminal? | Partially. Component rendering and many application paths work with fake streams; lifecycle and terminal modes need custom or PTY tests. |
| Does the harness exercise production code? | The fake harness uses Ink's renderer but intentionally enables debug output and replaces terminal streams, so it does not exercise the normal interactive erase path. |
| Can tests inject key and paste input? | Yes through stdin writes, including raw escape sequences and bracketed paste; no public mouse injection exists. |
| Can tests control clocks and settling? | Ink's own tests use fake timers and direct delays; `ink-testing-library` has no clock or `waitUntilRenderFlush` API. |
| Can tests inspect semantic layout, focus, cursor, or hit targets? | Focus and cursor can be inferred through output or custom components; there is no public hit-map inspection or virtual cell model. |
| Can tests simulate output failures? | The repository tests custom delayed and ended streams; the public utility has no failure-injection abstraction. |
| Are PTY or emulator tests present? | PTY tests using `node-pty` are present; a semantic terminal emulator was not found in the recorded checkout. |

Gemini CLI shows how a substantial application fills the gaps. Its application
test utilities provide `waitUntilReady`, fake timers, semantic assertions, and
snapshots around a custom `MaxSizedBox`; its tests settle the rendering system
before capturing a frame. That is strong application engineering, but it also
demonstrates that the complete harness belongs to the downstream application
rather than to `ink-testing-library`.

## Scenario Summary

| Scenario | Assessment | Explanation |
| --- | --- | --- |
| Form with focus traversal and modal | Supported, application-composed | `useFocus`, `useFocusManager`, `useInput`, and ordinary conditional React trees cover the basic model; modal scope restoration is not a built-in contract. |
| Large scrollable collection | Partial | Third-party virtual lists or measured scroll views are required; Ink itself constructs the normal React tree. |
| Streaming external output | Supported with `<Static>` | Static history plus a live region is a strong pattern; console and stream writes can still disturb live output if lifecycle sequencing is wrong. |
| Unicode-heavy text input | Partial to supported | Ink uses ANSI-aware width helpers and tests wide-character cases, but the public contract does not expose ArborUI's explicit grapheme and width-policy model. |
| Clipped overlay with mouse interaction | Visual clipping supported; pointer routing unsupported | Box clipping and ordering work, but coordinate hit testing and pointer capture are application responsibilities. |
| Resize during active updates | Supported with caveats | Ink listens to stdout resize and recalculates layout; width decreases trigger clearing, and custom components may need measurement coordination. |
| Deferred or failed output | Partial | Stream callbacks and closed-stream cleanup are tested; uncertain partial output does not produce a public full-repaint invalidation outcome. |
| Suspension to a child process | Supported in 7.1.0 | `suspendTerminal()` restores modes, exits alternate screen, and forces a redraw on resume. |
| Long idle periods | Supported by default; animations opt in | Normal updates do not continuously paint, while `useAnimation` owns a timer and render throttling. |
| Preserving native scrollback | Mode-dependent and incomplete | Alternate screen intentionally hides scrollback; main-screen overflow clearing has an open current issue and no explicit native-scrollback mode. |

## Lessons For ArborUI

### Adopt

Adopt the separation between declarative composition and terminal serialization.
Ink shows that a component tree plus a layout engine is a productive application
surface, even when the underlying renderer remains cell-oriented. ArborUI should
retain stable identity and separate it from ephemeral view descriptions, while
keeping its model-update-view and ownership rules rather than reproducing
React's hook lifecycle. This is consistent with the [ArborUI architecture](../../../docs/architecture.md)
and its retained tree design.

Adopt a clear append-only history primitive as a different contract from a live
scroll viewport. Ink's `<Static>` is a useful precedent: completed records can
leave the live layout tree, reducing the cost of unbounded logs. ArborUI should
make the same distinction in widget and terminal-mode APIs instead of treating
history, scrolling, and alternate-screen repaint as one configurable behavior.

Adopt Ink's practical input tests and lifecycle scenarios. Fragmented Escape
sequences, bracketed paste, raw-mode reference counting, resize boundaries,
fullscreen last-column behavior, cursor positioning, child-process handoff, and
stream barriers deserve focused tests. A library should test application-visible
behavior as well as parser functions.

Adopt a full-render reference path before optimizing. Ink's output construction
and line diff are understandable because the complete string is available before
`log-update` writes it. ArborUI should similarly compare optimized patches with a
simple complete-frame result and measure end-to-end bytes and latency before
adding dirty-subtree or damage-region complexity.

### Avoid

Avoid making a Node-like stream callback the equivalent of terminal acceptance.
ArborUI's prepared frame must remain uncommitted until the backend reports the
complete patch outcome. Deferred, failed, and state-unknown writes need distinct
semantics, and an uncertain write must force a full repaint. This is a genuine
difference from Ink's current boundary, not merely a preference for Rust APIs.

Avoid using alternate-screen mode as a generic answer to scrollback. Ink's
documentation is correct that alternate screen isolates the session, but that
contract cannot satisfy a conversation or log application that promises native
history. ArborUI should define alternate, main-screen inline, and native-
scrollback modes separately, with ownership and recovery tests for each.

Avoid requiring every application to rebuild pointer routing, scrolling, and
large-collection measurement. Ink's ecosystem proves that these features can be
layered on, but the resulting packages either assume fixed item heights, render
all children, manually remeasure on resize, or implement application-specific
truncation. ArborUI's current hit map and scroll widgets are valuable only if
their performance and stable-identity behavior are proven with a substantial
application.

Avoid treating string snapshots as a complete terminal test. Ink's fake-frame
approach is excellent for quick component assertions, but ArborUI should retain
semantic frame, style, cursor, hit-map, patch, and output-outcome assertions,
then use PTYs and terminal emulators where cell behavior cannot be represented by
a string.

### ArborUI Already Approaches Differently

ArborUI already has the strongest answer to Ink's output boundary: a prepared
frame and an explicit applied/deferred/state-unknown outcome. It also separates
grapheme storage, cell continuation invariants, terminal capabilities, UI
identity, runtime scheduling, and terminal state. The [rendering and text design](../../../docs/rendering-and-text.md)
defines explicit width policies and atomic wide-grapheme runs, while the
[terminal contract](../../../docs/terminal.md) makes physical-state invalidation
part of backend behavior.

ArborUI also approaches interaction as a first-class retained concern. Its
normalized mouse events, hit maps, capture and bubble routing, focus scopes, and
transactional UI-frame coordination directly address the gap exposed by Ink's
keyboard-only hooks. The public `arborui-test` harness likewise drives the real
runtime and renderer through an in-memory terminal rather than only collecting
debug strings.

These differences should not be presented as free superiority. Ink's React
ecosystem and component ergonomics are substantial advantages, and ArborUI's
full-frame work, ownership rules, and Rust lifetimes impose their own adoption
cost. The comparison supports a narrower claim: ArborUI is targeting guarantees
below Ink's extension boundary, not merely providing another JSX-like layout
library.

### Claims ArborUI Has Not Yet Proven

ArborUI has not yet proved that its full layout and painting path remains fast
for large dynamic trees, scrolling logs, or Unicode-heavy text at realistic
update rates. The repository's roadmap still calls for production-scale
application proof and a virtualized collection prototype. It also has not
stabilized main-screen inline or native-scrollback ownership, emulator-specific
behavior, or Unix job-control signals. The [compatibility notes](../../../docs/compatibility.md)
explicitly limit the high-level supported mode to alternate-screen fullscreen
use.

The strongest follow-up is not another feature inventory. It is a focused proof
matrix:

- Build a virtualized collection with stable item keys, variable-height
  measurement, focus, selection, overscan, and explicit backpressure, then
  compare it with `ink-virtual-list` and `ink-scroll-view` workloads.
- Inject output failure at every byte boundary in a PTY or terminal-emulator
  adapter and verify that ArborUI never commits a partial frame or dispatches an
  event against the wrong hit map.
- Compare a full-frame reference renderer with optimized patches for one changed
  cell, large scroll regions, overlays, resize storms, and wide graphemes.
- Add an explicit main-screen/native-scrollback prototype only after defining
  append history, viewport ownership, external output, resize recovery, and
  suspension semantics.
- Measure end-to-end input-to-write-complete latency, emitted bytes, allocations,
  full-repaint count, and idle CPU against a comparable Ink application rather
  than claiming performance from architecture alone.

## Evidence Appendix

### Ink Source And Documentation

- [Ink 7.1.0 release](https://github.com/vadimdemedes/ink/releases/tag/v7.1.0), published 2026-06-17; release notes include `suspendTerminal()`.
- [Pinned package metadata](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/package.json), source revision `25766aec618bd62030069f57dd081e5ebdd46add`.
- [Custom React reconciler](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/reconciler.ts), host nodes, mutation support, Yoga integration, and commit callbacks.
- [DOM and Yoga node model](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/dom.ts), text measurement and clipping-related node state.
- [Renderer and virtual output](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/renderer.ts),
  [output grid](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/output.ts),
  and [tree painter](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/render-node-to-output.ts).
- [Interactive lifecycle and output](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/ink.tsx),
  including resize, flush barriers, suspension, alternate screen, and clear fallback.
- [Render options and public instance](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/render.ts),
  including debug, interactive, alternate-screen, concurrent, and incremental modes.
- [Static append-only component](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/components/Static.tsx)
  and [public exports](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/index.ts).
- [Input parser](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/input-parser.ts),
  [keypress parser](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/parse-keypress.ts),
  [focus hook](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/hooks/use-focus.ts),
  and [cursor hook](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/src/hooks/use-cursor.ts).

### Ink Tests And Testing Utility

- [Ink render tests](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/test/render.tsx),
  including PTY fixtures, resize, full-height behavior, throttling, delayed writes, and flush ordering.
- [Ink terminal helper](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/test/helpers/term.ts), which uses `node-pty` with a fixed 100-column xterm-color process.
- [Ink render helpers](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/test/helpers/test-renderer.ts) and [fake stdout](https://github.com/vadimdemedes/ink/blob/25766aec618bd62030069f57dd081e5ebdd46add/test/helpers/create-stdout.ts).
- [`ink-testing-library` 4.0.0 package metadata](https://github.com/vadimdemedes/ink-testing-library/blob/4993171957dff60858bc9a860327f5d305696bc9/package.json) and [fake stream source](https://github.com/vadimdemedes/ink-testing-library/blob/4993171957dff60858bc9a860327f5d305696bc9/source/index.ts).

### Issues, Maintainer Statements, And Releases

- [Mouse hook issue #632](https://github.com/vadimdemedes/ink/issues/632), closed 2023-11-11;
  [owner statement](https://github.com/vadimdemedes/ink/issues/632#issuecomment-1806879782) says mouse support is not planned.
- [Scrolling primitives issue #765](https://github.com/vadimdemedes/ink/issues/765), open as of 2026-07-16;
  [maintainer proposal](https://github.com/vadimdemedes/ink/issues/765#issuecomment-3333113989) favors primitives for userland scrolling.
- [Scrollback issue #935](https://github.com/vadimdemedes/ink/issues/935), open as of 2026-07-16;
  [historical maintainer explanation](https://github.com/vadimdemedes/ink/issues/935#issuecomment-4230051376) describes the stale-frame and full-clear tradeoff.
- [Scrollback-preserving PR #936](https://github.com/vadimdemedes/ink/pull/936), open and unmerged as of 2026-07-16.
- [Declarative cursor PR #872](https://github.com/vadimdemedes/ink/pull/872), open and unmerged as of 2026-07-16; this confirms cursor positioning remains an active API boundary.
- [Ink 7.0.0 release and migration guide](https://github.com/vadimdemedes/ink/releases/tag/v7.0.0), including Node/React requirements, input changes, concurrent rendering, alternate screen, and `useBoxMetrics`.
- [Ink 6.7.0 release](https://github.com/vadimdemedes/ink/releases/tag/v6.7.0), including synchronized updates, cursor positioning, and stdout-drain changes.
- [Ink 6.8.0 release](https://github.com/vadimdemedes/ink/releases/tag/v6.8.0), including ended-stdout and static-output fixes.

### Ecosystem And Application Evidence

- [`ink-virtual-list` 0.2.3 package](https://github.com/archcorsair/ink-virtual-list/blob/6f6ed6a37943a56e84d5d6b0575e811a93f7c4f4/package.json),
  tag commit `6f6ed6a37943a56e84d5d6b0575e811a93f7c4f4`;
  [implementation](https://github.com/archcorsair/ink-virtual-list/blob/6f6ed6a37943a56e84d5d6b0575e811a93f7c4f4/src/VirtualList.tsx)
  and [tests](https://github.com/archcorsair/ink-virtual-list/blob/6f6ed6a37943a56e84d5d6b0575e811a93f7c4f4/tests/VirtualList.test.tsx).
- [`ink-scroll-view` 0.3.7 package](https://github.com/ByteLandTechnology/ink-scroll-view/blob/1d4b4b6cb5602657b5035e6be8257ed415823cb6/package.json),
  source revision `1d4b4b6cb5602657b5035e6be8257ed415823cb6`;
  [scroll wrapper](https://github.com/ByteLandTechnology/ink-scroll-view/blob/1d4b4b6cb5602657b5035e6be8257ed415823cb6/src/ScrollView.tsx),
  [measurement](https://github.com/ByteLandTechnology/ink-scroll-view/blob/1d4b4b6cb5602657b5035e6be8257ed415823cb6/src/ControlledScrollView.tsx),
  and [README contract](https://github.com/ByteLandTechnology/ink-scroll-view/blob/1d4b4b6cb5602657b5035e6be8257ed415823cb6/README.md#usage).
- [Gemini CLI package metadata](https://github.com/google-gemini/gemini-cli/blob/3ff5ba20fc1ad7d867218bbdb34756eb54d6eccb/packages/cli/package.json),
  which pins `ink: npm:@jrichman/ink@6.6.9` and React 19.2.4.
- [Gemini CLI `MaxSizedBox`](https://github.com/google-gemini/gemini-cli/blob/3ff5ba20fc1ad7d867218bbdb34756eb54d6eccb/packages/cli/src/ui/components/shared/MaxSizedBox.tsx)
  and [tests](https://github.com/google-gemini/gemini-cli/blob/3ff5ba20fc1ad7d867218bbdb34756eb54d6eccb/packages/cli/src/ui/components/shared/MaxSizedBox.test.tsx),
  inspected at commit `3ff5ba20fc1ad7d867218bbdb34756eb54d6eccb`.

### ArborUI Comparison Sources

- [ArborUI architecture](../../../docs/architecture.md)
- [ArborUI rendering and text](../../../docs/rendering-and-text.md)
- [ArborUI UI and runtime](../../../docs/ui-and-runtime.md)
- [ArborUI terminal contract](../../../docs/terminal.md)
- [ArborUI testing and roadmap](../../../docs/testing-and-roadmap.md)
- [ArborUI compatibility](../../../docs/compatibility.md)

All external sources were accessed on 2026-07-16. Links are pinned to commits
where available, and issue or pull-request state is recorded at access time.
Verified conclusions use implementation or tests; supported conclusions use
primary documentation; reported conclusions use maintainers, issues, or
downstream applications.
