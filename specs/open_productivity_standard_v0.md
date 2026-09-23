---
status: draft
version: "0"
date: 2026-08-01
author: Adrian Mindak (aricma) <adrian@aricma.org>
---

# OPS Specifications — Version 0 (draft)

This document is **version 0** of the **OPS Specifications**: the
specification of the Open Productivity Standard (OPS). Version 0 is a
draft: no version of OPS is released yet, so tools should not rely on
its stability, and any document claiming version `0` follows this
document.

A minimal, tool-agnostic data structure for exchanging tasks between
productivity tools. Any tool can export its tasks, any other tool can
import them.

## The model

One type: the **task**. A task may contain other tasks, nested
arbitrarily deep. An export is one or more root tasks; every task descends
from exactly one root. There are no other concepts — no epics, features,
backlog items, or lists. What a platform calls a task is carried in
`metadata`.

## Design goals

At its core, a task is a written line on a piece of paper with a checkmark
next to it.

```plaintext
[x] Buy Milks
[ ] Clean Room
[ ] Find Purpose
```

The standard keeps exactly that: `title` is the line — it must
explain itself — and `status` says whether it's done. Those two are the
only required fields because they're the only information you need to
understand a task at any level.

Everything else is metadata the reader brings. More tasks make a
project; group tasks and you have features; group features and
you have an epic. This game of nesting can be played to infinity, which is
exactly why the standard defines **no** such levels — only tasks, nested.

Everything beyond title and status — due dates, priorities, tags,
estimates, platform statuses — is real data, but every tool spells it
differently. Rather than standardize each one, the
standard keeps them as metadata, preserved verbatim.

This enables us to get three things:

- **Interoperability** — one open task structure that any productivity
  tool can export and import, so tasks travel between tools without
  losing title, status, notes, or hierarchy. Keep `metadata` around even
  when you don't understand it, and use self-explanatory keys
  (`due_date`, `status`, `priority`, `estimates`, `location`,
  `attachments`) so the next tool can guess their meaning. The model is
  the intersection of what tools share, not the union: new concepts
  arrive as metadata, never as new top-level fields.
- **Abstraction** — the structure is open and stable, so tools can
  expose their tasks through OPS-based APIs, and automations and
  analysis are written once against the OPS shape and work across every
  OPS-capable tool, instead of being re-implemented per platform.
- **Ownership** — your productivity data stays yours. Exports are
  lossless: re-importing into the same tool loses nothing, across
  tools the aim is none either — unknown details ride along in
  `metadata` instead of being dropped, so nothing stops you from
  leaving or returning to a tool when you want to.

Hierarchy is a reading of the graph, not a type. What a platform calls a
task — epic, feature, PBI, story — can be derived from a task's
position in the tree (projects and epics tend to be near the top, leaf
tasks are the concrete work) and from hints in `metadata` — keys like
`type` or `kind` that the source tool attached.
These heuristics are allowed but never required: the standard defines no
types, so no tool is wrong for not knowing one.

## Fields

| Field      | Type                   | Required | Notes |
|------------|------------------------|----------|-------|
| `title`    | string                 | yes      | The human-readable summary: the line on the paper. |
| `status`   | `"open"` / `"done"`    | yes      | `open` or `done`. |
| `version`  | string                 | no*      | The version the graph is modeled after. |
| `id`       | string                 | yes*     | Identifier for formats that reference tasks by id. |
| `notes`    | string                 | no       | Long-form text, plain or Markdown. |
| `subtasks` | task[]                 | no       | Nested task objects or child-id references, per serialization. |
| `metadata` | object                 | no       | Tool- or concern-specific data, keyed by tool or concern. |

`yes*` means required under conditions; `no*` means optional but
conditional — an asterisk always points at the rules.

## Rules

1. **Required fields** — `title` and `status` appear on every task.
2. **`id` and uniqueness** — task ids are required under conditions:
   only formats that reference tasks by id need them. Ids have to be
   unique across the whole graph. Nested serializations never need ids.
   Roots never need an id in any format. Though optional, ids can be
   attached anyway.
3. **`status` values** — can only be `open` or `done`.
4. **`subtasks` define hierarchy** — a task can have subtasks and
   exactly one parent. Tasks build an acyclic directed graph.
5. **`metadata` is the escape hatch** — anyone may add any key-value
   pairs, but keys must match `^[a-z0-9_]{3,}$` (lowercase letters,
   digits, and underscores, at least three characters; no dots, dashes,
   camelCase, etc.).
6. **`version` resolution** — optional; only root tasks may carry it.
   Each root defines the version its subtree is modeled after, and a
   version never appears below a root — different roots may declare
   different versions. It must be a released version of the OPS
   Specifications. If none is set, the latest released version applies.
7. **Attachments** — attachments are meant to be added as paths in
   `metadata` and resolved at the next import.
8. **Empty export** — a root task with no subtasks is a valid export,
   whether it represents an empty export or a single standalone task —
   the structure is identical.
9. **Field names** — the top-level field names are fixed: single
   lowercase English words, never renamed or translated.

## About metadata
*Common metadata keys and general recommendations*

`metadata` is open — anyone may add key-value pairs, as long as the
keys match the character set of rule 5. We recommend keeping them
flat: it keeps the metadata simple, self-explanatory, and readable.
The specifications define a task with very few required fields. This
allows us to keep the interaction with the data simple, fast, and
direct. However, many productivity tools already have a long list of
other information that they track. To support these and to make tool
interoperability simple, we want to give a few suggestions for common
attributes and how to spell them.

Recommended keys for the most common task attributes in the wild:

| Key            | Value |
|----------------|-------|
| `due_date`     | ISO 8601 date |
| `start_date`   | ISO 8601 date |
| `priority`     | free-form string (tool scales differ) |
| `tags`         | list of strings |
| `status`       | the platform's original status (e.g. "In review") — distinct from the model's `status` field |
| `estimates`    | number or string (tool units differ) |
| `location`     | string |
| `assignee`     | string naming a team member (name, id, or email) |
| `attachments`  | list of paths — relative to the export, or global (a URL, a path on the machine) |
| `created_at`   | ISO 8601 timestamp |
| `updated_at`   | ISO 8601 timestamp |
| `completed_at` | ISO 8601 timestamp |
| `url`          | string, link to the task in its source tool |

If teams follow the convention of using these attributes, parsing
tasks across tools will already be much simpler.

## Serializations

The model can be carried in any format that tools already speak. These
are general guidelines, not a closed list: the standard does not mandate
a single format, and any mapping that stays lossless and unambiguous is
welcome.

Formats fall into two families, and the use case decides which to pick:

- **Tree-preserving** — JSON, YAML, Markdown. `subtasks` holds
  nested task objects, so the graph structure is explicit and recursive
  traversal is straightforward. Best when the whole tree lives in memory
  anyway, and computation uses graph traversal algorithms for
  performant results.
- **Flat and streamable** — JSONL, CSV. `subtasks` holds child ids and
  each record appears on its own line, so a processor handles one record
  at a time and rebuilds the graph by resolving id lists. Best for large
  exports, logs, and pipe-style processing — not every format is
  data-transfer efficient, and these exist to be streamable and to
  perform local, context-based transformations in a very efficient way.

### CSV

CSV carries the flat family: one task per row. The first row is the
header and names the columns; it is required. Column order is free, and
this is the recommended order:

`id`, `title`, `status`, `version`, `notes`, `subtasks`, `metadata`,
then one column per metadata key.

Columns come in two kinds:

- **Reserved columns** are the top-level field names (`version`, `id`,
  `title`, `status`, `notes`, `subtasks`, `metadata`) and map to the
  model fields unchanged. The `metadata` column holds a JSON object,
  exactly as in the model.
- **Metadata columns** are every other column. A column names a metadata
  key, optionally prefixed with `metadata_` and optionally suffixed with
  a type:
  1. Strip one leading `metadata_` prefix, if present.
  2. If the prefix was present and the remainder ends in `_json`,
     `_string`, `_number`, or `_bool`, that suffix names the value type
     and is stripped.
  3. The remainder is the metadata key. It must match rule 5's charset;
     a document whose derived keys break it is invalid.

The prefix is optional and only needed to avoid a collision — with a
reserved column, or because the key itself starts with `metadata_`
(such a key is written prefixed, and stripping recovers it). A column
without the prefix is read as the literal key; a type suffix is
recognized only on a prefixed column. One key must not be supplied
twice.

Cell values:

- A prefixed column with a type suffix is read as that type. If the cell
  cannot be read as it, it is kept as a string — reading never fails.
- Otherwise, a cell that parses as JSON is read as that JSON value, and
  any other cell is kept as a string. So `3` is a number, `true` is a
  boolean, `["a","b"]` is a list, and `2026-07-01` is a string.
- An empty cell means the key is absent, not an empty value.

The `metadata` column and the metadata columns are merged; a metadata
column overrides the same key in the `metadata` object. Any non-reserved
column is metadata, so an unexpected column adds data instead of failing
the read. A row without `title` or `status` is invalid (rule 1).
`subtasks` is a JSON array of child ids, as in JSONL, and roots carry no
id. Values containing commas, quotes, or newlines are quoted per
RFC 4180.

### Markdown

Markdown is tree-preserving. Frontmatter carries the root's `version` and
`status`. The body carries the root and every task:

- The root's `title` is the first heading. Every other task is a checkbox
  list item — `- [ ]` is `open`, `- [x]` is `done` — and the text after
  the marker is the task's `title`.
- A task's content is indented two spaces below its own line; the root's
  content starts at column zero. A content line is read by its first
  character:
  - `- [` starts a child task.
  - `- key: value`, with a key from rule 5's charset, is a metadata entry.
  - any other line is a notes line, including a plain `key: value`.
- A task's `metadata` precedes its subtasks; values are read the same way
  as in CSV (a cell that parses as JSON is that value, anything else is a
  string).

Metadata is bulleted so it never collides with notes: a plain
`key: value` line stays prose, and a bullet that is neither `- [ ]` nor a
valid `- key: value` is a notes list item. Notes therefore cannot contain
a list item shaped like a metadata entry or a task marker, and blank
lines inside notes are not preserved. One metadata key must not appear
twice.

```markdown
# Acme product backlog

The product backlog for the Acme platform.

- created_at: 2026-01-01T09:00:00Z
- url: https://acme.example/backlog

- [ ] Fix memory leak in auth service
  Today description.
  anything: else?
  - priority: high
  - [ ] Reproduce the leak
    - location: Remote
```

Notes, metadata, and child tasks share one indentation level (two spaces
below the task), so notes can hold any text; blank lines between content
are cosmetic and never change nesting.

All examples live with the reference library, in
[`ops-lib/tests/fixtures/ops/examples/`](../ops-lib/tests/fixtures/ops/examples/)
(see its `README.md` for what each file shows).

## Conformance

How can software claim OPS conformance? In the following, we look at
what a reader, writer, and transformer must do. The obligations below
apply to whatever formats a tool claims to support.

**OPS Reader** — imports OPS documents.

- Must accept every valid document following the OPS Specifications.
- Must reject every invalid document claiming to follow the OPS
  Specifications.

**OPS Writer** — exports OPS documents.

- Must output only valid documents following the OPS Specifications.

**OPS Transformer** — imports and exports (converters, sync tools,
round-trippers).

- Everything the Reader and Writer must do.
- Must preserve the number and relation of all nodes and graphs per dataset.
- Must preserve all valid `metadata` verbatim across import → export cycles.

Conformance never means understanding metadata. No tool is required to
know what another tool's metadata means or to support every serialization.

The fixtures in
[`ops-lib/tests/fixtures/ops/tests/`](../ops-lib/tests/fixtures/ops/tests/)
express rules 1–9 as concrete valid
and invalid documents, one per rule and shape; they are the working
definition of "valid" for implementers.
