# Collection Lab Ratatui Comparison

This package contains matched Ratatui implementations for ArborUI's Collection
Lab list, table, and scrolling-log experiments. It is excluded from the product
workspace so Ratatui does not become part of ArborUI's facade-only example
dependency graph.

## Comparison Contract

Both implementations use the same application-owned visible-range providers,
generated labels, stable `u64` item keys, row measurements, overscan, viewport
dimensions, and action traces. The primary implementation constructs only the
visible and overscanned rows before painting.

The table workload additionally shares its complete model, responsive column
widths, Unicode records, selection policy, and deterministic producer updates.
ArborUI composes facade-only rows and controlled scrolling; Ratatui renders the
same window through its stateful `Table`. This is an application-level prototype,
not a stabilized ArborUI table widget.

The scrolling-log workload shares a bounded chronological history, stable
monotonic record keys, follow-tail policy, paused viewport anchoring, Unicode
records, and deterministic producer batches. Both sides construct only the
visible and overscanned records. It is application policy rather than a
stabilized log widget.

The deterministic contract covers:

- Fixed and variable-height rendering at explicit terminal sizes
- Active and selected stable identity
- Home, End, line, page, selection, reverse, and resize actions
- Character-frame equivalence after a canonical variable-height trace
- Bounded construction for one million logical rows
- Explicit unchanged redraw as distinct from no requested draw
- Responsive table columns with Unicode content
- Visible and offscreen background updates at stable row keys
- Table character-frame parity through narrow and wide resizes
- Bounded scrolling-log history and construction at one million records
- Follow-tail append, paused append, eviction anchoring, and log-frame parity

The timing benchmark measures a complete logical application turn: update,
visible-row construction, paint, and logical terminal diff. It excludes model
construction and terminal lifecycle. Alternating Down and Up keeps the measured
state bounded and avoids measuring an ever-growing scroll position.

The expanded action matrix uses 100,000 items and persistent fixtures. Untimed
actions restore a deterministic baseline between Page Down, End, resize,
selection, and reverse samples. Unchanged redraw sends Home while Home is
already active. Cold initial render is separate: it includes generated model
construction, logical terminal creation, and the first draw.

This is an application comparison, not a general framework ranking. The shared
visible-range algorithm is application policy and cannot be attributed to either
framework.

## Commands

From the repository root:

```text
just comparison-check
just comparison-bench-smoke
just comparison-bench
just comparison-output-metrics
just comparison-memory-metrics
just comparison-phase-metrics
```

Ratatui is pinned to 0.30.2, matching the research report dated 2026-07-16. The
comparison uses Rust 1.88.0 because that is Ratatui 0.30.2's MSRV; ArborUI's
product workspace remains pinned to Rust 1.85.0. Allocator, phase, latency, and
production ANSI probes remain separate because each instrumentation layer
changes the work being measured.

`comparison-output-metrics` passes real ArborUI patches and Ratatui buffer diffs
through each framework's Crossterm backend under fixed 48x12 ANSI16 conditions.
It reports bytes presented to the writer, writer callback counts, and flushes.
Writer callbacks are serializer operations, not operating-system syscall counts.
The resize case includes Ratatui's production clear before its full draw.

## First Local Result

One optimized run on 2026-07-17 used Rust 1.88.0 under Linux WSL2 on an Intel
Core Ultra 7 255H. Values below are Criterion point estimates for one alternating
Down or Up message through update, construction, paint, logical diff, and the
respective test backend.

Table background updates are deterministic messages so the benchmark isolates
the serialized application and render turn. It does not include producer sleep,
thread scheduling, or queue latency; Focus Queue separately covers bounded
external ingress.

Scrolling-log appends use the same deterministic boundary. A paused append
changes retained application state while preserving the visible projection;
follow-tail append moves the viewport to the newest retained record.

| Rows | Mode | ArborUI | Ratatui |
| ---: | --- | ---: | ---: |
| 1,000 | Fixed | 30.4 us | 9.72 us |
| 100,000 | Fixed | 30.2 us | 9.77 us |
| 1,000,000 | Fixed | 34.0 us | 11.6 us |
| 1,000 | Variable | 36.5 us | 12.7 us |
| 100,000 | Variable | 36.5 us | 12.0 us |
| 1,000,000 | Variable | 38.6 us | 14.3 us |

Both implementations remain approximately flat as logical row count grows,
which is the primary virtualization finding. The latency difference is not an
isolated renderer comparison: ArborUI includes runtime settlement, retained
reconciliation, hit geometry, and cloned test patches, while the Ratatui
application directly updates and redraws its immediate buffer. ArborUI reuses
retained geometry when reconciliation proves a line-navigation turn has no
layout-affecting change.

## Expanded Local Result

The same machine produced these Criterion point estimates on 2026-07-17 for
100,000-item action cases. Cold initial render includes model construction;
other rows exclude untimed baseline resets.

| Scenario | ArborUI fixed | Ratatui fixed | ArborUI variable | Ratatui variable |
| --- | ---: | ---: | ---: | ---: |
| Cold initial render | 16.7 ms | 16.1 ms | 17.0 ms | 16.3 ms |
| Page Down | 84.8 us | 9.65 us | 92.3 us | 11.5 us |
| End | 86.2 us | 9.99 us | 95.8 us | 12.3 us |
| Resize 48x12 to 48x16 | 125 us | 19.4 us | 143 us | 21.6 us |
| Selection | 33.3 us | 9.62 us | 38.4 us | 12.5 us |
| Reverse | 852 us | 740 us | 858 us | 720 us |
| Unchanged redraw | 15.2 us | 8.59 us | 16.0 us | 10.3 us |

Reverse is primarily the shared O(n) application policy: reversing 100,000
items and rebuilding providers. Criterion measured selection improvements of
30.4% fixed and 31.7% variable against the full-layout baseline. Reusing the
committed logical frame for a proven unchanged redraw subsequently improved that
case by 70.5% fixed and 74.2% variable against the immediately preceding stored
Criterion baseline. Reverse remained application dominated.
Conservative damaged-row repaint subsequently reduced fixed and variable
selection by 40.4% and 39.2% against the preceding documented point estimates;
line navigation benefits from the same focus-node damage tracking.

## Table Workload Result

The same 2026-07-17 environment produced these Criterion point estimates for
the matched table workload. The action cases use 100,000 rows unless shown
otherwise:

| Scenario | ArborUI | Ratatui |
| --- | ---: | ---: |
| Line navigation, 1,000 rows | 55.2 us | 198 us |
| Line navigation, 100,000 rows | 54.6 us | 206 us |
| Line navigation, 1,000,000 rows | 55.4 us | 204 us |
| Cold initial render, 100,000 rows | 11.8 ms | 12.3 ms |
| Page Down | 189 us | 204 us |
| Selection | 55.4 us | 215 us |
| Resize 48x12 to 48x16 | 279 us | 223 us |
| Visible background update | 181 us | 190 us |
| Offscreen background update | 32.5 us | 195 us |

Both line-navigation paths remain flat as logical row count grows. Ratatui's
stateful table reconstructs and lays out the visible widget rows on every draw;
ArborUI's retained path is faster for paint-only selection and unchanged visible
content in this workload. Resize remains faster in Ratatui. A visible producer
update changes text and therefore requires ArborUI recomposition and layout;
an offscreen update reconciles to unchanged visible content and reuses the
committed frame.

The production serializer probe reports `bytes/writer calls/flushes`:

| Scenario | ArborUI fixed | Ratatui fixed | ArborUI variable | Ratatui variable |
| --- | ---: | ---: | ---: | ---: |
| Initial render | 5265/3722/1 | 861/542/1 | 5259/3722/1 | 1047/689/1 |
| Page Down | 875/623/1 | 189/159/1 | 1243/905/1 | 249/216/1 |
| End | 1055/767/1 | 207/183/1 | 1095/797/1 | 247/213/1 |
| Resize | 7161/4986/1 | 1101/695/2 | 7155/4986/1 | 1407/926/2 |
| Selection | 785/588/1 | 157/132/1 | 1145/864/1 | 209/183/1 |
| Reverse | 875/623/1 | 189/159/1 | 899/641/1 | 213/177/1 |
| Unchanged redraw | 0/0/0 | 19/12/1 | 0/0/0 | 19/12/1 |

ArborUI's runtime suppresses an empty prepared patch before backend output.
Ratatui still invokes its production draw path for an empty diff, which emits
reset commands and flushes. These figures measure deterministic serialization,
not terminal-driver buffering or transport syscalls.

The corresponding table serializer result is:

| Scenario | ArborUI | Ratatui |
| --- | ---: | ---: |
| Initial render | 5429/3821/1 | 1203/816/1 |
| Page Down | 1129/815/1 | 294/244/1 |
| Resize | 7404/5133/1 | 1663/1131/2 |
| Selection | 370/273/1 | 92/73/1 |
| Visible background update | 102/68/1 | 65/44/1 |
| Offscreen background update | 0/0/0 | 19/12/1 |

The offscreen update changes shared application state but no visible row. ArborUI
therefore submits no patch, while Ratatui's empty production diff still emits
reset commands and one flush.

## Scrolling Log Workload Result

The same environment produced these Criterion point estimates for the matched
bounded scrolling log. Action cases start with 100,000 records; untimed reset
turns restore page and resize baselines:

| Scenario | ArborUI | Ratatui |
| --- | ---: | ---: |
| Line scrolling, 1,000 records | 141 us | 19.7 us |
| Line scrolling, 100,000 records | 136 us | 19.3 us |
| Line scrolling, 1,000,000 records | 147 us | 18.1 us |
| Cold initial render, 100,000 records | 12.7 ms | 12.6 ms |
| Page Up | 162 us | 20.0 us |
| Resize 48x12 to 48x16 | 249 us | 36.4 us |
| Append while following | 182 us | 18.5 us |
| Append while paused | 21.6 us | 15.6 us |

Both line-scrolling paths remain flat through one million retained records.
Ratatui's direct application adapter paints the visible records into its
immediate buffer with substantially less complete-turn overhead for active log
movement. ArborUI's retained path becomes most effective when a paused append
does not alter visible content.

The corresponding production serializer result is:

| Scenario | ArborUI | Ratatui |
| --- | ---: | ---: |
| Initial render | 5273/3718/1 | 1331/918/1 |
| Page Up | 2060/1522/1 | 472/408/1 |
| Resize | 7170/4982/1 | 1823/1268/2 |
| Append while following | 2330/1726/1 | 609/519/1 |
| Append while paused | 0/0/0 | 19/12/1 |

Paused append confirms the same no-visible-change behavior as the table's
offscreen update: ArborUI submits no patch, while Ratatui's empty production
diff emits reset commands and one flush.

## Allocation And Retained Memory

`comparison-memory-metrics` runs every case in a separate release-mode process
using DHAT 0.3.3. The profiler starts immediately before the named operation,
then records total allocations, allocated bytes, peak live bytes, and bytes
still retained while the result is alive. Dropping every measured result
returned the tracked live block and byte counts to zero.

The model and initial-render cases deliberately use different boundaries.
`model` measures generated application data and providers. `initial-render`
constructs that model before profiling, so its retained bytes represent the
framework harness and first settled frame rather than the O(n) item model:

| Items | Model retained | ArborUI fixed | ArborUI variable | Ratatui fixed | Ratatui variable |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1,000 | 148,987 | 97,484 | 92,988 | 82,944 | 82,944 |
| 100,000 | 14,899,987 | 97,484 | 92,988 | 82,944 | 82,944 |
| 1,000,000 | 148,999,987 | 97,484 | 92,988 | 82,944 | 82,944 |

Application-model memory scales linearly as expected. First-frame framework
memory is identical across all three logical collection sizes, which is the
memory-side bounded-virtualization result.

At 100,000 items, cells below are `allocated bytes/retained bytes` for each
isolated operation. Cold includes model construction; the other action fixtures
are constructed before profiling.

| Scenario | ArborUI fixed | Ratatui fixed | ArborUI variable | Ratatui variable |
| --- | ---: | ---: | ---: | ---: |
| Cold | 26,196,910/14,997,471 | 25,982,881/14,982,931 | 26,170,015/14,992,975 | 25,982,881/14,982,931 |
| Page Down | 122,177/44,884 | 0/0 | 106,354/42,772 | 0/0 |
| Resize | 302,653/123,428 | 165,888/165,888 | 267,462/118,988 | 165,888/165,888 |
| Selection | 73,621/44,692 | 0/0 | 73,054/42,532 | 0/0 |
| Reverse | 2,520,281/2,444,860 | 2,400,008/2,400,008 | 2,498,714/2,440,700 | 2,400,008/2,400,008 |
| Unchanged redraw | 56,648/39,892 | 0/0 | 47,880/35,492 | 0/0 |

The table model retains 152,000 bytes at 1,000 rows, 15,200,000 at 100,000, and
152,000,000 at one million on both sides. With model construction outside the
profiled boundary, first-render retained framework memory stays exactly 127,844
bytes for ArborUI and 82,944 bytes for Ratatui at all three sizes.

The shared log model retains 96,000 bytes at 1,000 records, 9,600,000 at
100,000, and 96,000,000 at one million on both sides. With model construction
outside the profiled boundary, first-render retained framework memory remains
approximately 101,200 bytes for ArborUI and exactly 82,944 bytes for Ratatui across
all three sizes. The first append into the configured spare history grows the
shared `VecDeque`; that application-owned capacity allocation is present on
both sides and is not attributed to either framework.

At 100,000 rows, table operation cells below are `allocated bytes/retained
bytes`:

| Scenario | ArborUI | Ratatui |
| --- | ---: | ---: |
| Page Down | 528,522/76,620 | 215,868/0 |
| Resize | 789,210/153,484 | 380,212/165,888 |
| Selection | 142,306/71,788 | 219,812/0 |
| Visible background update | 474,858/70,228 | 209,764/8 |
| Offscreen background update | 134,048/69,748 | 217,572/8 |
| Unchanged redraw | 134,040/69,740 | 225,740/0 |

## ArborUI Phase Attribution

ArborUI exposes opt-in timings for view construction, staged reconciliation,
layout, paint, diff, commit, post-commit refresh, and combined terminal backend
validation/serialization/write. Untimed rendering does not read the clock. The
headless comparison report averages 100 action samples and 20 initial-render
samples; initial render excludes model construction. Selected columns are shown
below in nanoseconds, while `comparison-phase-metrics` prints every phase.

| Mode | Scenario | Update | Stage/reconcile | Layout | Paint | Diff | Render total |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Fixed | Initial render | 0 | 15,666 | 67,588 | 62,834 | 36,816 | 207,911 |
| Fixed | Page Down | 427 | 4,492 | 30,634 | 40,894 | 5,467 | 91,853 |
| Fixed | End | 826 | 4,537 | 28,576 | 42,704 | 6,866 | 94,575 |
| Fixed | Resize | 2,977 | 5,637 | 37,624 | 51,972 | 19,638 | 130,267 |
| Fixed | Selection | 454 | 4,383 | 0 | 13,102 | 4,877 | 33,145 |
| Fixed | Reverse | 843,805 | 8,715 | 41,102 | 48,248 | 6,976 | 121,301 |
| Fixed | Unchanged redraw | 412 | 4,135 | 0 | 2,440 | 2,524 | 19,564 |
| Variable | Initial render | 0 | 12,807 | 65,707 | 65,413 | 16,983 | 184,606 |
| Variable | Page Down | 508 | 3,376 | 31,558 | 44,094 | 6,377 | 95,227 |
| Variable | End | 987 | 3,720 | 32,971 | 49,173 | 7,398 | 104,234 |
| Variable | Resize | 2,332 | 4,117 | 39,408 | 59,328 | 17,975 | 135,944 |
| Variable | Selection | 404 | 3,195 | 0 | 18,704 | 6,207 | 38,316 |
| Variable | Reverse | 812,190 | 6,817 | 42,408 | 52,977 | 7,209 | 126,907 |
| Variable | Unchanged redraw | 347 | 3,232 | 0 | 1,146 | 2,316 | 15,545 |

The table phase report adds:

| Scenario | Update | Stage/reconcile | Layout | Paint | Diff | Render total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Initial render | 0 | 25,669 | 127,876 | 62,387 | 13,855 | 262,082 |
| Page Down | 510 | 13,034 | 91,617 | 51,931 | 6,438 | 183,584 |
| Resize | 6,181 | 13,546 | 106,045 | 69,807 | 15,976 | 230,636 |
| Selection | 316 | 10,358 | 0 | 14,768 | 4,125 | 48,446 |
| Visible update | 580 | 11,389 | 81,373 | 50,918 | 2,830 | 165,987 |
| Offscreen update | 403 | 11,240 | 0 | 1,365 | 2,145 | 34,773 |

The scrolling-log phase report adds:

| Scenario | Update | Stage/reconcile | Layout | Paint | Diff | Render total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Initial render | 0 | 16,976 | 103,629 | 83,950 | 31,207 | 263,414 |
| Page Up | 492 | 4,905 | 58,632 | 61,406 | 11,202 | 150,966 |
| Resize | 1,971 | 4,732 | 64,640 | 81,978 | 19,149 | 188,356 |
| Append while following | 25,213 | 4,773 | 51,458 | 61,662 | 13,432 | 146,304 |
| Append while paused | 21,476 | 4,581 | 0 | 1,519 | 2,406 | 21,266 |

Paused append performs the shared record-formatting and storage update but
reconciles to unchanged visible content, skips layout, and reuses committed
logical output. Follow-tail append changes the visible window and follows the
normal layout and paint path.

Ordinary preparation skips layout when reconciliation reports only paint or no
changes. Paint-only work against the exact committed renderer generation clones
committed logical state, clears one full-width band covering invalid rows, and
replays intersecting painters in normal order. A no-change result reuses the
clone without invoking paint callbacks. Physical-state invalidation and renderer
mismatch retain complete-paint behavior, while `UiTree::prepare_full` remains a
separately callable full-layout/full-paint reference path. Hand-selected and
deterministic generated transitions compare complete buffers, patches, hit maps,
retained geometry, and committed renderer state. Reverse remains dominated by
the application-owned O(n) update. Ratatui's internal phase boundaries are not
exposed, so its comparison remains the complete-turn Criterion result rather
than a fabricated phase split.
