# RFC-004: The status values are `open` and `done`

*Scope: v0 model.*

## Context

A task's status is binary and spelled as a string. What remains is which
two values to spell. The alternatives, and why they lose:

- `open` / `closed` — familiar from issue trackers, but `closed` is not
  `done`: a task can be closed without being done (canceled, won't-do).
- `todo` / `done` — familiar from todo apps, but `todo` is jargon and is
  wrong for a task already started.
- `pending` / `done` — `pending` means waiting on something, not "not
  done".
- `incomplete` / `complete`, `unfinished` / `finished` — unambiguous but
  verbose, and `complete` is a third synonym for `done`.
- `unchecked` / `checked` — mirrors the Markdown checkbox literally, but
  it is a UI metaphor, and `checked` means nothing in JSON or CSV.
- `unresolved` / `resolved` — `resolved` is not `done`; a task can be
  resolved as won't-do.
- `new`, `active`, `ongoing` / `done` — each implies the task has or has
  not started, leaving the other half without a value.

Every candidate adds a meaning (how it ended, what it waits on), a
workflow position, or a UI metaphor. `open` / `done` is the only pair
where each value is exactly the other's negation and nothing more: `done`
is the impulse response of a task that is finished, and a task that is not
finished is still `open`.

## Decision

The two values are `open` and `done`. `done` is a finished task; `open` is
every task that is not done.

## Consequences

The pair reads as a task state, not a control, and maps directly onto the
Markdown checkbox — `- [ ]` is `open`, `- [x]` is `done`. A tool whose
real workflow has more states keeps them in `metadata`, so the choice
loses nothing.
