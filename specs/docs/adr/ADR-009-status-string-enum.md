# ADR-009: `status` is a string enum, not a boolean

**Status:** Pending.

## Context

"Done or not" is binary and could be a boolean — `is_done: false`. But a
boolean is a closed type: adding `in_progress` or `blocked` later would
change the field's type and break every reader. A string enum carries the
same binary today and can gain values without a schema change. `status`
also reads as a state, not a predicate — `"open"` / `"done"` mirror the
model's "[ ] / [x]" line, where `is_done: false` asks the reader to
negate.

## Decision

`status` stays a string enum with the values `open` and `done`.

## Consequences

The binary stays explicit, and the field can grow new values without
breaking existing documents or readers.
