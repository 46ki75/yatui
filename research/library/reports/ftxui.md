# FTXUI Research Report

## Evidence Header

```text
Research date: 2026-07-16
Project version: FTXUI 7.0.1
Project revision: c100eab535db2283b78d30fcb6d082a1f84fb683
Repository: https://github.com/ArthurSonzogni/FTXUI
Documentation version: Official documentation site labeled FTXUI 7.0.1; accessed 2026-07-16
Primary platform examined: Source inspection on Linux; no physical terminal reproduction
Report depth: Standard profile
```

The latest stable release at the start of this research was
[FTXUI 7.0.1](https://github.com/ArthurSonzogni/FTXUI/releases/tag/v7.0.1),
released on 2026-07-14. Implementation conclusions refer to the pinned commit
above. The current documentation site was checked for intent and examples, but
release-sensitive claims were verified against the tagged source. The project
advertises Linux and macOS as its main targets, with Windows and WebAssembly
support. No physical terminal or PTY reproduction was run for this report.

## Executive Assessment

FTXUI is a C++ functional terminal UI library that spans three related layers:
the `screen` layer owns cells and terminal serialization, the `dom` layer builds
and lays out hierarchical elements, and the `component` layer supplies retained
component objects, focus and event routing, terminal lifecycle, and an
application loop. It is therefore more than a renderer, but less opinionated
than a state-management framework: application state remains in user code and
components usually render from captured references or callbacks.

FTXUI is a strong fit for compact to medium-sized interactive C++ tools that
want declarative composition, built-in controls, keyboard and mouse interaction,
and a ready-made terminal loop. Its most consequential boundary for ArborUI is
not basic UI capability. The boundary is ownership and recovery: the default
runtime owns a concrete terminal process and writes
serialized output through `std::cout`, while the DOM expects the application to
describe a complete tree. Large virtualized collections, recoverable output
transactions, and explicit physical-screen uncertainty are outside that
contract or require custom elements and loops.

## Project Snapshot

FTXUI is a cross-platform C++ library with a functional style inspired by React.
The pinned build uses C++17 for consumers. Ordinary CMake builds require CMake
3.12; optional C++20 modules require CMake 3.28.2. Its README records the
proposition, platforms, packages, and three-module organization in the
[feature and module overview](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/README.md#L37-L124).

The public catalog includes buttons, input, menus, sliders, resizable splits,
modals, hover handling, and custom renderers. Recent releases fixed build,
signal, piped-input, color, Unicode-input, and rendering details without
changing the central architecture.

The pinned [json-tui source](https://github.com/ArthurSonzogni/json-tui/tree/5ec4942e7037a4343066c38d66ddc7957e175941)
is a JSON browser using FTXUI's three layers. It demonstrates ecosystem fit,
not physical-terminal reliability.

## Core Proposition

FTXUI makes a terminal UI look like a composition of values and functions. A
renderer returns an `Element`; elements are combined with `vbox`, `hbox`,
`dbox`, decorators, borders, sizing, clipping, and focus-related decorators.
An interactive component returns an element from `OnRender()` and handles an
`Event` from `OnEvent()`. This gives application authors a compact way to keep
visual composition near interaction logic without adopting a separate markup
language or runtime.

The library combines functional DOM composition with a component runtime.
Lower-level terminal libraries generally stop at input decoding or cell output;
FTXUI adds focus, controls, animation, terminal modes, signals, and a loop. The
tradeoff is that the DOM is reconstructed and laid out from the component tree;
it is not a keyed scene graph with automatic collection virtualization.

## Architecture

### Components And State

`Component` is a `std::shared_ptr<ComponentBase>` with children, rendering,
event, animation, focus, and parent/child operations. Its public contract is in
the [component base header](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/include/ftxui/component/component_base.hpp#L24-L112).
The tree is retained for interaction identity and focus, but the `Element`
returned for a render is a separate `std::shared_ptr<Node>` DOM value, usually
created from application state on each render.

`Container::Vertical`, `Horizontal`, `Tab`, and `Stacked` provide focus and
event routing. Events go to the focused child before navigation or custom
handling. `Renderer`, `CatchEvent`, and `Modal` are small extension points.

### DOM And Rendering

The DOM's `Node` contract has four primary phases: compute requirements, assign
a box, perform selection, and render. The pinned implementation recursively
computes child requirements, assigns boxes, allows up to twenty layout
iterations, selects content, sets the cursor, and renders into a `Screen`.
See the [Node contract](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/include/ftxui/dom/node.hpp#L21-L102)
and [render pipeline](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/src/ftxui/dom/node.cpp#L105-L177).

`Screen` is a rectangular grid of `Cell` values. Cells store UTF-8 graphemes and
styles; `ToString()` serializes the grid and `ResetPosition()` produces cursor
movement and optional clears. `App` chooses dimensions, renders the DOM, turns
the screen into terminal sequences, flushes stdout, clears the logical screen,
and marks the frame valid. The relevant
[draw implementation](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/src/ftxui/component/app.cpp#L1003-L1132)
is simple and inspectable, but is not an injected backend transaction.

### Runtime And Lifecycle

`App` inherits `Screen` and offers alternate-screen, primary-screen,
fit-component, fixed-size, and terminal-output modes. It installs raw input,
mouse, line-wrap, and screen settings; parses terminal replies; and restores
state on normal and signal paths. `WithRestoredIO()` temporarily uninstalls
hooks for child programs or ordinary stdin/stdout. POSIX Ctrl-Z also uninstalls
and reinstalls the configuration.

Tasks are a variant of events, closures, and animation tasks. `Post` and
delayed tasks are mutex-protected, and `RequestAnimationFrame()` asks for a
render. `Loop::RunOnce()` and `RunOnceBlocking()` support embedding in another
loop, as shown by the [custom-loop example](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/examples/component/custom_loop.cpp#L17-L53).
FTXUI supplies scheduling primitives, not an async runtime or backpressure
policy.

## Core Strengths

### Effective Composition And Extension

The DOM makes layout and overlays concise, while custom `Node` and
`ComponentBase` subclasses can replace rendering or event behavior. Standard
decorators cover much ordinary composition without manual cell painting.

### A Complete Interactive Baseline

Unlike a renderer-only library, FTXUI provides focus traversal, mouse capture,
selection, input controls, parsing, animations, and an application lifecycle.
`TerminalOutput` and `WithRestoredIO` support applications that combine an
interactive region with ordinary terminal or child-process I/O.

### Good Logical Unicode And Rendering Foundations

Cells can hold combining sequences, and the string layer exposes glyph
segmentation, width calculation, and cell-to-glyph mapping. Tests cover
combining marks, CJK fullwidth characters, emoji, and controls. The width table
is based on Unicode 13.0 data, so this is strong logical handling rather than a
guarantee that every terminal and newer sequence will agree.

### Practical Source-Level Testability

The same `Screen` and DOM paths can be tested without a terminal. Exact strings,
cell widths, component events, focus movement, signal paths, and stdout capture
are represented in repository tests, though not every physical failure mode.

## Limitations And Frustrations

### Full Trees Are Not Virtualized Collections

```text
Classification: Limitation with an extension-boundary cost
Requirement: Large scrollable collections with stable item identity
Library assumption: The application supplies a complete component/element tree
Observable friction: Child creation, requirement computation, and rendering scale with the supplied collection
Root architectural cause: Vertical containers and the default DOM pipeline recursively visit their children
Available workaround: Slice the visible range in application code or implement a custom Node/component
Cost of workaround: The application owns indexing, scroll anchoring, focus identity, and variable-height behavior
Upstream response: Issue #984 is closed; its example uses application-level visible-range slicing, not a generic virtual-list contract
Current status and version: No generic virtualization API found at v7.0.1
Evidence: Verified traversal; inferred absence of a built-in virtualization layer
Confidence: Medium
```

`VerticalContainer::OnRender()` and the base DOM requirement pass visit every
child. `frame` moves a focused item into view but does not virtualize child
construction. For an unbounded log or large collection, the application must
slice a visible window or write a custom element. [Issue #984](https://github.com/ArthurSonzogni/FTXUI/issues/984)
shows that workaround, which moves indexing and focus/scroll anchoring out of
the standard component contract.

### Output Is Serialized Through An Unrecoverable Stream Boundary

```text
Classification: Extension failure relative to ArborUI's output requirement
Requirement: Commit a prepared frame only after complete backend acceptance and recover after uncertain output
Library assumption: Writing the serialized screen to stdout and flushing is sufficient for the terminal session
Observable friction: Stream failure or partial delivery is not represented as applied, deferred, or unknown; the frame is still cleared and marked valid
Root architectural cause: App accumulates ANSI text in a string and writes it through std::cout rather than an outcome-aware backend
Available workaround: Use Screen/DOM without App and own output, or terminate/reinitialize after failure
Cost of workaround: Reimplement terminal ownership or give up the standard lifecycle and component loop
Upstream response: Recent releases improve signal and terminal integration, but no transactional output contract was found
Current status and version: Verified in v7.0.1 source
Evidence: App::Internal::TerminalFlush and Draw; reported integration issues #1209 and #723
Confidence: High for the source contract; medium for physical failure impact
```

`Draw()` serializes the screen, calls `TerminalFlush()`, then clears the screen
and sets `frame_valid_ = true`. `TerminalFlush()` performs
`std::cout << output_buffer << std::flush` without an acceptance result. Stream
failure or partial lower-level writes do not enter the frame state machine. The
usual response is to exit or restore the terminal, which does not meet a
commit-after-acceptance contract.

The public `Screen` API is an escape hatch, but raw mode, cursor queries, mouse
modes, signals, and recovery remain coupled to `App`. [Issues #1209](https://github.com/ArthurSonzogni/FTXUI/issues/1209)
and [#723](https://github.com/ArthurSonzogni/FTXUI/issues/723) document the
integration cost of mixing ordinary output with an active screen.

### Primary-Screen And Native-Scrollback Semantics Are Partial

```text
Classification: Tradeoff
Requirement: Preserve native scrollback or inline output while retaining reliable resize, cursor, and external-I/O behavior
Library assumption: A primary-screen or terminal-output mode can manage a live region with cursor repositioning
Observable friction: Resize and external output require cursor queries, clears, and careful coordination with the shared terminal stream
Root architectural cause: The live UI and ordinary terminal output share one physical cursor and one input/output protocol
Available workaround: Prefer the alternate screen, call WithRestoredIO for child I/O, or take over the loop and terminal integration
Cost of workaround: Alternate-screen applications lose native scrollback; custom integration owns terminal capability and cursor policy
Upstream response: Resize issue #951 was closed with a linked fix; current source still has special resize events and cursor-position queries
Current status and version: Supported but explicitly fragile outside alternate-screen mode in v7.0.1
Evidence: App mode comments and Draw/Signal implementation; issues #951, #723, and #1209
Confidence: High for the mode distinction; medium for emulator-specific failure rates
```

FTXUI distinguishes modes: alternate screen avoids preceding content, while
primary screen documents resize disturbance. `TerminalOutput` uses cursor
positioning and replies to operate in the primary screen; on width decrease it
may clear output after wrapping makes the display dirty. These measures mean
native scrollback is a negotiated protocol, not an immutable history buffer.

The [POSIX piped-input documentation](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/doc/posix_pipe.md#L1-L58)
also makes platform scope explicit: the `/dev/tty` input split is Linux/macOS
only, not a portable remote transport abstraction.

## Testing Strategy

The repository uses GoogleTest for controls, containers, modals, input parsing,
DOM layout, screen serialization, Unicode strings, and compatibility. Tests
inject `Event` values directly and assert state or exact `Screen::ToString()`
output. The [test registration](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/cmake/ftxui_test.cmake#L9-L79)
and [screen tests](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/src/ftxui/screen/screen_test.cpp#L49-L124)
show deterministic logical assertions rather than visual-only snapshots.

Unicode tests assert fullwidth widths, combining marks, glyph-to-cell mapping,
and controls. Parser tests cover UTF-8, escape timing, mouse reports, terminal
identification, and capability replies. Application tests capture stdout and
exercise signals, posting, fixed-size output, and teardown. Stdout capture is
not a PTY and cannot model cursor, wrapping, or physical scrollback.

FTXUI also builds six libFuzzer targets for terminal input, components, DOM
layout, canvas, UTF-8, and colors, as shown in the
[fuzzer configuration](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/cmake/ftxui_fuzzer.cmake#L1-L25).
The benchmark target is focused on DOM performance. A manual Linux emulator
script checks terminal identification, but is not a semantic emulator test.

Users can test DOM and components without a terminal and drive a custom `Loop`,
but no public full-application harness with controlled clocks,
run-until-settled semantics, or injected backend outcomes was found. No PTY
matrix was found for raw mode, suspend/resume, restoration, partial writes,
resize storms, or physical final-column behavior. The suite is strong for
logical correctness and parser robustness, not ArborUI's commit/recovery
guarantees.

## Common Scenario Assessment

| Scenario | Assessment |
| --- | --- |
| Form with focus and modal | Strong; built-in controls, containers, focus, and `Modal` cover the common path |
| Large keyed collection | Partial; visual scrolling exists, but visible-range construction and stable identity are application-owned |
| Streaming external updates | Supported through `Post`/closures; async runtime, cancellation, and backpressure are external |
| Unicode text input | Strong logical coverage; physical terminal and newer-width compatibility remain external |
| Overlay with clipping and mouse | Strong; `dbox`, `frame`, mouse capture, and modal routing are available |
| Resize during active updates | Supported in fullscreen; primary/terminal-output modes need cursor and protocol coordination |
| Deferred or failed output | Weak; no applied/deferred/unknown write result or forced recovery repaint |
| Suspend to child process | Supported through Ctrl-Z handling and `WithRestoredIO`, with platform-specific terminal behavior |
| Long idle periods | No redraw when the frame is valid, but the animation scheduler still wakes periodically |
| Native scrollback conversation | Partial; `TerminalOutput` provides a mode, not an immutable history/output contract |

## Lessons For ArborUI

ArborUI should adopt FTXUI's small widget and element extension points, the
separation between interaction and a logical cell surface, explicit terminal
modes, direct screen serialization, and parser fuzzing. Alternate screen,
primary screen, fit-to-component, and output regions should remain distinct
contracts.

ArborUI should avoid coupling frame validity to an ordinary stream flush. A
backend outcome must distinguish accepted, rejected, and uncertain output, with
uncertainty forcing a full repaint. It should also provide stable collection
identity and visible-range semantics, and route cursor/capability replies before
application events through one input owner.

The comparison does not prove that ArborUI's stronger contracts reduce total
complexity. FTXUI's simpler model is attractive for small applications, fatal
output failures, or user-owned event loops. ArborUI must measure against
idiomatic FTXUI rather than an awkward implementation.

Follow-up work should compare the same form, streaming dashboard, and large
collection; benchmark complete frames and bytes; add PTY tests for resize and
suspend; and inject partial output. A virtualized-collection prototype can test
the benefit of ArborUI's retained identity over FTXUI's custom-element path.

## Evidence Appendix

All sources were accessed on 2026-07-16 unless another date is listed.

| Claim | Source | Version or revision | Source date | Status and notes |
| --- | --- | --- | --- | --- |
| Stable release and release scope | [v7.0.1 release](https://github.com/ArthurSonzogni/FTXUI/releases/tag/v7.0.1), [pinned changelog](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/CHANGELOG.md#L7-L147) | `v7.0.1`, `c100eab` | 2026-07-14 | Verified; latest stable baseline |
| Three-layer architecture and platform intent | [README](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/README.md#L37-L124) and [CMake targets](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/CMakeLists.txt#L26-L170) | `c100eab` | 2026-07-14 | Verified |
| Component tree, focus, and event contract | [component base](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/include/ftxui/component/component_base.hpp#L24-L112), [containers](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/src/ftxui/component/container.cpp#L18-L176) | `c100eab` | 2026-07-14 | Verified |
| DOM phases and screen representation | [Node](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/include/ftxui/dom/node.hpp#L21-L102), [DOM render](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/src/ftxui/dom/node.cpp#L105-L177), [Cell](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/include/ftxui/screen/cell.hpp#L14-L54) | `c100eab` | 2026-07-14 | Verified |
| Output and lifecycle contract | [App API](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/include/ftxui/component/app.hpp#L28-L141), [App implementation](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/src/ftxui/component/app.cpp#L750-L805), [draw/flush](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/src/ftxui/component/app.cpp#L1003-L1132) | `c100eab` | 2026-07-14 | Verified; no physical failure reproduction |
| Unicode behavior | [string API](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/include/ftxui/screen/string.hpp#L29-L39), [Unicode tests](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/src/ftxui/screen/string_test.cpp#L11-L127) | `c100eab` | 2026-07-14 | Verified logically; physical terminal behavior unknown |
| Full-tree collection workaround | [frame implementation](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/src/ftxui/dom/frame.cpp#L39-L135), [issue #984](https://github.com/ArthurSonzogni/FTXUI/issues/984) | Source `c100eab`; issue closed, opened 2025-01-06 | 2025-01-06 | Verified traversal; workaround reported; no generic virtual list found |
| Output and primary-screen integration concerns | [issue #951](https://github.com/ArthurSonzogni/FTXUI/issues/951), [issue #723](https://github.com/ArthurSonzogni/FTXUI/issues/723), [issue #1209](https://github.com/ArthurSonzogni/FTXUI/issues/1209) | #951 closed; #723 open; #1209 closed | 2023-08-15 to 2026-03-04 | Reported; current source checked separately |
| Testing and fuzzing scope | [test registration](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/cmake/ftxui_test.cmake#L9-L79), [fuzzers](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/cmake/ftxui_fuzzer.cmake#L1-L25), [terminal emulator script](https://github.com/ArthurSonzogni/FTXUI/blob/c100eab535db2283b78d30fcb6d082a1f84fb683/tools/test_all_linux_terminal_emulator.sh#L1-L102) | `c100eab` | 2026-07-14 | Verified; no repository PTY/failure matrix found |
