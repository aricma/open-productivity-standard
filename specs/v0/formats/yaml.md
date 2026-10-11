# YAML

YAML carries the same mapping as [JSON](json.md): a task mapping is a forest
of one root, a sequence of task mappings is a forest of several, and
`subtasks` holds the child mappings. YAML calls a list a sequence and a map
a mapping.

YAML's extra syntax — anchors, aliases, tags, block scalars — is resolved
before the model is read, so it changes the bytes, never the values. A plain
scalar resolves to one of the model's own shapes or it is a string: `true` and
`false` are booleans, `null`, `~`, and an empty scalar are absent, a number is
a number, and every other plain scalar is the string it was written as —
`yes`, `on`, `1_000`, `0755`, `n`, an unquoted timestamp, or a set. A scalar
YAML resolves to a number the model cannot carry — `.inf`, `-.inf`, `.nan` —
is read as that spelling, as a string, so no value is dropped on the way. Because YAML is a superset of
JSON, every valid JSON document is also a valid YAML document.

`version` appears on a root mapping, and an unknown field makes the document
invalid, as in JSON. Unlike JSON, a mapping that repeats a key is not a YAML
mapping at all: the document is malformed, and a reader rejects it.
