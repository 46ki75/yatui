# Facade-Only Application Proof

Experiment date: 2026-07-16

## Question

Can a downstream ArborUI application implement and deterministically test a
controlled modal form with Unicode editing, keyed application state, focus
trapping, focus restoration, and pointer isolation without importing
implementation crates?

This is the first bounded slice of the production-scale application proof. It
does not attempt to prove streaming effects, multiple screens, virtualization,
or comparative ergonomics.

## Implementation

The existing [`Focus Queue`](../../../examples/focus-queue/) pilot now supports
editing a keyed task in a modal form. The form contains:

- A controlled grapheme-aware title input
- A controlled completion checkbox
- Explicit Save and Cancel actions
- Escape dismissal and scrim pointer isolation
- Forward and reverse focus traversal within the dialog
- Restoration to the originating keyed Edit control

The application continues to depend only on `arborui`. Its integration tests
use `arborui-test` plus the snapshot assertion library. The application does not
import an ArborUI implementation crate.

Two application-driven public widgets were added:

- `Checkbox`, which borrows a boolean value and emits the next value
- `Dialog`, which fills its containing overlay region with a focus scope,
  centers caller-supplied content, blocks lower pointer targets, and emits
  dismissal from Escape

Both widgets retain no application references or callbacks beyond the
frame-local `Element` value.

## Deterministic Evidence

The public application harness verifies:

- Opening the dialog focuses its title input.
- Tab and Shift-Tab wrap inside the active focus scope.
- Cancel preserves the original task and restores focus to its keyed Edit
  control.
- A scrim click remains in the dialog without activating the covered task row.
- Editing `a👩‍💻界` can delete the ZWJ emoji as one grapheme and save `a界`.
- Saving the controlled checkbox updates the task and summary.
- Character snapshots cover the open dialog and saved Unicode state, with
  semantic assertions for model and focus state.

The widget unit tests independently verify checkbox activation and that a dialog
owns focus, handles Escape, and replaces lower pointer targets.

Run the focused evidence with:

```console
cargo test -p arborui-widgets --all-features
INSTA_UPDATE=no cargo test -p arborui-example-focus-queue --test focus_queue --all-features
```

## Finding

The public boundary is sufficient for this modal form without application-level
focus flags or manual event broadcasting. The retained focus scope governs
keyboard traversal. Composited hit testing, explicit pointer-modal routing, and
captured-sequence suppression jointly govern pointer isolation.

The experiment also found a concrete composition constraint: the overlay host
must remain structurally stable while a dialog opens. Conditionally replacing
the application root with a stack removes the previously focused retained node,
so there is no identity to restore. Focus Queue now always renders the same stack
host and conditionally adds the keyed dialog layer. This preserves the
application subtree and restores the exact originating control.

## Limits And Next Evidence

This slice does not complete the production-scale proof. It leaves these
requirements open:

- Multiple application screens and screen-owned state
- External streaming work, cancellation, stale results, and effect settlement
- Bounded ingress and observable backpressure
- Fixed and variable-height visible-range collections
- Select and table controls driven by application requirements
- Validation and recoverable loading or error states
- A matched Ratatui-plus-application implementation
- Application-level measurements for code size, latency, allocations, emitted
  bytes, idle work, and retained memory

The next application slice should add external work and explicit settlement to a
second screen before collection APIs are selected. Virtualization should remain
a separate measured prototype rather than being inferred from the current
scroll view.
