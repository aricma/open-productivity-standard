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
productivity tools. Any tool can export its tasks, and any other tool can
import them.

## The model

One type: the **task**. A task may contain other tasks, nested
arbitrarily deep. An export is a **forest**: zero or more root tasks, each
the root of a tree; every task descends from exactly one root or is one.
A flat list with no root is simply a forest whose roots are the top-level
tasks. There are no other concepts — no epics, features, backlog items, or
lists. What a platform calls a task is carried in `metadata`.

## Design goals

At its core, a task is a written line on a piece of paper with a checkmark
next to it.

```plaintext
[x] Buy Milk
[ ] Clean Room
[ ] Find Purpose
```

The standard keeps exactly that: `title` is the line — it must
explain itself — and `status` says whether it's done. Those two are the
only required fields because that is all the information you need to
understand a task at any level.

Everything else is metadata the reader brings. More tasks make a
project; group tasks and you have features; group features and
you have an epic. This game of nesting can be played to infinity, which is
exactly why the standard defines **no** such levels — only tasks, nested.

Everything beyond title and status — due dates, priorities, tags,
estimates, platform statuses — is real data, but every tool spells it
differently. Rather than standardize each one, OPS keeps them as
metadata, preserved verbatim.

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
  expose their tasks through OPS-based APIs, and automation and
  analysis are written once against the OPS shape and work across every
  OPS-capable tool, instead of being re-implemented per platform.
- **Ownership** — your productivity data stays yours. Exports are
  lossless: re-importing into the same tool loses nothing, and across
  tools the aim is the same — unknown details ride along in
  `metadata` instead of being dropped, so nothing stops you from
  leaving or returning to a tool when you want to.

Hierarchy is a reading of the graph, not a type. What a platform calls a
task — epic, feature, PBI, story — can be derived from a task's
position in the tree (projects and epics tend to be near the top, leaf
tasks are the concrete work) and from hints in `metadata` — keys like
`type` or `kind` that the source tool attached.
These heuristics are allowed but never required: the standard defines no
types, so no tool is wrong for not knowing one.

---

## About the Task

A task is a record of a few named fields. This chapter defines the fields
and the rules that make a task valid.

### Fields

Every task is made of these fields. Two are required, the rest are
optional, and a tool that has nothing to put in a field leaves it out.

| Field      | Type                   | Required | Notes |
|------------|------------------------|----------|-------|
| `title`    | string                 | yes      | The human-readable summary: the line on the paper. |
| `status`   | `"open"` / `"done"`    | yes      | `open` or `done`. |
| `id`       | string                 | yes*     | Identifier for formats that reference tasks by id. |
| `notes`    | string                 | no       | Long-form text, plain or Markdown. |
| `subtasks` | task[]                 | no       | Nested task objects or child-id references, per serialization. |
| `metadata` | object                 | no       | Tool- or concern-specific data, keyed by tool or concern. |

`yes*` means required under conditions: a format that references tasks by
id needs one.

### Rules

A document that breaks one of these rules is invalid, and a reader must
reject it.

1. **Required fields** — `title` and `status` appear on every task.
   (DR-001)
2. **`subtasks` define hierarchy** — a task can have subtasks and
   exactly one parent. Tasks build an acyclic directed graph. (DR-002)
3. **`id` and uniqueness** — task ids are required under conditions:
   only formats that reference tasks by id need them. Ids have to be
   unique across the whole graph. Nested serializations never need ids.
   Roots never need an id in any format. Though optional, ids can be
   attached anyway. (RFC-001)
4. **`status` values** — can only be `open` or `done`. (RFC-004)
5. **`notes` carries no level-1 heading** — a task's `notes` must not
   contain a level-1 ATX heading. The H1 belongs to the title: in
   Markdown a `#` line inside notes reads back as a new root, splitting
   the task it belonged to. This constrains the value, so it holds in
   every serialization, not only in Markdown, and a deeper heading is
   unaffected. (RFC-007)
6. **`metadata` is the escape hatch** — anyone may add any key-value
   pairs, but keys must match `^[a-z0-9_]{3,}$` (lowercase letters,
   digits, and underscores, at least three characters; no dots, dashes,
   camelCase, etc.). (RFC-008, RFC-010)
7. **`metadata` values** — a value is one of the six shapes the model
   carries: string, number, boolean, `null`, list, map. Each
   serialization spells them its own way — JSON calls a list an array
   and a map an object, YAML a sequence and a mapping — and a value a
   format would resolve to some other type of its own, such as a YAML
   timestamp or set, is read as the string or list it was written as.
   Keys are constrained, values are not: a value is free data, with no
   character set, case rule, or language of its own, and nested values
   have no depth limit. A value that carries nothing — an empty string,
   list, map, or `null` — is read as absent. (RFC-011, RFC-026)
8. **A `done` task has no open subtask** — if a task is `done`, every
   subtask is `done` too. A task may be `open` with subtasks of either
   status; only the done → open direction is invalid. A reader must
   reject a document that breaks this rule. (RFC-012)
9. **Attachments** — attachments are references carried in
   `metadata`. (RFC-020)
10. **Empty export** — a root task with no subtasks is a valid export,
    whether it represents an empty export or a single standalone task —
    the structure is identical. A document with no tasks at all is an
    empty forest, and is also valid.

---

## Serializations

The model can be carried in any format that tools already speak. These
are general guidelines, not a closed list: the standard does not mandate
a single format, and any mapping that stays lossless and unambiguous is
welcome.

Formats fall into two families, and the use case decides which to pick:

- **Tree-preserving** — JSON, YAML, Markdown. `subtasks` holds
  nested task objects, so the graph structure is explicit and recursive
  traversal is straightforward. Best when the whole tree lives in memory
  anyway, and computation uses graph traversal algorithms to stay
  fast.
- **Flat and streamable** — JSONL, CSV. `subtasks` holds child ids and
  each record appears on its own line, so a processor handles one record
  at a time and rebuilds the graph by resolving id lists. Best for large
  exports, logs, and pipe-style processing. These formats exist to be
  streamable, and to make local, context-based transformations cheap.

### About the Version

A version names the OPS Specifications a document is modeled after. It is
optional, and one per document:

- A document declares **one** version: every declaration in it must be
  identical, or the document is invalid. A document never mixes versions.
- It must be a released version of the OPS Specifications. If none is set,
  the latest released version applies.
- A `version` appears on roots only, and never below a root.

Each serialization says where a document writes it: Markdown uses
frontmatter, and JSON, YAML, JSONL, and CSV write `version` on root
records. Because the field name is fixed, a reader can find the
declared version before parsing, and so choose the version of the
specifications to read it with.

### About metadata

The core fields are simple, so there is little to recommend about them.
The hard part is making `metadata` compatible across tools, and building
the heuristics that read another tool's export.

`metadata` is open — anyone may add key-value pairs, as long as the
keys match the metadata key character set. We recommend keeping them
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
| `priority`     | free-form string (tool scales differ); recommended: `A`, `B`, `C`, `D` (Eisenhower) |
| `tags`         | list of strings |
| `status`       | the platform's original status (e.g. "In review") — distinct from the model's `status` field |
| `estimates`    | number or string (tool units differ) |
| `location`     | string |
| `assignee`     | string naming a team member (name, id, or email) |
| `attachment`   | a URL or a path relative to the document |
| `attachments`  | a list of the same |
| `created_at`   | ISO 8601 timestamp |
| `updated_at`   | ISO 8601 timestamp |
| `completed_at` | ISO 8601 timestamp |
| `url`          | string, link to the task in its source tool |

One key carries a recommended scale, and every other spelling stays
valid. A tool with no scale of its own can use `priority` as `A` (urgent
and important), `B` (important, not urgent), `C` (urgent, not important),
or `D` (neither) — the four classes of the Eisenhower matrix, in the
order to work on them. It stays a recommendation: a tool with its own
scale keeps it, and a reader never compares priorities across tools.

If teams follow the convention of using these attributes, parsing
tasks across tools will already be much simpler.

### Attachments

Attachments are references carried in `metadata`, never the files
themselves. The OPS reader and writer do not resolve or validate them:
they read the references in and write them out, and the surrounding tool
resolves them, with the network and credentials it has.

The recommended keys are `attachment` for a single reference and
`attachments` for a list of them — a lone reference in `attachments`
counts as one, and a reader that finds both keys resolves both. A
reference is recommended to be a URL — `https://…` (presigned URLs
included) or `file://…` — or a path relative to the document; an absolute
location is a `file://` URL.

### JSON

A JSON document is a task object — a forest of one root — or an array of
task objects, a forest whose elements are the roots. Field names map to
the model unchanged, `subtasks` holds the child task objects, and the
tree is explicit and recursive.

`id` is optional, since JSON never references a task by id, and a root
never needs one. `version` appears on a root object, and every
declaration in a document must agree (see *About the Version*). An unknown field
makes the document invalid — a new concept arrives in `metadata`, not as a
new field.

### YAML

YAML carries the same mapping as JSON: a task mapping is a forest of one
root, a sequence of task mappings is a forest of several, and `subtasks`
holds the child mappings. YAML's extra syntax — anchors, aliases, tags,
block scalars — is resolved before the model is read, so it changes the
bytes, never the values. Because YAML is a superset of JSON, every valid
JSON document is also a valid YAML document.

### JSONL

JSONL carries the flat family: one task per line, each line a JSON
object, `subtasks` holding child ids rather than children. Each record
therefore stands alone and can be processed without its neighbors; blank
lines are ignored.

A reader rebuilds the forest by resolving those ids: a record that no
other record references is a root. It rejects a duplicate id, a
reference to an id that does not exist, a child with more than one
parent, and a cycle. `version` appears on root records, and
every declaration in a document must agree (see *About the Version*).

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
  3. The remainder is the metadata key. It must match the metadata key
     charset — a document whose keys break it is invalid.

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
the read. A row without `title` or `status` is invalid.
`subtasks` is a JSON array of child ids, as in JSONL, and roots carry no
id. `version` is a document property: every root row that declares it must
declare the same value, and absent means the latest release. Values
containing commas, quotes, or newlines are quoted per RFC 4180.

### Markdown

Markdown is tree-preserving and follows the document's outline. ATX
headings at column zero are tasks: heading level `L` is the task's depth,
so `#` is a root and `######` its six-level descendant. A heading's title
is the text after the `#`s. A heading that skips a level attaches to the
nearest shallower one.

A heading is `open` unless its title begins with `(Done) `, which makes it
`done`; `\(Done) ` at the start is a literal `(Done) ` and stays `open`.

A task's children may be deeper headings or checkbox list items, mixed:

- `- [ ]` is `open`, `- [x]` is `done`; the rest of the line is the title.
- A checkbox at column zero belongs to the nearest heading. A checkbox
  belongs to the nearest task whose content indent it matches: a heading's
  content indent is column zero, a checkbox's is its own indent plus two.
- A checkbox with no preceding task is a root.

Optional YAML frontmatter carries the document `version` and an optional
root `status` default: a root's `(Done) ` overrides it, and non-root
headings are never affected. Any other frontmatter key is invalid, and a
leading `---` block that is not a mapping is ordinary content.

Any other line in a task's scope is a note on that task; `- key: value`
with a valid metadata key is metadata — the value is the text after the
colon, so a value read from Markdown is a string. Content
before the first task is not part of the document.

```markdown
---
version: "0"
status: open
---

# Acme product backlog

The product backlog for the Acme platform.

- [ ] Fix memory leak in auth service
  - priority: high
  - [ ] Reproduce the leak

## (Done) Shipped in 2026.3

- [x] Retire the legacy importer
```

---

## Conformance

OPS defines the **core model** — the task model without `metadata`:
`title` and `status` required, `id`, `notes`, and `subtasks` optional —
and three levels of support. A tool claims the highest level it meets
for each serialization it supports.

**Level 1 — Export core.** The tool writes valid OPS documents of the core
model: `title` and `status`, plus `id`, `notes`, and `subtasks` where it
has them. It may omit `metadata`.

**Level 2 — Import and export core.** The tool also reads core documents
and derives the tasks from them, preserving the core model — including the
hierarchy in `subtasks`. It may discard `metadata`.

**Level 3 — Full, zero data loss.** The tool supports the core model plus
`metadata`: it reads and writes every field and preserves every node,
every relation, and all valid `metadata` verbatim across an import →
export cycle.

The obligations behind the levels:

**OPS Reader** — imports OPS documents.

- Must accept every valid document following the OPS Specifications.
- Must reject every invalid document claiming to follow the OPS
  Specifications.

**OPS Writer** — exports OPS documents.

- Must output only valid documents following the OPS Specifications.
- Should omit metadata fields that carry no value — an empty string,
  list, map, or `null`.

**OPS Transformer** — imports and exports (converters, sync tools,
round-trippers).

- Everything the Reader and Writer must do.
- Must preserve the number and relation of all nodes and graphs per dataset.
- Must preserve all valid `metadata` verbatim across import → export cycles.

A Level 1 tool is an OPS Writer, Level 2 adds the OPS Reader, and Level 3
adds the OPS Transformer. What a Level 1 or Level 2 tool handles is the
core model, not just the required fields; `metadata` is optional to
preserve below Level 3 and required at it. Conformance never means
understanding metadata: no tool is required to know what another tool's
metadata means, or to support every serialization.
