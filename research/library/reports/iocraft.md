# iocraft Research Report

## Evidence Header

```text
Research date: 2026-07-16
Project version: iocraft 0.8.4
Project revision: 65e34bf3d15f801293b6316ac474bb8ff5a8a377
Repository: https://github.com/ccbrown/iocraft
Documentation version: docs.rs iocraft 0.8.4; source documentation at the recorded tag
Primary platform examined: Source inspection and tests on Linux; no physical terminal reproduction
Report depth: Deep dive
Development comparison: main at 63268c5f259f688a5cd141cd1377218484fab8c8
```

The latest stable release at the start of this research was
[`iocraft-v0.8.4`](https://github.com/ccbrown/iocraft/releases/tag/iocraft-v0.8.4),
published on 2026-07-13. Source conclusions below refer to that tag unless a
finding explicitly names the later `main` commit. The development comparison is
important because `main` contains a post-release ANSI parser correction; it is
not silently treated as part of 0.8.4.

## Executive Assessment

iocraft is a Rust application framework for terminal interfaces, CLI output, and
dynamic text. Its central proposition is a React-like declarative component API
without requiring application authors to assemble a separate layout library,
widget system, render loop, and component-state convention. The README describes
`element!`, `#[component]`, hooks, Taffy flexbox layout, interactive elements, and
both terminal and non-terminal output as one coherent surface
([README](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/README.md#L16-L33)).

The design is more substantial than an immediate-mode renderer but narrower than
a full desktop-style UI framework. An element is a short-lived description of a
component and its properties. During a render loop, iocraft retains instantiated
components, their hooks, child identity, and Taffy nodes. Keys and component
types determine whether instances are recycled. A render pass updates that tree,
computes the complete layout, paints a new `Canvas`, and writes only changed
rows. This gives application authors local state and a compact declarative
programming model while retaining a simple cell-oriented output boundary.

The strongest use case is a small or medium Rust CLI that needs a polished
interactive form, dashboard, progress display, streaming status view, or inline
text interface and does not want to build focus, component state, and layout
conventions around Ratatui. The public mock-terminal loop is also a useful way to
test dynamic components with normalized key, mouse, and resize events without a
TTY.

The main costs are at the framework and terminal boundaries. Stable iocraft is
hard-wired to Crossterm through a private terminal implementation, broadcasts
terminal events rather than routing consumable UI events, and leaves focus as a
boolean prop convention. Its render loop can restore modes on ordinary drop, but
it has no public backend contract with applied/deferred/unknown output outcomes,
no visible panic restoration path, and no in-session recovery after a possibly
partial write. Its retained tree prevents component state from being rebuilt, but
layout, update, and painting still happen broadly on each pass and collections
are not virtualized.

For ArborUI, iocraft is a valuable warning against making a rigorous runtime feel
like an infrastructure project. Its declarative syntax, local hooks, explicit
inline/fullscreen entry points, and mock loop are practical strengths. ArborUI
should retain those ergonomics while keeping its stronger input ownership,
backend boundary, grapheme contract, and prepared-frame transaction. Neither
project has established end-to-end performance or physical-terminal correctness
with the evidence examined here.

## Project Snapshot

iocraft is a Rust 2021 workspace containing the `iocraft` library and its
procedural macro crate. Version 0.8.4 depends directly on Crossterm 0.29,
Futures, Taffy 0.5.2, `iocraft-macros`, `unicode-width`, `generational-box`, and
Regex. The `unstable-output-streams` feature exposes configurable output writers
and stdout/stderr selection, but the default API still owns the standard terminal
integration ([package manifest](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/Cargo.toml#L1-L28)).

The project spans three comparison categories: application framework, rendering
and widget library, and terminal integration. It is not merely a Canvas or
buffer package. `print` and `write` render a one-shot element, while
`render_loop()` and `fullscreen()` create the dynamic path. Inline rendering is
the default loop mode; fullscreen mode enters the alternate screen. The source
also includes `View`, `Text`, `TextInput`, `ScrollView`, `Button`, `Fragment`,
`MixedText`, and `ContextProvider`, a useful but intentionally compact standard
widget set ([component exports](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/components/mod.rs#L1-L23)).

Maintenance was active at the baseline. The 0.8.4 changelog records Unicode
cursor preservation, keyboard-enhancement probing, fixed-width deletion
scrolling, output corruption fixes, and new text styles
([changelog](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/CHANGELOG.md#L8-L21)).
The repository's current `main` then changed ANSI stripping in commit
`63268c5f259f688a5cd141cd1377218484fab8c8` (#214). That activity supports a
maturity assessment of active maintenance, not a claim that every terminal edge
case is solved.

## Core Proposition

The ordinary authoring experience is a declarative tree:

```rust
element! {
    View(border_style: BorderStyle::Round) {
        Text(content: "Hello, world!")
    }
}
```

The `element!` macro constructs typed elements and child collections. A custom
component is a Rust function annotated with `#[component]`; hooks supply state,
effects, futures, context, terminal events, and imperative references. The
result feels close to React or SwiftUI, but properties and handlers remain Rust
values and can borrow application data. The public API is intentionally flat,
with common types re-exported from the crate root and a prelude for normal
applications ([crate documentation](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/lib.rs#L1-L55)).

Unlike Ratatui, the application does not need to own the main redraw loop just to
obtain component-local state and flexbox layout. Unlike a terminal substrate,
iocraft also decides how components wait for changes, how output is redrawn, and
how standard controls consume events. It remains executor-independent in the
sense that `render_loop()` is a Future and applications can drive it with their
chosen executor; `use_future` itself requires a self-contained `Send + 'static`
future.

The proposition is strongest for applications whose whole visible UI can be
described every time a retained component changes. It is weaker for large
virtualized datasets, elaborate focus graphs, protocols requiring a custom
terminal backend, or applications that must survive uncertain physical output
and continue in the same session.

## Architecture

### Application And State Model

An `Element<'a, T>` is an uninstantiated component description containing an
`ElementKey` and properties. The key can hold any value satisfying
`Debug + Hash + Eq + Send + Sync + 'static`. Conversion to `AnyElement` can preserve borrowed properties,
while owned properties are used when appropriate
([element model](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/element.rs#L61-L132)).
This is an ergonomic answer to Rust ownership: the view can be rebuilt from
borrowed application state without requiring every prop to be cloned.

The render loop creates a `Tree` containing a root `InstantiatedComponent`, a
Taffy tree, and system context. Each instantiated component retains its concrete
component value, children, hooks, component helper, and Taffy node
([retained component](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/component.rs#L131-L186)).
When a component updates its children, iocraft removes the next matching key
from a `RemoveOnlyMultimap`. It reuses the instance only when the component
`TypeId` also matches; otherwise it creates a new Taffy node. The specialized
multimap uses a key-to-queue map and preserves insertion order for duplicate
keys ([recycling map](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/multimap.rs#L6-L14),
[reconciliation](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/render.rs#L151-L215)).

State is hook-owned rather than application-global. `use_state` stores a value in
a generational box owned by the hook and wakes the component when mutable access
changes it ([state hook](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/hooks/use_state.rs#L79-L165)).
`use_ref` provides component-owned mutable storage without requesting a redraw.
Contexts can be owned or borrowed and are scoped while children are updated.
The hook collection identifies hooks by call order; a different order panics,
which is documented as the same rule used by React hooks
([hook rules](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/hooks/mod.rs#L31-L41),
[hook lookup](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/hook.rs#L118-L135)).

This retained design solves a real problem: a counter, text cursor, scroll
offset, or child future survives ordinary view rebuilding when identity remains
stable. It does not provide a general retained scene graph. A one-shot
`element.render()` creates a new tree for that call, and the update path still
walks the component hierarchy on each render pass.

### Rendering And Layout

The render pass has a clear sequence. It updates the root and children, attaches
the current child node list to a wrapper, calls Taffy's
`compute_layout_with_measure`, allocates a new Canvas with the resulting size,
and recursively draws the retained component tree into clipped subviews
([render sequence](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/render.rs#L380-L461)).
`View` translates layout props into Taffy styles and establishes border and
overflow behavior; text components install measurement functions that wrap by
terminal width ([View](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/components/view.rs#L138-L229),
[Text](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/components/text.rs#L89-L135)).

`Canvas` is a two-dimensional vector of cells. A cell stores optional background
color and a string representing a character plus any zero-width code points that
were collected with it. Row equality trims trailing empty cells, so the
terminal implementation can skip unchanged rows. The writer tracks SGR and
background state, clears lines, and emits ANSI or plain text
([Canvas representation](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/canvas.rs#L77-L114),
[row diff support](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/canvas.rs#L251-L377)).

The retained tree therefore reduces state reconstruction, not all rendering
work. `render::terminal_render_loop` keeps the previous Canvas, calls the broad
render sequence, and asks the terminal to rewrite only different rows
([loop](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/render.rs#L463-L501)).
There is no dirty-subtree scheduler or collection virtualization in the baseline.
The cost is reasonable for modest screens and dynamic CLI content, but it is a
structural scaling limit for a large tree or high-frequency update stream. No
equivalent application benchmark was run, so this is an architectural inference,
not a performance measurement.

### Events, Effects, And Scheduling

The terminal event type has only three variants: key events, fullscreen mouse
events, and resize. Crossterm's event stream filters other input values before
they reach components ([event types](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/terminal.rs#L21-L88),
[conversion](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/terminal.rs#L357-L385)).
The terminal stores subscriber queues. `wait()` reads one stream and clones each
event into every live subscriber; `use_terminal_events` invokes a callback for
all events, while `use_local_terminal_events` filters mouse coordinates to the
component rectangle. Key and resize events still reach every local subscriber
([broadcast and local filtering](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/hooks/use_terminal_events.rs#L91-L193),
[subscriber delivery](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/terminal.rs#L669-L715)).

This is simple and composable for a single handler or a small form. It is not a
capture-target-bubble system. `Button` and `TextInput` use local events but
manually check a `has_focus` prop for keyboard handling. There is no built-in
focus traversal, first responder, event consumption, or mouse hit map. An
application can build those policies above the hooks, but each component must
cooperate with application state.

The render loop waits on either the root component's change future or the
terminal event stream. `use_future` polls one component-bound future and drops it
when the hook is dropped. `use_effect` hashes dependencies and runs a closure
after component update; it offers no cleanup function. `use_async_handler` puts
each invoked future into a `Vec` and polls all queued futures, with no documented
bound or backpressure policy ([future hook](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/hooks/use_future.rs#L14-L81),
[effect hook](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/hooks/use_effect.rs#L10-L73),
[async handler](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/hooks/use_async_handler.rs#L15-L70)).

### Terminal Ownership And Lifecycle

The high-level loop constructs a private `Terminal` backed by a private
`TerminalImpl`; the concrete `StdTerminal` uses Crossterm for raw mode, mouse
capture, alternate screen, keyboard-enhancement probing, cursor movement, and
event decoding. `fullscreen()` enters the alternate screen, while inline mode
rewrites the current canvas region. The terminal tracks previous canvas height
and size, uses absolute row positioning in fullscreen mode, and falls back to a
full clear in some inline overflow cases ([terminal implementation](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/terminal.rs#L114-L128),
[writes and resize behavior](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/terminal.rs#L199-L355)).

Drop disables raw mode, mouse capture, and keyboard flags, leaves the alternate
screen when needed, and shows the cursor. A synchronized-update guard also
attempts to close its update envelope during drop
([restoration](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/terminal.rs#L430-L467),
[synchronized update](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/terminal.rs#L744-L761)).
These are useful ordinary-shutdown safeguards, but they are best-effort cleanup,
not a transaction protocol. `write_canvas` writes directly to the destination;
an I/O error returns from the render loop. There is no public way to distinguish
no bytes written from a partially applied patch, and no renderer state that can
force a full repaint after an uncertain write. `use_output` additionally ignores
several write, flush, and cursor-query errors while printing above the UI
([output hook](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/hooks/use_output.rs#L73-L151)).

## Core Strengths

### Declarative Rust Ergonomics

`element!` and `#[component]` turn a complete Rust component into a readable
tree, while the flat prelude hides internal module organization. Props can borrow
data and handlers can be ordinary closures. This is a meaningful reduction in
application ceremony compared with assembling a renderer, state store, and
component convention independently. The React-like model is not just marketing:
the implementation actually retains component instances and hook slots between
updates.

### Retained Identity With Small Primitives

`ElementKey`, component `TypeId`, and the O(1) recycling map provide a focused
answer to dynamic child identity. A keyed input or list item can keep its state
when siblings are inserted, and unkeyed repeated elements have deterministic
queue-order recycling. This is narrower than a general scene graph but gives
application authors the state continuity they usually expect from a component
framework.

### Integrated Flexbox, Text, And Inline Output

Taffy handles layout, components draw into clipped Canvas subviews, and the same
element can render plain text to a pipe or ANSI output to a terminal. Inline and
fullscreen modes are explicit configuration choices rather than separate widget
systems. `ScrollView` supports keyboard and mouse scrolling and an auto-scroll
mode, while `TextInput` includes wrapping, cursor rendering, and multiline
editing. This breadth makes iocraft useful before an application has its own
widget library.

### A Practical Logical Test Seam

`mock_terminal_render_loop` drives the production component update, layout, hook,
and Canvas path while replacing physical output with a stream of Canvas values.
Tests can inject `TerminalEvent` values, collect each frame, inspect text or
cells, and assert component behavior. Internal `avt` tests take a second step by
feeding generated ANSI into a virtual terminal and checking rows and cursor
position. The seam is not a physical terminal, but it is substantially more
useful than testing only isolated string helpers.

### Active Correction Of Terminal Edge Cases

The release history shows maintenance around the difficult parts: Unicode cursor
preservation, fixed-width editing scroll, keyboard protocol probing, output
interleaving, row-level diffing, and terminal cleanup. The later `main` commit
adds regression tests for long CSI parameters and DCS/APC sequences. This is
evidence of a project that responds to concrete terminal failures, even though it
does not justify universal terminal compatibility claims.

## Limitations And Frustrations

### Backend Extensibility Stops At Crossterm

```text
Classification: Extension failure
Requirement: Use a different terminal input/output implementation or integrate a specialized backend
Library assumption: Crossterm is the concrete terminal boundary for the high-level loop
Observable failure or friction: Application authors cannot supply a public backend object or replace input decoding
Root architectural cause: TerminalImpl and Terminal are crate-private; the public loop constructs StdTerminal
Available workaround: Fork iocraft, patch the private terminal layer, or only replace configured output writers
Cost of workaround: A fork tracks terminal, lifecycle, and API changes; a writer does not replace input or capabilities
Upstream response: Issue #202 says this was not possible; draft PR #210 proposes TerminalBackend
Current status and version: Not available in stable 0.8.4; PR #210 was open and draft on 2026-07-16
Evidence: Verified source boundary; supported by maintainer issue and open draft PR
Confidence: High
```

The stable package exposes Crossterm event types and a render-loop configuration,
not a backend trait. The `unstable-output-streams` feature helps a CLI keep a TUI
on stderr while stdout remains pipeable, but the documentation itself records
Crossterm operations that bypass the supplied writer. That is an output-routing
escape hatch, not backend independence. The maintainer's response to
[issue #202](https://github.com/ccbrown/iocraft/issues/202) explicitly said a
separate backend was not currently possible. The open draft
[PR #210](https://github.com/ccbrown/iocraft/pull/210) is strong evidence that
the boundary is recognized and being redesigned, but its proposed trait is
development behavior and must not be credited to 0.8.4.

### Broadcast Input Requires Application-Owned Focus

```text
Classification: Limitation and tradeoff
Requirement: A form or overlay must route each key or pointer event to one intended target
Library assumption: Components can subscribe to a shared terminal event stream and decide locally whether to act
Observable failure or friction: Key and resize events reach every local subscriber; handlers have no consume or stop operation
Root architectural cause: TerminalEvents is a broadcast queue and the API has no focus manager or routed event object
Available workaround: Keep a focused-component flag in application state and make every handler check it
Cost of workaround: Focus traversal, modal priority, propagation, and duplicate-handler prevention become application conventions
Upstream response: Open issue #113 proposes consumable bubbling events and a first-responder mechanism
Current status and version: Behavior verified in stable 0.8.4; issue #113 remained open on the research date
Evidence: Verified source; supported by the maintainer's issue proposal
Confidence: High
```

Mouse-local filtering is helpful: coordinates are translated and out-of-bounds
mouse events are omitted. It does not generalize to keyboard focus. `Button`
checks `has_focus` before acting on Enter or Space; `TextInput` checks the same
flag, but neither establishes that flag or traverses focus. An overlay can be
composed visually, yet its keyboard and mouse ownership still depends on the
application's state discipline. The open [event mechanism issue #113](https://github.com/ccbrown/iocraft/issues/113)
describes exactly the missing concepts: dispatch, consumption, bubbling, and a
first responder. A single global event handler is a workable small-application
solution, but it becomes a coordination layer the framework could otherwise
provide.

### Terminal Recovery Is Cleanup, Not A Committed-Frame Contract

```text
Classification: Limitation relative to ArborUI's recovery requirement
Requirement: Continue after deferred or partially applied output while knowing whether a full repaint is required
Library assumption: A terminal write error ends the current loop and normal Drop restoration is sufficient
Observable failure or friction: A direct write can fail after an unknown prefix; no public outcome reports physical state
Root architectural cause: Canvas diffing and terminal writes are coupled to a private Crossterm implementation without staged commit state
Available workaround: Install an application panic hook, exit after output failure, or fork the terminal layer to keep a shadow frame
Cost of workaround: In-session recovery and consistent panic presentation must be rebuilt outside the library
Upstream response: Open issue #103 requests visible panic output after fullscreen teardown
Current status and version: Drop restoration exists in 0.8.4; panic and partial-write recovery are not a public contract
Evidence: Verified source; issue report supports the panic-path frustration
Confidence: High for the API limitation, medium for physical failure impact
```

The library does get important lifecycle details right for ordinary paths. A
fullscreen terminal enters the alternate screen, raw mode is enabled when the
event stream starts, and Drop attempts to disable modes and show the cursor. A
virtual terminal test covers row diffing and cursor placement. Those safeguards
do not answer what happens when `write_all` or a flush applies only a prefix. The
render loop returns the I/O error, and the application normally stops polling;
there is no `Applied`, `Deferred`, or `StateUnknown` result and no full-repaint
flag that survives to the next attempt. The open [panic-output issue #103](https://github.com/ccbrown/iocraft/issues/103)
also shows a user-facing consequence of alternate-screen ownership: a panic can
be printed into a screen that disappears before the developer sees it.

Inline overflow has a more specific history. [Issue #118](https://github.com/ccbrown/iocraft/issues/118)
reported scrolling and duplicate-render behavior when dynamic output exceeded the
terminal. The maintainer adopted an Ink-like whole-terminal clear in 0.7.12,
and the current code still has a full-clear fallback when an old canvas fills the
terminal. That is a reasonable mode-specific mitigation, not proof of general
scrollback or suspend/resume correctness.

### Broad Rendering And No Virtualized Collection Boundary

```text
Classification: Performance tradeoff
Requirement: Sustain large collections or high-rate updates without work proportional to the whole UI tree
Library assumption: A retained component tree plus row-level output diff is sufficient for normal screens
Observable failure or friction: Each changing pass updates components, computes Taffy layout, allocates a Canvas, and paints all retained children
Root architectural cause: Retention preserves identity but does not provide dirty-subtree layout/paint or visible-range virtualization
Available workaround: Window the collection in application code, reduce redraw frequency, or use use_output for append-only history
Cost of workaround: The application owns viewport policy and loses some declarative simplicity
Upstream response: No general virtualization or incremental-tree contract was found at the recorded revision
Current status and version: Intentional architecture in stable 0.8.4; no workload was benchmarked
Evidence: Verified render sequence; inferred user cost
Confidence: Medium
```

The output diff is useful because unchanged rows produce no terminal bytes, and
the loop waits when neither a component nor the terminal has a change. It does
not avoid the update/layout/paint work needed to discover those unchanged rows.
`ScrollView` clips children and manages offsets, but it accepts a vector of
children and does not turn a million-item collection into a visible-window
provider. This is not a defect for a dashboard or short form. It is a meaningful
boundary for a chat log, large table, or animated tree unless the application
builds its own windowing.

### Text Correctness Is Better Than `char`, But Not A Grapheme Contract

```text
Classification: Limitation and ecosystem tradeoff
Requirement: Cursor movement, editing, wrapping, and physical placement must agree for grapheme clusters and terminal widths
Library assumption: unicode-width plus character-oriented byte offsets are an adequate logical model
Observable failure or friction: TextInput moves by Unicode scalar boundaries and Canvas derives cells from character widths; terminals may disagree on emoji and ambiguous width
Root architectural cause: The baseline has no explicit grapheme identity or configurable terminal width policy
Available workaround: Restrict input, add application-level editing, or validate against target terminals
Cost of workaround: Compatibility policy and custom editing behavior leave the normal component path
Upstream response: 0.8.4 shipped a Unicode insertion-cursor fix; main separately fixes ANSI stripping
Current status and version: Active maturity area in 0.8.4; main at 63268c5 contains later ANSI corrections
Evidence: Verified source and release tests; terminal compatibility inference
Confidence: Medium
```

This finding should be read carefully. iocraft is not treating each Rust `char`
as an independent cell. `Canvas` groups zero-width code points with preceding
text, measures strings with `unicode-width`, and includes compatibility padding
for terminals known to mishandle VS16 emoji. `TextInput` handles byte offsets,
wide CJK characters, wrapping, and multiline vertical movement. The stable
release specifically fixed cursor preservation after Unicode insertion and tests
Kanji input. However, cursor left/right and deletion use `char_indices`, not a
grapheme-segmentation contract, and the physical terminal remains outside the
logical Canvas test. The post-release [ANSI parser commit](https://github.com/ccbrown/iocraft/commit/63268c5f259f688a5cd141cd1377218484fab8c8)
adds a more correct ECMA-48 CSI grammar and DCS/APC/PM/SOS handling, showing
that even text preprocessing remains an active boundary.

## Testing Strategy

iocraft tests the logical component and terminal paths separately, with one
particularly useful integration seam between them.

Direct component tests call `element!(...).to_string()` or `.render(None)` and
inspect Canvas text, cells, styles, or exact ANSI bytes. The `Text` tests cover
wrapping, alignment, Unicode emoji, ANSI stripping, and style output. `TextInput`
tests drive key press, repeat, release, CJK, overflow, multiline, and cursor
movement cases. These tests exercise production components and hooks, not a
parallel fake widget implementation ([Text tests](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/components/text.rs#L271-L373),
[TextInput tests](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/components/text_input.rs#L629-L904)).

`MockTerminalConfig` accepts a stream of `TerminalEvent` values. The public
`mock_terminal_render_loop` runs the real tree, reconciliation, hooks, Taffy
layout, and Canvas painting, while the mock terminal returns each output Canvas
through a stream. Tests can therefore inject key, mouse, and resize events and
assert every frame or the settled final frame. The button, terminal-event,
terminal-size, async-handler, and render-loop tests use this pattern. It is a
strong headless application slice, although it does not encode actual ANSI bytes
or exercise Crossterm's terminal state machine ([mock API](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/element.rs#L234-L270),
[mock terminal](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/terminal.rs#L470-L568)).

The internal terminal tests use `avt::Vt` as a virtual terminal. They feed
initial Canvas output and row-level diffs into the emulator, then assert visible
rows and cursor positions for inline and fullscreen rewriting, shrinking and
growing canvases, scrolling at the bottom of the screen, styled text, and no-op
frames. This is a good semantic test of ANSI serialization without opening a
PTY ([virtual terminal tests](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/terminal.rs#L811-L960),
[diff tests](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/terminal.rs#L1038-L1277)).

The repository CI runs formatting, build, tests, clippy with warnings denied, and
documentation on Ubuntu and Windows using Rust 1.94.0. A separate Ubuntu job
generates Codecov coverage. The local stable checkout also passed
`cargo test --workspace` during this research. These checks give reasonable
cross-platform compile and logical-test confidence
([CI](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/.github/workflows/commit.yaml#L1-L38),
[check tasks](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/Makefile.toml#L4-L56)).

Important gaps were not found at the recorded revision. There is no public
application harness with a run-until-idle operation, controllable clock, command
completion, focus inspection, or structured model assertions. The mock stream
can inject events, but the terminal mock's writer always succeeds. The source
contains no PTY or terminal-emulator lifecycle suite for raw mode, panic,
suspend, signal handling, or restoration; `avt` tests only the logical output
path. No general fuzz/property suite or output fault matrix was found. The
terminal test itself says there is unfortunately little that can be tested for
the real terminal and leaves emulation as a TODO
([real-terminal test](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/terminal.rs#L763-L809)).

| Capability | iocraft 0.8.4 |
| --- | --- |
| Render components without a terminal | Strong: production Canvas path |
| Drive a dynamic component headlessly | Strong: `mock_terminal_render_loop` |
| Inject key, mouse, and resize events | Supported through `MockTerminalConfig` |
| Inspect text and cells | Supported through Canvas APIs |
| Test ANSI semantics | Strong internally with `avt` |
| Test full physical lifecycle | Not found at the recorded revision |
| Inject partial or failed writes | Not provided by the public mock |
| Control time and async settlement | Application/executor-defined; no public clock |
| Focus and event propagation assertions | Application-defined |
| PTY or emulator compatibility matrix | `avt` only; no PTY matrix found |
| Property or fuzz testing | Not found at the recorded revision |
| Cross-platform CI | Strong compile and logical-test coverage |

## Common Scenario Assessment

| Scenario | Assessment |
| --- | --- |
| Form with focus, validation, and modal | Visual controls and local handlers exist; traversal, validation, modal priority, and focus state are application-owned |
| Large scrollable collection | `ScrollView` provides clipping, scrolling, scrollbar, and auto-scroll; children are still eagerly represented and painted |
| Streaming external updates | `use_future`, state hooks, and `use_output` support it; task backpressure and output failure semantics are weak |
| Unicode-heavy text input | Width-aware wrapping and byte-offset editing are useful; grapheme and physical-terminal policy remain incomplete |
| Overlay with clipping and mouse interaction | Taffy overflow and local mouse coordinates work; no retained hit map or propagation model exists |
| Resize during active updates | Resize events update hooks and the loop refreshes size before rendering; physical resize recovery is not transactional |
| Deferred or failed output | Ordinary I/O errors terminate the loop; no deferred/unknown outcome or retry contract |
| Suspension to a child process | Dropping the loop restores ordinary modes; explicit suspend, resume, capability reacquisition, and forced repaint are not public APIs |
| Long idle periods | The loop waits for component or terminal changes rather than continuously repainting |
| Native scrollback conversation | Inline mode and `use_output` support append-oriented output; editing already-scrolled history is not the same contract |

## Lessons For ArborUI

### Adopt Or Preserve

+ Keep the application entry point small. A declarative macro, a component
  attribute, and a prelude can expose a sophisticated runtime without exposing
  its reconciliation machinery.
+ Preserve retained identity as a focused primitive. Explicit keys plus type
  checks solve dynamic state continuity without requiring every application to
  understand the whole retained tree.
+ Make the ordinary widget path pleasant. Flexbox, text measurement, borders,
  input, scrolling, inline output, and fullscreen output should be available
  together, not left as architecture exercises.
+ Provide a public mock loop that exercises production reconciliation, hooks,
  layout, and painting. Pair it with semantic Canvas assertions and virtual
  terminal tests rather than relying on string snapshots alone.
+ Keep output modes explicit. Inline, alternate-screen, append-only output, and
  future native-scrollback modes have different ownership and recovery rules.

### Avoid Or Keep Different

+ Do not make a broadcast event queue the only interaction boundary. ArborUI's
  retained focus, hit testing, propagation, capture, and protocol-response
  ownership should remain first-class, even if a broadcast or global shortcut
  API is offered as an escape hatch.
+ Do not hide backend choice inside a private terminal implementation if backend
  independence is a product requirement. The public trait must cover input,
  output, capabilities, lifecycle, and failure outcomes coherently.
+ Keep prepared-frame commit separate from physical writes. iocraft's direct
  Canvas write path is simple, but it cannot promise a correct next frame after a
  partial write.
+ Treat panic restoration, suspend/resume, and output errors as correctness
  paths. Drop-based best effort is useful but insufficient for the documented
  ArborUI contract.
+ Do not claim Unicode correctness from `unicode-width` and logical tests alone.
  Keep grapheme identity, width policies, continuation-cell invariants, and PTY
  or emulator evidence visible in the design.

### Claims Not Yet Proven

iocraft demonstrates that hooks and retained component identity can make a Rust
TUI concise, but this research did not measure whether its broad update/layout/
paint path is faster or slower than Ratatui for representative applications. It
also did not establish how its restoration behaves under real panic, signal,
multiplexer, or partial-write conditions. ArborUI should likewise avoid claiming
that its transaction and grapheme machinery will improve user-visible behavior
until fault-injection and physical-terminal tests demonstrate that benefit.

### Follow-Up Work

1. Build the same moderate form and streaming dashboard with iocraft and the
   ArborUI facade; compare application-owned focus code, test code, and failure
   handling rather than comparing library feature lists.
2. Add an iocraft-style component test example to ArborUI documentation and
   compare the ergonomics of Canvas assertions, styled-cell assertions, and the
   application harness.
3. Benchmark idle, one-cell, large-list, resize-storm, Unicode, overlay, and
   streaming workloads with equal viewport sizes and record emitted bytes,
   complete-turn latency, allocations, and idle CPU.
4. Add PTY and failure-injection cases to ArborUI before claiming that its
   prepared-frame transaction is a practical differentiator.
5. Prototype explicit inline and native-scrollback contracts independently;
   iocraft's issue #118 shows why append-only history and a live redraw region
   should not be treated as one screen mode.

## Evidence Appendix

All sources below were accessed or inspected on 2026-07-16. Stable source links
use commit `65e34bf3d15f801293b6316ac474bb8ff5a8a377`; development links use
commit `63268c5f259f688a5cd141cd1377218484fab8c8`.

| Claim | Source | Version or revision | Source date | Accessed | Status | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| Stable baseline and release | [iocraft-v0.8.4](https://github.com/ccbrown/iocraft/releases/tag/iocraft-v0.8.4) | tag `iocraft-v0.8.4`; commit `65e34bf` | 2026-07-13 | 2026-07-16 | Verified | Latest stable release at research start |
| Versioned API documentation | [docs.rs iocraft 0.8.4](https://docs.rs/iocraft/0.8.4/iocraft/) | 0.8.4 | 2026-07-13 | 2026-07-16 | Supported | Version-matched generated API documentation |
| Package scope and dependencies | [Cargo manifest](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/Cargo.toml#L1-L28) | `65e34bf` | 2026-07-13 | 2026-07-16 | Verified | Rust 2021, Crossterm, Taffy, hooks/state dependencies |
| Project intent and examples | [README](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/README.md#L16-L100) | `65e34bf` | 2026-07-13 | 2026-07-16 | Supported | Establishes declarative, React-like, fullscreen, and example scope |
| Element keys and borrowed props | [Element source](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/element.rs#L61-L132) | `65e34bf` | 2026-07-13 | 2026-07-16 | Verified | Key and AnyElement conversion behavior |
| Retained component state | [Component source](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/component.rs#L131-L221) | `65e34bf` | 2026-07-13 | 2026-07-16 | Verified | Component, child, hook, and Taffy-node retention |
| Key recycling | [Multimap](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/multimap.rs#L6-L73) and [reconciliation](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/render.rs#L151-L215) | `65e34bf` | 2026-07-13 | 2026-07-16 | Verified | O(1) key lookup and insertion-order duplicate handling |
| Hooks and generational state | [Hook rules](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/hooks/mod.rs#L31-L41), [hook slots](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/hook.rs#L118-L135), and [state](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/hooks/use_state.rs#L79-L165) | `65e34bf` | 2026-07-13 | 2026-07-16 | Verified | React-like ordering and component-owned state |
| Full update/layout/paint sequence | [Render source](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/render.rs#L380-L461) | `65e34bf` | 2026-07-13 | 2026-07-16 | Verified | Taffy layout and fresh Canvas per pass |
| Row-level output diff | [Canvas source](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/canvas.rs#L251-L377) and [render loop](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/render.rs#L463-L501) | `65e34bf` | 2026-07-13 | 2026-07-16 | Verified | Diff reduces output, not broad render work |
| Event categories and broadcast | [Terminal source](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/terminal.rs#L21-L128) and [delivery](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/terminal.rs#L669-L715) | `65e34bf` | 2026-07-13 | 2026-07-16 | Verified | Key, fullscreen mouse, resize; cloned to subscribers |
| Local event filtering | [Terminal event hook](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/hooks/use_terminal_events.rs#L91-L193) | `65e34bf` | 2026-07-13 | 2026-07-16 | Verified | Mouse is spatially filtered; key/resize are not consumed |
| Backend boundary | [Private terminal implementation](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/terminal.rs#L114-L128) and [loop construction](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/element.rs#L455-L518) | `65e34bf` | 2026-07-13 | 2026-07-16 | Verified | No stable public backend trait |
| Backend alternatives are not currently possible | [Issue #202](https://github.com/ccbrown/iocraft/issues/202) | Open | 2026-05-25 and 2026-07-03 | 2026-07-16 | Supported | Maintainer response says not currently possible |
| Backend trait proposal | [Draft PR #210](https://github.com/ccbrown/iocraft/pull/210) | Open draft; head `7cbd6f8` | 2026-07-03 | 2026-07-16 | Reported | Development proposal, not shipped in 0.8.4 |
| Drop restoration and synchronized update | [Terminal lifecycle](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/terminal.rs#L430-L467) and [guard](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/terminal.rs#L744-L761) | `65e34bf` | 2026-07-13 | 2026-07-16 | Verified | Best-effort Drop cleanup |
| Panic output concern | [Issue #103](https://github.com/ccbrown/iocraft/issues/103) | Open | 2025-05-23 | 2026-07-16 | Reported | Fullscreen panic output concern not independently reproduced |
| Inline overflow history | [Issue #118](https://github.com/ccbrown/iocraft/issues/118) | Closed; fixed in 0.7.12 | 2025-08-11 to 2025-09-20 | 2026-07-16 | Reported | Maintainer adopted whole-terminal clear mitigation |
| Text and input behavior | [Text source](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/components/text.rs#L89-L268) and [TextInput source](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/components/text_input.rs#L348-L627) | `65e34bf` | 2026-07-13 | 2026-07-16 | Verified | Width-aware, character-oriented editing |
| Stable Unicode cursor fix | [PR #212](https://github.com/ccbrown/iocraft/pull/212) and [changelog](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/CHANGELOG.md#L16-L21) | shipped in 0.8.4 | 2026-07-13 | 2026-07-16 | Verified | Release change, not grapheme proof |
| Post-release ANSI correction | [Main commit #214](https://github.com/ccbrown/iocraft/commit/63268c5f259f688a5cd141cd1377218484fab8c8) and [tests](https://github.com/ccbrown/iocraft/blob/63268c5f259f688a5cd141cd1377218484fab8c8/packages/iocraft/src/strip_ansi.rs#L4-L110) | `63268c5` | 2026-07-14 | 2026-07-16 | Verified | Corrects CSI grammar and strips DCS/APC/PM/SOS |
| Mock application test path | [Element mock API](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/element.rs#L234-L270) and [mock terminal](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/terminal.rs#L470-L568) | `65e34bf` | 2026-07-13 | 2026-07-16 | Verified | Production tree and hooks; physical writer replaced |
| Virtual terminal output tests | [AVT setup and tests](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/terminal.rs#L811-L960) | `65e34bf` | 2026-07-13 | 2026-07-16 | Verified | Semantic rows and cursor assertions |
| CI and coverage | [Commit workflow](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/.github/workflows/commit.yaml#L1-L38) and [Makefile tasks](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/Makefile.toml#L4-L56) | `65e34bf` | 2026-07-13 | 2026-07-16 | Verified | Ubuntu/Windows checks and Ubuntu coverage |
| PTY, fuzz, and fault-injection facilities not found | [Pinned repository tree](https://github.com/ccbrown/iocraft/tree/65e34bf3d15f801293b6316ac474bb8ff5a8a377) and [real-terminal test note](https://github.com/ccbrown/iocraft/blob/65e34bf3d15f801293b6316ac474bb8ff5a8a377/packages/iocraft/src/terminal.rs#L763-L809) | `65e34bf` | 2026-07-13 | 2026-07-16 | Inferred | Searched source, tests, workflows, and package manifests |
