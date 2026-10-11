# RFC-002: The top-level status is binary

*Scope: v0 model.*

## Context

Every task manager distinguishes "not done" from "done", but agrees on
nothing else. Real tools carry `in_progress`, `blocked`, `in_review`,
`backlog`, and more, each with its own meaning and its own vocabulary.
Making the model's `status` that rich would make OPS the union of every
tool's workflow: a tool that does not model `blocked` would have to invent
a value on export, and every other tool would have to guess what a value
means on import.

## Decision

`status` is a binary state: a task is either done or not done. Richer,
tool-specific states belong in `metadata` under a key such as `status`,
where each tool can spell its own without the model growing.

## Consequences

Every document's core is readable by every tool without guessing, and a
tool's real workflow state still survives a round trip in `metadata`.
