# Open Productivity Standard (OPS)

> An open, tool-agnostic data model for exchanging tasks between
> productivity tools.

## Mission

Your tasks should live wherever you work best — and move when you do.
OPS is one open task structure that any productivity tool can export and
import, so your work travels between tools without losing title,
status, notes, or hierarchy. Any tool that reads or writes OPS can
exchange tasks with any other: unknown details ride along in
`metadata` instead of being dropped, so every export stays lossless.

```text
[ Notion ]  ──┐                         ┌──> [ Linear ]
[ Todoist ] ──┼──> [ OPS (the spec) ] ──┼──> [ Obsidian ]
[ Jira ]    ──┘                         └──> [ Anything ]
```

OPS has three goals:

- **Interoperability** — one open task structure any tool can export and
  import. Tasks travel between tools without losing title, status,
  notes, or hierarchy.
- **Abstraction** — APIs, automations, and analysis written once against
  the OPS shape work across every OPS-capable tool.
- **Ownership** — your data stays yours. Unknown details ride along in
  `metadata` instead of being dropped, so every export stays lossless.

A startup can track its work in markdown files in a repo today and move
into bigger productivity tools later — no migration tools to write,
none to buy: OPS speaks both.

Companies get the same freedom at scale: because the structure is open
and stable, tools can expose their tasks through OPS-based APIs, and
teams build automations and analysis once — against their whole body of
productivity data, across any OPS-capable tool — instead of rewriting
them for every platform.

## The model in a few lines

A task needs only a **title** and a **status** — a line on paper and a
checkmark. Everything else is optional `metadata`, carried verbatim so
exports stay lossless. Hierarchy lives in `subtasks`. Here is the same
list in the two formats people reach for first:

```json
{
  "title": "Get ready for the trip",
  "status": "open",
  "metadata": { "due_date": "2026-08-15", "priority": "high" },
  "subtasks": [
    { "title": "Book the flights", "status": "done" },
    { "title": "Find the passport", "status": "done" },
    { "title": "Pack the suitcase", "status": "open" }
  ]
}
```

```markdown
---
version: "0"
status: open
---

# Get ready for the trip

- due_date: 2026-08-15
- priority: high

- [x] Book the flights
- [x] Find the passport
- [ ] Pack the suitcase
```

## Big exports stream

Exports get big: 200 GB of tasks leaving one tool for another, piped
through serverless processing units. Nested data makes that pipe
expensive — a reader must hold the whole tree in memory before acting
on a single task, so cost scales with the export, not the question.

So OPS also speaks flat, streamable formats — JSONL and CSV, one task
per line, processed one record at a time, constant memory. Hierarchy is
child id lists instead of nesting; a consumer rebuilds the graph by
resolving them.

## Nested or flat at a glance

| Aspect   | Nested (tree-preserving) | Flat (streamable)    |
|----------|--------------------------|----------------------|
| Formats  | JSON, YAML, Markdown     | JSONL, CSV           |
| Memory   | whole tree in memory     | one record at a time |
| Best for | tree traversal           | large exports, logs  |

The same model in both — convertible without loss.

## The standard, proven by code

The standard lives in [`specs/`](specs/): start with the
[specification (v0 draft)](specs/open_productivity_standard_v0.md) or the
[spec README](specs/README.md).

It is more than prose: a working [reference library in Rust](ops/lib/)
reads and writes OPS losslessly, with round-trips checked against a
portable [conformance corpus](test-corpus/) of plain-text cases any
implementation can run. The library consumes the corpus in place, so
spec, corpus, and implementation cannot drift apart.

## Repository

- [`specs/`](specs/) — the standard, its [changelog](specs/CHANGELOG.md),
  and [decision records](specs/adr/README.md).
- [`ops/`](ops/) — the reference library: [`ops/lib/`](ops/lib/) source,
  [`ops/tests/`](ops/tests/) tests, [README](ops/README.md).
- [`test-corpus/`](test-corpus/) — the portable conformance corpus.
- [`maskfile.md`](maskfile.md) — local tasks (`mask setup`, `mask ci`, …).

License: MIT (see [`LICENSE`](LICENSE)).
