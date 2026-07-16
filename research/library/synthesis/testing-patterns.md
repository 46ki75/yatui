# Cross-Project Testing Patterns

Research baseline: 2026-07-16

Terminal UI tests operate at different semantic layers. A passing snapshot at one
layer does not prove behavior at another. This catalog combines the project
reports with ArborUI's current test surface.

## Test Layers

| Layer | Contract proved | Representative projects | Main blind spot |
| --- | --- | --- | --- |
| Pure semantics | Editing, geometry, focus, state transitions, parser units | Prompt Toolkit, FTXUI, ArborUI | Integration and physical behavior |
| Logical frame | Cells, graphemes, styles, clipping, cursor, hit geometry | Ratatui, iocraft, ArborUI | ANSI serialization and terminal interpretation |
| Serialized patch | Exact control sequences and transition shape | Bubble Tea, Terminal.Gui, Rich | Whether a terminal accepted or interpreted the bytes |
| Virtual terminal | Resulting cells, cursor, modes, scroll regions, replies | tcell | Native lifecycle, OS signals, real transport failures |
| Complete app harness | Runtime, updates, effects, layout, routing, render, settlement | Textual, Terminal.Gui, OpenTUI, ArborUI | Raw mode, signals, emulator differences |
| Fault injection | Accepted, deferred, rejected, partial, and unknown output | ArborUI logical backend | Real kernel and terminal behavior |
| PTY or ConPTY | Raw mode, process lifecycle, signals, suspend, restoration | Ink internals, narrow ArborUI suite | Screen contents unless paired with an emulator |
| Physical compatibility | Terminal, multiplexer, font, width, final-column behavior | Mostly manual project matrices | Determinism and broad automation |
| Fuzz and property tests | Fragmentation, malformed input, Unicode sequences, state-machine invariants | FTXUI, ArborUI | End-to-end lifecycle unless the harness is integrated |

## Project Coverage

Terms in this table are deliberately narrow: `yes` means a meaningful facility
was found at the pinned revision, `partial` means a useful but incomplete seam,
and `not found` is limited to the searched revision.

| Project | Logical frame | Serialized output | Virtual terminal | Complete app | Clock or settlement | Output faults | PTY/lifecycle |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Ratatui | Yes | Partial | No | Partial | No | No | Limited repository tests |
| Bubble Tea | Partial | Yes | Not found | Partial | Application-owned | Writer injection only | Not found |
| Textual | Yes | Partial | Not found | Yes | Heuristic settlement | Not found | Not found |
| Ink | Frame capture | Partial | Not found | Partial public utility | Internal fake timers | Stream errors, not frame outcomes | Internal node-pty cases |
| OpenTUI | Yes | Partial | Not found | Yes, renderer-centered | Manual clock and visual idle | Partial memory writer | Not found |
| iocraft | Yes | No | No | Yes, component loop | No structured settlement | Not found | Not found |
| FTXUI | Yes | stdout capture | No | Not public | No | Not found | Manual emulator script |
| Terminal.Gui | Partial | Yes | No | Yes | Virtual time and iterations | Short-write helper only | Not found |
| Prompt Toolkit | Output discarded | No | No | Logical session | Real asyncio | Not found | Not found |
| tcell | Yes | Yes | Yes | Out of scope | Out of scope | Writes always succeed in mock | Limited substrate cases |
| notcurses | Yes | Partial | No | Out of scope | Out of scope | Not found | Manual release checks |
| tview | SimulationScreen seam | No | Through dependency | Not found | Not found | Not found | Not found |
| blessed | Screenshots | Partial | No | Ad hoc | Not found | Not found | Interactive fixtures |
| Rich | Capture | Yes | No | Out of scope | Refresh control | Not found | Out of scope |
| PTerm | Render-to-string | Partial | No | Out of scope | Forced dimensions | Not found | Out of scope |
| Spectre.Console | Segment capture | Partial | No | Out of scope | TimeProvider | Not found | Out of scope |
| Gum | No | No | No | Process smoke tests | Out of scope | Not found | Not found |
| gocui | Limited view buffers | No | No | Not found | Not found | Not found | Not found |
| PyTermGUI | Recording terminal | Partial | No | Not found | Not found | Not found | Not found |
| ArborUI | Yes | Patch inspection | Not yet | Yes | Manual time and settling | Outcome scripts | Narrow PTY/ConPTY |

## Reusable Patterns

### Production-Path Headless Applications

Textual's Pilot, Terminal.Gui's AppTestHelper, OpenTUI's test renderer, and
iocraft's mock loop all avoid reducing tests to isolated widgets. Their strongest
shared pattern is to run real application and rendering code with replaceable
terminal dependencies.

ArborUI already follows this pattern through `arborui-test::TestApp`, which drives
the production runner, UI tree, renderer, and terminal transaction path. This is
a feature to preserve as public API, not an internal test utility.

### Explicit Dimensions And Capabilities

Ratatui, Rich, PTerm, Spectre.Console, Bubble Tea, and Terminal.Gui show the value
of making terminal dimensions and capabilities deterministic. Every visual
contract should state its terminal size. Capability-dependent output should use
an injected profile rather than ambient detection.

ArborUI controls dimensions but should audit whether downstream tests can set all
relevant capability and width-policy inputs as easily as Spectre.Console's test
console.

### Semantic Assertions Alongside Snapshots

Prompt Toolkit emphasizes returned values, cursor position, history, selection,
and editing state. Ratatui and iocraft expose logical cells directly. These
patterns make failures easier to diagnose than byte snapshots alone.

ArborUI snapshots should continue to retain at least one assertion about model,
focus, cursor, hit target, style, patch, or effect state. Character snapshots are
the default visual contract, not the only assertion.

### Controlled Time And Settlement

Terminal.Gui's virtual time and iteration waits, Textual's Pilot pause, OpenTUI's
manual clock, and Spectre.Console's TimeProvider demonstrate complementary parts
of deterministic time.

ArborUI should keep the following concepts separate:

- Renderer idle: no frame is currently requested.
- Visual settlement: queued visible changes have rendered.
- Model settlement: all currently deliverable messages have been processed.
- Effect settlement: no owned effect can still produce a result.
- Quiescence: no progress is possible without external input or clock advance.

The current harness controls time and settles its scheduler, but future effect
ownership must make these states explicit rather than relying on sleeps.

### Emulator As Semantic Oracle

tcell's MockTerm is the clearest reusable pattern not yet present in ArborUI. It
feeds production terminal sequences to an emulator and exposes resulting cells,
cursor, style, modes, and protocol behavior.

An ArborUI emulator layer should test:

- Patch serialization and cursor movement
- Alternate-screen entry and exit
- Autowrap and final-column policy
- Wide-cell overwrite and continuation behavior
- Scroll regions and erase operations
- Synchronized-update envelopes
- Terminal query and reply routing

It should not replace native PTY tests or output-fault tests.

### Outcome And Fault Matrix

Most examined renderers equate a completed write or flush with advancing logical
state. ArborUI's outcome model is unusual enough to warrant a dedicated matrix:

| Injected outcome | Required committed state | Required next render |
| --- | --- | --- |
| Applied | Advance UI, hit map, and renderer baseline together | Diff from accepted frame |
| Deferred before progress | Preserve previous committed state | Retry prepared work or regenerate safely |
| Rejected before progress | Preserve previous committed state | Policy-dependent retry or error |
| Unknown after possible progress | Preserve logical baseline and invalidate physical state | Complete repaint |
| Serialization failure before output | Preserve previous state | No physical invalidation unless output began |
| Failure while closing synchronized update | Treat physical state as unknown | Complete repaint and lifecycle repair |

Current unit and `arborui-test` coverage proves the high-level state transitions.
The remaining gap is systematic byte-boundary injection through serialization and
native output adapters.

### PTY And Physical Tests

A PTY proves process and terminal-driver behavior, not screen semantics. Pair it
with an emulator when asserting cursor or cell state. The native matrix should
cover:

1. Normal open, render, and Drop restoration.
2. Panic after raw mode and alternate-screen entry.
3. Failed cleanup followed by best-effort raw-mode recovery.
4. Suspend to a child process and resume with forced repaint.
5. Stop, continue, termination, disconnect, and resize storms.
6. Query timeout and late reply while user input is queued.
7. Final-column and wide-grapheme behavior in selected emulators and multiplexers.

## Recommended Additions

| Priority | Addition | Evidence source | Acceptance criterion |
| --- | --- | --- | --- |
| High | Virtual-terminal oracle | tcell | Production ANSI produces asserted cells, cursor, and modes. |
| High | Byte-boundary output-fault target | Cross-project output findings | Unknown progress always invalidates and next success repaints fully. |
| High | Broader PTY lifecycle matrix | Ink internals and project gaps | Panic, suspend, signal, resize, and cleanup contracts pass on supported platforms. |
| High | Explicit effect settlement states | Textual, Terminal.Gui, OpenTUI | Tests can distinguish visual idle from pending owned effects. |
| Medium | Input fragmentation and query-reply tests | Prompt Toolkit and OpenTUI | Fragmented UTF-8 and protocol replies never leak as user keys. |
| Medium | Locale and capability matrix | notcurses and Spectre.Console | Width, color, keyboard, and fallback behavior are injected and reproducible. |
| Medium | Reference-versus-optimized render properties | Ratatui and FTXUI simplicity | Any incremental path matches complete rendering for generated trees. |
| Medium | Matched cross-framework application tests | All direct alternatives | Equivalent scenarios compare code, assertions, settling, and failure handling. |

## Snapshot Policy

- Always use an explicit terminal size.
- Prefer character or semantic frame snapshots for stable visual contracts.
- Use byte snapshots only when serialization shape is the contract.
- Use emulator-state snapshots when terminal interpretation is the contract.
- Retain semantic assertions for focus, cursor, model, selection, or hit targets.
- Never update snapshots automatically in CI.
- Treat PTY transcripts as diagnostic artifacts unless their byte stream is the
  deliberate contract.
