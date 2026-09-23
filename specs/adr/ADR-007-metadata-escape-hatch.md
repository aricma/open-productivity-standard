# ADR-007: `metadata` is the escape hatch

**Status:** Accepted.

## Context

The model's field names are fixed single lowercase English words, never
renamed or translated. Tools track far more attributes than title and
status — due dates, priorities, tags, estimates, platform statuses — and
spell them differently. Standardizing each attribute would make the model
a union of every tool's taxonomy, and new attributes keep arriving. An
open field keeps tool-specific data without standardizing it.

## Decision

Every task may carry an optional `metadata` object; tool- or
concern-specific key-value pairs live there, preserved verbatim. New
concepts arrive as metadata, never as new top-level fields.

## Consequences

Round-trips stay lossless: unknown details ride along in `metadata`
instead of being dropped.
