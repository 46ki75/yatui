# Spectre.Console Library Survey

## Evidence Header

```text
Research date: 2026-07-16
Project version: Spectre.Console 0.57.2 (latest stable NuGet package)
Project revision: tag 0.57.2 -> bbbb5729dde27b58deee44f447a788eea46ee451
Repository: https://github.com/spectreconsole/spectre.console
Documentation version: Spectre website revision 1c1ab78f0c74f43d506f7137eec4ab32f9182918
Primary platform examined: Source and test inspection on Linux; no physical terminal or PTY reproduction
Report depth: Brief survey
```

The NuGet catalog marks `0.57.2` as non-prerelease, published 2026-07-02, and
records the pinned repository commit above. A newer `0.57.3-alpha.0.1` exists
but is excluded. GitHub's matching `0.57.2` release is labeled prerelease,
which is a channel-label discrepancy rather than a silently mixed baseline.

## Project And Scope

Spectre.Console is a C#/.NET library for cross-platform console applications.
The package targets `net8.0`, `net9.0`, `net10.0`, and `netstandard2.0`; it is
framework-portable rather than an OS-specific terminal backend. Its primary
category is a **presentation/CLI toolkit**: it combines rich renderables,
bounded live output, progress/status displays, and interactive prompts. It is
not a full-screen application framework with a retained application tree,
focus manager, event loop, or scheduler.

## Distinctive Strength

The core `IRenderable` contract separates `Measure` from `Render`, producing
styled `Segment` values rather than writing directly to the terminal. Tables,
panels, grids, layouts, charts, and custom renderables compose through this
model. `LiveDisplay` re-renders a mutable target in place through a render hook;
`Progress` adds task columns, refresh policy, `TimeProvider` injection, and a
non-interactive fallback. Typed text, confirmation, selection, multi-selection,
validation, masking, and cancellation make short interactive workflows easy.

Capability detection covers ANSI, color, Unicode, interactivity, dimensions,
and alternate buffers, with ASCII fallbacks when appropriate. Cell measurement
uses the bundled width machinery, and the pinned tests cover CJK characters,
variation selectors, ZWJ sequences, and regional-indicator flags. This is an
unusually polished path from formatted output to a small, capability-aware CLI
experience without requiring an application architecture.

## Important Limitation Or Tradeoff

**Classification: tradeoff. Requirement: ArborUI needs a long-running
full-screen application with explicit terminal ownership, recoverable frame
commit, serialized interaction, and deterministic complete-application tests.**
Spectre.Console scopes ownership to console operations: live and progress
displays run as exclusive, in-place output sessions, and the [live-mode
guide](https://github.com/spectreconsole/website/blob/1c1ab78f0c74f43d506f7137eec4ab32f9182918/Spectre.Docs/Content/console/how-to/live-rendering-and-dynamic-updates.md)
warns that live mode is not thread-safe or concurrent with prompts, progress,
or status. The
library does expose `AlternateScreen(Action)` with `finally`-based exit, but it
is a synchronous screen scope, not a complete runtime or failure-recovery
contract. The pinned public API exposes no prepared-frame commit or physical
screen invalidation boundary.

The workaround is to supply the application state, event loop, focus/input
policy, resize and signal handling, recovery policy, and PTY tests yourself,
then call Spectre renderables inside that layer or wrap it in
`AlternateScreen`. The cost is duplicating the runtime and lifecycle machinery
that ArborUI intends to provide. This is an intentional extension boundary,
not evidence that Spectre.Console is defective for presentation-oriented CLI
work.

## Testing Observation

`Spectre.Console.Testing` provides an injectable `TestConsole` backed by a
`StringWriter`, fixed default dimensions, capability controls, and
`TestConsoleInput` queues for keys and text. The official testing guide
recommends accepting `IAnsiConsole`, so tests exercise production renderable
and prompt code rather than a separate renderer. Repository tests combine
unit assertions with Verify text expectations and include cell-width and
alternate-screen cases. No PTY or terminal-emulator harness was found in the
pinned `Spectre.Console.Testing` package or repository test project: real
cursor behavior, raw terminal input, resize/signals, and partial-write recovery
remain outside this test boundary.

## Relevance To ArborUI

Spectre.Console is an adjacent reference, not a direct full-screen competitor.
ArborUI should borrow its renderable measurement/render split, capability
fallbacks, injectable console seam, and deterministic clock seam. Its narrow
scope also reinforces ArborUI's distinction: a complete application harness,
transactional physical-screen state, and explicit lifecycle recovery are
additional value, not requirements a presentation toolkit should be expected
to supply.

## Best Sources

- [NuGet 0.57.2 catalog entry](https://api.nuget.org/v3/catalog0/data/2026.07.02.21.40.40/spectre.console.0.57.2.json) (stable metadata and source commit; accessed 2026-07-16).
- [GitHub 0.57.2 commit](https://github.com/spectreconsole/spectre.console/commit/bbbb5729dde27b58deee44f447a788eea46ee451) (tagged implementation; accessed 2026-07-16).
- [IRenderable](https://github.com/spectreconsole/spectre.console/blob/bbbb5729dde27b58deee44f447a788eea46ee451/src/Spectre.Console/Rendering/IRenderable.cs),
  [live/progress](https://github.com/spectreconsole/spectre.console/blob/bbbb5729dde27b58deee44f447a788eea46ee451/src/Spectre.Console/Live/LiveDisplay.cs),
  and [prompts](https://github.com/spectreconsole/spectre.console/blob/bbbb5729dde27b58deee44f447a788eea46ee451/src/Spectre.Console/Prompts/TextPrompt.cs)
  (implementation; accessed 2026-07-16).
- [Alternate-screen implementation](https://github.com/spectreconsole/spectre.console/blob/bbbb5729dde27b58deee44f447a788eea46ee451/src/Spectre.Console/Extensions/AnsiConsoleExtensions.Screen.cs),
  [cell tests](https://github.com/spectreconsole/spectre.console/blob/bbbb5729dde27b58deee44f447a788eea46ee451/src/Spectre.Console.Tests/Unit/CellTests.cs),
  and [TestConsole](https://github.com/spectreconsole/spectre.console/blob/bbbb5729dde27b58deee44f447a788eea46ee451/src/Spectre.Console.Testing/TestConsole.cs)
  (implementation/tests; accessed 2026-07-16).
- [Pinned rendering-model documentation](https://github.com/spectreconsole/website/blob/1c1ab78f0c74f43d506f7137eec4ab32f9182918/Spectre.Docs/Content/console/explanation/understanding-rendering-model.md),
  [live-mode guide](https://github.com/spectreconsole/website/blob/1c1ab78f0c74f43d506f7137eec4ab32f9182918/Spectre.Docs/Content/console/how-to/live-rendering-and-dynamic-updates.md),
  and [testing guide](https://github.com/spectreconsole/website/blob/1c1ab78f0c74f43d506f7137eec4ab32f9182918/Spectre.Docs/Content/console/how-to/testing-console-output.md)
  (official docs; accessed 2026-07-16).
