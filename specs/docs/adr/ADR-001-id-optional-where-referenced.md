# ADR-001: `id` is optional — required only where a format references by id

**Status:** Accepted.

## Context

Trees serialize two ways: nested objects, or flat lists that map edges by
id. Nested formats need no ids; flat formats need ids to reference nodes.

## Decision

`id` is optional on every task. Flat formats require ids on non-root
tasks. An id, when present, is unique across the whole graph.

## Consequences

Nested documents stay minimal; flat documents get a stable reference
key; roots never need an id but may carry one.
