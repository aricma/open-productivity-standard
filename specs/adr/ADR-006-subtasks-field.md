# ADR-006: `subtasks` instead of `children`

**Status:** Accepted.

## Context

Tools name hierarchy levels differently: epics, features, stories,
subtasks. Modeling named levels would make the standard a union of every
tool's taxonomy, so the hierarchy field must be a neutral, self-explanatory
word — candidates: `children`, `nodes`, `tasks`, `subtasks`. `nodes` is
too abstract; `children` evokes DOM trees. The field names the
objects below a task, so it should say what those objects are and nothing
about the task itself. `tasks` fails: a task with a full `tasks` field
reads as if it isn't itself a task. Every node is a task; `subtasks` says
what the field holds without implying a separate type.

## Decision

Hierarchy is expressed by the `subtasks` field on every task. A task is a
parent exactly when it has `subtasks`; an epic, a feature, and a subtask
are all the same node type. A root is just a task nothing references.

## Consequences

One recursive rule covers the whole forest — no special types, no level
names, no rules about who may be a parent. What a platform calls a task is
derived from position and metadata hints, never enforced.
