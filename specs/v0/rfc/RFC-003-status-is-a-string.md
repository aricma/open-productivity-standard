# RFC-003: `status` is a string, not a boolean

*Scope: v0 model.*

## Context

A binary state could be a boolean — `is_done: true` — and a field name
plus a boolean is compact in code. But the value has to read in the
plain-text formats OPS targets, and `true` / `false` does not: it is a
machine value, not a task state, and it does not explain itself in a JSON,
YAML, CSV, or Markdown document a person reads.

A boolean is also not one representation. Formats spell it differently —
`true` / `false`, `0` / `1`, `yes` / `no` — and CSV has no boolean type at
all, so the value would be inferred or quoted anyway. The compactness is
mostly imaginary too: `true` and `false` are four and five characters,
against four each for `open` and `done`, and a JSON boolean saves only its
quotes.

A string sidesteps all of it. It is the plain-text default in every
format, it says what it means, and it leaves the representation open to
grow.

## Decision

`status` is a string, not a boolean — specifically a two-value string
enum that represents the binary state. The two values it takes are a
separate decision.

## Consequences

Any format a person reads shows the state as a word, and a reader never
has to interpret a format's boolean dialect. Because the value is a
string, a richer set could be added later without changing the field's
type — a decision for a future RFC, not a promise.
