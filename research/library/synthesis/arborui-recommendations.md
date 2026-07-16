# ArborUI Recommendations

Research baseline: 2026-07-16

These recommendations adjudicate the 76 records in
[`lesson-ledger.yaml`](lesson-ledger.yaml) against ArborUI's current
implementation and roadmap. They do not change the architecture by themselves.
Accepted changes should become tests, prototypes, benchmarks, roadmap work, or
architecture decision records as specified below.

## Decision Rules

- `Preserve` means the behavior is implemented and the research supports keeping
  it as an invariant.
- `Adopt` means the pattern has sufficiently strong evidence and a bounded next
  step.
- `Avoid` means the recurring mechanism conflicts with a current ArborUI
  requirement.
- `Investigate` means the requirement is relevant but the API, impact, or cost is
  not established.
- `Defer` means the lesson is useful but outside the current application profile.
- `Non-goal` means ArborUI should explicitly decline the requirement unless its
  product scope changes.

Project recurrence is not a vote. Recommendation strength considers evidence
quality, project scope, independent implementations, and applicability to
ArborUI's full-screen stateful application profile.

## Preserve

### P1. Transactional Prepared-Frame Commitment

**Decision:** Preserve as a core invariant.

ArborUI currently prepares and diffs without changing committed renderer state,
then commits UI state, hit geometry, and renderer baseline only after an applied
backend outcome. Unknown or erroneous output invalidates physical state so the
next success repaints completely. See the current
[`Renderer` transaction](../../../crates/arborui-render/src/frame.rs) and
[`Runner` outcome handling](../../../crates/arborui-runtime/src/runner.rs).

This directly addresses a recurring extension boundary in Ratatui, Bubble Tea,
Textual, Ink, OpenTUI, iocraft, FTXUI, Terminal.Gui, Prompt Toolkit, tcell,
notcurses, tview, blessed, gocui, and PyTermGUI.

**Do next:** Extend byte-boundary and synchronized-update fault injection. Do not
claim that this guarantee drives adoption until physical failures and application
cost are measured.

### P2. Coherent Identity, Geometry, And Routed Interaction

**Decision:** Preserve keyed retained identity, transactionally committed hit
geometry, focus scopes, pointer capture, and capture-target-bubble routing.

Textual provides the strongest positive precedent. Ink and iocraft expose the
cost of pointer routing or focus policy outside the framework. gocui shows that
identity and z-order can remain conceptually inexpensive.

ArborUI already implements the relevant behavior in the
[`UI tree`](../../../crates/arborui-ui/src/tree.rs) and
[`focus model`](../../../crates/arborui-ui/src/focus.rs).

**Do next:** Add representative modal, clipping, drag capture, and dynamic keyed
collection examples. Keep global shortcuts explicit rather than reverting to
broadcast-first input.

### P3. Borrowed Ephemeral Elements With Owned Retained State

**Decision:** Preserve the ownership boundary while treating its ergonomic
advantage as unproven.

iocraft confirms that borrowed ephemeral properties and keyed retained state are
a viable Rust-specific design. Terminal.Gui and Textual show the local-state and
lifecycle convenience of durable objects. ArborUI should keep retained state from
holding borrowed application references, but must still compare application code
and diagnostics against durable-view models.

**Do next:** Use a dynamic form and variable collection to measure keys,
fingerprints, ownership boilerplate, diagnostics, and retained memory.

### P4. Public Production-Path Application Harness

**Decision:** Preserve `arborui-test` as the downstream full-application test
surface.

Textual, Terminal.Gui, OpenTUI, and iocraft demonstrate that public headless
application testing materially improves framework usability. ArborUI's
[`TestApp`](../../../crates/arborui-test/src/app.rs) already drives the real
runner, UI tree, renderer, and terminal transaction path with controlled time,
events, settling, and output outcomes.

**Do next:** Keep examples dependent only on `arborui` plus `arborui-test` in
tests. Add explicit effect settlement and capability injection as those runtime
contracts mature.

### P5. Runtime, UI, Widget, Layout, And Backend Independence

**Decision:** Preserve current crate boundaries and facade-only application use.

Ratatui, tcell, Prompt Toolkit, tview, and Terminal.Gui show the long-term value
of narrow widget and backend seams. They also show the failure mode where replacing
output or event-loop policy requires abandoning the standard application layer.

ArborUI currently keeps Crossterm inside its adapter, Taffy private to layout, UI
independent of terminal/runtime, runtime independent of widgets, and applications
on the `arborui` facade.

**Do next:** Complete the planned public API review before stabilizing a widget
trait or adding another backend. Test that a custom backend retains standard
runtime, focus, widgets, and application harness behavior.

### P6. Grapheme-Level Logical Correctness

**Decision:** Preserve complete grapheme cells, continuation invariants, explicit
width policy, and grapheme-boundary editing.

FTXUI, tcell, notcurses, Rich, and Prompt Toolkit provide independent positive
precedents. ArborUI's implementation is already substantial in
[`arborui-text`](../../../crates/arborui-text/src/measure.rs) and the renderer.

**Do next:** Add emulator and physical compatibility evidence. Never describe a
logical width policy as universal terminal agreement.

## Adopt

### A1. Add A Virtual-Terminal Oracle

**Decision:** Adopt the pattern, not a specific dependency yet.

tcell's production-path emulator tests provide the clearest missing layer between
ArborUI patch tests and native PTY tests. The oracle should apply production ANSI
and expose cells, cursor, modes, scroll regions, and protocol replies.

**Acceptance criteria:** Assert alternate-screen transitions, final-column
behavior, wide-cell overwrite, cursor placement, erase behavior, synchronized
updates, and query replies. Keep the emulator separate from the abstract backend
and native PTY suites.

**Target:** Testing roadmap and a dependency/prototype decision.

### A2. Make Settlement States Explicit

**Decision:** Adopt explicit terminology and test controls for renderer idle,
visual settlement, model settlement, effect settlement, and quiescence.

Terminal.Gui, Textual, OpenTUI, Bubble Tea, and Ink show that `idle` is otherwise
ambiguous. ArborUI already controls time and scheduler progress, but future owned
effects and external producers need a visible settlement contract.

**Acceptance criteria:** A test with a visually idle screen and pending effect can
assert both states without sleeps. Shutdown policy states whether owned effects
are cancelled, drained, or detached.

**Target:** Runtime API experiment before stabilization.

### A3. Add Capability And Locale Injection To Test Ergonomics

**Decision:** Adopt the test seam.

Spectre.Console demonstrates comprehensive capability injection; notcurses uses
locale and terminal release matrices; Bubble Tea fixes color profile and size in
golden tests.

**Acceptance criteria:** Downstream tests can select width policy, color level,
keyboard support, synchronized updates, focus, paste, mouse, and related terminal
capabilities without ambient environment dependence.

**Target:** `arborui-test` and terminal capability API review.

### A4. Preserve Simple Component-Level Rendering Tests

**Decision:** Adopt a documented low-ceremony path in addition to `TestApp`.

Ratatui buffers, Rich capture, PTerm render-to-string, and Spectre.Console
renderables make focused component testing inexpensive. A complete runtime should
not be mandatory when the contract is only measurement and painting.

**Acceptance criteria:** A custom component can be measured and painted at a
fixed size with semantic cell assertions and no terminal session.

**Target:** Testing documentation; avoid adding a second rendering implementation.

### A5. Prioritize Ordinary Application Controls And Proof

**Decision:** Adopt a focused ecosystem investment, not a large core catalog.

Ratatui, Textual, Bubble Tea, Ink, Terminal.Gui, blessed, and the presentation
toolkits all demonstrate that reusable controls, examples, and familiar
composition affect adoption independently of architecture.

**Acceptance criteria:** Deliver the planned checkbox, select, modal/dialog, and
table path; build one substantial facade-only application with forms, overlays,
streaming effects, dynamic rows, Unicode input, and deterministic tests.

**Target:** Existing application-driven widget and example milestone.

## Avoid

### V1. Do Not Advance Baselines During Serialization

Prepared state must remain immutable until the backend reports its outcome.
Serialization, buffering, stream callbacks, synchronized-update closure, and
flush are not interchangeable acceptance points.

### V2. Do Not Treat Output Modes As Flags On One Ownership Contract

Full-screen alternate buffer, inline region, native scrollback, and bounded live
presentation have different history, cursor, resize, external-output, and repair
semantics. Shared widgets are desirable; shared ownership assumptions are not.

### V3. Do Not Add Capability Queries With Multiple Input Readers

ArborUI currently delegates complete events to one Crossterm reader. Any future
query protocol must be integrated into one owner with fragmentation, timeout,
late-reply, and downgrade behavior.

### V4. Do Not Call A Callback Queue An Effect System

Serialization alone does not establish ownership, cancellation, backpressure,
stale-result handling, shutdown, or deterministic settlement.

### V5. Do Not Equate Clipping With Virtualization

The current scroll view and list construct the complete child tree. Any future
large-collection claim must account for construction, reconciliation, layout,
painting, focus, selection, and variable-height measurement.

### V6. Do Not Make Architecture-Only Adoption Claims

Transactional recovery, backend independence, borrowed elements, and grapheme
correctness are real contracts. The reports do not prove they outweigh widget
maturity, familiar ecosystems, documentation, or application boilerplate.

## Investigate

| ID | Question | Why evidence is insufficient | Required evidence |
| --- | --- | --- | --- |
| I1 | Does transactional output recovery prevent failures users encounter often enough to justify its cost? | Competitor boundaries are source-verified, but physical impact was rarely reproduced. | Byte-boundary faults, PTY cases, issue corpus, and matched complexity measurements. |
| I2 | What visible-range API fits ArborUI identity and layout? | Repeated absence proves a boundary, not the correct generic API. | Fixed and variable-height list, table, tree, log, filtering, focus, and anchoring prototypes. |
| I3 | Which inline or native-scrollback modes should exist? | Other projects show value and fragility, but ownership contracts differ. | Explicit mode designs plus resize, query, external output, suspension, and recovery tests. |
| I4 | How much effect ownership belongs in the runtime? | Caller burden is common, but framework ownership can constrain host runtimes. | Screen-owned cancellation, stale result, bounded producer, shutdown, and embedding prototypes. |
| I5 | Are borrowed elements and explicit invalidation ergonomic at production scale? | No matched substantial application exists. | Equivalent ArborUI and alternative implementations with code and diagnostic review. |
| I6 | Does full layout and painting become a practical ceiling? | Full traversal is known; impact is not measured. | Large dynamic tree, streaming dashboard, and idle workloads with latency, allocations, bytes, and CPU. |
| I7 | Should terminal input be split below Crossterm into ArborUI-owned byte parsing? | Prompt Toolkit and OpenTUI show benefits; current Crossterm support is simpler. | Query negotiation and fragmented-input prototype without backend type leakage. |
| I8 | Can widget authoring remain compositional without a formal trait? | Current custom elements work, but third-party ecosystem evidence is absent. | Two external-style custom widgets using only the proposed stable surface. |

## Defer

### D1. Bounded Presentation And Shell Mode

Rich, PTerm, Spectre.Console, and Gum provide useful render-to-string,
degradation, stdout/stderr, exit-status, and subprocess patterns. These should be
revisited only if ArborUI adopts a bounded CLI or shell-helper product goal. They
must not weaken the current full-screen runtime contract.

### D2. Additional Backends Before Contract Review

tcell and Prompt Toolkit show the value of transport abstraction, but another
backend added too early can freeze accidental API. Complete the backend-author
surface review first, then use a second implementation to test it.

### D3. Native Rendering Core

OpenTUI and notcurses demonstrate capabilities of native cells, compositing, and
specialized terminal support. ArborUI has no evidence that a native core is
needed for its current workloads. Benchmark before accepting build,
distribution, safety, and binding costs.

## Non-Goals

### N1. Universal Physical Width Correctness

ArborUI should guarantee its documented logical segmentation and width policy,
not identical display across every terminal, font, Unicode version, and
multiplexer.

### N2. One Universal Screen-Mode Contract

Shared rendering components may span modes, but alternate-screen ownership,
inline regions, append-only history, and native scrollback should not be forced
into one behavioral contract.

### N3. Mandatory Async Runtime Or Concurrent Model Mutation

Textual demonstrates the value of deep asyncio integration; other projects show
host-runtime flexibility. ArborUI should keep model updates serialized and
effects runtime-neutral unless evidence requires a stronger commitment.

### N4. React-Compatible Hooks, CSS Cascading, Or Pluggable Layout Engines

These systems have real benefits, but they would change ArborUI's product and
extension boundaries. The research does not justify reversing the existing
non-goals.

### N5. A Large Built-In Widget Catalog

The lesson is to make ordinary applications viable, not to put every table,
tree, editor, chart, and formatter in the core workspace. Stabilize extension
surfaces and support ecosystem controls while keeping the standard catalog
focused.

## Roadmap Impact

| Existing area | Research adjustment |
| --- | --- |
| Lifecycle and capabilities | Add query ownership, emulator semantics, and broader PTY acceptance criteria. |
| Application-driven widgets | Keep planned controls; make the substantial facade-only app the primary ergonomics proof. |
| Large collections | Require multiple provider prototypes before selecting a generic virtualization API. |
| Performance | Preserve complete rendering as the reference implementation and measure before dirty-subtree work. |
| API stabilization | Review application, widget-author, backend-author, and test-author surfaces separately. |
| ADRs | Record output outcome semantics, mode separation, effect ownership, collection providers, and parser ownership once decisions are ready. |

No ADR directory currently exists. The research should feed the planned decision
process rather than creating retrospective ADRs for unsettled questions.
