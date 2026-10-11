# RFC-017: The core data model

*Scope: v0 model.*

## Context

The model has six fields: `title` and `status` required (DR-001), `id`,
`notes`, and `subtasks` optional, and `metadata`, the escape hatch.
Conformance below full support has to name what a tool must
handle, and "everything except `metadata`" is the line that matters — the
whole task structure without the metadata burden.

Saying only `title` and `status` went too far the other way. It let a
Level 2 tool discard `subtasks` and flatten the tree, and a migration that
loses the hierarchy is not a migration.

## Decision

OPS defines the **core model**: every field except `metadata`.

```text
Core {
    title:    string             required
    status:   "open" | "done"    required
    id:       string             optional
    notes:    string             optional
    subtasks: task[]             optional
}
```

It is the task model with the escape hatch removed. `subtasks` holds
nested tasks or child-id references, per serialization. `id` is optional
and required only where a flat serialization references tasks by id: a
nested structure omits ids, a flat one needs them to reference a child.

A Level 1 or Level 2 tool handles the core model; a Level 3 tool handles
the core model plus `metadata`.

## Consequences

Level 2 keeps its hierarchy: a tool can drop `metadata` and still
round-trip the task tree, which is what a migration needs. Only the
metadata burden stays a full-support requirement, so DR-001's promise —
any tool can write and read a valid document without inventing data —
holds down to title and status.

The core model is a subset, not a second type. A core document is a valid
OPS document; a full document is a core document with `metadata`. No tool
has to model `metadata` to understand the core, and no format changes: the
core is expressed in the same serializations, with `metadata` absent.
