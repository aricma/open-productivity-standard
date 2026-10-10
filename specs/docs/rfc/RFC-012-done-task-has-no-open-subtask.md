# RFC-012: A `done` task has no open subtask

## Context

`status` is binary and a task may nest (DR-002), which leaves one
combination undefined: a task marked `done` whose subtask is still `open`.
Tools that export a finished project with a leftover open item produce
exactly this, so it appears in real data.

Two readings were available. Treat `status` as purely local — each task
says only whether it is finished, and a parent may be `done` while work
below it is not. Or treat it as consistent — finishing a task finishes
what it contains, so a `done` parent implies every subtask is `done`.

## Decision

A `done` task must have no open subtask; every subtask of a `done` task
is itself `done`. An `open` task may contain subtasks of either status.
The rule is a reader obligation: readers must reject a document that
breaks it.

Applying it at every task makes it transitive — a `done` root implies all
of its descendants are `done` — without any extra traversal.

## Consequences

The graph is internally consistent: a task cannot claim to be finished
while its contents are not, and a `done` root is a complete, finished
subtree.

The trade-off is a hard interop constraint. A tool that exports a done
parent with a leftover open child now emits an invalid document and must
resolve it before exporting — for example by keeping the parent `open`,
or by closing the child. The rejected alternative, leaving `status`
local, would accept those exports but would make "is this task
finished?" depend on whether its subtasks are read too.
