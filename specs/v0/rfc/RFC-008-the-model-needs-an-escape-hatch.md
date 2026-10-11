# RFC-008: The model needs an escape hatch

*Scope: v0 model.*

## Context

The model keeps its field names fixed and its field list short, on purpose:
few required fields keep interaction with the data simple, fast, and
direct. The other side of that choice is everything the model does not
carry. Tools track due dates, priorities, tags, estimates, assignees,
platform statuses, links, attachments — each spells them its own way, and
each tracks a few that nobody else has.

Standardizing them is not available to us. Every attribute we name becomes
a top-level field, the model becomes the union of every tool's taxonomy,
and each addition is a change every reader has to learn — while the
attributes keep arriving. The model answers with one open field instead.

## Decision

The model has exactly one open field. Every task may carry it, its
contents are preserved verbatim, and a concept the fixed fields do not
cover is carried inside it rather than as a new field.

Its name, the shape of its keys, and the shape of its values are each
their own decision.

## Consequences

The fixed model stays small and stable while everything else has somewhere
to live, and a round trip stays lossless: a tool that does not understand a
key can still carry it through.

The open field is not a place to standardize. A concept every tool shares
belongs in the model as a field; what is left over is by definition
neither required nor guaranteed to be understood by anyone.
