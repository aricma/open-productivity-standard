# RFC-005: Long-form text is an optional string

## Context

After the title, long-form text is the most universal content: every tool
has a description, body, or comment field. It is the task's content, not
data about the task. It can be long, multi-line, and Markdown-formatted —
awkward as a single-line CSV cell.

## Decision

The field is an optional string: plain multi-line text.

Markdown is a valid value, not a separate type. We do not distinguish
formatted from unformatted text, so there is no rich-text structure in the
model and no per-format variant of the field.

## Consequences

Every serialization maps the same type — one optional string — and a tool
that stores plain text and one that stores Markdown both round-trip
through it unchanged. A tool with no long-form content omits the field.

A tool that wants to render Markdown renders it; one that wants to show
text shows it. The choice of how to interpret the value is the reader's,
not something the document has to declare.
