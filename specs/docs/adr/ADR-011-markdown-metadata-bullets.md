# ADR-011: Markdown metadata is a bulleted `key: value`

**Status:** Accepted.

## Context

An earlier draft hid per-task metadata in HTML comments
(`<!-- metadata: {...} -->`). Comments are invisible in rendered Markdown,
so the data is only readable in source and does not survive editors that
strip comments. Users want to see and edit metadata where they see the
task.

Two alternatives were tried and rejected. Indenting metadata one level
below notes puts a parent's metadata below its own child markers, grows
the indent on every nesting level, and leaves a plain `key: value` line in
notes ambiguous. Escaping notes lines with a leading `|` is unambiguous,
but the markers render literally and a run of them can trip GFM table
detection.

## Decision

Metadata is a bulleted `key: value` line at the task's content indent —
`- key: value`, with a key from rule 5's charset. Notes are the plain
lines at that indent, so a plain `key: value` stays prose and only a
bullet is metadata. Child tasks are `- [ ]` / `- [x]` bullets at the same
indent. The root's metadata moves from frontmatter into the body;
frontmatter keeps only `version` and `status`.

## Consequences

Task data is readable and editable in rendered Markdown, metadata reads as
an ordinary list, and notes and metadata never collide — no extra
indentation, no escape characters. The trade-off: a notes list item that
looks like `- key: value` or `- [ ]` is read as metadata or a child, and
blank lines inside notes are not preserved.
