# ADR-008: metadata keys must match a charset

**Status:** Accepted.

## Context

`metadata` is open, so key spelling decides interoperability. Every tool
spells the same concepts differently (`dueDate`, `Due_Date`, `dd`,
`fälligAm`); without an agreed spelling, every key is opaque to every
other tool. Self-explanatory snake_case English keys cross tool boundaries,
get compared and guessed by other tools. Unconstrained keys can't be
parsed by heuristics, so tools reach for company- or tool-specific
prefixes and suffixes. One constraint fixes both: a charset — a key in
plain lowercase letters, digits, and underscores is clear enough to guess
and simple enough to write.

## Decision

Metadata keys must match `^[a-z0-9_]{3,}$` — lowercase letters, digits,
underscores, at least three characters; no dots, dashes, or camelCase. The
charset is enforced for readers and writers alike: a document whose
metadata keys break it is invalid. Flat, self-explanatory keys are a
recommendation, not a rule.

## Consequences

Keys stay uniform, heuristic-parseable, and self-explanatory across tools.
Any key in the charset stays expressible; keys outside it are rejected by
conforming readers.
