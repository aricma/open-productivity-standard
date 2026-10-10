# RFC-021: A metadata key is unique and never a field

## Context

`metadata` is open, so nothing stops a key from spelling `title` or
`status` — a word the model already uses for a field — and nothing stops a
document from writing the same key twice. Both leave a reader guessing:
does this `title` mean the field, and which of two `due_date` values is
the task's?

One collision is deliberate. The common keys use `status` for the
platform's own status word (`In review`), which the model's binary
`status` cannot hold. A key that spells a field name is not always a
mistake.

## Decision

Two rules.

A metadata key is unique within a task. A key supplied twice is invalid —
in every format, including those where a duplicate can be written.

Metadata never carries a field. A field's value comes from the field, and
a metadata key that spells the same word is a different datum: `status` in
metadata is the platform's status, never the model's. A reader must not
read a field, or a substitute for one, out of the metadata.

## Consequences

A lookup returns one value, and a duplicate is an error instead of a coin
flip. Spellings that shadow a field stay legal where they are honest —
the same word for closely related data — and a tool that finds a metadata
`title` knows it is the tool's own and not the model's.
