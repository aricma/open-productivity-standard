# ADR-005: `notes` is a model field, not metadata

**Status:** Accepted.

## Context

After the title, long-form text is the most universal content: every tool
has a description, body, or comment field. It is the task's content, not
data about the task. It can be long, multi-line, and Markdown-formatted —
awkward as a single-line CSV cell.

## Decision

`notes` is an optional top-level string field (plain or Markdown).

## Consequences

Long-form content gets an explicit, uniform mapping in every
serialization; a tool without notes omits the field.
