# Bubble Tea Research Report

## Evidence Header

```text
Research date: 2026-07-16
Project version: charm.land/bubbletea/v2 v2.0.8
Project revision: fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1
Repository: https://github.com/charmbracelet/bubbletea
Documentation version: pkg.go.dev charm.land/bubbletea/v2@v2.0.8; README.md and UPGRADE_GUIDE_V2.md at the recorded revision
Primary platform examined: Source and test inspection on Linux; no physical terminal or PTY reproduction
Report depth: Deep dive
```

The baseline is the
[`v2.0.8` release](https://github.com/charmbracelet/bubbletea/releases/tag/v2.0.8),
released on 2026-07-03. The tag object is
`960311fe80ea87e6d16cae980f5785b0d23ec102`; its dereferenced source commit is
`fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1`. Version-sensitive conclusions below
refer to that source revision unless another revision or issue is named.
Documentation and external evidence were accessed on 2026-07-16.

## Executive Assessment

Bubble Tea is an application framework, not a terminal drawing library with an
optional event loop. Its core contract is The Elm Architecture: an
application-owned model implements `Init`, `Update`, and `View`; messages enter
a central loop; updates return commands; and the framework renders the resulting
view. The v2 API makes the terminal configuration part of the declarative view
as well as the content. Alternate-screen mode, mouse reporting, focus reporting,
bracketed paste, cursor state, colors, window title, keyboard enhancements, and
a native progress-bar request can all be described by `tea.View`.

The framework's strongest boundary is the integration it owns. Bubble Tea
translates terminal input through Ultraviolet, serializes model updates, runs
commands, handles signals and resize notifications, manages raw terminal state,
and drives a cell-oriented renderer. This produces a compact application model
for dashboards, forms, chat-like tools, pagers, and long-running command-line
applications. The companion Bubbles repository supplies stateful components such
as lists, text inputs, text areas, viewports, tables, spinners, and file
pickers. Lip Gloss supplies styling and compositing above the string-based view
boundary.

The same boundary leaves several ArborUI requirements outside the framework.
Bubble Tea has no retained keyed UI tree, built-in focus or hit-target registry,
command cancellation contract, deterministic clock, run-until-idle operation, or
public complete-application simulator. Its renderer accepts ordinary writer
errors and has no public applied/deferred/unknown output result. Inline output
and native scrollback are supported by cursor-manipulation paths, but unmanaged
output is a separate state machine from the rendered view. These are not all
defects in Bubble Tea's intended model. Some are deliberate flexibility; some
are maturity gaps; some are concrete lifecycle or physical-terminal failure
cases.

For ArborUI, Bubble Tea validates the value of a small reducer-like update
boundary, explicit terminal modes, one input owner, and a production renderer
that can be driven with in-memory input and output. It does not displace
ArborUI's reasons for retained identity, prepared-frame commit, physical-state
invalidation, or a deterministic application harness. Bubble Tea also
demonstrates the adoption cost ArborUI must confront: a strong framework
contract is less useful if common widgets, examples, and integration escape
hatches are not equally mature.

## Project Snapshot

Bubble Tea is written in Go and describes itself as a framework for rich
terminal user interfaces based on The Elm Architecture. The v2 module requires
Go 1.25.0 and changes the import path to `charm.land/bubbletea/v2`. It is
maintained by Charmbracelet alongside Bubbles, Lip Gloss, and Ultraviolet. The
[pinned README](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/README.md#L10-L30)
establishes the intended inline, full-window, and mixed scope.

The project category is application framework. It owns the update scheduler,
command execution, terminal input, rendering cadence, and terminal lifecycle. It
does not own domain state, application navigation, layout constraints, focus
policy, or a retained component graph. Those are assembled by the application
and by higher-level component libraries.

The latest release has active maintenance rather than a frozen compatibility
posture. The
[upgrade guide](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/UPGRADE_GUIDE_V2.md#L8-L43)
changes the module path, changes `View() string` to `View() tea.View`, replaces
imperative terminal commands with view fields, and changes key and mouse message
types. Bubbles v2.1.1, Crush at
[commit `9b36b44`](https://github.com/charmbracelet/crush/tree/9b36b44fa16122868d803106c500b726e4d7608b),
and gh-dash's
[v2 migration](https://github.com/dlvhdr/gh-dash/commit/076821f7f60cd74d942e75b39d47326c73d9daae)
provide ecosystem evidence.

## Core Proposition

Bubble Tea makes a terminal application look like a small state machine. The
model owns state and implements three methods:

```go
type Model interface {
    Init() Cmd
    Update(Msg) (Model, Cmd)
    View() View
}

type Cmd func() Msg
```

The v2 definitions are in
[`tea.go`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/tea.go#L48-L90).
`Msg` is an alias of `ultraviolet.Event`, so key, mouse, paste, focus,
clipboard, capability, terminal-version, and resize events share the same
message boundary. A command is an I/O operation that returns one message when it
completes. The application handles that message in `Update`, changes its state,
and may return another command.

This is more than a convenient callback convention. The model is the application
state boundary, and `Update` is the only place where the framework applies an
incoming message to that state. A child component can be another model value
with its own `Update` and `View`, while the parent decides which child receives
a message and where its rendered string appears. The application therefore gets
a clear unidirectional data flow without adopting an async runtime or a
prescribed domain architecture.

The principal v2 change is the `View` value. The
[migration guide's declarative-view section](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/UPGRADE_GUIDE_V2.md#L39-L103)
moves terminal feature selection from `NewProgram` options and imperative
commands into fields returned by `View()`. The content remains a styled string,
but the same value can declare alternate-screen ownership, mouse mode, focus
reporting, cursor placement and style, colors, window title, bracketed paste
policy, keyboard enhancements, and progress-bar state. The renderer compares the
latest view with its previous view and applies mode transitions as part of
rendering.

## Architecture

### Application And State Model

Bubble Tea retains the latest `Model` value inside `Program`, but it does not
retain a UI tree. The application retains domain state, child component state,
focus index, modal stack, scroll positions, stable IDs, and any asynchronous
task bookkeeping. `Update` can type-switch on a message, update one or more
child models, and combine their returned commands. The composable example
demonstrates this directly: the parent keeps a focus enum, updates either a
timer or spinner, and returns a `tea.Batch` of child commands
([`examples/composable-views/main.go`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/examples/composable-views/main.go#L62-L133)).

This model is simple and flexible, but it makes interaction architecture
application-owned. There is no framework focus manager, event capture/bubble
phase, retained node identity, keyed reconciliation, or automatic hit-test
registry. The clickable example instead builds Lip Gloss layers, assigns IDs,
and uses `View.OnMouse` to map coordinates back to those IDs
([`examples/clickable/main.go`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/examples/clickable/main.go#L210-L258)).

### Commands And Scheduling

`Cmd` is intentionally small: a function invoked by Bubble Tea that eventually
returns a `Msg`. `Batch` starts commands concurrently with no ordering
guarantee; `Sequence` runs them serially in order
([`commands.go`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/commands.go#L7-L54)).
`Tick` and `Every` are one-shot commands; applications return another command
after handling the tick if they want a repeating timer.

The command dispatcher receives returned commands through a channel and invokes
each non-nil command in its own goroutine. Bubble Tea catches command panics by
default and sends an error into the program. The source explicitly documents the
lifetime tradeoff: the dispatcher does not wait for a long-running command
because that would make shutdown slow, and commands cannot be cancelled by the
framework, so their goroutines remain until the command returns
([`tea.go`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/tea.go#L698-L738)).
An external context can kill the program, but it is not automatically passed
into arbitrary commands. `Exec` is a special case that blocks the event loop
while handing control to a child process.

This is effective for short requests and timers. It does not define
cancellation, effect identity, backpressure, deadlines, or deterministic
completion. A long command must implement its own context handling and
stale-result policy.

### Event Loop And Input

`Program.Run` establishes the runtime in a deliberate sequence. It selects
input, installs the signal handler, initializes raw terminal state, determines
the initial size, creates the renderer, sends color-profile, size, and
environment messages, starts the input reader and renderer, calls `Init`,
renders the initial view, starts resize handling, and enters the event loop
([`tea.go`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/tea.go#L988-L1144)).
The event loop translates Ultraviolet events, applies an optional filter,
handles framework messages, calls `Update`, schedules the returned command, and
renders the resulting model
([`tea.go`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/tea.go#L741-L883)).
Updates are serialized by this loop even though input and commands arrive from
separate goroutines.

The input translation boundary is broad.
[`input.go`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/input.go#L7-L54)
maps Ultraviolet key press and release, mouse click, motion, release and wheel,
paste, focus, clipboard, capability, keyboard-enhancement, mode-report,
terminal-version, and window-size events to Bubble Tea messages. The public
`Program.Send` method provides a message-injection path for external integration
and tests. `WithInput`, `WithOutput`, `WithEnvironment`, `WithWindowSize`,
`WithColorProfile`, `WithoutSignals`, and `WithContext` provide useful seams
without exposing the private renderer implementation.

Resize is platform-dependent. On Unix, `SIGWINCH` invokes `checkResize`, which
queries the output terminal and sends `WindowSizeMsg`. The Windows
implementation's `listenForResize` is explicitly a no-op because Windows has no
`SIGWINCH`
([Unix implementation](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/signals_unix.go#L12-L33),
[Windows implementation](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/signals_windows.go#L6-L10)).
A closed [issue #1601](https://github.com/charmbracelet/bubbletea/issues/1601)
reported missing Windows Terminal resize events in v2.0.0; this research did not
reproduce v2.0.8 or establish which lower-level paths, if any, now compensate.

### Rendering And Output

The default `cursedRenderer` uses an Ultraviolet `ScreenBuffer` and
`TerminalRenderer`. It converts the ANSI-styled `View.Content` into cells,
clears and redraws the logical buffer, applies mode and cursor changes, passes
the cell buffer to Ultraviolet, and flushes the resulting terminal output.
Bubble Tea owns the integration and view-state transitions; Ultraviolet owns the
lower-level cell and terminal rendering algorithms. The relevant source
initializes the cell buffer in
[`cursed_renderer.go`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/cursed_renderer.go#L18-L48),
builds frames and handles frame-area changes in
[the flush path](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/cursed_renderer.go#L256-L311),
and submits cells and cursor state through
[Ultraviolet's terminal renderer](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/cursed_renderer.go#L460-L510).

The renderer is frame-rate driven rather than purely invalidation driven.
`NewProgram` clamps the rate to 60 FPS by default and 120 FPS maximum, and the
ticker flushes even when the view is unchanged. An unchanged-view fast path
keeps the work low, but an idle application still wakes regularly. No idle
benchmark was run.

Bubble Tea supports inline and alternate-screen behavior through the same view
contract. Inline content derives its frame height from the content, while
alternate-screen content uses the terminal dimensions. A view can set cursor
position, colors, mouse mode, focus reporting, bracketed paste, keyboard
enhancements, and title. The renderer queries synchronized output and
Unicode-core terminal modes when it believes the environment supports them. If
synchronized output is available, updates can be wrapped in terminal
synchronization mode; otherwise cursor hiding is used as a best-effort flicker
reduction.

Unmanaged output is a separate path. `tea.Println` becomes a `printLineMessage`,
and `insertAbove` calculates line widths, moves the cursor, inserts lines,
writes directly to the underlying writer, and resets the renderer's tracked
cursor position
([`cursed_renderer.go`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/cursed_renderer.go#L706-L776)).
It is useful for inline logs and history, but it is not a retained scrollback
model or a transaction with the screen renderer. The closed
[issue #1666](https://github.com/charmbracelet/bubbletea/issues/1666) records a
request for a raw history-printing path because applications combining native
clears, large history pushes, and inline rendering can encounter cursor and
artifacting friction. The issue is evidence of a real integration boundary, not
proof that every inline application fails.

### Terminal Lifecycle And Extension Boundary

Bubble Tea puts input into raw mode when the input or output is a terminal,
restores terminal state on normal shutdown, and catches panics by attempting
cleanup before reporting the panic. `ReleaseTerminal` stops input and the
renderer and restores the prior state; `RestoreTerminal` reinitializes input and
the renderer, checks for a size change, and flushes queued commands
([`tea.go`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/tea.go#L1317-L1365)).
Unix suspension uses the same release/restore path. A signal-channel leak in
`suspendProcess` was fixed and merged by
[PR #1674](https://github.com/charmbracelet/bubbletea/pull/1674); the v2.0.8
source contains the corresponding `defer signal.Stop(c)`
([`tty_unix.go`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/tty_unix.go#L37-L47)).
This is a useful example of labeling a historical defect as fixed rather than
treating it as a current limitation.

The public extension boundary is intentionally narrower than the internal
architecture. Applications can supply input, output, environment, initial size,
color profile, message filters, contexts, and messages. Models and views are
public. The renderer interface, renderer state, screen buffer, and flush policy
are private. There is no public `WithRenderer` or backend contract that lets an
application replace the default output transaction policy while retaining the
Bubble Tea event loop. An application can wrap `io.Writer`, use `Raw`, use
`Exec`, or fork the framework, but it cannot cleanly substitute a renderer that
reports applied versus uncertain output through the public API.

## Core Strengths

### A Small, Coherent Application Loop

The `Init`/`Update`/`View` contract is easy to explain and scales from a counter
to a multi-pane application. Every external event becomes a message, every side
effect becomes a command result, and the central loop serializes model mutation.
The framework also exposes `Program.Send` for external event sources without
requiring the application to expose mutable state. This gives application
authors a clear boundary between event acquisition and state transition.

The value is architectural rather than merely ergonomic. A child model can be
tested and composed independently, while a parent owns routing and layout. The
cost is that teams must establish their own focus, identity, and component
conventions. For applications that already have a domain reducer or need to
integrate with an existing Go program, that flexibility is a significant
advantage over a framework that mandates its own runtime.

### Declarative Terminal State In V2

Moving terminal mode selection into `View` reduces scattered imperative state.
The application declares whether the current view wants alternate-screen mode,
mouse reporting, focus events, a cursor, a title, or keyboard enhancements. The
renderer compares the previous and current view and performs the transitions.
This is particularly valuable when an application intentionally switches between
inline and full-window contexts, because the desired terminal state is visible
beside the content that requires it.

The design also makes testing options more explicit. `WithWindowSize` and
`WithColorProfile` let tests control two environmental inputs that otherwise
produce unstable output. The migration is not free, but the resulting contract
is more coherent than mixing startup options, update-time mode commands, and
renderer state.

### Capable Terminal Integration

Bubble Tea handles much more than key presses. The v2 input boundary includes
paste phases, mouse motion and wheel events, focus changes, clipboard responses,
terminal capability reports, keyboard enhancement reports, and resize messages.
The renderer integrates color profiles, synchronized output where available,
grapheme-width negotiation, cursor style and color, alternate-screen
transitions, and raw terminal restoration. This is a substantial amount of
difficult terminal behavior behind a small application API.

The release stream includes fixes for wide-character rendering, keyboard
enhancements, tab-stop restoration, and suspension signal cleanup. The open
shutdown-garbage issue below shows both the remaining edge surface and active
maintenance.

### A Practical Component Ecosystem

Bubbles supplies production-shaped stateful components instead of only drawing
primitives. Its v2.1.1 list component provides filtering, pagination, selection,
help, and delegate customization. Text input and text area components include
focus state, cursor handling, character limits, validation, suggestions,
wrapping, scrolling, and dynamic sizing. Viewport, table, progress, spinner,
timer, file picker, paginator, and help components cover common application
needs. The component models follow the same `Update` and `View` convention, so
they compose naturally with the framework.

Crush uses a top-level model with explicit screen and dialog state, application
IDs and caches, custom overlays, and UI tests. gh-dash and Superfile provide
additional v2 migration evidence. These applications show that the framework
supports complex tools, but do not prove it minimizes total code for every
workload.

### Useful In-Memory Runtime Seams

The core tests run `Program` with byte buffers, fixed dimensions, explicit color
profiles, and deterministic command sequences. That exercises real
initialization, dispatch, rendering, and shutdown without a physical terminal.
These seams do not solve asynchronous testing, but they avoid requiring every
test to open a TTY.

## Limitations And Frustrations

### Complete-Application Determinism Stops At The Runtime Boundary

```text
Classification: Maturity problem and extension failure
Requirement: Drive a complete application deterministically, inject input and resize, control time and effects, wait for settlement, and inspect semantic view state
Library assumption: Applications can test Update and View directly or construct their own Program-level test setup
Observable failure or friction: No public simulator provides synchronous Init/Update/View execution, fake clocks, command completion, run-until-idle, semantic focus inspection, or stable application snapshots
Root architectural cause: The production runtime is goroutine- and ticker-driven, while Program exposes only message injection and byte-oriented output capture
Available workaround: Unit-test models, use Program with byte buffers and fixed size, or adopt/maintain a separate teatest-style harness
Cost of workaround: Tests duplicate runtime coordination, depend on sleeps or output polling for asynchronous behavior, and can become color- or terminal-size-sensitive
Upstream response: Issue #1654 proposes charm-test; it was open with no assigned implementation or linked pull request at access time
Current status and version: Not found in the public v2.0.8 application API or pinned repository; related example tests are commented out
Evidence: Verified API and test search; reported testing gap in issue #1654 and commented example tests
Confidence: High for the public API boundary, medium for ecosystem-wide absence
```

The repository has strong internal runtime tests:
[`tea_test.go`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/tea_test.go#L69-L335)
runs real `Program` instances with byte buffers, while
[`screen_test.go`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/screen_test.go#L61-L159)
compares fixed-size output against golden files. The user-facing gap is visible
in the simple example: its `teatest` tests are commented out because color
output and terminal sizing made them fail or flaky
([`examples/simple/main_test.go`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/examples/simple/main_test.go#L3-L88)).
The open
[charm-test proposal](https://github.com/charmbracelet/bubbletea/issues/1654)
asks for the synchronous simulator and mock-clock decisions ArborUI requires.

### Commands Have No Framework Cancellation Or Effect Lifetime

```text
Classification: Limitation relative to structured async effects
Requirement: Cancel work when a screen closes or the program exits, bound shutdown, and prevent stale results from surviving the owning task
Library assumption: Cmd functions are short enough to finish eventually, or command authors handle their own cancellation
Observable failure or friction: A long-running Cmd goroutine remains alive until the function returns; an external context kills the Program but is not automatically passed to the Cmd
Root architectural cause: Cmd is only func() Msg and the dispatcher deliberately launches it without a cancellation argument
Available workaround: Close over an application-owned context, use channels and select statements, tag results with screen/task identity, and make every effect observe cancellation
Cost of workaround: Cancellation, stale-result filtering, and ownership become application conventions; third-party components cannot assume a common task lifetime
Upstream response: The source documents the behavior as an intentional shutdown-latency tradeoff; no replacement effect contract was found in v2.0.8
Current status and version: Current behavior in v2.0.8
Evidence: Verified in the command dispatcher and Cmd definition
Confidence: High
```

The source explicitly documents that command goroutines are not waited on or
cancelled because doing so would increase shutdown latency
([`tea.go`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/tea.go#L716-L733)).
`Program.Send` drops results after shutdown, but does not reclaim command work
or its resources. This is acceptable for short effects and a meaningful boundary
for streams, subprocesses, subscriptions, and screen-scoped tasks.

### Physical Output Is Not A Transaction

```text
Classification: Limitation and extension failure
Requirement: Commit the logical frame only after complete output acceptance, distinguish partial from unknown writes, and force a full repaint after uncertainty
Library assumption: A terminal output error is exceptional and normally ends the session
Observable failure or friction: Ordinary writer errors do not report how many terminal operations were accepted; the output buffer is reset after Write, renderer flush errors are discarded by the ticker, and no public state invalidation follows an uncertain write
Root architectural cause: The renderer and writer use ordinary io.Writer error semantics, while the private renderer interface owns the screen baseline
Available workaround: Exit after an output error, wrap the writer for diagnostics, or fork/replace the renderer outside the public Program API
Cost of workaround: In-session recovery is unavailable without taking over lifecycle and rendering policy; a wrapper cannot infer terminal state from a partial write
Upstream response: No commit-after-acceptance or physical-state invalidation contract was found at v2.0.8
Current status and version: Current output path in v2.0.8
Evidence: Verified write and flush control flow; physical consequence inferred because no partial-write reproduction was run
Confidence: High for the control flow, medium for terminal-specific outcomes
```

`Program.flush` resets its output buffer after `Write`, and the renderer ticker
ignores both program and renderer flush errors
([`tea.go`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/tea.go#L1213-L1237),
[ticker](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/tea.go#L1392-L1419)).
A writer can accept a prefix before returning an error, but Bubble Tea does not
mark the screen unknown or force a full repaint. This is reasonable if output
failure ends the session, but it is below ArborUI's recovery requirement, and
the private renderer interface prevents a clean replacement through `Program`
options.

### Terminal Protocol Responses Can Escape A Short-Lived Program

```text
Classification: Bug
Requirement: Restore the terminal without leaving capability-response bytes for the shell or parent terminal reader
Library assumption: Asynchronous mode reports will be consumed before the input reader is cancelled and the TTY is restored
Observable failure or friction: A very short program can leave DECRPM responses such as synchronized-output or Unicode-core reports in the terminal input queue, producing garbage characters after exit
Root architectural cause: v2 queries terminal modes during startup, then restores the TTY without draining late input responses
Available workaround: Avoid ultra-short runs on affected terminals, disable the rendering path, or apply a downstream drain patch
Cost of workaround: Short-lived commands and spinners can corrupt the user's shell output; disabling the renderer removes the feature that triggered the query
Upstream response: Issue #1590 is open; PR #1692 proposes a platform-specific bounded input drain and was still open at access time
Current status and version: Reproduced and reported against v2; not fixed in the pinned v2.0.8 source
Evidence: Source confirms the capability query and restore path; issue #1590 and PR #1692 provide an external reproduction and proposed fix
Confidence: Medium
```

`Run` queries synchronized-output and Unicode-core support after starting input,
then the restore path returns the TTY without an input drain
([startup](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/tea.go#L1098-L1115),
[restore](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/tty.go#L31-L38)).
[Issue #1590](https://github.com/charmbracelet/bubbletea/issues/1590) supplies
the reproduction;
[PR #1692](https://github.com/charmbracelet/bubbletea/pull/1692) proposes
bounded platform-specific draining and remained open. The suspend signal leak is
different: its fix was merged and is present in v2.0.8.

### Inline And Native-Scrollback Ownership Is A Separate, Fragile Contract

```text
Classification: Tradeoff and extension failure for native-scrollback applications
Requirement: Preserve terminal history, append external output, clear or reflow history, and repaint the live region without cursor races or duplicated content
Library assumption: Inline view content and the insert-above mechanism can coordinate all output that changes the main screen
Observable failure or friction: Unmanaged insertion calculates widths and moves the cursor directly, while the renderer tracks a separate cell buffer; native clears, large history pushes, resize, and external output can require application-specific sequencing
Root architectural cause: Inline output shares a physical terminal with the live renderer but is not represented in the retained view or a transactional history model
Available workaround: Keep history in application state, use the supplied insert-above path conservatively, sequence Raw/ClearScreen operations, and avoid external writes during rendering
Cost of workaround: Applications must understand cursor protocol, terminal wrapping, width calculation, and renderer state; a true native-scrollback mode cannot be treated as fullscreen mode with one option changed
Upstream response: Issue #1666 records a raw-print request and is closed without a corresponding v2.0.8 API; resize reports include both fixed historical cases and unresolved platform questions
Current status and version: Inline mode is supported; robust native-scrollback ownership remains application-defined in v2.0.8
Evidence: Verified insert-above implementation and application examples; reported integration friction in issue #1666
Confidence: High for the extension boundary, medium for the severity of individual terminal artifacts
```

Bubble Tea deserves credit for supporting inline applications, but inline mode
has different ownership semantics. `insertAbove` writes cursor movement and
content directly to the writer while ordinary `View` rendering uses the cell
buffer. Native clears or external output can invalidate assumptions invisible to
the logical view.
[Issue #1666](https://github.com/charmbracelet/bubbletea/issues/1666) records
this friction; the current API has `Raw`, `ClearScreen`, and `Println`, but no
native-scrollback transaction or durable history contract.

ArborUI's current alternate-screen baseline should not count this as a defect
against ordinary fullscreen applications. It is a warning about future modes:
inline regions and native scrollback should be explicit contracts with cursor
ownership, immutable history, external-output policy, resize recovery, and
physical invalidation rather than a boolean added to the fullscreen renderer.

## Testing Strategy

Bubble Tea's repository testing model is strongest at the framework and renderer
seams, and weaker at the public complete-application seam.

### Production Runtime Unit And Integration Tests

The core tests instantiate real `Program` values with `bytes.Buffer` input and
output. They drive initial rendering, injected key input, `Quit`, external
`Quit`, `Kill`, `Wait`, context cancellation, message filters, batch and
sequence commands, command panics, program panics, and `Program.Send`. This is
not a parallel fake update function: it exercises the production event loop and
shutdown path.
[`TestTeaModel`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/tea_test.go#L69-L90)
is the minimal example; the surrounding tests cover the lifecycle cases.

The repository's `Taskfile.yaml` runs `go test -race -count 4 -cpu 1,4 ./...`,
which is a useful race and scheduling stress policy
([pinned task](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/Taskfile.yaml#L11-L15)).
Repeating tests on one and four CPUs is appropriate for a goroutine-driven
framework. It does not make wall-clock timers deterministic or test physical
terminal behavior.

### Golden Terminal Output Tests

`screen_test.go` fixes the initial window to 80 by 24, fixes the color profile
to ANSI256, fixes `TERM`, injects a sequence of view options, captures output in
a buffer, and compares bytes with `golden.RequireEqual`. Cases cover
alternate-screen entry and exit, mouse modes, cursor visibility, bracketed
paste, keyboard release reporting, colors, clear screen, clipboard requests, and
terminal color requests. These tests exercise real renderer output and
terminal-mode transitions without a physical terminal. The repository stores the
expected `.golden` files under `testdata`.

The boundary is deliberately byte-oriented. A golden file can catch a changed
escape sequence, ordering regression, or missing cleanup sequence, but it does
not apply the output to a terminal emulator and inspect the resulting cells,
scrollback, cursor, or alternate-screen state. It also does not model a writer
that accepts only a prefix, returns an error after terminal effects, or delays a
capability response. The renderer tests therefore complement, but do not
replace, PTY or virtual-terminal tests.

### User-Facing Test Controls And Gaps

The public API has good environmental controls: `WithInput`, `WithOutput`,
`WithEnvironment`, `WithWindowSize`, `WithColorProfile`, `WithoutSignals`,
`WithContext`, and `Program.Send`. These allow applications to run production
`Program` instances in memory and to inject messages. They do not control the
renderer ticker, `time.Now`, `Tick` timers, command goroutines, command
completion, or a run-until-idle condition. A test that needs to wait for an
asynchronous command commonly needs timeouts, sleeps, output polling, or an
application-specific synchronization message.

The commented example tests are important negative evidence. They reference
`teatest`, test a complete application, inject text and key messages, inspect
final output and final model, and wait for output changes. They are disabled
because color-profile behavior differs for buffer output, and another test is
marked flaky because initial terminal size is not concrete enough. This means
the repository recognizes the desired test shape but does not make it a stable,
maintained public contract at v2.0.8.

### Failure Injection, PTY, And Performance

The suite exercises panic and cancellation paths, but no complete fault matrix
for partial writes, delayed terminal responses, backpressure, physical recovery,
or PTY behavior was found at the recorded revision. It also has no end-to-end
benchmark contract for complete turns, idle wakeups, emitted bytes, queue
latency, or output recovery. These are searched-revision observations, not
claims that applications cannot add such tests.

| Capability                                 | Bubble Tea v2.0.8                                             | Evidence status       |
| ------------------------------------------ | ------------------------------------------------------------- | --------------------- |
| Test production `Program` without a TTY    | Supported with in-memory input/output                         | Verified              |
| Fixed initial dimensions and color profile | Supported                                                     | Verified              |
| Inject application messages                | `Program.Send`                                                | Verified              |
| Golden terminal escape output              | Repository-owned `.golden` tests                              | Verified              |
| Semantic terminal-emulator snapshots       | Not found in the pinned repository                            | Inferred from search  |
| Control clocks and timers                  | Application-defined                                           | Verified API boundary |
| Wait for command settlement                | Not provided                                                  | Verified API boundary |
| Inject output failures and recover         | Custom writer can inject errors; recovery policy not provided | Supported/inferred    |
| PTY lifecycle coverage                     | Not found in the pinned repository                            | Inferred from search  |
| Race and scheduling stress                 | `-race -count 4 -cpu 1,4` task                                | Verified              |

## Common Scenario Assessment

| Scenario                                         | Assessment                                                                                                                                                                                                               |
| ------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Form with focus, validation, and modal           | Bubbles text input and text area provide editing and validation primitives. Focus traversal, modal policy, and event routing are application-owned.                                                                      |
| Large scrollable collection with stable identity | Bubbles list provides filtering, pagination, delegates, and index/global-index helpers. Its item contract has `FilterValue()` but no stable key or keyed reconciliation; applications must preserve identity themselves. |
| Streaming external updates                       | `Program.Send` and commands support external messages. Cancellation, coalescing, ordering across concurrent commands, and backpressure are application-owned.                                                            |
| Unicode-heavy text input and editing             | Ultraviolet and Bubbles handle substantial width, grapheme, cursor, wrapping, and editing behavior. Physical terminal and font differences remain outside logical tests.                                                 |
| Overlay with clipping and mouse interaction      | Lip Gloss layers plus `View.OnMouse` can implement overlays and hit testing. IDs, clipping policy, and event targeting are application-owned.                                                                            |
| Resize during active updates                     | Initial size and Unix `SIGWINCH` are supported. Windows resize behavior is platform-dependent and was not established for v2.0.8.                                                                                        |
| Deferred, partial, or failed output              | Ordinary writer errors exist, but applied/unknown outcomes, logical commit-after-acceptance, and forced full repaint are not part of the public contract.                                                                |
| Suspension to a child process and resume         | Release/restore lifecycle is supported on Unix; the historical `SIGCONT` channel leak was fixed before v2.0.8. Physical output and child-process coordination still deserve PTY tests.                                   |
| Long idle periods                                | The unchanged-view path is cheap, but the renderer ticker still wakes at the configured frame rate. No idle CPU benchmark was run.                                                                                       |
| Conversation preserving native scrollback        | Inline output and insert-above primitives exist. Durable history, native clear coordination, external output, and physical recovery remain application-defined.                                                          |

The Bubbles list supports filtering and `GlobalIndex`, but its
[`Item`](https://github.com/charmbracelet/bubbles/blob/d2b2217d6352ce04183623d66d4266115419733c/list/list.go#L33-L60)
contract and mutation APIs
([`list.go`](https://github.com/charmbracelet/bubbles/blob/d2b2217d6352ce04183623d66d4266115419733c/list/list.go#L374-L450))
remain filter- and slice-index based. It is a reasonable widget contract, not
retained UI identity.

## Lessons For ArborUI

### Adopt Or Preserve

- Keep a small application facade with an explicit `Init`/update/view or
  equivalent reducer boundary. Bubble Tea demonstrates that this model is
  understandable and useful for both small and complex programs.
- Keep terminal feature state declarative with the view or prepared frame that
  needs it. v2's movement of alternate-screen, mouse, cursor, paste, and
  keyboard settings into `View` is a strong state-ownership decision.
- Use one input owner. Protocol replies for capability, cursor, clipboard, and
  keyboard negotiation must be routed separately from application events before
  they can be interpreted as user input.
- Make in-memory tests exercise the production update and rendering path. Fixed
  size, fixed color profile, injected input, and a semantic frame representation
  should be ordinary APIs rather than private test tricks.
- Provide common widgets, examples, and integration recipes as part of the
  architecture. Crush and the Bubbles ecosystem show that application authors
  evaluate the whole path to a working tool, not only the runtime contract.
- Treat alternate-screen, inline, fixed-region, and native-scrollback behavior
  as distinct modes. Ratatui's explicit viewport modes and Bubble Tea's
  different inline paths both support this conclusion.

### Avoid Or Change

- Do not make `Cmd`-like effects unconditionally uncancellable. Retain effect
  ownership, cancellation, deadlines, and stale-result identity in the runtime
  or a public effect contract.
- Do not treat a generic `io.Writer` error as enough information to commit a
  physical frame. ArborUI should preserve prepared-frame commit semantics and
  invalidate physical state after any partial or uncertain write.
- Do not hide the renderer extension point while promising recoverable output.
  Either expose the required backend contract or make the recovery guarantee an
  internal invariant that applications can rely on without replacing the
  renderer.
- Do not mix unmanaged inline output with a live renderer without an explicit
  ownership and synchronization contract.
- Do not infer stable identity from list position. Dynamic collections, focus,
  mouse targets, and retained state need explicit keys.

### Claims ArborUI Has Not Yet Proven

Bubble Tea makes the case for stronger guarantees only if ArborUI can show a
lower total cost for real applications. ArborUI has not yet demonstrated that
retained elements, explicit invalidation, prepared-frame transactions, and
grapheme-level invariants improve application author productivity enough to
offset a smaller widget ecosystem and more implementation machinery. It has also
not shown that a full layout and paint pass remains within practical latency for
large collections, or that its runtime can outperform a simpler event loop in
equivalent workloads. These should remain benchmark and prototype questions, not
assumptions.

The harness must drive production code, control asynchronous completion, expose
semantic focus and hit-target assertions, and include PTY or emulator tests for
behavior a logical buffer cannot represent. A polished widget catalog and
application examples may matter more to adopters than an untested
physical-output contract.

### Follow-Up Experiments

1. Implement the same moderate form-and-list application in idiomatic Bubble
   Tea/Bubbles and ArborUI. Compare application-owned focus, stable identity,
   async cancellation, test code, and terminal lifecycle code rather than
   comparing framework source size.
2. Build a virtual terminal test for Bubble Tea's golden output and ArborUI's
   prepared-frame path. Inject resize storms, Unicode boundary cases, overlays,
   and partial writer failures.
3. Add a deterministic Bubble Tea harness prototype with a fake clock and
   command scheduler. Measure which missing APIs are necessary for real Bubbles
   components, not just toy models.
4. Benchmark idle wakeups, one-cell updates, large-list updates, overlays,
   streaming messages, and resize bursts under equal terminal sizes and
   workloads.
5. Prototype ArborUI inline and native-scrollback modes separately from the
   current alternate-screen runtime. Specify cursor ownership, external output,
   history, and recovery before adding a shared abstraction.

## Evidence Appendix

All sources below were accessed on 2026-07-16 unless otherwise stated.

| Claim                                   | Source                                                                                                                                                                                                                                                                              | Version or revision                               | Source date               | Accessed   | Status             | Notes                                                                         |
| --------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------- | ------------------------- | ---------- | ------------------ | ----------------------------------------------------------------------------- |
| Stable release baseline                 | [`v2.0.8` release](https://github.com/charmbracelet/bubbletea/releases/tag/v2.0.8)                                                                                                                                                                                                  | `v2.0.8`, source `fc707bb`                        | 2026-07-03                | 2026-07-16 | Verified           | Selected latest stable baseline at research start                             |
| Module and Go version                   | [`go.mod`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/go.mod#L1-L16)                                                                                                                                                                  | `fc707bb`                                         | 2026-07-02                | 2026-07-16 | Verified           | `charm.land/bubbletea/v2`, Go 1.25.0                                          |
| Elm model contract                      | [`tea.go` Model and View](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/tea.go#L48-L90)                                                                                                                                                  | `fc707bb`                                         | 2026-07-02                | 2026-07-16 | Verified           | `Msg` is an Ultraviolet event alias; v2 View is a struct                      |
| Batch and sequence semantics            | [`commands.go`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/commands.go#L7-L54)                                                                                                                                                        | `fc707bb`                                         | 2026-07-02                | 2026-07-16 | Verified           | Concurrent unordered batch and ordered sequence                               |
| Declarative v2 migration                | [`UPGRADE_GUIDE_V2.md`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/UPGRADE_GUIDE_V2.md#L39-L103)                                                                                                                                      | `fc707bb`                                         | 2026-07-02                | 2026-07-16 | Supported          | Version-matched migration documentation                                       |
| Program initialization and event loop   | [`tea.go` Run and eventLoop](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/tea.go#L741-L883)                                                                                                                                             | `fc707bb`                                         | 2026-07-02                | 2026-07-16 | Verified           | Input translation, filters, Update, commands, render                          |
| Command goroutine lifetime              | [`tea.go` handleCommands](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/tea.go#L698-L738)                                                                                                                                                | `fc707bb`                                         | 2026-07-02                | 2026-07-16 | Verified           | Source explicitly states commands are not cancellable                         |
| Renderer cell-buffer integration        | [`cursed_renderer.go`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/cursed_renderer.go#L18-L48) and [flush path](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/cursed_renderer.go#L256-L311) | `fc707bb`                                         | 2026-07-02                | 2026-07-16 | Verified           | Uses Ultraviolet ScreenBuffer and TerminalRenderer                            |
| Inline unmanaged output                 | [`cursed_renderer.go` insertAbove](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/cursed_renderer.go#L706-L776)                                                                                                                           | `fc707bb`                                         | 2026-07-02                | 2026-07-16 | Verified           | Direct cursor manipulation and writer output                                  |
| Suspend signal leak fixed               | [`tty_unix.go`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/tty_unix.go#L37-L47) and [PR #1674](https://github.com/charmbracelet/bubbletea/pull/1674)                                                                                  | v2.0.8; merged 2026-04-13                         | 2026-04-13                | 2026-07-16 | Verified           | Historical issue #1673 closed by merged PR                                    |
| Short-lived shutdown garbage            | [Issue #1590](https://github.com/charmbracelet/bubbletea/issues/1590) and [PR #1692](https://github.com/charmbracelet/bubbletea/pull/1692)                                                                                                                                          | Issue and proposed fix; v2.0.8 source lacks drain | 2026-02-18 and 2026-05-06 | 2026-07-16 | Reported/supported | Issue and PR remained open at access time; source confirms query/restore path |
| Windows resize boundary                 | [`signals_windows.go`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/signals_windows.go#L6-L10) and [issue #1601](https://github.com/charmbracelet/bubbletea/issues/1601)                                                                | v2.0.8 source; issue reported v2.0.0              | 2026-02-28                | 2026-07-16 | Verified/reported  | No physical reproduction; issue is closed                                     |
| Complete application test proposal      | [Issue #1654](https://github.com/charmbracelet/bubbletea/issues/1654)                                                                                                                                                                                                               | Open                                              | 2026-04-01                | 2026-07-16 | Reported           | No assignee or linked implementation                                          |
| Runtime tests                           | [`tea_test.go`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/tea_test.go#L69-L335)                                                                                                                                                      | `fc707bb`                                         | 2026-07-02                | 2026-07-16 | Verified           | Real Program with byte buffers and lifecycle cases                            |
| Golden screen tests                     | [`screen_test.go`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/screen_test.go#L61-L207) and [testdata](https://github.com/charmbracelet/bubbletea/tree/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/testdata)                              | `fc707bb`                                         | 2026-07-02                | 2026-07-16 | Verified           | Fixed size, profile, environment, and byte golden output                      |
| Test command                            | [`Taskfile.yaml`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/Taskfile.yaml#L11-L15)                                                                                                                                                   | `fc707bb`                                         | 2026-07-02                | 2026-07-16 | Verified           | Race and repeated CPU scheduling test                                         |
| Disabled application tests              | [`examples/simple/main_test.go`](https://github.com/charmbracelet/bubbletea/blob/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/examples/simple/main_test.go#L3-L88)                                                                                                                      | `fc707bb`                                         | 2026-07-02                | 2026-07-16 | Verified           | Commented teatest tests cite color and size flakiness                         |
| Bubbles component and identity contract | [`list.go` Item and delegate](https://github.com/charmbracelet/bubbles/blob/d2b2217d6352ce04183623d66d4266115419733c/list/list.go#L33-L60) and [list mutation APIs](https://github.com/charmbracelet/bubbles/blob/d2b2217d6352ce04183623d66d4266115419733c/list/list.go#L374-L450)  | Bubbles v2.1.1, `d2b2217`                         | 2026-07-04                | 2026-07-16 | Verified           | Filter value and slice indices, not keyed reconciliation                      |
