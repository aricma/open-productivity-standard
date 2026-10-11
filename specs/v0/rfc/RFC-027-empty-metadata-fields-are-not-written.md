# RFC-027: An empty metadata field is not written

*Scope: v0 model.*

## Context

"Present but empty" should not become a second way to spell "not set",
and an empty field only grows the document for nothing.

## Decision

We recommend that a writer omit a metadata key that says nothing — an
empty string, list, map, or `null` — to keep the exported data small.

A reader does not add an empty key to the model, and does not carry it:
a key whose value carries nothing is read as absent. Even if a writer
emits empty keys, the next reader drops them.

## Consequences

A lean document has a key only when the key has something to say, and an
empty value does not survive a round trip.
