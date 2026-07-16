# Recurring Failure Modes And Mitigations

Research baseline: 2026-07-16

This catalog groups recurring mechanisms rather than counting complaints. Each
entry states the affected requirement, the architectural cause, the common
workaround, and the corresponding ArborUI action. Lesson IDs refer to
[`lesson-ledger.yaml`](lesson-ledger.yaml).

## Logical State Advances Before Output Acceptance

**Requirement:** A renderer baseline must describe a frame known to have been
accepted by the backend.

**Recurring mechanism:** A renderer updates its shadow buffer or clears dirty
state while constructing output, before `flush`, or without checking the complete
write result. This appears across direct renderers, stream-based frameworks, and
terminal substrates.

**Observed mitigation:** Exit on output failure, expose a manual `Sync` or hard
refresh, or let applications replace the renderer.

**Workaround cost:** Continuing safely requires crossing the renderer/backend
boundary. A normal application callback cannot reconstruct whether a prefix was
applied or which baseline the renderer now believes.

**ArborUI action:** Preserve immutable prepared frames and joint commit only after
`Applied`. Treat `StateUnknown` and output errors after possible progress as
physical invalidation.

**Qualification:** The logical contract is verified in ArborUI, but its practical
frequency and adoption value are not. Fatal output remains a reasonable simpler
contract for many local applications.

**Lessons:** `RATATUI-OUTPUT-RECOVERY-01`,
`BUBBLE-TEA-OUTPUT-RECOVERY-03`, `TEXTUAL-OUTPUT-RECOVERY-03`,
`INK-OUTPUT-RECOVERY-03`, `OPENTUI-OUTPUT-RECOVERY-02`,
`IOCRAFT-OUTPUT-RECOVERY-03`, `FTXUI-OUTPUT-RECOVERY-03`,
`TCELL-OUTPUT-RECOVERY-03`.

## Write Completion Is Confused With Physical Certainty

**Requirement:** Distinguish serialization, transport acceptance, queued delivery,
and terminal state.

**Recurring mechanism:** A stream callback, successful flush, synchronized-update
envelope, or completed buffer write is treated as proof that the terminal applied
the intended patch.

**Observed mitigation:** Best-effort cleanup, repaint after suspension, writer
backpressure, or explicit caller-owned buffering.

**Workaround cost:** These mechanisms improve ordering and ordinary robustness but
cannot determine the physical state after a partial or externally interrupted
write.

**ArborUI action:** Document exactly what `Applied` means for each backend. A local
backend may accept responsibility after complete OS-level write; a buffered or
remote backend needs a later invalidation path if delivery fails.

**Lessons:** `INK-OUTPUT-RECOVERY-03`, `OPENTUI-OUTPUT-RECOVERY-02`,
`NOTCURSES-OUTPUT-RECOVERY-02`, `SPECTRE-CONSOLE-OUTPUT-RECOVERY-02`.

## Terminal Modes Are Treated As One Configurable Renderer

**Requirement:** Define cursor, history, resize, external output, and recovery
ownership for each output context.

**Recurring mechanism:** Alternate screen, inline region, main-screen footer,
append-only history, and native scrollback are selected through flags while
sharing assumptions about erasure and repaint.

**Observable friction:** Cursor-position queries race input, resize can invalidate
the region origin, external output bypasses the renderer, and immutable history
cannot be repaired like an owned viewport.

**Observed mitigation:** Query before input starts, require application cursor
tracking, redraw from a known anchor, or restrict support to full screen.

**ArborUI action:** Keep the current high-level full-screen contract explicit.
Design inline regions and native scrollback as separate modes with their own
failure and recovery semantics.

**Lessons:** `RATATUI-INPUT-PROTOCOL-04`,
`BUBBLE-TEA-TERMINAL-MODES-01`, `PYTERMGUI-TERMINAL-MODES-01`.

## Terminal Replies Compete With Application Input

**Requirement:** One owner must frame the byte stream and route protocol replies
before normal events.

**Recurring mechanism:** A renderer performs a cursor or capability query while a
separate event reader is already consuming stdin.

**Observable friction:** The application receives reply bytes as keys, the query
times out, or another reader consumes an incomplete sequence.

**Observed mitigation:** Stop the event reader, query only during startup, or use
a single incremental parser.

**ArborUI action:** Preserve one active local reader. Before capability negotiation
or inline mode, define a query state machine with fragmentation, timeout, late
reply, downgrade, and override tests.

**Lessons:** `RATATUI-INPUT-PROTOCOL-04`,
`PROMPT-TOOLKIT-INPUT-PROTOCOL-01`, `OPENTUI-INPUT-PROTOCOL-01`.

## Callback Queues Are Mistaken For Effect Ownership

**Requirement:** Async work needs ownership, cancellation, stale-result handling,
backpressure, and deterministic settlement.

**Recurring mechanism:** A framework serializes callback execution or message
delivery but leaves producer lifetime and completion policy to the application.

**Observable friction:** Removed screens receive stale results, shutdown leaves
work running, fast producers grow queues, and tests cannot know when effects have
settled.

**Observed mitigation:** Host-runtime cancellation tokens, generations in model
state, bounded application channels, or waiting through sleeps and polling.

**ArborUI action:** Keep model mutation serialized. Investigate effect identity,
component or screen ownership, cancellation, and bounded ingress without making a
particular async runtime mandatory.

**Lessons:** `BUBBLE-TEA-EFFECTS-SCHEDULING-02`,
`TERMINAL-GUI-EFFECTS-SCHEDULING-04`,
`OPENTUI-EFFECTS-SCHEDULING-04`, `BLESSED-EFFECTS-SCHEDULING-02`.

## Broadcast Input Pushes Focus Policy Into Applications

**Requirement:** Focus, modal priority, capture, propagation, clipping, and z-order
must select one coherent interaction target.

**Recurring mechanism:** Components subscribe independently to keys or pointer
events and inspect local flags or rectangles.

**Observable friction:** Duplicate handlers fire, overlays leak input, pointer
targets disagree with clipping, and drag capture is difficult to preserve through
tree changes.

**Observed mitigation:** Application-managed focused IDs, subscription ordering,
or manual modal flags.

**ArborUI action:** Preserve committed hit geometry, focus scopes, pointer capture,
and capture-target-bubble routing. Keep global shortcuts explicit rather than
making all input broadcast.

**Lessons:** `TEXTUAL-IDENTITY-INTERACTION-01`,
`INK-IDENTITY-INTERACTION-02`, `IOCRAFT-IDENTITY-INTERACTION-02`,
`GOCUI-IDENTITY-INTERACTION-01`.

## Scroll Containers Are Mistaken For Virtualized Collections

**Requirement:** Work for large data should scale with the visible range while
selection, focus, and anchoring use stable identity.

**Recurring mechanism:** A scroll view clips output after all children have been
constructed, reconciled, laid out, or painted.

**Observable friction:** Large collections pay full tree cost, and application
slicing must recreate variable-height measurement, focus retention, overscan, and
scroll anchoring.

**Observed mitigation:** Specialized table providers, pagination, application
slicing, caches, or custom nodes.

**ArborUI action:** Prototype a specialized data-provider and visible-range
contract before adding generic virtualization. Preserve a complete-render
reference path and measure tree work, output bytes, latency, and allocations.

**Lessons:** `TEXTUAL-LAYOUT-VIRTUALIZATION-02`,
`INK-LAYOUT-VIRTUALIZATION-05`, `FTXUI-LAYOUT-VIRTUALIZATION-02`,
`TVIEW-LAYOUT-VIRTUALIZATION-01`.

## Headless Tests Bypass Production Behavior

**Requirement:** A public harness should exercise the same scheduler, layout,
interaction, renderer, and transaction code used in production.

**Recurring mechanism:** Tests instantiate isolated widgets, discard output, use a
parallel fake renderer, or require a real terminal during application creation.

**Observable friction:** Application tests need sleeps, cannot inject resize or
effects, and miss integration failures in focus, settlement, and output recovery.

**Observed mitigation:** In-memory streams, simulation screens, internal-only test
helpers, screenshots, or PTY wrappers.

**ArborUI action:** Preserve `arborui-test` as a downstream public crate. Keep
logical, emulator, fault, and PTY tests separate so no one fake path is treated as
complete evidence.

**Lessons:** `BUBBLE-TEA-TESTING-04`, `TEXTUAL-TESTING-04`,
`INK-TESTING-04`, `OPENTUI-TESTING-03`, `TERMINAL-GUI-TESTING-01`,
`TVIEW-TESTING-04`, `GOCUI-TESTING-03`.

## Logical Unicode Is Presented As Universal Compatibility

**Requirement:** Distinguish grapheme segmentation, width policy, cell placement,
editing boundaries, serialization, and physical display.

**Recurring mechanism:** Correct logical width tables or snapshots are generalized
into claims about all terminals, fonts, Unicode versions, and final-column
behavior.

**Observable friction:** Ambiguous width, emoji presentation, ZWJ sequences,
autowrap, and continuation-cell overwrites differ across environments.

**Observed mitigation:** Configurable width policies, conservative final-column
rules, compatibility tables, or manual emulator checks.

**ArborUI action:** Preserve grapheme-level invariants and explicit width policy,
but describe physical compatibility as an empirical matrix. Add emulator and
selected physical-terminal tests before stronger claims.

**Lessons:** `FTXUI-TEXT-UNICODE-01`, `TCELL-TEXT-UNICODE-01`,
`NOTCURSES-TEXT-UNICODE-01`, `RICH-TEXT-UNICODE-01`.

## Architecture Is Expected To Substitute For Ecosystem

**Requirement:** Users need to build ordinary applications with reasonable code,
diagnostics, controls, documentation, and examples.

**Recurring mechanism:** A framework compares internal guarantees while ignoring
widget breadth, familiar composition models, templates, migration cost, and the
availability of substantial applications.

**Observable friction:** A technically stronger runtime may still require more
application code and carry higher adoption risk.

**Observed mitigation:** Ecosystem crates, facade-only examples, compatibility
layers, or adopting a host ecosystem such as React.

**ArborUI action:** Keep the focused core catalog, but prioritize representative
forms, dialogs, tables, text workflows, diagnostics, and one production-scale
facade-only application. Do not claim lower total cost before matched evidence.

**Lessons:** `RATATUI-ECOSYSTEM-ERGONOMICS-05`,
`BUBBLE-TEA-ECOSYSTEM-ERGONOMICS-05`,
`TEXTUAL-ECOSYSTEM-ERGONOMICS-05`, `INK-ECOSYSTEM-ERGONOMICS-01`,
`OPENTUI-ECOSYSTEM-ERGONOMICS-05`.

## Mitigation Principles

Across these failure modes, the most reusable mitigation principles are:

1. Make ownership explicit at subsystem boundaries.
2. Separate preparation from accepted commitment.
3. Preserve a simple correctness oracle before optimizing incrementally.
4. Test production paths with controlled dependencies.
5. Treat output modes as contracts, not flags.
6. Model stable identity where state or interaction must survive reconstruction.
7. Record the workaround and the subsystem boundary it crosses.
8. Preserve the cost and benefit of the competing design rather than labeling it
   simply correct or incorrect.
