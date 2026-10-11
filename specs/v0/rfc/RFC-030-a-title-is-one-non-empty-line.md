# RFC-030: A title is one non-empty line

*Scope: v0 model.*

## Context

`title` is required, and the standard calls it "the line" — the one line
that says what the task is. Nothing yet constrains the value itself, so
both `""` and `"two\nlines"` are accepted on the wire today.

Neither survives every serialization. Markdown writes a heading as `#`
followed by the title, so a line break inside the title becomes a second
line — read back as a note, or as a task — and an empty title writes
`# `, which reads back as a task with no title at all. The other formats
carry both, so the same task is valid in JSON and invalid in Markdown.

## Decision

A title is one non-empty line: `title` contains no line break and is not
the empty string.

## Consequences

Every serialization carries a title the same way, so a task read from one
format writes to another and reads back unchanged. A writer that would
otherwise emit a heading with nothing after it, or a line that splits its
own title, gets a rejection instead of a document that reads back as
something else.

The constraint is on the value, so it holds in every serialization and not
only in Markdown — the same reasoning as the H1 rule for `notes`
(RFC-007).
