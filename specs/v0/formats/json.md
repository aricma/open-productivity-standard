# JSON

A JSON document is a task object — a forest of one root — or an array of
task objects, a forest whose elements are the roots. Field names map to the
model unchanged, `subtasks` holds the child task objects, and the tree is
explicit and recursive.

JSON calls a list an array and a map an object. A value that JSON would
resolve to some other type of its own is read as the string or list it was
written as.

`id` is optional, since JSON never references a task by id, and a root never
needs one. `version` appears on a root object, and every declaration in a
document must agree (see [About the version](../model.md#about-the-version)).
An unknown field makes the document invalid — a new concept arrives in
`metadata`, not as a new field. An object that repeats a key resolves to its
last occurrence: JSON permits the repeat, and a task carries one value per key
(RFC-023).
