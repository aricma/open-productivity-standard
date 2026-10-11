# RFC-011: Metadata values are JSON values

*Scope: v0 model.*

## Context

A key needs a value, and the model has to say what a value may be. The
narrow answer is a string: every format can carry text, and text needs no
agreement at all. But tools do not hold their data as text. A priority is
a number, a flag is a boolean, tags are a list, and an integration's own
payload is a structure nobody outside it knows. Forcing all of that into a
string means a convention per type — `"true"`, `"1,2,3"` — and a parser
per key, which is exactly the guessing the key charset exists to avoid.

The open answer is any JSON value, and it is what the serializations
already assume: JSON and YAML carry the value natively, CSV either names
the type in a column suffix or reads the cell as JSON, and CSV's
`metadata` column holds a JSON object exactly as the model has it.

## Decision

A metadata value is a JSON value: a string, a number, a boolean, null, an
array, or an object. The model adds no types of its own and puts no depth
limit on a nested value.

Keys and values are separate concerns. The charset constrains keys only; a
value is free text or free data, with no charset, no case rule, and no
language requirement.

A format that cannot express a type carries the value as text. Markdown is
the case that matters: a `key: value` bullet has nowhere to put a type, so
a value read from Markdown is a string. The text is kept; the type is not.

## Consequences

Values move between the typed formats unchanged, so a tool that stores a
list or a structure keeps it, and nothing has to be stringly typed.

The cost is that a non-string value is only as portable as the formats it
passes through. A JSON document holding `"priority": 3` written out as
Markdown and read back holds a string `3`, because Markdown had nowhere to
record the difference. A tool that needs the type across a Markdown round
trip writes it where a type is expressible, or accepts the string.

An absent key and an empty value are different things: a format spells
absence as a missing cell or line, while a key present with a null value
is a value that happens to be null.
