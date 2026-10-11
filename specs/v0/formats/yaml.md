# YAML

YAML carries the same mapping as [JSON](json.md): a task mapping is a forest
of one root, a sequence of task mappings is a forest of several, and
`subtasks` holds the child mappings. YAML calls a list a sequence and a map
a mapping.

YAML's extra syntax — anchors, aliases, tags, block scalars — is resolved
before the model is read, so it changes the bytes, never the values. A value
YAML would resolve to a type of its own, such as a timestamp or a set, is
read as the string or list it was written as. Because YAML is a superset of
JSON, every valid JSON document is also a valid YAML document.

`version` appears on a root mapping, and an unknown field makes the document
invalid, as in JSON.
