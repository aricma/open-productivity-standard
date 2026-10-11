# RFC-031: The standard fixes the model, not the bytes

*Scope: v0 model.*

## Context

Two tools that claim the same format have to exchange documents, so
something has to be fixed. Bytes are the easy thing to fix and the wrong one:
key order, indentation, quoting style, and a trailing newline are layout,
every serializer spells them differently, and pinning them would make the
standard a serialization format instead of a data model — and would fail
every tool whose writer already emits valid, readable documents another way.

## Decision

What OPS fixes is the model and the mapping onto each syntax: the field names,
the shapes of the values, the rules, and what each construct means. Byte
layout is not part of the standard.

Two consequences carry the weight instead:

- **Comparison is semantic.** Two documents are the same when they read as
  the same model, so a conformance case compares models and never bytes.
- **A writer's output is stable.** Reading your own bytes and writing them
  again produces the same bytes, so reformatting a file never churns it, and
  a document has one spelling per implementation rather than one for
  everybody.

## Consequences

A tool may lay a document out however it likes as long as it reads back to the
same model, so an implementation conforms without matching another's bytes.
Idempotence is what keeps that safe: within one implementation, a document has
exactly one spelling, which is also what lets the reference writer double as a
formatter.
