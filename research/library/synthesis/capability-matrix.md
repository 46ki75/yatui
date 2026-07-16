# Cross-Project Capability And Extension Matrix

Research baseline: 2026-07-16

This matrix synthesizes the project reports by requirement. It is not a quality
ranking. A capability outside a project's intended layer is marked out of scope
rather than missing. Detailed observations and evidence status are in
[`lesson-ledger.yaml`](lesson-ledger.yaml).

## Status Vocabulary

| Status | Meaning |
| --- | --- |
| Native | The project owns a direct contract for the capability. |
| Application policy | The project exposes enough primitives, but the application supplies the policy. |
| Extension | The capability requires replacing or extending a documented subsystem. |
| Partial | A narrower contract is native, but an important part remains external. |
| Out of scope | The capability is intentionally outside the examined project layer. |
| Not found | No supported facility was found at the pinned revision. |

These statuses describe API boundaries, not implementation quality. They also do
not imply equivalent semantics. For example, a memory screen, a virtual terminal,
and a PTY are all useful test facilities but prove different properties.

## Direct Architectural Alternatives

| Requirement | Ratatui | Bubble Tea | Textual | Ink | OpenTUI | iocraft | ArborUI |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Serialized application updates | Application policy | Native reducer | Native message loop | React policy | Application or binding policy | Native component loop | Native runtime |
| Stable retained identity | Application policy | Application policy | Native widget tree | Native React keys | Native scene and bindings | Native keyed components | Native keyed UI state |
| Routed focus and pointer interaction | Application policy | Application policy | Native | Keyboard native; pointer not found | Native hit grid | Partial; broadcast input | Native capture-target-bubble |
| Structured effect ownership and cancellation | Out of scope | Partial; commands lack ownership cancellation | Native workers with asyncio policy | React and Node policy | Application or binding policy | Partial hook/runtime policy | Partial; serialized commands, incomplete ownership controls |
| Generic visible-range collections | Application policy | Application policy | Not found; specialized controls | Extension or ecosystem | Not established | Not found | Planned; eager list today |
| Grapheme-aware logical cells | Native | Native through renderer dependency | Native | Partial terminal-string model | Native cell model | Native canvas model | Native |
| Outcome-aware frame commit | Not found | Not found | Not found | Not found | Not found | Not found | Native |
| Explicit physical-state invalidation | Manual full redraw | Not found as public recovery | Not found | Not found | Internal repaint latch | Not found | Native |
| Separate full-screen and inline contracts | Native viewport modes | Native modes, shared renderer | Multiple drivers and modes | Native inline and static output | Native renderer modes | Native inline and full screen | Full screen native; inline undefined |
| Public production-path app harness | Partial logical terminal | Partial program seams | Native Pilot | Partial frame capture | Native renderer harness | Native mock loop | Native TestApp |
| Physical lifecycle tests | Limited project coverage | Not found at pinned revision | Not found at pinned revision | Internal PTY regressions | Not found at pinned revision | Not found at pinned revision | Narrow native PTY/ConPTY suite |

### Interpretation

- Ratatui demonstrates the value of a small rendering boundary. Missing
  application policy is intentional, not a framework defect.
- Textual demonstrates the coherence available when identity, geometry, focus,
  workers, styling, and testing share one framework-owned object graph.
- Bubble Tea demonstrates how far a small serialized reducer and command model can
  go while leaving identity, cancellation, and deterministic settlement to
  applications.
- Ink demonstrates the adoption advantage of reusing a mature component model,
  while also showing the cost of leaving pointer routing and terminal recovery
  outside that model.
- OpenTUI demonstrates rich cell, input, hit-grid, and headless-renderer contracts,
  but its renderer scheduler is not a serialized application effect system.
- iocraft is the closest Rust comparison for borrowed ephemeral descriptions and
  retained keyed state. Its broadcast-first input model provides a useful
  contrast with ArborUI's routed interaction.
- ArborUI's unusual current contract is not full-screen rendering by itself. It
  is the joint commit of UI state, hit geometry, renderer baseline, and accepted
  backend output.

## Substrate And Toolkit Lessons

| Project | Layer | Strong boundary to reuse | Boundary that remains above or outside the project | ArborUI conclusion |
| --- | --- | --- | --- | --- |
| tcell | Terminal substrate | Screen and terminal emulation | Application state, focus, effects, settlement | Preserve runtime-independent terminal contracts; add an emulator oracle. |
| notcurses | Terminal substrate and compositor | Grapheme cells, planes, render-to-buffer | General app runtime and transactional acceptance | Study plane composition; avoid process-global ownership in reusable layers. |
| Python Prompt Toolkit | Input and interactive CLI toolkit | Incremental input, buffers, editing, injectable I/O | Domain model, effect policy, full-screen recovery | Reuse parser and semantic-test lessons without treating prompt scope as incomplete. |
| FTXUI | Widget and application library | Functional components, DOM layout, logical screens | Generic virtualization and output recovery | Keep a full-render correctness path; add a visible-range seam only with evidence. |
| Terminal.Gui | Application framework | Retained controls, virtual time, complete app harness | Transactional output and framework effect ownership | Match its test ergonomics while preserving ArborUI's output contract. |
| tview | Widget and application library | Simple primitives and table data source | Event-loop and output policy are coupled to Application | Keep widget, runtime, and backend seams independently usable. |
| blessed | Retained widget framework | Broad controls and low-level escape hatches | Deterministic settlement and recovery | Widget breadth and escape hatches matter even when architecture is less strict. |
| gocui | Minimal retained UI | Named views, simple focus, explicit z-order | Modal routing, headless construction, write recovery | Keep simple applications inexpensive; do not require a real terminal to construct them. |
| PyTermGUI | Lightweight framework | Shared widgets across inline and full-screen modes | Complete harness and output recovery | Reuse widgets where possible, but define each output mode independently. |

## Adjacent Presentation And Shell Tools

| Project | Applicable lesson | Runtime criteria |
| --- | --- | --- |
| Rich | Composable renderables, cell measurement, capture, deliberate degradation | Retained identity, effects, and transactional recovery are outside scope. |
| PTerm | Render-to-string, writer injection, bounded prompts and progress | Full-screen ownership and application settlement are outside scope. |
| Spectre.Console | Separate measure/render, capability injection, deterministic time | Its alternate-screen scope is cleanup-oriented, not a retained runtime. |
| Gum | stdout/stderr separation, exit status, shell composition, PTY-aware child commands | Process composition is not retained application identity. |

The absence of an outcome-aware frame commit in these tools is not four
independent findings against them. It is one expected consequence of their
bounded presentation scope.

## Extension Boundary Matrix

| Boundary | Useful precedent | Recurring failure | ArborUI state | Decision |
| --- | --- | --- | --- | --- |
| Application facade | Ratatui facade; Ink React entry; Bubble Tea model | Applications import implementation crates or assemble policy repeatedly | `arborui` facade is implemented; examples use it exclusively | Preserve and validate with a larger application. |
| Widget authoring | Ratatui Widget; FTXUI Node/Component; Terminal.Gui View | Custom controls require replacing runtime or backend behavior | Composition, custom paint, and handlers are public; formal Widget trait is not implemented | Review the smallest stable widget-author surface before 1.0. |
| Terminal backend | tcell Screen/Tty; Prompt Toolkit Input/Output; Terminal.Gui drivers | A custom backend forfeits standard lifecycle or event-loop behavior | Backend-neutral `TerminalBackend`; Crossterm isolated | Preserve; add a second implementation only after contract review. |
| Prepared render | notcurses render-to-buffer; FTXUI DOM-to-screen | Logical state advances during serialization or before write acceptance | Prepare, commit, discard, and invalidate are implemented | Preserve as a core invariant. |
| Input owner | Prompt Toolkit parser; OpenTUI byte parser | Cursor replies race another terminal reader | One local Crossterm reader exists, but ArborUI owns no byte parser | Keep single ownership; investigate parser and query state before inline mode. |
| Large-data provider | tview TableContent | Scroll containers still construct and traverse all children | Current list is eager; visible-range work is planned | Prototype specialized collection providers before generalizing. |
| Effect integration | Textual workers; Bubble Tea commands; native task ecosystems | Callback queues are called effect systems without ownership or settlement | Runtime-neutral futures and timers exist; cancellation/backpressure are incomplete | Add policy only where tests show application-level value. |
| Test driver | Textual Pilot; Terminal.Gui AppTestHelper; OpenTUI test renderer | Fake paths bypass production scheduling or rendering | `arborui-test` drives the production runner and renderer | Preserve public status and expand physical/emulator layers. |

## Correlation And Weighting

- tview inherits important terminal behavior from tcell; those reports are not
  independent confirmations of the output model.
- Projects using ordinary streams often share the same flush and error
  assumptions. Repetition establishes prevalence, not independent proof of user
  impact.
- Deep reports carry more weight for application architecture than brief surveys.
- Adjacent tools carry substantial weight for formatting, degradation, capture,
  and shell composition, but little weight for retained runtime policy.
- A project-specific benchmark cannot establish cross-project performance unless
  workload, environment, and measured contract are equivalent.

## Current ArborUI Gaps

The comparison supports these as current gaps rather than established defects:

1. No generic visible-range collection or external data-provider contract.
2. No virtual-terminal oracle that applies production ANSI and inspects semantic
   terminal state.
3. Narrow PTY coverage for panic, signals, suspension, child handoff, resize, and
   emulator-specific behavior.
4. No byte-oriented input parser or capability-query state machine owned by
   ArborUI; the Crossterm backend supplies complete events.
5. Incomplete effect ownership, cancellation, ingress bounds, and backpressure.
6. A small widget catalog and no production-scale proof of framework ergonomics.
7. Undefined inline-region and native-scrollback ownership contracts.
8. No measured evidence that transactional recovery improves enough real sessions
   to drive adoption, despite the logical contract being implemented and tested.
