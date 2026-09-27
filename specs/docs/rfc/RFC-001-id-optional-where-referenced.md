# RFC-001: `id` is optional — required only where a format references by id

## Context

OPS supports two task structures, each for its own optimization:

- **Flat** — for streaming large amounts of data.
- **Nested** — for parsing without resolving references, and because nested
  structures read more simply in plain-text formats like JSON, YAML, and
  Markdown.

A nested structure carries hierarchy inline: a child sits inside its
parent, so no reference has to be resolved and an id is only a value a
tool may attach. A flat record stands alone, so the only way to express
parentage is a reference — and that reference is the task's `id`. What
`id` is for depends on the structure, not the task.

## Decision

`id` is optional on every task. A flat serialization requires it on every
task it references; a nested serialization never needs it. An id, when
present, is unique across the whole graph.

## Consequences

`id` exists for the flat structure. A nested document carries hierarchy
inline and never needs one; a flat document requires it on every record it
references. A converter adds ids when flattening and may drop them when
nesting.

The requirement is per serialization: the same dataset is valid in JSON
with no ids and valid in JSONL only once every referenced record has one.
Roots are never referenced, so no format needs an id on a root.

An id is about parentage, not linkage. A `subtasks` entry points at a
child's id so the tree can be rebuilt, and that is the only relation an id
expresses. A relation between tasks that is orthogonal to the hierarchy —
blocking, duplicating, relating — belongs in `metadata`, where a tool can
define it without the model growing.

An id is also local to the document. It is not a permanent identity, and
it is not how you find the same task across tools or across a round trip —
another tool may reassign or drop it. If you need identity that survives,
keep a tool-scoped key in `metadata`, such as `jira_id: "<uuid>"`, and let
each tool keep its own.
