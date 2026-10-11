# Formats

The model in [model.md](../model.md) is what OPS standardizes. A **format**
is a mapping of that model onto a concrete syntax — JSON, YAML, JSONL, CSV,
Markdown, or any other.

A format document is **normative**: claiming a format binds a tool to its
rules. That is what makes exports reproducible — read, write, and read
again has to come back to the same model, and two tools that claim the same
format have to agree on the bytes. Nothing requires a tool to claim any
format, and this list is not closed: any mapping that stays lossless and
unambiguous is welcome.

Formats fall into two families, and the use case decides which to pick:

- **Tree-preserving** — JSON, YAML, Markdown. `subtasks` holds nested
  task objects, so the graph structure is explicit and recursive
  traversal is straightforward. Best when the whole tree lives in memory
  anyway, and computation uses graph traversal algorithms to stay fast.
- **Flat and streamable** — JSONL, CSV. `subtasks` holds child ids and
  each record appears on its own line, so a processor handles one record
  at a time and rebuilds the graph by resolving id lists. Best for large
  exports, logs, and pipe-style processing. These formats exist to be
  streamable, and to make local, context-based transformations cheap.

| Format | Document | Family | Where `version` goes |
|--------|----------|--------|----------------------|
| JSON | [json.md](json.md) | tree | a root object |
| YAML | [yaml.md](yaml.md) | tree | a root mapping |
| JSONL | [jsonl.md](jsonl.md) | flat | a root record |
| CSV | [csv.md](csv.md) | flat | a root row |
| Markdown | [markdown.md](markdown.md) | tree | frontmatter |

A format's rules are the model's rules plus the constraints its syntax
adds, and a format that can carry every valid model adds none. Where a
format cannot — Markdown cannot carry a level-1 heading inside `notes` —
the format document says what a writer must do.
