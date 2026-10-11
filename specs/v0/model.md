---
status: draft
version: "0"
date: 2026-08-01
author: Adrian Mindak (aricma) <adrian@aricma.org>
---

# OPS Specifications — Version 0: the model (draft)

This document is **version 0** of the **OPS Specifications**: the
specification of the Open Productivity Standard (OPS). Version 0 is a
draft: no version of OPS is released yet, so tools should not rely on
its stability, and any tasks claiming version `0` follow this document.

It specifies the **model**: what a task is, and the rules that make a
collection of tasks valid. How the model is written into a concrete
syntax is a separate, optional [format](formats/README.md), and nothing
here requires a tool to speak a particular one.

A minimal, tool-agnostic data structure for exchanging tasks between
productivity tools. A tool exports its tasks as a document in one of the
formats, and a tool that reads that format imports them.

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
  tool can export and import in a shared format, so tasks travel between
  tools without losing title, status, notes, or hierarchy. Keep `metadata`
  around even when you don't understand it, and use self-explanatory keys
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
| `notes`    | string                 | no       | Long-form text; often Markdown itself. |
| `subtasks` | task[]                 | no       | Nested task objects or child-id references, per format. |
| `metadata` | object                 | no       | Tool- or concern-specific data, keyed by tool or concern. |
| `version`  | string                 | no       | The OPS Specifications this document follows; roots only. |

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
   unique across the whole graph. Nested formats never need ids.
   Roots never need an id in any format. Though optional, ids can be
   attached anyway. (RFC-001)
4. **`status` values** — can only be `open` or `done`. (RFC-004)
5. **`metadata` is the escape hatch** — anyone may add any key-value
   pairs, but keys must match `^[a-z0-9_]{3,}$` (lowercase letters,
   digits, and underscores, at least three characters; no dots, dashes,
   camelCase, etc.). (RFC-008, RFC-010)
6. **`metadata` values** — a value is one of the six shapes the model
   carries: string, number, boolean, `null`, list, map. Keys are
   constrained, values are not: a value is free data, with no character
   set, case rule, or language of its own, and nested values have no
   depth limit. A value that carries nothing — an empty string, list,
   map, or `null` — is read as absent. (RFC-011, RFC-027)
7. **A `done` task has no open subtask** — if a task is `done`, every
   subtask is `done` too. A task may be `open` with subtasks of either
   status; only the done → open direction is invalid. A reader must
   reject a document that breaks this rule. (RFC-012)
8. **Attachments** — attachments are references carried in
   `metadata`. (RFC-022)
9. **A metadata key is unique** — a task carries at most one value per
   key. (RFC-023)
10. **Empty export** — a root task with no subtasks is a valid export,
    whether it represents an empty export or a single standalone task —
    the structure is identical. A document with no tasks at all is an
    empty forest, and is also valid.
11. **A title is one non-empty line** — `title` contains no line break
    and is not empty. A title is the line that explains the task: a
    break inside it splits it into two, and an empty one explains
    nothing. (RFC-030)

## About metadata

The core fields are simple, so there is little to recommend about them.
The hard part is making `metadata` compatible across tools.

`metadata` is open — anyone may add key-value pairs, as long as the
keys match the metadata key character set, and it belongs to a task; a
document itself carries none. A reader carries the pairs without
interpreting them, and an application decides what to use. We recommend
keeping the pairs flat, which keeps them simple, self-explanatory, and
readable.

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
scale keeps it under a namespaced key, and a reader never compares
priorities across tools.

Data that fits none of these keys goes under a namespaced key — the
tool's name plus an underscore, `jira_ticket_id` — so one tool's extra
fields never collide with another's.

If teams follow the convention of using these attributes, parsing
tasks across tools will already be much simpler.

## Attachments

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

## About the version

A version names the OPS Specifications a document is modeled after. It is
optional, and one per document:

- A document declares **one** version: every declaration in it must be
  identical, or the document is invalid. A document never mixes versions.
- It must be a released version of the OPS Specifications. If none is set,
  the latest released version applies.
- A `version` appears on roots only, and never below a root.

Each [format](formats/README.md) says where a document writes it. Because
the field name is fixed, a reader can find the declared version before
parsing, and so choose the version of the specifications to read it with.

---

## Conformance

OPS defines the **core model** — the task model without `metadata`:
`title` and `status` required, `id`, `notes`, and `subtasks` optional —
and three levels of support. A tool claims the highest level it meets
for each format it supports.

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
metadata means, or to support every format.
