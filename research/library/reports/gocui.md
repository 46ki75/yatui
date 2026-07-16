# gocui Research Survey

## Evidence Header

```text
Research date: 2026-07-16
Project version: github.com/jroimartin/gocui v0.5.0 (latest tagged module)
Project revision: de100503e6c3afcfd3ed101e7263ada4166f634c
Repository: https://github.com/jroimartin/gocui
Documentation version: pkg.go.dev gocui@v0.5.0 and pinned doc.go; accessed 2026-07-16
Primary platform examined: Linux source inspection; termbox-go Unix/Windows scope checked; no physical terminal reproduction
Report depth: Brief survey
```

**Project and version:** gocui is a Go 1.16, BSD-3-Clause package built on
`termbox-go` v1.1.1. `v0.5.0`, released 2021-08-14, is the latest tagged Go
module on the research date. The Go proxy also lists the unreleased
[pseudo-version](https://proxy.golang.org/github.com/jroimartin/gocui/@v/v0.0.0-20250501121535-0e75b37a4ce7.info)
`v0.0.0-20250501121535-0e75b37a4ce7`, which corresponds to the default branch.
That branch's latest commit only removed a stale README project list; no later
tagged release exists. This is maintenance-light rather than actively evolving.
Source behavior below is the immutable `v0.5.0` tag, not an unreleased default-branch assumption ([release and commit](https://github.com/jroimartin/gocui/releases/tag/v0.5.0),
[current-branch commit](https://github.com/jroimartin/gocui/commit/0e75b37a4ce7cb6e09f23ccd71a7d466301a7467)).

**Category and scope:** gocui is a **minimal retained view-based UI package**
in the rendering/widget-library category. `Gui` owns a full-screen native
terminal through termbox, named `View` buffers, keybindings, and a synchronous
event/redraw loop. It targets Unix and Windows terminal implementations supplied
by termbox, not inline output, remote transports, or headless applications.

**Distinctive strength:** A view is a durable named window implementing
`io.ReadWriter`; it retains text, cursor, origin, wrapping, scrolling, editing,
colors, and frame state. `Manager.Layout` runs on every loop iteration, so
applications can recompute manual coordinate layouts after resize. Views overlap
in insertion order and can be moved to the top or bottom, making simple popups
cheap. Global or view-specific keybindings, optional mouse bindings, one focused
`CurrentView`, and a replaceable `Editor` cover basic forms and navigation. This
is an unusually small path from `NewGui` to a working dashboard, but focus is a
single pointer and overlays have no modal stack or propagation policy
([pinned API documentation](https://github.com/jroimartin/gocui/blob/de100503e6c3afcfd3ed101e7263ada4166f634c/doc.go),
[views and focus](https://github.com/jroimartin/gocui/blob/de100503e6c3afcfd3ed101e7263ada4166f634c/gui.go#L125-L258)).

`MainLoop` polls termbox in a goroutine, accepts queued `Update` callbacks, and
flushes after each event batch. Each flush clears termbox's back buffer, reruns
managers, redraws every view, detects size changes, and calls `termbox.Flush`;
resize therefore has a simple redraw path but no explicit resize transaction.
`NewGui` enters termbox's raw/alternate-screen lifecycle and `Close` restores it
only when called. gocui's cell model stores one rune per logical cell: wrapping
uses rune counts, with no grapheme segmentation or gocui-level width policy.
termbox applies runewidth during output, so wide and combining text can diverge
from gocui's cursor, wrap, and hit-test coordinates
([pinned implementation](https://github.com/jroimartin/gocui/blob/de100503e6c3afcfd3ed101e7263ada4166f634c/gui.go#L349-L467),
[text model](https://github.com/jroimartin/gocui/blob/de100503e6c3afcfd3ed101e7263ada4166f634c/view.go#L198-L360)).

**Important limitation or tradeoff:** **Classification: limitation relative
to ArborUI's recoverable-output requirement.** gocui's `flush` discards the
error returned by `termbox.Flush`; termbox updates its front buffer before its
final `io.Copy` can fail. A partial write can therefore leave physical output
uncertain while the library has no applied/deferred/unknown result or automatic
full repaint. The workaround is to terminate and reinitialize, or fork/wrap the
global termbox path and add invalidation; either gives up the minimal lifecycle
and carries terminal-ownership cost. Evidence status is verified for the
source/API boundary; physical partial-write behavior was not reproduced here
([gocui flush](https://github.com/jroimartin/gocui/blob/de100503e6c3afcfd3ed101e7263ada4166f634c/gui.go#L421-L467),
[termbox flush](https://github.com/nsf/termbox-go/blob/2ff630277754813b198ae96036e28e254d2c72bf/api.go#L170-L220)).

**Testing observation:** No `*_test.go`, mock screen, event driver, snapshot
helper, or complete-application harness was found in the tagged gocui tree.
`View.Buffer` and related accessors permit limited content assertions, but
public `NewGui` immediately initializes a real termbox terminal. Complete-app
tests therefore need a PTY or a forked abstraction; input injection, controlled
clocks, settlement, Unicode boundaries, resize/lifecycle, and failed-output
recovery are not covered ([tagged tree](https://github.com/jroimartin/gocui/tree/de100503e6c3afcfd3ed101e7263ada4166f634c)).

**Relevance to ArborUI:** Retain gocui's inexpensive named identity, explicit
view z-order, application-friendly `Update` boundary, and custom-editor seam.
ArborUI must go further with width-correct text, explicit focus/overlay routing,
prepared-frame commit and physical-state invalidation, panic/suspend recovery,
and a deterministic complete-application harness. gocui is a useful minimal
counterexample, not evidence that those runtime guarantees are unnecessary.

**Best dated sources:** All sources were accessed 2026-07-16. The [Go module
metadata](https://proxy.golang.org/github.com/jroimartin/gocui/@v/v0.5.0.info) and
[GitHub release](https://github.com/jroimartin/gocui/releases/tag/v0.5.0) date the
baseline to 2021-08-14; the [pinned module file](https://github.com/jroimartin/gocui/blob/de100503e6c3afcfd3ed101e7263ada4166f634c/go.mod),
[pinned implementation](https://github.com/jroimartin/gocui/tree/de100503e6c3afcfd3ed101e7263ada4166f634c),
and [termbox dependency source](https://github.com/nsf/termbox-go/tree/2ff630277754813b198ae96036e28e254d2c72bf) provide the
version-matched behavior evidence.
