# JSONL

JSONL carries the flat family: one task per line, each line a JSON object,
`subtasks` holding child ids rather than children. Each record therefore
stands alone and can be processed without its neighbors; blank lines are
ignored.

A reader rebuilds the forest by resolving those ids: a record that no other
record references is a root. It rejects a duplicate id, a reference to an id
that does not exist, a child with more than one parent, and a cycle.
`version` appears on root records, and every declaration in a document must
agree (see [About the version](../model.md#about-the-version)).
