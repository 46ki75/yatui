# Collection Lab Ratatui Comparison

This package contains the matched Ratatui implementation for ArborUI's
Collection Lab experiment. It is excluded from the product workspace so Ratatui
does not become part of ArborUI's facade-only example dependency graph.

## Comparison Contract

Both implementations use the same application-owned visible-range providers,
generated labels, stable `u64` item keys, row measurements, overscan, viewport
dimensions, and action traces. The primary implementation constructs only the
visible and overscanned rows before painting.

The deterministic contract covers:

- Fixed and variable-height rendering at explicit terminal sizes
- Active and selected stable identity
- Home, End, line, page, selection, reverse, and resize actions
- Character-frame equivalence after a canonical variable-height trace
- Bounded construction for one million logical rows
- Explicit unchanged redraw as distinct from no requested draw

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

| Rows | Mode | ArborUI | Ratatui |
| ---: | --- | ---: | ---: |
| 1,000 | Fixed | 53.7 us | 9.44 us |
| 100,000 | Fixed | 56.3 us | 9.35 us |
| 1,000,000 | Fixed | 56.3 us | 9.45 us |
| 1,000 | Variable | 63.1 us | 11.4 us |
| 100,000 | Variable | 64.0 us | 11.8 us |
| 1,000,000 | Variable | 64.5 us | 11.8 us |

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
| Selection | 55.9 us | 9.50 us | 63.2 us | 11.6 us |
| Reverse | 852 us | 740 us | 858 us | 720 us |
| Unchanged redraw | 15.2 us | 8.59 us | 16.0 us | 10.3 us |

Reverse is primarily the shared O(n) application policy: reversing 100,000
items and rebuilding providers. Criterion measured selection improvements of
30.4% fixed and 31.7% variable against the full-layout baseline. Reusing the
committed logical frame for a proven unchanged redraw subsequently improved that
case by 70.5% fixed and 74.2% variable against the immediately preceding stored
Criterion baseline. Reverse remained application dominated.

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
| Reverse | 2,520,281/2,444,860 | 2,400,008/2,400,008 | 2,498,714/2,440,700 | 2,400,008/2,400,008 |
| Unchanged redraw | 56,648/39,892 | 0/0 | 47,880/35,492 | 0/0 |

## ArborUI Phase Attribution

ArborUI exposes opt-in timings for view construction, staged reconciliation,
layout, paint, diff, commit, post-commit refresh, and combined terminal backend
validation/serialization/write. Untimed rendering does not read the clock. The
headless comparison report averages 100 action samples and 20 initial-render
samples; initial render excludes model construction. Selected columns are shown
below in nanoseconds, while `comparison-phase-metrics` prints every phase.

| Mode | Scenario | Update | Stage/reconcile | Layout | Paint | Diff | Render total |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Fixed | Initial render | 0 | 12,146 | 51,492 | 45,810 | 28,307 | 157,088 |
| Fixed | Page Down | 458 | 3,963 | 25,391 | 36,238 | 5,015 | 80,281 |
| Fixed | End | 755 | 3,806 | 23,488 | 34,061 | 5,009 | 75,122 |
| Fixed | Resize | 2,744 | 4,298 | 29,798 | 44,376 | 15,095 | 105,024 |
| Fixed | Selection | 365 | 3,687 | 0 | 33,449 | 4,356 | 50,041 |
| Fixed | Reverse | 709,661 | 6,440 | 31,063 | 39,010 | 5,113 | 94,687 |
| Fixed | Unchanged redraw | 306 | 3,483 | 0 | 978 | 2,114 | 14,728 |
| Variable | Initial render | 0 | 11,174 | 57,793 | 54,901 | 13,756 | 157,272 |
| Variable | Page Down | 524 | 3,237 | 31,173 | 43,324 | 7,205 | 93,997 |
| Variable | End | 696 | 3,212 | 27,940 | 43,901 | 6,726 | 91,092 |
| Variable | Resize | 1,941 | 3,201 | 32,376 | 50,863 | 15,161 | 112,567 |
| Variable | Selection | 392 | 3,092 | 0 | 43,096 | 5,711 | 62,432 |
| Variable | Reverse | 670,477 | 5,062 | 32,715 | 46,046 | 5,977 | 102,201 |
| Variable | Unchanged redraw | 338 | 2,669 | 0 | 1,029 | 2,096 | 16,803 |

Ordinary preparation skips layout when reconciliation reports only paint or no
changes. A no-change result against the exact committed renderer generation also
clones committed logical state without invoking paint callbacks. Physical-state
invalidation and renderer mismatch retain complete-paint behavior, while
`UiTree::prepare_full` remains a separately callable full-layout/full-paint
reference path. Hand-selected and deterministic generated transitions compare
complete buffers, patches, hit maps, retained geometry, and committed renderer
state. Paint remains the largest measured phase for selection. Reverse remains
dominated by the application-owned O(n) update. Ratatui's internal phase
boundaries are not exposed, so its comparison remains the complete-turn
Criterion result rather than a fabricated phase split.
