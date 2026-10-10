# RFC-009: The open field is called `metadata`

## Context

The open field needs a name, and the name has to be plain enough that a
reader who has never seen this document guesses it right. The field holds
data *about* the task, supplied by a tool or a concern the standard does
not model. It is not the task's own content — long-form text already
carries that — and it is not one of the fixed attributes.

The near misses give themselves away. `properties` and `attributes` come
from object models and say nothing about who supplies them. `extra`,
`misc`, and `other` say "leftovers", which invites dumping anything there,
including things that belong in a field. `data` is the whole task, not a
part of it. `meta` is an abbreviation that has to be explained anyway.
A vendor-style prefix imports a convention from specifications that
reserve prefixes for their own extensions, and this field is not an
extension. `extensions` does not fit either: a value inside this field has
no behavior attached, so nothing is being extended.

`metadata` is the word for exactly this: data about the data. It is what
file formats, photo libraries, and content tools call the block of
attributes beside the content, so a reader arriving at a task file knows
what the field is before reading any definition — which is the whole point
of the name.

## Decision

The open field is called `metadata`.

## Consequences

One word, unabbreviated and unprefixed, and the same word in every format:
a JSON key, a YAML mapping, a CSV column, a Markdown bullet. A tool with
its own name for such a block maps that name onto `metadata`, and nothing
else in the model claims the word.
