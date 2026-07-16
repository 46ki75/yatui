# Notcurses Research Report

## Evidence Header

```text
Research date: 2026-07-16
Project version: Notcurses 3.0.17
Project revision: 77672788db0765ab868abafebbaadd8cfe133781
Repository: https://github.com/dankamongmen/notcurses
Documentation version: notcurses.com v3.0.17; accessed 2026-07-16
Primary platform examined: Source inspection on Linux; no physical terminal reproduction
Report depth: Standard profile
```

The latest stable release at the start of this research was
[Notcurses v3.0.17](https://github.com/dankamongmen/notcurses/releases/tag/v3.0.17),
released on 2025-10-28. Source conclusions refer to the tagged commit above;
development `master` at `b26048eebc74d5d254717d3332fa484718f9efe6` was inspected
separately and is not stable-release evidence.

## Executive Assessment

Notcurses is best classified as a **terminal substrate**: a C library that owns
terminal capability detection, input decoding, cell and plane storage,
compositing, rasterization, terminal lifecycle, and optional multimedia. It is
not a complete application framework.

Its strongest proposition is an unusually capable retained terminal surface.
Planes form z-ordered piles, cells represent Unicode extended grapheme clusters
with styles and channels, and visuals can degrade from pixel protocols to
Unicode blitters. It also provides explicit TUI and CLI modes, mouse and resize
events, and a C API usable from multiple language wrappers.

The same boundary limits its direct suitability for ArborUI. Application state,
focus policy, event routing, effects, scheduling, and complete application
testing remain caller responsibilities. Notcurses has useful recovery tools,
especially `notcurses_refresh`, but its normal output contract is still a
`FILE`-oriented rasterizer rather than a backend transaction that commits a
prepared frame only after complete acceptance. ArborUI should learn from the
substrate, not treat it as an alternative application runtime.

## Project Snapshot And Core Proposition

Notcurses is an Apache-2.0 C17 library built with CMake 3.21 or newer. Its
required runtime/build dependencies include terminfo and GNU libunistring;
FFmpeg, OpenImageIO, GPM, QR-Code-generator, C++, and Python are optional. The
tagged README advertises Linux, FreeBSD, Windows, macOS, Unicode EGCs, 24-bit
color, Sixel, Kitty graphics, keyboard protocols, and both TUI and CLI modes
([README and requirements](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/README.md#L1-L120)).

The project is a low-level library with some built-in widgets, not an Elm-,
React-, or message-driven framework. Included widgets such as menus, selectors,
readers, reels, trees, and progress bars are constructed on planes; they do not
define a general retained component tree or application scheduler
([API overview](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/doc/man/man3/notcurses.3.md#L69-L177)).
This makes Notcurses attractive when an existing C/C++ program needs a powerful
terminal surface and control of its event loop. It is less attractive when the
library is expected to own model updates, focus, task lifetimes, or settlement.

## Architecture

### Planes, Cells, And Rendering

An `ncplane` is a retained rectangular framebuffer with parent-relative geometry,
styles, channels, a base cell, a user pointer, and optional resize callback.
Planes are totally ordered on a z-axis and can also be bound into parent/child
forests. A context may contain several independent piles. The plane API exposes
direct movement, reparenting, resizing, scrolling, clipping, and cell access,
but not a general constraint layout system
([plane contract](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/doc/man/man3/notcurses_plane.3.md#L21-L148)).

Each `nccell` stores one extended grapheme cluster, style bits, foreground and
background channels, and width. Longer EGCs are kept in a plane-associated pool;
erasing or destroying the plane invalidates associated cell storage. This is a
strong terminal-correctness primitive, but callers must respect the C lifetime
and ownership rules ([cell representation](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/doc/man/man3/notcurses_cell.3.md#L13-L21),
[EGC storage](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/doc/man/man3/notcurses_cell.3.md#L137-L163)).

Rendering first reduces a pile from its z-order to a cell matrix, then
rasterizes that matrix and the terminal state into control sequences and EGCs.
`ncpile_render` can run concurrently for distinct piles; rasterization is
blocking and only one rasterization operation may proceed at a time. The pile
must not be modified while its frame is being rasterized
([render pipeline](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/doc/man/man3/notcurses_render.3.md#L25-L69)).
This separation is useful for custom render-to-buffer integrations, but it is
not a retained widget reconciliation system.

### Input, Effects, And Ownership

Notcurses puts stdin into non-canonical, non-blocking mode and exposes blocking,
deadline-based, nonblocking, and vector input APIs. `notcurses_inputready_fd`
provides a descriptor for integration with `poll`-style loops. Keyboard events
are Unicode codepoints or synthesized key values; the documented `ncinput`
contract explicitly does not represent complete EGCs. Mouse coordinates,
modifiers, `NCKEY_RESIZE`, and `NCKEY_SIGNAL` are available. Escape sequences
must arrive completely or are rejected/playback as literal input
([input contract](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/doc/man/man3/notcurses_input.3.md#L24-L117)).

The application still owns the event loop, model, focus, command routing,
timers, asynchronous work, cancellation, and backpressure. `ncvisual_stream`
and subprocess/file-descriptor widgets provide subsystem hooks, but no general
scheduler or serialized effect queue was found in the tagged API. The library
supports multithreaded use through strict caller rules rather than
internal locking: distinct piles can be operated concurrently, one thread may
read input, and a pile cannot be modified while it is being rendered
([threading rules](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/doc/man/man3/notcurses.3.md#L155-L177)).

### Terminal Lifecycle And Modes

`notcurses_init` owns terminal-facing setup around a caller-provided `FILE`,
stdin, termios state, terminal queries, and signal handlers. Only one Notcurses
or direct-mode context may be active in a process. TUI mode enters the alternate
screen by default. `NCOPTION_NO_ALTERNATE_SCREEN` and `NCOPTION_CLI_MODE` support
the primary screen, preserved cursor, and scrolling output; these are distinct
ownership modes, not an application history model
([initialization and modes](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/doc/man/man3/notcurses_init.3.md#L66-L110)).

SIGWINCH becomes `NCKEY_RESIZE`; the next render or refresh resizes the standard
plane. SIGCONT causes a signal event and a full rebuild on the next rasterization.
`notcurses_stop` unregisters handlers and restores attributes, palette, cursor,
and the alternate screen. It is undefined to use the context concurrently with
shutdown ([stop behavior](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/doc/man/man3/notcurses_stop.3.md#L15-L51)).

## Core Strengths

### Deep Terminal Representation

The cell/EGC model directly represents the correctness unit that ArborUI cares
about: graphemes, wide cells, styles, channels, and clipping. Visuals add image,
video, pixel, Sixel, Kitty, and Unicode-blitter paths without requiring a
separate rendering library ([visual API](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/doc/man/man3/notcurses_visual.3.md#L127-L160)).
This is substantially richer than a plain ANSI writer and is a credible choice
for modern terminal emulators where media is a requirement.

### Useful Rendering Boundaries

The explicit virtual-render/rasterize split, multiple piles, and render-to-buffer
API give advanced users more control than a monolithic draw call. A custom
application can generate a frame without immediately writing it or partition
independent work across piles. The tradeoff is that concurrency and frame
lifetime are exposed as C-level obligations rather than hidden by a runtime.

### Lifecycle And Capability Knowledge

Notcurses has unusually detailed terminal capability handling, including terminfo,
device queries, truecolor, keyboard protocols, alternate-screen transitions,
resize signals, and restoration. `notcurses_refresh` is an explicit hard redraw
for external terminal damage, rather than assuming that internal damage tracking
knows everything about the physical screen
([refresh contract](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/doc/man/man3/notcurses_refresh.3.md#L15-L34)).

## Limitations And Frustrations

### No Complete Application Runtime

```text
Classification: Tradeoff
Requirement: Retained interaction identity, focus, serialized updates, effects, and application settlement
Library assumption: The caller owns the application around a powerful terminal surface
Observable friction: Focus, routing, timers, redraw policy, and component task lifetimes are assembled by each application
Root architectural cause: The public abstraction is planes and input records, not a retained component/runtime tree
Available workaround: Build an application layer above Notcurses or use a separate framework
Cost of workaround: The caller owns conventions, scheduling, and full-application test infrastructure
Upstream response: Intentional library boundary; included widgets do not change it
Current status and version: Verified in 3.0.17
Evidence: Supported by API scope and source; no runtime was found
Confidence: High for the boundary, medium for duplicated application cost
```

This is not a defect for a C program that already has an event loop. It is a
direct mismatch with ArborUI's primary profile. A form or dashboard can use
Notcurses planes and included controls, but the application must decide input
routing, focus, asynchronous model updates, and frame settlement. No public
run-until-settled harness was found.

### Output Is Not A Transactional Frame Contract

```text
Classification: Extension failure relative to ArborUI's physical-screen requirement
Requirement: Commit a prepared frame only after complete backend acceptance; repaint fully after partial or uncertain output
Library assumption: Rasterizing to a FILE or caller-written buffer is the terminal output boundary
Observable friction: There is no applied/deferred/unknown write outcome tied to logical frame state
Root architectural cause: Output is serialized through the rasterizer/FILE path rather than an outcome-aware backend transaction
Available workaround: Use render-to-buffer, own the complete write, and call notcurses_refresh after an error
Cost of workaround: The application owns output recovery and still cannot prove what a stream partially delivered
Upstream response: Documentation requires refresh after a render-to-buffer error; no transaction protocol was found
Current status and version: Verified in 3.0.17
Evidence: Supported by the render and refresh contracts; not physically reproduced
Confidence: High for the public API boundary
```

The closest supported recovery behavior is explicit: if
`ncpile_render_to_buffer` fails, subsequent frames may be out of sync and
`notcurses_refresh` must be called. That is valuable caller-directed repair, not
ArborUI's prepared-frame commit rule. The default `notcurses_render` path
returns an error and tracks failed renders, but exposes no backend acceptance
state for an application's physical baseline.

### Terminal Negotiation And Embedding Are Strongly Owned

```text
Classification: Tradeoff, with a reported compatibility risk
Requirement: Predictable startup in varied transports and multiple independent UI contexts
Library assumption: One process owns a real, correctly described terminal and its stdin
Observable friction: One-context enforcement, global terminal state, capability queries, and strict TERM/terminfo expectations constrain embedding
Root architectural cause: Notcurses is a terminal owner, not an injected transport/backend
Available workaround: Disable selected signal/mode features, use a separate process, or provide a known compatible terminal environment
Cost of workaround: Reduced capability, platform-specific setup, or process isolation
Upstream response: Extensive terminal documentation; PR #2930 addresses issue #2929 on development history
Current status and version: Stable behavior verified; later issue response is not part of v3.0.17
Evidence: Verified startup contract; compatibility problem reported upstream
Confidence: High for ownership, medium for terminal-specific impact
```

Initialization sends terminal queries and the project documents that an
unrecognizable Primary Device Attributes response can make `notcurses_init`
hang ([terminal requirements](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/TERMINALS.md#L12-L42)).
That is a reasonable tradeoff for capability-rich modern terminals, but a
serious integration concern for remote, multiplexed, test, or newly emulated
terminals. Issue [#2929](https://github.com/dankamongmen/notcurses/issues/2929)
reports a WSL/Windows Terminal hang; development PR
[#2930](https://github.com/dankamongmen/notcurses/pull/2930) is evidence of
ongoing response, not proof of a stable-release fix.

### Python Is An Optional, Native, Partial Surface

The repository contains both an older CFFI wrapper and a newer compiled C
extension. The CFFI README explicitly says coverage is "nowhere near complete";
both setup paths depend on an installed native `libnotcurses` or a native CFFI
build, and PyPI labels the package Beta. Python users therefore inherit
Notcurses's terminal ownership and C-lifetime assumptions, not a pure-Python
application framework. This is a maturity/ecosystem limitation, and ArborUI
should provide a deliberate Python facade if Python integration becomes a goal
([CFFI metadata](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/cffi/setup.py#L49-L78),
[CFFI native build](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/cffi/src/notcurses/build_notcurses.py#L5-L12),
[compiled wrapper](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/python/setup.py#L39-L78)).

## Testing Strategy

Notcurses uses CTest and a Doctest executable named `notcurses-tester`. The
registered test is serial and links the full C++ wrapper, libunistring, terminfo,
and test data. The source suite covers cells, EGC pools, wide text, geometry,
planes, stacking, piles, scrolling, resize, output, visuals, palettes, and
included widgets. This is meaningful production-path testing rather than a
separate fake renderer ([CTest registration](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/CMakeLists.txt#L864-L901),
[test fixture](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/src/tests/main.cpp#L17-L27)).

The fixture initializes a real Notcurses context with no alternate screen and
drained input. The runner requires a usable locale and `TERM`; resize and
render tests call production APIs and inspect return values and dimensions.
The release checklist adds multimedia configurations, `LANG=C` and French
locale runs, Valgrind, manual demos, multiple terminal geometries, and Sixel and
Kitty checks ([release checklist](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/doc/testing-checklist.md#L1-L31)).

At the recorded revision, repository search found no PTY suite, virtual-terminal
emulator oracle, golden terminal snapshot contract, complete application
harness, controlled clock, or partial-write fault injection. The Windows CI
workflow explicitly leaves CTest disabled until a ConPTY environment exists
([Windows workflow](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/.github/workflows/windows_test.yml#L63-L87)).
The test strategy is strong for cell algorithms, input decoding, and real
terminal-oriented integration, but it does not establish ArborUI's physical
screen transaction or deterministic application-settlement guarantees.

## Common Scenario Assessment

| Scenario | Assessment |
| --- | --- |
| Form with focus and modal | Partial; planes and widgets exist, but focus and routing are caller policy |
| Large scrollable collection | Partial; reels/trees help, but no general keyed data/runtime contract was found |
| Streaming external updates | Substrate support exists through descriptors, subprocesses, and visual streams; scheduling is external |
| Unicode-heavy editing | Strong cell/EGC output; input is codepoint-oriented and editor state is external |
| Overlay with clipping and mouse | Strong plane/z-order primitives and mouse coordinates; hit routing is external |
| Resize during updates | Supported through SIGWINCH, `NCKEY_RESIZE`, and render/refresh; physical PTY coverage is limited |
| Deferred or failed output | Partial recovery guidance, but no accepted/unknown write transaction |
| Native scrollback conversation | CLI/scrolling mode exists; immutable application-owned history does not |

## Lessons For ArborUI

ArborUI should adopt the explicit distinction between a terminal substrate and an
application runtime. Notcurses demonstrates the value of a cell model that
stores complete EGCs, widths, styles, channels, and ownership-aware backing
storage. Its plane forest and render/raster split are useful reference points
for overlays, clipping, compositing, and backend-independent frame generation.
ArborUI should also preserve explicit screen modes, one input owner, capability
diagnostics, a hard-refresh operation, and a clear shutdown contract.

ArborUI should avoid treating a `FILE` write, a successful rasterization call,
or a normal terminal flush as proof that the physical screen accepted a complete
patch. The Notcurses workaround confirms that a hard repaint is useful, but the
ArborUI backend contract should represent accepted, rejected, and uncertain
output and invalidate physical state on uncertainty. It should also avoid
letting terminal capability negotiation block indefinitely and should define
whether multiple sessions require separate processes or injected backends.

This research does not prove that ArborUI's stronger runtime or transaction
model reduces total application cost. Notcurses may be the better choice for a
media-rich C program, a custom event loop, or an application that values direct
terminal control over framework policy. Follow-up work should compare an
equivalent form and streaming dashboard, inject partial writes into both
rendering paths, and add a PTY or virtual-terminal test to validate ArborUI's
claimed recovery benefit.

## Evidence Appendix

All sources below were accessed on 2026-07-16 unless another date is listed.
Source links are pinned to the stable commit unless marked otherwise.

| Claim | Source | Version or revision | Status and notes |
| --- | --- | --- | --- |
| Stable release and release date | [v3.0.17 release](https://github.com/dankamongmen/notcurses/releases/tag/v3.0.17) | `v3.0.17`, commit `7767278`; 2025-10-28 | Verified; stable baseline |
| Scope, modes, piles, widgets, and threading | [notcurses API manual](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/doc/man/man3/notcurses.3.md#L16-L177) | `7767278` | Verified |
| Plane, cell, EGC, and render contracts | [plane manual](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/doc/man/man3/notcurses_plane.3.md#L21-L148), [cell manual](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/doc/man/man3/notcurses_cell.3.md#L137-L163), [render manual](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/doc/man/man3/notcurses_render.3.md#L25-L69) | `7767278` | Verified; no physical failure reproduction |
| Input and lifecycle | [input manual](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/doc/man/man3/notcurses_input.3.md#L89-L213), [init manual](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/doc/man/man3/notcurses_init.3.md#L66-L236), [stop manual](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/doc/man/man3/notcurses_stop.3.md#L15-L51) | `7767278` | Verified |
| Terminal query compatibility | [TERMINALS.md](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/TERMINALS.md#L12-L42) | `7767278` | Verified documentation; physical behavior not reproduced |
| Development compatibility response | [issue #2929](https://github.com/dankamongmen/notcurses/issues/2929), [PR #2930](https://github.com/dankamongmen/notcurses/pull/2930), [master commit](https://github.com/dankamongmen/notcurses/commit/b26048eebc74d5d254717d3332fa484718f9efe6) | Development state inspected 2026-07-16 | Reported; not silently applied to v3.0.17 |
| Unit and release testing | [CTest setup](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/CMakeLists.txt#L864-L901), [tests](https://github.com/dankamongmen/notcurses/tree/77672788db0765ab868abafebbaadd8cfe133781/src/tests), [testing checklist](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/doc/testing-checklist.md#L1-L31) | `7767278` | Verified; no PTY/emulator/failure matrix found |
| Python ecosystem | [CFFI README](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/cffi/README.md#L1-L13), [CFFI setup](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/cffi/setup.py#L49-L78), [native build](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/cffi/src/notcurses/build_notcurses.py#L5-L12), [compiled setup](https://github.com/dankamongmen/notcurses/blob/77672788db0765ab868abafebbaadd8cfe133781/python/setup.py#L39-L78), [PyPI 3.0.17](https://pypi.org/project/notcurses/3.0.17/) | `3.0.17`; PyPI release 2025-10-28 | Supported/verified; Beta and native dependency |
