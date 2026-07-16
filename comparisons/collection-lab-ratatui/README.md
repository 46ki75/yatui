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
```

Ratatui is pinned to 0.30.2, matching the research report dated 2026-07-16. The
comparison uses Rust 1.88.0 because that is Ratatui 0.30.2's MSRV; ArborUI's
product workspace remains pinned to Rust 1.85.0. Allocator and production
ANSI-byte adapters remain separate follow-up measurements because allocator
instrumentation perturbs latency and logical test backends do not exercise
production serialization.

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
| 1,000 | Fixed | 83.9 us | 9.50 us |
| 100,000 | Fixed | 82.1 us | 9.53 us |
| 1,000,000 | Fixed | 80.5 us | 12.4 us |
| 1,000 | Variable | 95.2 us | 11.8 us |
| 100,000 | Variable | 94.9 us | 11.2 us |
| 1,000,000 | Variable | 93.3 us | 11.4 us |

The million-row fixed Ratatui sample had substantial outliers and a wide
11.3-13.6 us confidence interval. Both implementations remain approximately
flat as logical row count grows, which is the primary virtualization finding.
The latency difference is not an isolated renderer comparison: ArborUI includes
runtime settlement, retained reconciliation, layout, hit geometry, and cloned
test patches, while the Ratatui application directly updates and redraws its
immediate buffer.

## Expanded Local Result

The same machine produced these Criterion point estimates on 2026-07-17 for
100,000-item action cases. Cold initial render includes model construction;
other rows exclude untimed baseline resets.

| Scenario | ArborUI fixed | Ratatui fixed | ArborUI variable | Ratatui variable |
| --- | ---: | ---: | ---: | ---: |
| Cold initial render | 20.6 ms | 28.7 ms | 19.5 ms | 24.0 ms |
| Page Down | 118 us | 14.0 us | 143 us | 12.1 us |
| End | 97.7 us | 10.5 us | 103 us | 13.0 us |
| Resize 48x12 to 48x16 | 142 us | 20.0 us | 143 us | 22.9 us |
| Selection | 80.1 us | 9.49 us | 91.4 us | 11.8 us |
| Reverse | 817 us | 755 us | 839 us | 780 us |
| Unchanged redraw | 78.8 us | 8.60 us | 87.9 us | 11.7 us |

Cold initial render and fixed Page Down had wide intervals and substantial
outliers. Reverse is primarily the shared O(n) application policy: reversing
100,000 items and rebuilding providers. The other cases retain the same
framework-level work differences described above.

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
