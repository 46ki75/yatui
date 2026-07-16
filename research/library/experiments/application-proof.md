# Facade-Only Application Proof

Experiment dates: 2026-07-16 and 2026-07-17

## Question

Can a downstream ArborUI application implement and deterministically test a
controlled modal form with Unicode editing, keyed application state, focus
trapping, focus restoration, and pointer isolation without importing
implementation crates?

Can the same application add a second screen with externally produced updates,
cooperative cancellation, stale-result rejection, explicit settlement, and
recoverable errors through the public runtime and test facades?

These are the first two bounded slices of the production-scale application
proof. They do not attempt to prove bounded ingress, virtualization, or
comparative ergonomics.

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

The second slice adds a persistent Queue and Activity navigation row. Activity
screen state remains owned by the application while either screen is visible.
Starting activity launches a demonstration producer on an operating-system
thread. It sends generation-tagged items and completion through `EventProxy`.
The application owns a cooperative cancellation signal, advances the generation
when cancelling or restarting, and ignores every item, completion, or failure
whose generation is no longer current.

The activity state machine distinguishes idle, running, cancelled, completed,
and failed states. Failures expose a Retry action. The application retains at
most 32 accepted log items with stable keys and renders the newest first. This
bound limits application memory after update processing; it does not bound the
runtime's ingress queue or provide producer backpressure.

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
- Screen navigation preserves queue and timer state and keeps the keyed
  navigation control focused across recomposition.
- A controlled launcher receives the production `EventProxy` and cancellation
  signal without sleeping in tests.
- Proxy-delivered items settle deterministically through `TestApp::settle`.
- A barrier-coordinated worker thread observes cancellation and submits raced
  items and completion from the preceding generation, which are rejected.
- Failure is recoverable through a new generation; stale completion cannot
  settle the retry.
- Accepted history remains at 32 items, with semantic assertions for the
  retained range.
- Character snapshots cover idle and completed Activity states, including the
  maximum retained-history viewport.

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

The external-work slice required no runtime API change. `EventProxy` is
sufficient for an external producer to submit owned messages while application
updates remain serialized. Cancellation is necessarily cooperative at this
layer, and generation checks are still required because a producer may race the
cancel request with an already prepared item or terminal result.

`TestApp::settle` can deterministically drain messages that a controlled
producer has already submitted. It cannot infer whether an arbitrary external
producer will send more work later. Explicit application settlement state is
therefore part of the tested contract rather than an implication of visual idle.

The second screen also exposed a layout constraint. A nested screen container
with percentage height retained its content-derived minimum and could clip the
persistent navigation under a long log. Giving the screen a zero flex basis and
allowing it to grow within the root keeps the footer visible while the inner
scroll view clips the retained log.

## Limits And Next Evidence

This slice does not complete the production-scale proof. It leaves these
requirements open:

- Bounded ingress and observable backpressure
- Fixed and variable-height visible-range collections
- Select and table controls driven by application requirements
- Form validation and broader loading or error recovery
- A matched Ratatui-plus-application implementation
- Application-level measurements for code size, latency, allocations, emitted
  bytes, idle work, and retained memory
- Integration with a real service, subprocess, or async executor rather than the
  demonstration thread producer

The next application slice should prototype bounded, observable ingress and
choose an explicit reject, coalesce, or replace-latest policy. Virtualization
should remain a separate measured prototype rather than being inferred from the
current scroll views.
