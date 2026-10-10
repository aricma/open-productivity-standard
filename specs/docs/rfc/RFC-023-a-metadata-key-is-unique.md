# RFC-023: A metadata key is unique

## Context

`metadata` is open, so nothing stops a document from writing the same key
twice — a JSON or YAML object with `due_date` repeated, two Markdown
bullets with the same key, two CSV columns with the same header, or a key
in the `metadata` object that a column repeats. A reader that takes one
silently is guessing, and two readers that guess differently read
different documents.

## Decision

A metadata key is unique within a task. A key supplied twice is invalid in
every format: a repeated JSON or YAML key, a repeated Markdown bullet, two
CSV columns with the same header, or the same key in both the `metadata`
object and a column. A reader fails rather than taking the first or the
last, and there is no override.

## Consequences

A lookup returns one value, and a duplicate is an error instead of a coin
flip. A document that repeats a key is simply not valid.
