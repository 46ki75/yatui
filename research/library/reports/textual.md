# Textual Research Report

## Evidence Header

```text
Research date: 2026-07-16
Project version: Textual 8.2.8
Project revision: 1d99508b928a771b51e1a527319c6b87dcff9e05
Repository: https://github.com/Textualize/textual
Documentation version: Release source documentation at the recorded commit; live documentation accessed 2026-07-16 with deployment revision unavailable
Primary platform examined: Source and test inspection on Linux; no physical terminal reproduction
Report depth: Deep dive
```

All current implementation conclusions in this report refer to Textual 8.2.8 at
the recorded revision unless another version is named. The selected commit is the
`v8.2.8` tag and was created on 2026-06-30. The release notes list fixes for
Kitty extended-key parsing and clicks in Screen padding, plus changed
`super+backspace` and `alt+backspace` behavior in `Input` and `TextArea`
([release source](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/CHANGELOG.md#L8-L18)).

## Executive Assessment

Textual is a Python application framework for terminal user interfaces. Its
central abstraction is a retained tree of `App`, `Screen`, and `Widget` objects
styled and laid out through a CSS-like language. A message-pump runtime built on
`asyncio` owns event delivery, timers, reactive updates, workers, screen stacks,
and repaint scheduling. Rich supplies much of the text, style, segment, and
terminal-cell representation. Textual can also run headlessly for tests, inline
under a shell prompt, or through a web driver.

Textual is strongest when an application author wants a complete, productive
framework rather than a rendering primitive. Forms, dashboards, inspectors,
file browsers, log viewers, Markdown viewers, modal screens, focus traversal,
mouse interaction, CSS theming, and asynchronous work are all represented in one
coherent API. The official examples are not only static widgets: `code_browser.py`
combines a directory tree, a scrollable code view, reactive state, and syntax
highlighting; `mother.py` mounts a chat transcript and streams worker-thread
results into it ([code browser](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/examples/code_browser.py#L23-L88), [streaming example](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/examples/mother.py#L43-L107)).

The same completeness is the principal tradeoff. Textual chooses a runtime,
widget-tree ownership model, CSS cascade, and Python object identity policy. It
does not merely render an application-owned model. That makes ordinary
interaction much easier than assembling a framework above a cell renderer, but
it also means that large retained trees, CSS invalidation, terminal output
recovery, and application lifecycle edges are framework concerns. The current
implementation has useful compositor dirty-region and visible-widget paths, but
it does not provide a general application-level virtualization contract for
unbounded collections. Its physical output path has no accepted/deferred/unknown
write result, and its standard headless test path cannot represent PTY, terminal
emulator, or partial-write behavior.

For ArborUI, Textual is evidence that a high-level TUI framework can make focus,
messages, workers, screens, CSS composition, and deterministic interaction tests
valuable as one product. It is not evidence that every retained-mode feature
should be copied. ArborUI should preserve its explicit separation between model,
ephemeral view, retained identity, renderer, runtime, and terminal session; keep
the prepared-frame commit contract; and prove the performance cost of its
invalidation model. Textual also shows that widget breadth, examples, and
debugging tools can outweigh architectural elegance in adoption decisions.

## Project Snapshot

Textual is an MIT-licensed Python package presented as a "Modern Text User
Interface framework." It is an application framework, not just a widget or
terminal backend library. The package declares Python `^3.9`, is typed, and the
8.2.8 metadata advertises Python 3.9 through 3.14 on Linux, macOS, and Windows.
Its core runtime depends on Rich, `markdown-it-py`, `mdit-py-plugins`,
`typing-extensions`, and `platformdirs`; syntax highlighting is an optional
Tree-sitter-based extra ([pinned package metadata](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/pyproject.toml#L1-L27), [dependencies](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/pyproject.toml#L47-L76)).

The project labels the release production/stable and ships a substantial widget
catalog, documentation, examples, developer tools, and an official snapshot
plugin. Its CI matrix covers Ubuntu, Windows, and macOS for Python 3.9 through
3.14 with coverage and snapshot artifacts ([release CI](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/.github/workflows/pythonpackage.yml#L17-L56)).
Full-screen terminal apps are the primary mode; inline mode, headless tests, and
the web driver are explicit additional drivers. The direct comparison category
is application framework, not a lower-level Rich presentation library.

## Core Proposition

Textual makes a terminal application feel like a small web application without
requiring the author to implement the browser-like machinery. The author
subclasses `App` or `Widget`, yields children from `compose()`, writes CSS, and
handles messages, events, reactive state, and actions. The application can add or
remove widgets at runtime through `mount`, `remove`, and screen operations.

The framework's value is not merely less code for drawing cells. It supplies a
retained DOM, selector queries, CSS inheritance and pseudo-classes, focus and
mouse targeting, bubbling messages, key bindings, actions, screen navigation,
timers, workers, lifecycle hooks, headless interaction, and visual snapshots.
The [official events guide](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/docs/guide/events.md#L5-L18)
describes the intended model directly: every `App` and `Widget` has a message
queue processed by an `asyncio` task.

This differs materially from Ratatui's application-owned immediate-mode boundary:
Textual optimizes for rapid construction of stateful applications and accepts
more framework policy in return.

## Architecture

### Application, DOM, And State

`App` is the root message pump and owns one or more screen stacks. A `Screen`
occupies the terminal-sized view, and widgets form the child hierarchy. The
`compose` helper consumes a generator, validates that it yields widget objects,
and builds the nested result before mounting it
([composition implementation](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/compose.py#L12-L99)).
The initial composition and later `mount` operations are therefore imperative
mutations of a retained tree. Reactive attributes can request a repaint, layout,
binding refresh, or full recomposition. Recomposition explicitly removes child
widgets and calls `compose()` again; it is not a general keyed reconciliation
algorithm ([reactive flags](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/reactive.py#L124-L163), [recomposition documentation](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/docs/guide/reactivity.md#L229-L259)).

`NodeList` stores actual widget objects in a list and set, tracks updates for
caching, and maintains an ID-to-widget map for fast queries. A widget remains
the same Python object until it is removed, so focus, scroll position, reactive
attributes, worker ownership, and other object-local state naturally survive
ordinary updates. Duplicate IDs are rejected at one level of the tree
([NodeList](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/_node_list.py#L29-L58), [ID insertion](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/_node_list.py#L118-L161)).

The application model is not separated from widget state by the framework. A
small app may keep state in reactive widgets; a larger app may put it in `App`
and update children from handlers. This is ergonomic, but unlike ArborUI it does
not enforce one model owner or prohibit retained references to application data.
CSS classes, pseudo-classes, variables, transitions, and layout properties are
also part of retained-tree state. Reactive assignments schedule refreshes, while
`recompose=True` deliberately recreates child objects.

### Layout, Composition, And Rendering

Textual's layout engine arranges child widgets into placements with regions,
margins, z-order, fixed and overlay flags, and a spatial map. A layout result
can query only placements visible in a region, while fixed items remain visible
regardless of scrolling ([layout result](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/layout.py#L20-L81)).
The compositor retains a full widget-to-geometry map, visible map, layers,
visible widget regions, and dirty regions. It has a `reflow_visible` fast path
for scrolling and a full reflow path for ordinary layout changes
([compositor state and reflow](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/_compositor.py#L280-L477)).

Rendering is retained at the geometry and damage level, but not every widget is
a retained cell surface. A widget can return a Rich renderable, Textual
`Content`, or another visual protocol object. The compositor asks visible
widgets for strips and crops them to clipped regions. A full update renders all
screen regions; a partial update turns dirty rectangles into line spans and
renders only the affected portions ([render selection](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/_compositor.py#L1031-L1119), [partial updates](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/_compositor.py#L1140-L1183)).

Individual widgets add caches. `Widget.BLANK` lets a large scrolling container
avoid painting its own content; `DataTable` caches rows, cells, renderables, and
lines and has stable `RowKey` and `ColumnKey` values. These optimizations do not
turn arbitrary children or external data into a bounded virtual list.

Rich `Segment` values and cell-width functions form the logical text boundary.
`_cells.cell_len` delegates to Rich, while immutable-like `Content` caches cell
lengths and layout calculations ([cell functions](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/_cells.py#L1-L44), [Content model](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/content.py#L1-L8)).
This is stronger than treating Python characters as cells, but physical cursor
and wrapping behavior still belongs to the terminal.

### Events, Focus, And Actions

Terminal drivers normalize key, mouse, paste, resize, focus, and protocol events.
The App receives input first for global handling. Key bindings are checked along
the focused widget's ancestor chain, and an unhandled key is forwarded to the
focused widget or screen. Mouse events use compositor geometry and style metadata
to select the topmost widget; mouse capture overrides ordinary hit testing for a
drag or release. `Screen._forward_event` then forwards the event to the target's
message pump ([mouse targeting and capture](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/screen.py#L1820-L1951), [binding chain](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/app.py#L3934-L3988)).

Textual's propagation model is target-first bubbling rather than ArborUI's
explicit capture-target-bubble phases. A message handler runs through the
class handler chain, can prevent default behavior, and can stop message
bubbling. If `bubble` is enabled, the message pump forwards the same message to
the parent after local handlers run. The official documentation calls this out
as the mechanism by which a child control gives a container or App a chance to
respond ([event propagation guide](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/docs/guide/events.md#L42-L82), [message dispatch](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/message_pump.py#L707-L840)).

Actions are allow-listed methods named `action_*`, optionally addressed through
`app`, `screen`, or `focused` namespaces. Bindings can be dynamically enabled,
disabled, hidden, or shown through `check_action`. This gives a reusable command
surface without exposing arbitrary method execution from action strings
([actions guide](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/docs/guide/actions.md#L1-L31), [action dispatch](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/app.py#L4167-L4284)).

Focus is a reactive `Screen` property. Its chain is derived from the retained
tree; changes update pseudo-class styling, queue focus/blur events, scroll the
target into view, and refresh bindings. Screen stacks provide modal navigation.

### Scheduling, Messages, And Workers

Every App and widget has a message queue and an asyncio task. The pump processes
one message at a time, can coalesce messages that declare themselves replaceable,
dispatches handlers, publishes a message signal, and inserts idle callbacks
when the queue is empty. Timers, `call_next`, `call_later`, and
`call_after_refresh` are all integrated with this queue
([message loop](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/message_pump.py#L634-L705)).
`call_after_refresh` is especially useful for updates whose geometry must settle
before a callback runs. Startup initializes the asyncio primitives explicitly for
older Python compatibility ([queue initialization](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/message_pump.py#L155-L180)).

The runtime is intentionally serialized per message pump, not per entire
application object graph. A slow event handler blocks that pump, and the
documentation directs authors to move network, subprocess, or CPU work to a
worker. `@work` and `run_worker` manage asyncio tasks or thread workers, expose
`PENDING`, `RUNNING`, `CANCELLED`, `ERROR`, and `SUCCESS` states, provide
exclusive groups, and tie worker cleanup to the owning DOM node
([worker state](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/worker.py#L118-L183), [worker guidance](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/docs/guide/workers.md#L47-L109)).

Thread workers do not make widget mutation generally thread-safe. The documented
boundary is `App.call_from_thread` for UI calls and `post_message` for sending a
message. Thread cancellation is cooperative: a running thread may continue
until it checks its cancellation state. This is a practical integration design,
but it puts the burden of batching, ordering, and backpressure for high-rate
external streams on the application.

### Terminal Ownership And Lifecycle

The `Driver` abstraction owns terminal mode setup, input disabling, output writes,
start and stop operations, and optional suspend/resume. Drivers exist for
full-screen Unix, inline Unix, Windows, headless, and web/remote operation. The
base contract is intentionally small: `write(str)` and `flush()` do not return a
write outcome, and the lifecycle methods do not expose a terminal-state snapshot
([driver contract](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/driver.py#L17-L173)).

The Linux full-screen driver enters the alternate screen, enables mouse and
focus reporting, configures raw input, optionally enables Kitty keyboard
protocols, starts an input thread, and queues output through `WriterThread`.
Resize signals become messages. Inline mode stays on the main screen, writes
padding and redraws beneath the shell prompt, and queries cursor position with
the terminal response sequence `ESC [ 6 n`. The compositor has separate inline
rendering code and tracks the prior inline height. This is a real second mode,
not just a fullscreen viewport with a different height. The relevant source is
the [fullscreen startup](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/drivers/linux_driver.py#L196-L324),
[inline startup and input](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/drivers/linux_inline_driver.py#L121-L245),
and [inline compositor](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/_compositor.py#L120-L162).

`App.suspend()` stops application mode, restores terminal access, redirects
stdout and stderr for the child operation, starts application mode again, emits
resume signals, and requests a layout refresh. Unix process suspension is a
separate `SIGTSTP`/`SIGCONT` path. Documentation supports external editors and
other terminal programs, but Textual Web does not support suspension
([suspend implementation](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/app.py#L4717-L4788), [suspension example](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/docs/examples/app/suspend.py#L8-L20)).

### Extension Surface

Application authors extend `App`, `Screen`, and `Widget`, provide CSS, handlers,
actions, reactive attributes, visual objects, and custom drivers. The Rich visual
protocol is a useful escape hatch for plots, syntax, Markdown, and formatting.

The central message pump, CSS engine, compositor, focus model, and asyncio policy
are internal machinery. A custom driver can change I/O and lifecycle, but must
fit the `write`/`flush` contract; a custom widget cannot replace reconciliation
without taking ownership of more framework internals.

## Core Strengths

### A Complete Interaction Model

Textual supplies the parts that a cell-rendering library leaves to the
application: focus, keyboard binding resolution, mouse targeting, capture,
event bubbling, modal screen stacks, actions, notifications, cursor placement,
and widget lifecycle. This makes a form with validation and an overlay a normal
framework task rather than a collection of application conventions. The `Input`,
`TextArea`, `DataTable`, `ListView`, `Tree`, `Select`, and screen APIs are
designed to participate in the same routing and styling model. `compose()` and
context-manager containers make nested structure compact, while `mount()` and
`remove()` support dynamic content without application coordinate calculations.

IDs and CSS selectors make the retained tree inspectable, while the compositor
exposes geometry and hit-testing information. ArborUI should preserve this shared
identity principle without copying CSS wholesale.

### Compositor, Clipping, And Hit Testing Agree

Textual does not independently implement visual layering and mouse targeting.
The compositor retains widget regions, clips visible content, builds layer order,
and answers widget/style queries from the same arrangement. Dirty regions and
partial rendering reduce output for localized updates. The spatial map provides
window queries rather than scanning every placement for every pointer event
([spatial map](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/_spatial_map.py#L15-L103)).

An overlay, scrollbar, or captured drag is less likely to disagree with what the
user sees because composition and hit testing use one geometry source. This is
more advanced than a simple full-frame widget painter.

### Workers Make External Effects Usable

The worker API addresses a common TUI failure mode: a network request or blocking
file read inside a message handler freezes input and painting. Async workers,
thread workers, cancellation, exclusive groups, lifecycle cleanup, state events,
and `call_from_thread` give application authors a documented path for effects.
The `mother.py` example is especially relevant: it mounts a prompt and response,
then feeds chunks from a blocking model iterator back to the UI through a thread
worker. This is a concrete pattern for external streaming rather than a
theoretical scheduler feature.

Workers are tied to their originating DOM node, so removing a screen or widget
cancels its work. ArborUI's command scheduler should retain this ownership and
cancellation story while keeping commands runtime independent.

### Strong Headless And Visual Test Ergonomics

`App.run_test()` runs the real application in a `HeadlessDriver` and returns a
`Pilot`. Pilot can press keys, click, move the mouse, resize the terminal, wait
for message pumps, and wait for the process to become idle. The official plugin
adds SVG visual snapshots, and the repository has a large snapshot suite covering
widgets, layouts, focus, scrolling, tables, overlays, and many Unicode-adjacent
rendering cases.

This boundary uses the same App, Screen, Widget, compositor, queues, and workers
rather than a model-only fake. It still has physical-terminal gaps, but makes
full-application interaction testing a first-class feature.

## Limitations And Frustrations

Textual's framework runtime and state ownership are an intentional tradeoff, not
a defect for its target applications. Its App, Screen, and Widget queues, CSS
invalidation, lifecycle, and workers belong to the framework-owned object graph;
external work is adapted through workers, `call_from_thread`, `post_message`, or a
custom driver, while application-owned state remains a convention. This is why a
dashboard or form is quick to assemble, but it costs integration boundaries for a
host with another event loop, strict reducer ownership, or a requirement that
retained UI state never hold application references. The behavior is current in
8.2.8 and is supported by the message-pump, Worker, and Driver source discussed
above. ArborUI should prove that its extra separation reduces integration cost.

### Large Collections Are Retained, Not Generally Virtualized

```text
Classification: Limitation and extension boundary
Requirement: Display an unbounded or externally paged collection with stable item identity and bounded memory/work
Library assumption: A scrolling UI is represented by mounted child widgets or by a DataTable containing all rows and cells
Observable failure or friction: Large widget lists retain every child; DataTable retains all rows, cell data, row heights, and a y-offset entry per rendered line
Root architectural cause: Textual's retained DOM and widget-local collection implementations do not expose a generic visible-range/data-provider contract
Available workaround: Use DataTable's caches, Widget.BLANK, pagination, manual chunking, or a specialized custom widget
Cost of workaround: Application-specific data windowing and selection/focus mapping; external data and stable identity must be coordinated manually
Upstream response: Large-list performance fixes have shipped, but the million-cell lazy-view request was closed as not planned
Current status and version: Current source behavior in 8.2.8; no generic virtualization API found at the recorded revision
Evidence: DataTable source plus issues #1892 and #5163
Confidence: High for source boundary; medium for end-user performance cost; Evidence status: Verified and Reported
```

Textual has useful partial answers: `DataTable` assigns stable keys, caches cell
and row renderables, and renders requested lines rather than a widget per cell;
the compositor culls non-visible placements. These features outperform a naive
list of deeply nested child widgets, but do not make the data source lazy.

The pinned `DataTable` constructor stores `_data`, `rows`, and location maps, and
`_y_offsets` is built with one entry for each line in all ordered rows
([DataTable storage](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/widgets/_data_table.py#L733-L869)).
The `ListView` API mounts `ListItem` objects and navigates its `_nodes` list
([ListView](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/widgets/_list_view.py#L19-L29), [mounting](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/widgets/_list_view.py#L236-L259)).
Issue #1892 requested a lazy, chunked view for about a million cells and is
currently marked "Closed as not planned" ([issue #1892](https://github.com/Textualize/textual/issues/1892)).
Issue #5163 documents a 500-1000-widget selectable list becoming unresponsive;
it closed with a performance pull request, evidence of optimization but not a
general virtual-list contract ([issue #5163](https://github.com/Textualize/textual/issues/5163), [linked PR #5164](https://github.com/Textualize/textual/pull/5164)).

For ArborUI, visible-range construction and stable keyed identity should be
explicit APIs. Measure before adding a general virtualization layer; Textual
shows both its value and the cost of retrofitting it onto retained children.

### CSS Invalidation Can Traverse Large Subtrees

```text
Classification: Maturity problem and performance tradeoff
Requirement: Focus, hover, class, and disabled-state changes should remain responsive in large retained trees
Library assumption: Style changes may need to be applied to a node and its descendants because selectors and pseudo-classes can affect them
Observable failure or friction: update_styles walks every descendant and applies stylesheet updates; a local interaction can trigger broad style/layout work
Root architectural cause: CSS cascade and pseudo-class propagation are integrated with retained DOM invalidation rather than a narrowly scoped typed invalidation graph
Available workaround: Reduce descendant counts, use custom widgets, batch updates, cache content, or apply an application-level workaround around hot paths
Cost of workaround: CSS expressiveness, component granularity, or framework internals become part of performance tuning
Upstream response: Issue #6524 reported measured overhead and was closed without a linked fix visible in the fetched issue metadata; the pinned source still walks descendants
Current status and version: Source path remains present in 8.2.8
Evidence: Pinned App.update_styles implementation and issue #6524
Confidence: High for traversal; medium for workload cost; Evidence status: Verified and Reported
```

The source is unambiguous: `App.update_styles` calls `node.walk_children` with
`with_self=True` before applying the stylesheet
([implementation](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/app.py#L2504-L2523)).
Some traversal is necessary for CSS semantics, and caches/coalescing mean this
is not proof that every focus operation is slow.

Issue #6524, opened in May 2026, reports a 2,290-call session and roughly 962 ms
of Python overhead in a complex app, claiming the behavior affects Textual 7.3.0
and 8.2.5 ([issue #6524](https://github.com/Textualize/textual/issues/6524)). The
issue is a user measurement, not a reproduction in this research, and its
current page does not link a fix. The current source confirms the relevant
subtree walk at 8.2.8, so the structural concern is supported while the exact
workload cost remains reported.

This is the clearest risk of combining a CSS cascade with a retained tree: local
state can have global invalidation consequences. ArborUI's typed invalidation is
narrower in intent, but still requires realistic benchmarks.

### Output Is Not A Recoverable Frame Transaction

```text
Classification: Limitation relative to ArborUI's terminal-recovery requirement; extension failure for in-session recovery
Requirement: Commit logical visual state only after the complete patch is accepted, and force a full repaint after an uncertain write
Library assumption: A driver write/flush is sufficient and output failure is normally an application-fatal terminal error
Observable failure or friction: Driver.write returns no outcome; Unix output is queued asynchronously; dirty regions are cleared while constructing the update; a writer-thread failure has no application acknowledgement path
Root architectural cause: The Driver contract has write(str) and flush() rather than a staged patch/outcome protocol
Available workaround: Exit and restore the terminal, or implement a custom driver with synchronous/stateful output and its own recovery policy
Cost of workaround: In-session recovery is not supplied by the normal compositor/runtime path; custom drivers must recreate physical-state tracking
Upstream response: No accepted/deferred/unknown output contract was found at the recorded release
Current status and version: Current in 8.2.8
Evidence: Driver, App display, CompositorUpdate, Screen dirty-region clearing, and WriterThread source
Confidence: High; Evidence status: Verified for the contract, Inferred for a particular physical failure sequence
```

Textual's normal output path is optimized for ordinary terminal success. The
compositor decides between full and partial updates; rendering the update clears
its dirty regions before returning the `CompositorUpdate`. The screen then clears
dirty-widget state after `_display`, and `_display` calls `driver.write` followed
by `driver.flush`. A failed or partial write is not represented as `Applied`,
`Deferred`, or `StateUnknown` ([screen refresh](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/screen.py#L1218-L1233), [compositor update](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/_compositor.py#L1096-L1183)).

On Linux, `WriterThread` has a bounded queue of 30 strings. `write` blocks when
the queue is full, but returns after enqueueing, not after the terminal accepts
the bytes. The background thread calls the underlying file's `write` and
`flush`; there is no result channel, exception propagation, or physical-screen
shadow state ([writer implementation](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/drivers/_writer_thread.py#L12-L68)).
Most terminal applications terminate on a broken output stream, and the driver
lifecycle attempts cleanup. A host requiring continued operation after a slow,
closed, or partial write cannot obtain that guarantee from a widget or handler.

ArborUI's prepared-frame transaction is therefore a semantic difference, not
just a more complicated diff API. Textual optimizes for the successful path;
ArborUI accepts staging and invalidation for recoverable sessions.

### Inline And Process Suspension Have Terminal-Specific Edge Cases

```text
Classification: Reported bug and lifecycle tradeoff
Requirement: Leave the terminal usable for a child process or shell, then resume with a correct complete application view across supported platforms and modes
Library assumption: Driver-specific stop/start and cursor tracking can restore application mode
Observable failure or friction: Inline mode depends on cursor queries and main-screen positioning; process suspension has platform-specific signal behavior; an open report describes raw-terminal and blank-screen failures
Root architectural cause: Terminal modes, cursor origin, shell scrollback, signals, and application rendering share one process-global stream
Available workaround: Prefer App.suspend() context management, restrict platform/mode combinations, or restart and force a full redraw after resumption
Cost of workaround: Applications must test the exact OS, terminal, inline/fullscreen mode, and child-process handoff; native scrollback semantics remain distinct from alternate-screen repainting
Upstream response: Historical App.suspend issue is closed; issue #6298 remains open at access and reports action_suspend_process failure
Current status and version: App.suspend and inline mode are implemented in 8.2.8; no physical reproduction here
Evidence: Current driver/app source, suspend tests, issues #5528 and #6298
Confidence: Medium; Evidence status: Verified for lifecycle code, Reported for physical failures
```

`App.suspend()` stops reading and writing, runs external code with output
redirected, restarts the driver, sends a resume signal, and refreshes layout. A
headless test verifies the calls and signals
([suspend test](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/tests/test_suspend.py#L19-L63)).
It does not verify termios, process groups, cursor position, or physical-screen
changes made by a child.

Inline mode has a different ownership problem: the compositor emits a cursor
query and the inline input thread consumes responses while parsing user input.
Native scrollback is therefore not equivalent to an alternate-screen repaint;
it needs explicit append, cursor, clear, and resize contracts.

Closed issue #5528 is historical evidence of a child-editor restore failure;
open #6298 reports raw-terminal and shell failures in `action_suspend_process`
across Linux and macOS ([#5528](https://github.com/Textualize/textual/issues/5528),
[#6298](https://github.com/Textualize/textual/issues/6298)). Neither was
reproduced. ArborUI should retain PTY tests for suspend, handoff, resume,
signals, and inline cursor ownership.

### Strict UTF-8 Input Can Turn Malformed Bytes Into A Lifecycle Failure

```text
Classification: Bug
Requirement: Malformed or non-UTF-8 terminal input must be contained, diagnosed, or replaced without silently losing application input
Library assumption: Terminal input is valid UTF-8 and can be decoded by a strict incremental decoder
Observable failure or friction: Invalid bytes raise UnicodeDecodeError in the input thread; the thread exits through the driver's exception path and the application may panic or stop responding to input
Root architectural cause: Byte decoding is performed before parser recovery, with errors='strict' and no backend-level malformed-input policy
Available workaround: Enforce a UTF-8 locale/input source, patch the driver to replace or reject invalid sequences explicitly, or use a wrapper transport
Cost of workaround: Platform and deployment configuration become part of application correctness; replacement changes user-visible text semantics
Upstream response: Open issue #6456 proposes errors='replace' for Linux, inline, and web drivers
Current status and version: Strict decoder remains in the pinned 8.2.8 source
Evidence: Driver source and issue #6456; not reproduced here
Confidence: Medium; Evidence status: Verified source and Reported failure
```

The Linux, inline, and web drivers each construct
`getincrementaldecoder("utf-8")().decode` without an error policy. Linux and
inline schedule `app.panic` when the input thread raises; web logs the exception.
The source confirms the boundary in these implementations:
[Linux input](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/drivers/linux_driver.py#L403-L468),
[inline input](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/drivers/linux_inline_driver.py#L106-L177),
and [web input](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/drivers/web_driver.py#L184-L214).

Issue #6456 describes invalid locale bytes, binary input, and fragmented invalid
paste as triggers and proposes replacement decoding. See [issue #6456](https://github.com/Textualize/textual/issues/6456).
It was not reproduced; ArborUI's parser fuzzing should include invalid UTF-8 and
input-thread survival.

## Testing Strategy

### Production-Path Headless Tests

Textual's strongest test boundary is `App.run_test()`: the ordinary App runtime
runs with a `HeadlessDriver`, including composition, CSS, queues, layout, focus,
workers, and handlers. Its async `Pilot` injects keys, mouse events, and resize
events ([testing guide](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/docs/guide/testing.md#L57-L84), [headless driver](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/drivers/headless_driver.py#L10-L66)).

Pilot's `press`, `click`, `mouse_down`, `mouse_up`, `hover`, and
`resize_terminal` APIs are useful application-level primitives. A click is
usually routed through the screen's geometry and message path; the source only
bypasses the outer driver event path where needed to make deterministic tests
possible. Pilot waits for all existing App and widget callbacks, then uses a
CPU-time versus wall-time heuristic to wait for process idleness
([Pilot settlement](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/pilot.py#L473-L549), [idle heuristic](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/_wait.py#L8-L41)).

The harness supports ordinary state assertions; repository tests cover
exceptions in `compose`, actions, and workers, mouse down/up/click behavior, and
message order
([Pilot exception tests](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/tests/test_pilot.py#L50-L120), [driver interaction tests](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/tests/test_driver.py#L7-L127)).
This is a good model for `arborui-test`: inject semantic events, settle the real
runtime, and assert focus, model, and visual state together.

The settlement contract is weaker than a manually controlled clock: `Pilot.pause()`
infers idleness from process CPU time. Workers and timers can be timing-sensitive,
and `run_test()` cannot exercise raw mode, terminal responses, physical
restoration, output backpressure, or child processes.

### Unit And Component Tests

Focused tests cover geometry, layout, CSS, reactivity, messages, focus, screens,
compositor scrolling, text, segments, widgets, workers, and driver translation.
They often call production methods; compositor tests inspect `visible_widgets`,
and driver tests call `process_message` then settle through Pilot.

Unicode coverage is logical rather than physical: `Content`, `Strip`, segment,
and input tests cover wide emoji, cropping, cell lengths, and cursor movement,
but not ambiguous-width policy, final-column autowrap, or a particular emulator.

### Snapshot Tests

`pytest-textual-snapshot` produces SVG screenshots from a headless application;
the guide requires report review before `--snapshot-update`. The suite covers
layouts, widgets, focus, scrolling, DataTable, Markdown, themes, selection, and
examples ([snapshot guidance](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/docs/guide/testing.md#L181-L321), [snapshot suite](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/tests/snapshot_tests/test_snapshots.py#L66-L80)).

Snapshots are valuable for CSS and visible behavior that is tedious to assert
cell by cell. The tests are not a transaction oracle: an SVG does not show
whether a physical output write was partial, whether the cursor wrapped in a
real terminal, or whether a screen was restored after a signal. The source
revision search found no PTY suite, terminal-emulator oracle, generalized
partial-write fault matrix, or property/fuzz suite for those boundaries at the
recorded revision. This is a repository-scope observation, not a claim that no
Textual user has built such tests.

### Platform CI And Failure Coverage

CI runs the pytest suite on Ubuntu, Windows, and macOS across Python 3.9-3.14,
with coverage. This catches compatibility regressions, but the subprocess pipe
test is not a PTY lifecycle or partial-output test.

The principal gap is the physical terminal contract: raw mode, alternate-screen
restoration, inline cursor queries, last-column behavior, signals, suspension,
slow output, partial writes, and malformed bytes. ArborUI should combine this
headless pattern with PTY/emulator tests, output fault injection, semantic
snapshots, and explicit settlement controls.

## Common Scenario Assessment

| Scenario | Assessment |
| --- | --- |
| Form with focus, validation, and modal | Strong: focus chain, Input/TextArea, reactive state, actions, and Screen stacks are integrated |
| Large scrollable collection with stable identity | Partial: DataTable has row keys and caches; generic widget lists retain all children and no general lazy provider was found |
| Streaming output from external events | Strong: workers, `call_from_thread`, messages, and the `mother.py` example provide a direct pattern |
| Unicode-heavy text input and editing | Strong logical cell-width and input coverage; physical width, invalid bytes, and emulator differences remain outside headless tests |
| Overlay with clipping and mouse interaction | Strong: compositor layers, clipping, spatial lookup, focus, and mouse capture share geometry |
| Resize during active updates | Supported through resize events and layout invalidation; resize storms and driver-specific cursor behavior need physical tests |
| Deferred, partial, or failed terminal output | Partial to weak: output is queued/written, but no accepted/unknown outcome or recovery contract is exposed |
| Suspension to a child process | Supported by `App.suspend()` and Unix signal paths; cross-platform physical restoration remains reported and unverified here |
| Long idle periods | Good when the queue and timers are quiet; there is no separate application-level idle/backpressure contract |
| Conversation preserving native scrollback | Partial: inline mode preserves ordinary output above an inline app, but immutable append-only history and recovery are application/mode concerns |

## Lessons For ArborUI

### Adopt Or Preserve

- Make a complete application harness a first-class public API. Textual's
  `run_test()` and `Pilot` demonstrate that event injection, layout, workers,
  and visual assertions are more useful together than as separate mocks.
- Keep target selection, z-order, clipping, focus, mouse capture, and rendering geometry derived from one retained arrangement.
- Give asynchronous effects names, lifecycle ownership, cancellation, error states, and a documented thread boundary.
- Provide common widgets, examples, themes, templates, and diagnostics alongside architectural guarantees; Textual's ecosystem is an adoption advantage.
- Treat output modes as explicit contracts. Textual's fullscreen and inline drivers show that cursor origin, resize, clear, and scrollback behavior cannot be hidden behind one generic viewport flag.
- Test the optimized compositor against a simpler reference and retain the patch-replay properties in [`docs/testing-and-roadmap.md`](../../../docs/testing-and-roadmap.md).

### Avoid Or Limit

- Do not let a successful headless snapshot stand in for terminal lifecycle correctness. Add PTY, emulator, and failure-injection coverage before claiming restoration or output recovery.
- Do not make a local widget interaction traverse an entire retained subtree unless cascade semantics require it. Benchmark focus, hover, CSS class changes, and screen transitions at realistic tree sizes.
- Do not make every collection a list of retained widgets. Offer a specialized visible-range/key contract for large data, and keep arbitrary widget composition available for smaller collections.
- Do not accept a background writer with no acknowledgement if the public
  guarantee includes continued operation after uncertain output. Physical state
  should be explicit and invalidated after a possible partial write.
- Do not assume valid UTF-8, terminal width agreement, or final-column behavior merely because Rich produces correct logical cell lengths.

### Problems ArborUI Already Approaches Differently

ArborUI's planned model-update-view boundary gives one application state owner,
allows an ephemeral view to borrow application data, and forbids retained state
from retaining those borrows. That is stricter than Textual's Python object model
and directly addresses a Rust-specific lifetime hazard. The intended separation
of application data, view description, component identity, layout, visual frame,
terminal session, and command scheduler is also clearer than Textual's
framework-owned object graph ([ArborUI architecture](../../../docs/architecture.md)).

ArborUI's prepared-frame transaction addresses a different requirement from
Textual's normal compositor. The prepared visual frame is committed only after
the backend accepts the complete patch; `Deferred` and `StateUnknown` have
defined renderer actions. That is a stronger physical-screen contract, but it
will cost implementation complexity and must be demonstrated with failures that
matter to users ([ArborUI rendering contract](../../../docs/rendering-and-text.md#prepared-frame-transaction)).

ArborUI also deliberately avoids a mandatory async runtime, keeps Crossterm and
layout-engine types behind crate boundaries, and plans a normalized terminal
capability/input layer. Textual shows the convenience of choosing asyncio and
Rich; it does not show that the choice is wrong for a Rust facade intended to be
embedded in varied application runtimes.

### Claims ArborUI Has Not Yet Proven

- A retained identity tree plus explicit invalidation is more ergonomic than Textual's CSS/reactive model for ordinary applications.
- Prepared-frame commit solves failures users encounter often enough to justify its complexity.
- A smaller widget catalog will not outweigh stronger contracts during adoption.
- Full layout/painting and ArborUI's width policies perform well in realistic apps and terminals.
- Runtime independence and backend isolation reduce integration cost; main-screen modes can share useful abstractions with alternate-screen rendering.

### Follow-Up Experiments

1. Implement the same moderate form/dashboard in Textual and facade-only ArborUI, comparing application code, tests, widgets, and error handling.
2. Benchmark focus/CSS changes, one-cell updates, large tables, retained child lists, streaming logs, Unicode, overlays, and resize storms.
3. Add output fault injection and Linux PTY scenarios for alternate screen, inline queries, last-column writes, suspend/resume, child handoff, and invalid UTF-8.
4. Prototype bounded visible-range data before committing to general virtualization, then compare snapshot review and semantic assertions.

## Evidence Appendix

All sources below were accessed on 2026-07-16. Links are pinned to the Textual
8.2.8 commit unless noted; issue state may change.

| Claim | Source | Version or revision | Source date | Accessed | Status | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| Stable release baseline | [v8.2.8 tag](https://github.com/Textualize/textual/releases/tag/v8.2.8) and [pinned changelog](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/CHANGELOG.md#L8-L18) | `1d99508` / `v8.2.8` | 2026-06-30 | 2026-07-16 | Verified | Latest stable baseline selected for this research |
| Compositor and partial rendering | [Compositor](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/_compositor.py#L280-L477) and [render updates](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/_compositor.py#L1031-L1183) | `1d99508` | 2026-06-30 | 2026-07-16 | Verified | Maps, layers, dirty regions, visible and partial updates |
| Large collection storage | [DataTable source](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/widgets/_data_table.py#L733-L869) and [ListView source](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/widgets/_list_view.py#L134-L259) | `1d99508` | 2026-06-30 | 2026-07-16 | Verified | Full row/widget retention; caches and visible rendering are partial mitigations |
| Lazy large DataTable request | [Issue #1892](https://github.com/Textualize/textual/issues/1892) | Opened 2023-02-27; closed as not planned | 2026-07-16 | 2026-07-16 | Reported | Million-cell lazy/chunked view request; current issue state observed |
| Large-list and CSS scaling | [Issue #5163](https://github.com/Textualize/textual/issues/5163), [PR #5164](https://github.com/Textualize/textual/pull/5164), [App.update_styles](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/app.py#L2504-L2523), and [issue #6524](https://github.com/Textualize/textual/issues/6524) | `1d99508` | 2024-10-23 and 2026-05-07 | 2026-07-16 | Reported and Verified | Performance reports plus current descendant-walk source |
| Output/display contract | [Driver](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/driver.py#L134-L173), [App._display](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/app.py#L3821-L3887), and [WriterThread](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/drivers/_writer_thread.py#L9-L68) | `1d99508` | 2026-06-30 | 2026-07-16 | Verified | No write outcome or physical-state acknowledgement found |
| Suspend orchestration | [App.suspend](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/app.py#L4717-L4788) and [suspend test](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/tests/test_suspend.py#L19-L63) | `1d99508` | 2026-06-30 | 2026-07-16 | Verified | Headless driver verifies calls/signals, not physical restoration |
| Suspension and inline lifecycle reports | [Issue #5528](https://github.com/Textualize/textual/issues/5528) and [issue #6298](https://github.com/Textualize/textual/issues/6298) | Historical/current | 2025-02-15 and 2025-12-31 | 2026-07-16 | Reported | Neither reproduced |
| Strict UTF-8 input | [Linux driver](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/drivers/linux_driver.py#L403-L468), [inline driver](https://github.com/Textualize/textual/blob/1d99508b928a771b51e1a527319c6b87dcff9e05/src/textual/drivers/linux_inline_driver.py#L106-L177), and [issue #6456](https://github.com/Textualize/textual/issues/6456) | `1d99508` | 2026-03-30 | 2026-07-16 | Verified and Reported | Strict decoder; proposed replacement not reproduced |
| PTY/fault-injection gap | [Pinned repository tests](https://github.com/Textualize/textual/tree/1d99508b928a771b51e1a527319c6b87dcff9e05/tests) | `1d99508` | 2026-06-30 | 2026-07-16 | Inferred | Searched tests and workflows; no general PTY/emulator/output-fault matrix found |
