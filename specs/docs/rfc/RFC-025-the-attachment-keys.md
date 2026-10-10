# RFC-025: The recommended keys are `attachment` and `attachments`

## Context

Both spellings exist in the wild: one file is `attachment`, several are
`attachments`. Both are obvious, so we accept both.

## Decision

We recommend `attachment` (one value) and `attachments` (a list; a lone
value in it counts as one). A writer commits to one of them — writing
both is not recommended.

A reader resolves both: it tries `attachment` first, then `attachments`,
and keeps every value it can resolve.

## Consequences

A writer keeps its spelling, a reader knows two names, and a document
carrying both loses nothing — each key is resolved in turn.
