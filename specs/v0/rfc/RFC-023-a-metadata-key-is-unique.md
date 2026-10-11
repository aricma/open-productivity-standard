# RFC-023: A metadata key is unique

*Scope: v0 model.*

## Context

`metadata` is open, so nothing stops a document from writing the same key
twice. A reader that ends up holding two values for one key has to guess
which one is the task's, and two readers can guess differently.

## Decision

A metadata key is unique within a task.

- **Given** a task, **when** a metadata key is looked up, **then** it maps
  to at most one value.

## Consequences

A lookup returns one value or none. How a format resolves a repeated key
is left to the parser of that format.
