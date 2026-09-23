# Examples

Every file in this folder is a **valid** document per
[`specs/open_productivity_standard_v0.md`](../../../../../../specs/open_productivity_standard_v0.md).
These examples are the reference library's test fixtures; the standard
itself lives in `specs/`.

This is not an exhaustive catalog: we show a valid example per shape and
serialization, not every combination. There are **no invalid examples**
here — deliberately. If you want to test a parser's error handling,
construct invalid documents yourself from the rules in the spec.

## The shared document

Most files carry the same content, so the mappings are comparable. The
forest pair adds a second tree.

- Root task: `Acme product backlog` (`id: export-root` in JSON)
- Root `notes`: the product-backlog paragraph, in every format that
  carries the shared document
- `t1` — Fix memory leak in auth service (open; carries `notes` and
  `metadata`)
  - `t1r` — Reproduce the leak (leaf; carries only `metadata`)
  - `t1a` — Ship the fix (done; carries `metadata`)
- `metadata` showcases every recommended key from the Common metadata
  chapter, spread across the tree: root `created_at`/`updated_at`/`url`,
  `t1` `priority`/`start_date`/`due_date`/`tags`/`status`/`assignee`,
  `t1r` `location`, `t1a` `estimates`/`attachments`/`completed_at`

## Nested (tree-preserving)

### `nested-tree-rich-metadata.json`

Canonical JSON, the reference form: one task object per node, children
nested in `subtasks`. The root carries `version`, `notes`, and an `id` it
doesn't need (ids are optional in tree serializations); `metadata` sits on
more than one level, and `t1r` is a leaf whose only data is `title`,
`status`, and a little `metadata`.

### `nested-tree-rich-metadata.yaml`

The same document in YAML. Nesting is expressed by indentation, not
brackets; the mapping is otherwise identical to JSON. Note `due_date` is
quoted so it stays a string instead of parsing as a YAML date.

### `nested-minimal-empty-export.json`

The minimal valid export: a root task with only `version`, `title`, and
`status`. Demonstrates rule 8 — a root task with no `subtasks` is a valid
export whether it stands for an empty export or a single task — and that
the root needs no `id`.

### `nested-task-with-notes.json`

A single task with multi-line `notes`, to show the `notes` field on its
own: the description is model content, not metadata, so it stays a
top-level string in every serialization.

### `nested-checkbox-tree-with-metadata.md`

Frontmatter carries the root's `version` and `status`; the `#` heading is
the root's `title`, the paragraph below it the root's `notes`, and the
`- key: value` bullets the root's `metadata`. The tree is a nested
checkbox list, `[ ]` = `open` and `[x]` = `done`. Every task's content
sits at one indent: `- [` starts a child task, a `- key: value` bullet is
metadata, and a plain line is notes — so `anything: else?` stays prose
next to metadata that reads as an ordinary list. No HTML comments and no
JSON blobs.

### `nested-minimal-checkbox-tree.md`

The smallest Markdown document: frontmatter `version`+`status`, an `#`
heading root, and two checkbox tasks. No notes or metadata, to show the
core shape without the extras.

## Flat and streamable

### `flat-tree-id-lists.jsonl`

The shared tree linearized: one task per line, one line per record. The
root has no `id` and its `subtasks` lists the ids of its children; every
child record carries its own `id` and lists its own children the same way.
A streaming format — a consumer processes records line by line and
rebuilds the tree by resolving the id lists, no parent references needed.

### `flat-tree-metadata-columns.csv`

The shared tree as rows, metadata spread over columns. A metadata column
is the key (`priority`), optionally prefixed `metadata_` when it would
collide with a reserved column (`metadata_status` carries the `status`
metadata key), and optionally type-suffixed (`metadata_estimates_number`).
A cell is plain text unless it parses as JSON — `estimates` is the number
`3`, `tags` is a JSON list, while `2026-08-15` stays a string. Empty cells
mean absent. Resolution works exactly like JSONL: roots have no `id`;
child rows carry ids and list their children's ids in `subtasks`.

### `flat-minimal-two-columns.csv`

The smallest valid CSV: just `title` and `status`, one row. Shows that
every other column — including `id`, `notes`, `subtasks`, and all metadata
— is optional.

### `flat-tree-metadata-prefix-and-types.csv`

A focused CSV example for the column rules: `metadata_status` (the
`metadata_` prefix avoiding the reserved `status` column) plus
`metadata_estimates_number`, `metadata_flagged_bool`, `metadata_tags_json`,
and `metadata_code_string` (the type suffixes, including forcing the
string `"3"` to stay a string).

### `flat-forest-two-roots.jsonl`

Two root tasks in one file — "Work" and "Personal" — a forest, as JSONL.
Both roots carry no `id`; each lists its children's ids in `subtasks`. No
task is shared between trees: every task has exactly one parent in the
whole file, and ids are unique across it. This is how a flat export of
several lists (or several tools' data merged) can be streamed as one file.

### `flat-forest-metadata-json-and-columns.csv`

The same forest as CSV, and the only example that uses the optional
`metadata` JSON column together with metadata columns: the "Work" root
keeps its metadata in the JSON column, its child spreads the same keys
over columns. On read the two are merged, and a metadata column overrides
the same key from the JSON column.
