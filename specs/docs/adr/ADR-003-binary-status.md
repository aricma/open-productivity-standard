# ADR-003: Binary status (`open` | `done`)

**Status:** Accepted.

## Context

Every task manager distinguishes "not done" from "done", but agrees on
nothing else. Requiring `in_progress`, `blocked`, `in_review`, or
`backlog` would exclude tools that don't model them or force lossy
guessing on import. The two values must be short and self-explanatory:
`open` and `done` are both four characters, a simple balanced pair —
`pending`/`done` are not. This is the same minimum-intersection argument
as ADR-002, applied to statuses instead of fields.

## Decision

The only status values are `open` and `done`. Anything richer is platform
metadata, under a `metadata` key like `status`.

## Consequences

Every document's core is readable by every tool without guessing. Richer
states live in `metadata`, so round-trips stay lossless.
