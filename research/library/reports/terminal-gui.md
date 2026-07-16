# Terminal.Gui Research Report

## Evidence Header

```text
Research date: 2026-07-16
Project version: Terminal.Gui v2.4.17
Project revision: d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6
Repository: https://github.com/tui-cs/Terminal.Gui
Documentation version: v2 DocFX sources at d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6; rendered docs accessed 2026-07-16
Primary platform examined: Source inspection on Linux; no physical terminal reproduction
Report depth: Standard profile
```

## Executive Assessment

Terminal.Gui is a full .NET terminal application framework, not merely a cell renderer or widget catalog. Its v2 design gives each application an `IApplication` context, a retained `View` hierarchy,
nested modal sessions, a single UI loop, and deferred driver output. The baseline is
[v2.4.17](https://github.com/tui-cs/Terminal.Gui/releases/tag/v2.4.17), released 2026-07-07, targeting .NET 10.

The framework fits conventional forms, dialogs, administrative tools, file browsers, dashboards, and full-screen or inline CLI interfaces. Its main strengths are retained interaction state, mature layout
and widget composition, explicit driver boundaries, and a capable headless application harness. V2 also removes much of the global-state coupling of older usage.

The main ArborUI-relevant weakness is terminal output recovery. Terminal.Gui retries short Unix writes, but the public path does not report an applied, deferred, or unknown frame outcome.
Dirty cells are
cleared during
emission and the ANSI implementation ignores a failed native write. A partial write therefore has no transaction or mandatory full-repaint contract. Other gaps are intentional: retained views require
application-managed identity and disposal, while background work uses `Invoke` and timers.

## Project Snapshot

Terminal.Gui is a C# `net10.0` application framework with nullable references, trimming, and Native AOT compatibility enabled. It targets Windows, macOS, and Linux/Unix, and includes views for forms,
menus, tables, trees, text editing, charts, dialogs, and file management. Windows, .NET `System.Console`, and ANSI drivers are registered publicly. The repository includes a UICatalog, DocFX documentation,
integration and stress tests, benchmarks, and Native AOT smoke coverage. Adoption was not measured.

The project is primarily an **application framework** in the research taxonomy, with an integrated renderer, layout engine, terminal lifecycle, and widget library. Claims refer to the tagged release unless
marked as documentation intent or historical v1 behavior.

## Core Proposition

Terminal.Gui makes a stateful terminal application look like a conventional GUI program. An application creates an `IApplication`, initializes a driver, adds long-lived `View` objects to a
`SuperView`, and runs an
`IRunnable`. The framework owns input translation, focus, layout, invalidation, cursor updates, drawing, and output. A dialog can be a runnable session with a typed result. Unlike a rendering library
such as
Ratatui, applications do not assemble their own focus policy, modal stack, or redraw loop.

The v2 API is instance-based by default:
[`Application.Create()` and `IApplication`](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/docfx/docs/application.md#L1-L15)
replace singleton-oriented usage; the obsolete static gateway remains for migration. Views persist, mark themselves or descendants dirty, and draw into a driver buffer; the driver writes changed cells
on the next iteration.

## Architecture

### Application And State

`IApplication` owns initialization, the driver, main-thread identity, timers, the session stack, navigation, and disposal. `Begin` pushes an `IRunnable` onto a `ConcurrentStack`; the top runnable becomes
modal
while lower sessions remain running and can draw. `Run` wraps begin, iteration, stop, and end. `RunAsync` observes cancellation but drives the same UI loop. Framework-created runnables are disposed automatically;
caller-created runnables remain caller-owned.

Nested forms and modals have a clear contract: adding a view transfers ownership to the parent, and disposing it recursively disposes subviews. See the
[v2 application documentation](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/docfx/docs/application.md#L211-L240).

### Retained Hierarchy And Layout

`View` is the base for visible elements, input, layout, and drawing. Each view has a `SuperView`, ordered `SubViews`, an application context, focus properties, adornments, and disposal hooks.
Layout is
declarative through `Pos` and `Dim`; the resolved `Frame` is relative to parent content. `SetNeedsLayout` and `SetNeedsDraw` propagate work to the next iteration. Margin, border, and padding adornments
participate without being ordinary application content.

Drawing is retained and region-aware rather than a full-tree repaint on every call. A view clears dirty content, draws subviews in z-order, draws text and custom content, resolves line canvases, and updates
clip
regions. Child-only invalidation can avoid repainting the parent's content. The [drawing documentation](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/docfx/docs/drawing.md#L56-L83)
documents this lifecycle and invalidation API.

### Events, Focus, And Scheduling

Keyboard and mouse input is translated by a driver-specific processor. Views expose commands, cancellable work-pattern events, mouse handling, focus eligibility, tab behavior, popovers, and navigation
hooks.
`ApplicationNavigation` tracks the focused view, advances focus through the hierarchy, and updates the cursor after drawing. This supports forms, menus, overlays, and mouse-targeted controls.

The main loop drains a raw input queue, runs ANSI response scheduling, polls size, performs layout and drawing, updates the cursor, then runs timers. Background code must use `app.Invoke()`; it
executes immediately
on the UI thread or schedules a zero-duration callback. The
[multitasking guide](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/docfx/docs/multitasking.md#L5-L13) warns
that direct background view mutation is unsupported.

### Rendering, Drivers, And Terminal Modes

The output buffer stores cells, attributes, graphemes, dirty state, and raster commands. `OutputBase.Write` walks dirty rows and cells, batches ANSI sequences, handles wide graphemes and raster output,
and clears dirty
flags as it emits. ANSI can run in degraded buffer-only mode without a terminal, which is useful for CI. The [driver registry](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/Terminal.Gui/Drivers/DriverRegistry.cs#L7-L59)
and factories permit injected components.

Full-screen ANSI mode activates the alternate screen, while inline mode stays in primary scrollback and renders a measured region at the cursor row. Startup queries and size monitors coordinate cursor
position, size,
and first rendering. They are separate ownership contracts. Resize, suspend, cursor handling, and capabilities remain driver responsibilities.

## Core Strengths

### A Complete Interaction Model

Terminal.Gui supplies modal session ownership, focus traversal, commands, mouse targeting, cursor updates, timers, cancellation-aware runs, and lifecycle
disposal. A form with validation and a modal dialog can
use
ordinary `View` composition.

### Composition And Visual Breadth

The retained tree, `Pos`/`Dim`, clipping, scrolling, popovers, and adornments support complex screens without application-defined geometry conventions. Built-in views include controls, tables, trees,
editors,
menus, charts, dialogs, and file views. This breadth matters because architectural guarantees do not replace common controls.

### Explicit Integration Boundaries

The three drivers and `IComponentFactory` split platform input, output, size detection, and ANSI parsing from application and view layers. `Application.Create()` removes most test dependence on a process-wide
singleton. Custom drivers and outputs remain possible without replacing the retained model.

### Application-Level Testability

`AppTestHelper` runs the real application with injectable ANSI input, output, size monitoring, fixed size, virtual time, and `WaitIteration`. It injects keys, mouse events, resize, and UI-thread actions.
Its ANSI snapshot
helper records the driver's escape stream rather than a parallel mock representation.

## Limitations And Frustrations

### Output Has No Transactional Recovery Contract

```text
Classification: Limitation relative to ArborUI's recovery requirement
Requirement: Commit a prepared frame only after complete backend acceptance and repaint after uncertain output
Library assumption: Output failures can be handled as ordinary write failures or by ending the session
Observable failure or friction: Dirty cells are cleared during emission; failed native writes are not surfaced as a frame outcome
Root architectural cause: IOutput.Write returns void and the output buffer has no applied/deferred/unknown transaction state
Available workaround: Terminate and reinitialize, force a full redraw through application-specific invalidation, or replace the output implementation
Cost of workaround: Recovery policy is outside the framework and cannot reliably reconstruct physical state after an uncertain write
Upstream response: No transaction or physical-state invalidation API was found at the tagged revision
Current status and version: Verified in v2.4.17
Evidence: OutputBase.Write; AnsiOutput.Write; UnixIOHelper.TryWriteAll and its unit tests
Confidence: High
```

`UnixIOHelper.TryWriteAll` retries short writes and returns `false` for zero or failed progress. However,
[`OutputBase.Write`](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/Terminal.Gui/Drivers/Output/OutputBase.cs#L110-L155)
marks cells clean during emission.
[`AnsiOutput`](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/Terminal.Gui/Drivers/AnsiDriver/AnsiOutput.cs#L212-L241)
ignores the Boolean result and catches native output exceptions.
Tests cover helper retries, not frame transactions or physical resynchronization. Abandoning the session may be acceptable, but it does not satisfy ArborUI's recoverable full-screen contract.

### Retained Views Shift Identity And Ownership Into Applications

```text
Classification: Tradeoff
Requirement: Reconcile changing model data without stale UI state, accidental retained references, or disposal bookkeeping
Library assumption: Views are durable objects whose parent owns their child lifetime
Observable failure or friction: Dynamic collections require application-managed view creation, reuse, ordering, removal, and disposal
Root architectural cause: The framework exposes a retained object hierarchy, not a keyed declarative reconciliation contract
Available workaround: Build a keyed adapter/reconciliation layer or use specialized ListView/TableView/TreeView state models
Cost of workaround: Extra identity, lifecycle, and model-to-view synchronization code
Upstream response: Retained views and ownership are documented design choices, not an open defect
Current status and version: Verified hierarchy; no general keyed reconciliation API found in v2.4.17
Evidence: View hierarchy, View.Dispose, application documentation
Confidence: Medium
```

Retained mode is not intrinsically wrong; this is a cost relative to ArborUI's ephemeral `Element` model. `Add` transfers child ownership and `Dispose` recursively releases subviews.
That suits dialogs and
durable
controls, but changing collections must synchronize framework objects with model identity. Specialized widgets may solve this locally; no framework-wide keyed reconciliation contract was found.
ArborUI should retain
its borrowed-element boundary only if prototypes show meaningful application cost.

### Async Work Is An Integration Boundary, Not A Framework Effect System

```text
Classification: Tradeoff
Requirement: Serialize external events and asynchronous effects with cancellation, backpressure, and deterministic settlement
Library assumption: Applications own background tasks and marshal every UI mutation to the main thread
Observable failure or friction: Producers must call Invoke, manage timer/task lifetimes, coalesce updates, and stop work when views end
Root architectural cause: The main loop owns UI mutation, while effects remain ordinary .NET tasks, channels, and callbacks
Available workaround: Use channels or tasks, cancellation tokens, app.Invoke, and explicit disposal handlers
Cost of workaround: Application-specific scheduling and lifecycle policy; no built-in run-until-idle or backpressure contract
Upstream response: Documented and intentional single-threaded UI model
Current status and version: Supported in v2.4.17
Evidence: ApplicationMainLoop, IApplication.Invoke, multitasking documentation, RunAsync tests
Confidence: High
```

The design becomes infrastructure for streaming output, high-rate events, or effects outliving views. `RunAsync` cancels sessions but does not make arbitrary work framework-owned. ArborUI should
expose an effect boundary only if settlement and backpressure are testable. Inline mode leaves external output, cursor ownership, and native scrollback as
mode-specific concerns.

## Testing Strategy

Terminal.Gui has strong logical and application-harness testing. `AppTestHelper` constructs `ApplicationImpl` with injectable ANSI input, output, and size monitoring, runs the actual loop, and uses a
`VirtualTimeProvider`. `WaitIteration` schedules actions through `IApplication.Invoke`, waits for the iteration, and propagates failures. Helpers inject keys, mouse events, resize, and live-driver assertions.

The snapshot path is stronger than a character dump:
[`AssertAnsiSnapshot`](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/Tests/AppTestHelpers/AppTestHelper.Snapshot.cs#L6-L92)
captures normalized, byte-exact ANSI at an explicit size and stops the app on mismatch. Integration tests exercise the real harness. Unit tests cover layout, clipping, focus, input, timers, startup gates,
inline sizing, Unicode, and drivers. `UnixIOHelperTests` inject short-write behavior.

The blind spot is the physical terminal transaction boundary. The test tree contains no general PTY or terminal-emulator suite, and no application test demonstrates partial `IOutput.Write` followed by
a guaranteed full repaint. The harness is excellent for deterministic headless behavior, but captured output does not prove raw-mode restoration, emulator cursor behavior, final-column wrapping, suspend/resume,
or broken-pipe recovery.
Snapshots still need semantic assertions and physical lifecycle tests.

## Common Scenario Assessment

| Scenario | Terminal.Gui assessment |
| --- | --- |
| Form with focus, validation, and modal | Strong: retained focus/navigation, commands, runnable sessions, and dialogs fit directly; validation remains application logic. |
| Large scrollable collection with stable identity | Partial: ListView, TableView, and TreeView exist, but stable model identity and reconciliation are widget/application concerns. |
| Streaming external updates | Supported with `Invoke`; coalescing, cancellation, and backpressure are application-owned. |
| Unicode-heavy input and editing | Strong logical support for runes, graphemes, and wide cells; physical terminal/font differences remain unproven here. |
| Overlay with clipping and mouse | Strong: popovers, adornments, clipping, focus, and mouse APIs are first-class. |
| Resize during active updates | Supported through size monitors and explicit inline/full-screen tests. |
| Deferred or failed output | Partial: short writes retry, but uncertain physical state has no transaction/repaint contract. |
| Suspend to a child process | Driver API exists, but orchestration and platform behavior were not reproduced. |
| Long idle periods | Application can block between iterations; timer and update policy remains application-dependent. |
| Native scrollback conversation | Inline mode is supported, but surrounding output and cursor ownership require an explicit integration contract. |

## Lessons For ArborUI

**Adopt:** Keep a complete application harness separate from backend tests. Make fixed size, key/mouse injection, virtual time, iteration settlement, and ANSI or character snapshots easy. Treat
alternate-screen and
inline/native-scrollback as distinct contracts. Preserve a narrow driver boundary and a small widget author API. Focus, modal sessions, cursor ownership, and disposal are valuable policy.

**Avoid:** Do not clear dirty state before backend acceptance. Do not hide write failures behind a `void` output method. Do not assume a headless buffer test validates lifecycle, cursor queries, Unicode
final-column
behavior, or suspend/resume. Measure whether retained identity and invalidation reduce application code before adding their complexity.

**Already different:** ArborUI's prepared-frame commit and physical-state invalidation are stronger than the observed output contract. Its borrowed elements avoid retaining application references, while
Terminal.Gui retains
view objects and ownership. Evaluate these as costs and guarantees, not automatic superiority.

**Still unproven:** ArborUI has not demonstrated better end-to-end performance, real-terminal Unicode compatibility, or that transaction recovery justifies its complexity. Compare the same form, collection,
streaming
view, resize storm, and failed-output fixture in both frameworks, including application code and tests.

## Evidence Appendix

All sources were accessed on 2026-07-16. Source links are pinned to `d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6` unless noted.

| Claim | Source | Version or revision | Status | Notes |
| --- | --- | --- | --- | --- |
| Stable release baseline | [v2.4.17 release](https://github.com/tui-cs/Terminal.Gui/releases/tag/v2.4.17) | v2.4.17, 2026-07-07 | Verified | Latest stable release used for this report |
| .NET target and SDK | [`Terminal.Gui.csproj`](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/Terminal.Gui/Terminal.Gui.csproj#L17-L35) and [`global.json`](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/global.json) | `d0a0ed9` | Verified | `net10.0`, SDK `10.0.100` |
| Instance application and session model | [Application architecture](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/docfx/docs/application.md#L1-L15) and [`IApplication`](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/Terminal.Gui/App/IApplication.cs#L105-L168) | `d0a0ed9` | Verified | Current v2 behavior |
| Retained hierarchy and disposal | [`View.Hierarchy`](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/Terminal.Gui/ViewBase/View.Hierarchy.cs#L5-L22) and [`View.Dispose`](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/Terminal.Gui/ViewBase/View.cs#L56-L141) | `d0a0ed9` | Verified | Parent owns and disposes subviews |
| Layout and invalidation | [`View.Layout`](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/Terminal.Gui/ViewBase/View.Layout.cs#L18-L49) and [drawing docs](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/docfx/docs/drawing.md#L19-L32) | `d0a0ed9` | Verified | Deferred, dirty-region rendering |
| Main loop and async boundary | [`ApplicationMainLoop`](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/Terminal.Gui/App/MainLoop/ApplicationMainLoop.cs#L32-L194), [`Invoke`](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/Terminal.Gui/App/ApplicationImpl.Run.cs#L44-L105), and [multitasking docs](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/docfx/docs/multitasking.md#L7-L13) | `d0a0ed9` | Verified | Single UI thread, queue, timers, and Invoke |
| Drivers and modes | [`DriverRegistry`](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/Terminal.Gui/Drivers/DriverRegistry.cs#L7-L59) and [`AnsiOutput`](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/Terminal.Gui/Drivers/AnsiDriver/AnsiOutput.cs#L61-L166) | `d0a0ed9` | Verified | ANSI supports degraded, fullscreen, and inline paths |
| Output recovery boundary | [`OutputBase.Write`](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/Terminal.Gui/Drivers/Output/OutputBase.cs#L110-L155), [`AnsiOutput.Write`](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/Terminal.Gui/Drivers/AnsiDriver/AnsiOutput.cs#L212-L241), and [`UnixIOHelper.TryWriteAll`](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/Terminal.Gui/Drivers/UnixHelpers/UnixIOHelper.cs#L291-L335) | `d0a0ed9` | Verified | Retry exists; transaction and resync do not |
| Short-write tests | [`UnixIOHelperTests`](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/Tests/UnitTestsParallelizable/Drivers/UnixIOHelperTests.cs#L18-L57) | `d0a0ed9` | Verified | Tests helper behavior, not full application recovery |
| Application harness and snapshots | [`AppTestHelper`](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/Tests/AppTestHelpers/AppTestHelper.cs#L12-L45), [input injection](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/Tests/AppTestHelpers/AppTestHelper.Input.cs#L137-L165), and [ANSI snapshots](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/Tests/AppTestHelpers/AppTestHelper.Snapshot.cs#L6-L92) | `d0a0ed9` | Verified | Real app loop, virtual time, injected input, exact ANSI |
| Historical v1 distinction | [v1-to-v2 migration guide](https://github.com/tui-cs/Terminal.Gui/blob/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/docfx/docs/migratingfromv1.md#L29-L79) | Historical/current docs at `d0a0ed9` | Supported | Static API is historical compatibility, not the recommended v2 model |
| Physical test gap | [Pinned repository test tree](https://github.com/tui-cs/Terminal.Gui/tree/d0a0ed9b150d3fc8aacf4ab07b7f7d91264fe6d6/Tests) | `d0a0ed9` | Inferred | No general PTY/emulator or full output-fault recovery suite found in the searched tree |
