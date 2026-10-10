# RFC-023: A metadata value is single-line

## Context

Markdown writes a value as the rest of a bullet — `- key: value` — and one
line is all it has. JSON and YAML can hold a line break inside a value, so
whether a document is writable depends on where it is going. Left to each
writer, a line break inside a value means an escape invented here, a
truncated value there, and a document that reads differently in every
tool.

Long-form text already has a field. `notes` is where prose with line breaks
belongs; a metadata value is an attribute.

## Decision

A metadata value must not contain a line break. A document with one is
invalid in every format, JSON and YAML included.

A writer writes a structured value as a single line — a list or an object
as compact JSON — and refuses rather than truncates when a value cannot be
written on one line.

## Consequences

Every value the model can hold is writable in every format, so no format is
lossier than another for this. A tool that must keep a multi-line payload
encodes it inside the value (`\n`) and reads its own encoding back: the
text survives, and the escape stays that tool's business.
