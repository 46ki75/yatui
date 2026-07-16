# Blessed Research Report

## Evidence Header

```text
Research date: 2026-07-16
Project version: blessed 0.1.81 (latest npm release)
Project revision: eab243fc7ad27f1d2932db6134f7382825ee3488
Repository: https://github.com/chjj/blessed
Documentation version: README at eab243fc7ad27f1d2932db6134f7382825ee3488
Primary platform examined: Source inspection on Linux; no physical terminal reproduction
Report depth: Standard profile
```

The npm baseline is 0.1.81, published in 2015, with Node.js `>= 0.8.0` and no
runtime dependencies. Its release tag is `a45575fee63fac158fd467087ec172f657bfec6b`;
source inspection used the later repository head above, dated 2016-01-04.
Forks are ecosystem evidence, not current `blessed` behavior.

## Executive Assessment

`blessed` is a retained-mode Node.js terminal UI library with a substantial
widget catalog and unusually direct terminal control. An application creates a
`Screen`, attaches `Node` and `Element` objects, handles events, mutates widget
state, and explicitly calls `screen.render()`. The library owns the alternate
screen, terminal capability lookup, input decoding, focus bookkeeping, cell
buffers, and ANSI output, but it does not own an application event loop or
asynchronous effect model.

Its strongest use case is a focused full-screen Node application that wants
ready-made forms, lists, text input, scrolling, overlays, mouse support, and
low-level escape-sequence access without adopting a larger framework. It is a
poor baseline for ArborUI's full application contract when the requirement is a
settled, serializable update model, deterministic complete-application tests,
or recovery after an uncertain terminal write. Those are not merely API style
differences: the renderer updates its physical-screen comparison state while
writing through a buffered stream, and the package provides no transaction
boundary around that operation.

## Project Snapshot

`blessed` is JavaScript running on Node.js. It presents itself as a high-level
terminal interface library, not an application framework. Its primary category
is retained rendering and widgets, with a terminal substrate embedded in the
`Program` object. The target is an interactive full-screen application using
the alternate buffer. The built-in surface includes boxes, text, lists, tables,
forms, buttons, textboxes, scrollable elements, layouts, logs, images, and a
terminal widget. The README names `slap` and `blessed-contrib` among users or
related applications. The release and repository baselines are old: npm's
latest release remains 0.1.81, while the inspected repository head is from
2016.

## Core Proposition

The proposition is direct construction of a terminal application from a
retained widget tree. A `Node` owns a parent, children, screen association, and
event emitter behavior. `Element` adds a rectangular position, content, style,
border, clipping, and input registration. A developer can therefore express a
form or dashboard as ordinary JavaScript objects, attach listeners, and use
the same tree for painting, focus, hit testing, and event bubbling.

Unlike a formatting library, `blessed` owns terminal modes and emits terminal
control sequences. Unlike a framework with a prescribed reducer or async
runtime, it leaves state transitions and redraw policy to the application. The
combination is productive for small and medium tools: the application can use
high-level widgets for common interactions and drop to `Program` for raw
cursor, mouse, terminfo, child-process, or transport behavior. The cost is
that application policy is distributed across callbacks and explicit render
calls rather than represented by one testable runtime contract.

## Architecture

### Retained Tree And Layout

`Node` builds a mutable parent-child tree and assigns a screen to each node.
`Element` stores position expressions such as absolute values, percentages,
`center`, and `shrink`; `test/widget-pos.js` asserts the resulting coordinates.
Elements paint in child order, so later children act as overlays. Focus is
stored on a screen history stack, while events emitted by an element can bubble
as `element <event>` through its ancestors. Identity, focus scope, and lifecycle
are mutable object state rather than serialized application state.

### Rendering And Text

`Screen.alloc()` creates a pending cell matrix and an output comparison matrix.
`Screen.render()` traverses the children, lets each element paint into the
pending matrix, then calls `draw()` for every row. `draw()` skips clean rows,
compares cells with the previous output matrix, and emits cursor movement,
attributes, erase operations, and terminal scrolling where useful. `smartCSR`,
`fastCSR`, BCE, painter ordering, and terminfo capabilities reduce output for
the alternate-screen case. The design is a practical cell diff, not retained
dirty-subtree rendering: a render still walks the visible tree.

`fullUnicode` enables East Asian widths, UTF-16 surrogate pairs, and combining
characters. `unicode.charWidth()` has a width table and optional
`NCURSES_CJK_WIDTH` policy. This is better than byte-oriented text, but it is
option-gated and does not establish a modern grapheme-cluster contract.
Terminal-specific behavior, including the README's iTerm2 caveat, remains
visible to applications.

### Events, Scheduling, And Lifecycle

`Program` parses keypresses and mouse sequences from a Node input stream,
broadcasts them to programs sharing that stream, and emits output resize events.
Listeners turn on raw input or mouse handling. There is no scheduler, effect
queue, cancellation model, backpressure policy, or settled-render concept;
examples and handlers call `screen.render()` themselves.

`Screen.enter()` selects the alternate buffer, hides the cursor, sets the scroll
region, and allocates buffers. `leave()` restores the normal buffer, cursor, and
mouse state. `pause()` and `resume()` hand the terminal back temporarily, while
`spawn()` and `sigtstp()` support child-process and suspend workflows. Resize
handling reallocates and immediately renders the tree. These facilities are
useful, but the lifecycle is imperative and output recovery is not a separate
contract.

## Core Strengths

### Terminal-Aware Rendering

`blessed` integrates terminfo/termcap parsing, alternate-screen lifecycle,
cursor control, mouse protocols, resize handling, CSR, BCE, and a two-buffer
cell renderer. Applications can optimize output without importing a separate
terminal substrate, while `Program` remains available when a widget is the
wrong abstraction.

### Broad Composable Widget Surface

The retained tree makes forms, overlays, focus traversal, scrolling, and mouse
hit testing accessible through ordinary objects. `Form` collects focusable
descendants and implements tab, arrow, and optional vi traversal; `ScrollableBox`
supports keyboard and mouse scrolling, scrollbar interaction, and automatic
scrolling to focused descendants. This is a significant advantage over a
renderer that only paints rectangles.

### Practical Escape Hatches

Applications can supply streams, including a telnet-style socket, change the
terminal definition, use `Program` directly, spawn a foreground process, dump
I/O, or inspect an SGR screenshot. Custom widgets can subclass existing
prototypes. `slap`, `vtop`, and `blessed-contrib` show that this boundary was
useful for substantial tools; adoption size was not measured here.

## Limitations And Frustrations

### 1. Stale Release And Governance Boundary

**Classification:** Governance problem and maturity problem.

**Requirement:** A long-running application needs a maintained runtime baseline
and an authoritative place to resolve terminal regressions.

**Library assumption:** The high-level API and old Node compatibility floor are
adequate for downstream applications.

**Observable failure or friction:** The latest npm release is 0.1.81 from 2015
and the inspected repository head is from 2016. The open
[archive-project issue](https://github.com/chjj/blessed/issues/418) records the
maintenance concern. `blessed-contrib` is an add-on; `neo-blessed`, `reblessed`,
`blessed-ng`, and `unblessed` are separate continuation or modernization paths
rather than one compatible authority.

**Root architectural cause:** Release ownership and project governance, not the
retained widget model.

**Available workaround:** Pin the old package, maintain a private patch, adopt
a fork, or build on an add-on such as `blessed-contrib`.

**Cost of workaround:** Each path creates compatibility ownership; forks can
diverge in terminal behavior and API details.

**Upstream response:** No later blessed release was found in the recorded npm
baseline; the issue remains historical evidence of the governance boundary.

**Current status and version:** Verified for npm 0.1.81 and `eab243f`; confidence
high for release age and medium for future maintenance.

**Evidence:** [package manifest](https://github.com/chjj/blessed/blob/eab243fc7ad27f1d2932db6134f7382825ee3488/package.json),
[npm release](https://www.npmjs.com/package/blessed/v/0.1.81), and the linked
issue.

### 2. Application Scheduling And Large Collections Stay Outside The Contract

**Classification:** Tradeoff; extension failure for applications needing a
framework-owned effect or virtualization contract.

**Requirement:** A stateful application needs serialized events, settling,
cancellation, and predictable large-collection cost.

**Library assumption:** The application can own callbacks, timers, redraws, and
collection policy.

**Observable failure or friction:** Examples explicitly call `screen.render()`.
`Screen.render()` traverses children on each pass, and `ScrollableBox` computes
scroll extent from child geometry unless a list-specific optimization applies.
There is no general virtual list, stable item-key contract, effect queue, or
redraw backpressure API.

**Root architectural cause:** `blessed` is a mutable retained widget library,
not an application runtime.

**Available workaround:** Manually batch updates and renders, paginate or reuse
elements, write a custom widget, or bypass the tree with `Program`.

**Cost of workaround:** Scheduling, focus, cancellation, and model-to-widget
identity become application conventions; bypassing the tree loses composition
and hit testing.

**Upstream response:** The public API and examples consistently expose explicit
render calls rather than a replacement scheduler.

**Current status and version:** Verified or inferred from `eab243f`; confidence
high for the API boundary and medium for workload-specific performance.

**Evidence:** [`Screen.render()`](https://github.com/chjj/blessed/blob/eab243fc7ad27f1d2932db6134f7382825ee3488/lib/widgets/screen.js#L718-L758),
[`ScrollableBox`](https://github.com/chjj/blessed/blob/eab243fc7ad27f1d2932db6134f7382825ee3488/lib/widgets/scrollablebox.js#L181-L215),
and the [form example](https://github.com/chjj/blessed/blob/eab243fc7ad27f1d2932db6134f7382825ee3488/example/simple-form.js#L65-L87).

### 3. Incomplete Headless And Output-Recovery Contract

**Classification:** Maturity problem and limitation.

**Requirement:** Tests should drive a complete application without a physical
terminal and simulate partial, deferred, or failed output.

**Library assumption:** Interactive display tests and a logical screenshot are
sufficient for terminal correctness.

**Observable failure or friction:** The README says most tests are interactive
and asks the programmer to judge the display. `Program._buffer()` schedules a
flush on the next tick; `flush()` writes and clears it without a completion or
backpressure contract. `Screen.draw()` updates its comparison matrix while
constructing output. No prepared frame waits for backend acceptance.

**Root architectural cause:** ANSI generation and stream writes are integrated
with the mutable screen comparison state, while testing is fixture-oriented.

**Available workaround:** Capture a supplied output stream, use `screenshot()`
or dump logs, force a full reallocation after uncertainty, and add an external
PTY or terminal emulator harness.

**Cost of workaround:** A capture proves logical output but not physical screen
state; each application must rebuild recovery rules and protocol coverage.

**Upstream response:** The README mentions a possible future vttest-like
approach; no complete application harness or failure-injection facility was
found at `eab243f`.

**Current status and version:** Test documentation and source are verified; the
partial-write consequence is an architectural inference. Confidence is high for
the testing gap and medium for untested stream failure paths.

**Evidence:** [README testing section](https://github.com/chjj/blessed/blob/eab243fc7ad27f1d2932db6134f7382825ee3488/README.md#L2314-L2318),
[`Program._buffer()` and `flush()`](https://github.com/chjj/blessed/blob/eab243fc7ad27f1d2932db6134f7382825ee3488/lib/program.js#L1632-L1669),
and [`Screen.draw()`](https://github.com/chjj/blessed/blob/eab243fc7ad27f1d2932db6134f7382825ee3488/lib/widgets/screen.js#L1053-L1214).

## Scenario Evaluation

| Scenario | Status | Finding |
| --- | --- | --- |
| Form, validation, and modal | Supported, validation application-owned | `Form`, focus traversal, buttons, bubbling, and child ordering are built in. |
| Large scrollable collection | Partial | Scrolling exists; general virtualization and stable item identity do not. |
| Streaming external output | Partial | Node events and manual renders work; scheduling and backpressure are external. |
| Unicode-heavy input and editing | Partial | `fullUnicode` handles important widths and surrogates, with option and terminal caveats. |
| Overlay, clipping, and mouse | Supported | Retained children, painter order, focus, and mouse hit testing cover the case. |
| Resize during updates | Supported, imperative | Resize reallocates and renders, but no settling or transactional recovery is promised. |
| Deferred or failed writes | Unsupported contract | Output buffering exists; physical-state invalidation is not defined. |
| Suspend or child process | Supported, imperative | `pause`, `resume`, `spawn`, and `sigtstp` restore terminal modes. |
| Long idle period | Partial | An application can stop rendering, but there is no idle scheduler or policy. |
| Native terminal scrollback conversation | Unsupported by `Screen` | The normal screen owns the alternate buffer; inline behavior requires lower-level use. |

## Testing Strategy

Testing at the recorded revision mixes assertions, interactive fixtures, output
logs, and visual inspection. `test/widget-pos.js` sets an explicit terminal size
and asserts percentage, center, shrink, absolute, and relative coordinates.
`test/widget-unicode.js` exercises CJK, surrogate, combining, and emoji-like
code points, mainly for visual inspection. `test/program-mouse.js` exercises
mouse, focus, keypress, and resize paths in a live terminal. `test/widget-record.js`
collects repeated `screen.screenshot()` values into JSON frames.

Tests use `Screen`, `Program`, and real widget rendering rather than a parallel
mock renderer. `dump` records input/output, and `screenshot()` exposes logical
SGR output. However, the README says most tests are interactive, and no test
script or complete application driver was found at this revision. There is no
standard injection API for input, resize, external events, or clocks; no settled
async model; no partial-write failure injection; and no evidence of a virtual
terminal or PTY lifecycle suite.

The main blind spot is the difference between a correct logical cell matrix and
an accepted physical patch. A screenshot validates text, styles, clipping, and
some Unicode behavior, but not whether a stream accepted all bytes or whether
the next diff starts from trustworthy physical state. The collection is useful
for widget development and manual compatibility work, but not a deterministic
full-application contract.

## Lessons For ArborUI

ArborUI should adopt the practical parts: a small terminal capability boundary,
explicit output modes, a cell renderer, Unicode width tests, composable widgets,
overlays, and a low-level escape hatch. `blessed` also shows the value of easy
screenshots and input/output capture, even when they are not the whole test
solution.

ArborUI should avoid treating a mutable rendered buffer as committed physical
state. A prepared frame should be committed only after the backend accepts the
complete patch; an uncertain write should invalidate physical state and force a
full repaint. It should also avoid making every application invent scheduling,
focus routing, async settlement, and test-driving conventions. The comparison
must be fair: `blessed` is deliberately flexible, and that flexibility is a
benefit for applications that already have an event loop.

ArborUI already approaches the most consequential boundary differently through
its runtime, public facade, headless test harness, and explicit render
transaction rules. It has not yet proven that those guarantees will be cheaper
than `blessed`'s direct JavaScript object model, that large retained trees will
meet practical latency targets, or that its terminal modes will cover native
scrollback and child-process handoff without ambiguity.

Follow-up work should benchmark a dashboard and large collection at fixed
sizes, inject fragmented and rejected writes, run PTY tests for resize and
suspend/resume, and compare the code required for a validated modal form.
Unicode tests should include width policy, combining sequences, final-column
behavior, and semantic assertions in addition to character snapshots.

## Evidence Appendix

### Primary Sources

- [Pinned README](https://github.com/chjj/blessed/blob/eab243fc7ad27f1d2932db6134f7382825ee3488/README.md), accessed 2026-07-16.
- [`Node`](https://github.com/chjj/blessed/blob/eab243fc7ad27f1d2932db6134f7382825ee3488/lib/widgets/node.js), [`Element`](https://github.com/chjj/blessed/blob/eab243fc7ad27f1d2932db6134f7382825ee3488/lib/widgets/element.js)
- [`events.js`](https://github.com/chjj/blessed/blob/eab243fc7ad27f1d2932db6134f7382825ee3488/lib/events.js), accessed 2026-07-16.
- [`Screen`](https://github.com/chjj/blessed/blob/eab243fc7ad27f1d2932db6134f7382825ee3488/lib/widgets/screen.js), [`Program`](https://github.com/chjj/blessed/blob/eab243fc7ad27f1d2932db6134f7382825ee3488/lib/program.js)
- [`unicode.js`](https://github.com/chjj/blessed/blob/eab243fc7ad27f1d2932db6134f7382825ee3488/lib/unicode.js)
- [`Form`](https://github.com/chjj/blessed/blob/eab243fc7ad27f1d2932db6134f7382825ee3488/lib/widgets/form.js), accessed 2026-07-16.
- [`test/widget-pos.js`](https://github.com/chjj/blessed/blob/eab243fc7ad27f1d2932db6134f7382825ee3488/test/widget-pos.js), [`test/widget-unicode.js`](https://github.com/chjj/blessed/blob/eab243fc7ad27f1d2932db6134f7382825ee3488/test/widget-unicode.js)
- [`test/widget-record.js`](https://github.com/chjj/blessed/blob/eab243fc7ad27f1d2932db6134f7382825ee3488/test/widget-record.js)
- [`test/program-mouse.js`](https://github.com/chjj/blessed/blob/eab243fc7ad27f1d2932db6134f7382825ee3488/test/program-mouse.js), accessed 2026-07-16.

### Releases And Ecosystem

- [npm `blessed@0.1.81`](https://www.npmjs.com/package/blessed/v/0.1.81), latest release baseline examined.
- [npm `blessed-contrib`](https://www.npmjs.com/package/blessed-contrib), latest reported version 4.11.0, published 2022-06-13; widget add-on rather than the blessed core.
- [npm `neo-blessed`](https://www.npmjs.com/package/neo-blessed), latest reported version 0.2.0, published 2018-06-13; fork ecosystem evidence.
- [npm `reblessed`](https://www.npmjs.com/package/reblessed), latest reported version 0.2.1, published 2023-02-12; fork ecosystem evidence.
- [blessed-ng](https://github.com/blessed-ng/blessed) is a separate fork, while [unblessed](https://github.com/vdeantoni/unblessed) is a newer TypeScript modernization with a compatibility package.
- [slap](https://github.com/slap-editor/slap), [vtop](https://github.com/MrRio/vtop), and [blessed-contrib](https://github.com/yaronn/blessed-contrib) were inspected as ecosystem evidence.
- Application and ecosystem pages accessed 2026-07-16.

### Qualifications

The source and test conclusions are verified by inspection at the pinned
revision. No physical terminal, PTY, terminal emulator, benchmark, or injected
stream failure was run. Claims about partial-write recovery and large-collection
cost are therefore architectural inferences, not reproduced failures. Issue
and fork activity establishes ecosystem and governance context, not a universal
quality ranking.
