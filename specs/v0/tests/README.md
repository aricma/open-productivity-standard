# OPS Conformance Corpus

Language-agnostic, file-based conformance cases for
[version 0 of the Open Productivity Standard](../model.md). Any
implementation can run them; nothing here is specific to one language.

## A case is a directory

```text
tests/
├── <format>/valid/<case>/{claim.txt, given.<ext>}
├── <format>/invalid/<case>/{claim.txt, given.<ext>}
├── <format>/roundtrip/<case>/{claim.txt, given.<ext>, expectation.<ext>}
└── transform/<case>/{claim.txt, given.<ext>, expectation.<ext>}
```

`<format>` is one of `json`, `yaml`, `jsonl`, `csv`, `markdown`, and
`<ext>` is that format's own file extension. Every case directory holds:

| File | Required | Meaning |
|---|---|---|
| `claim.txt` | yes | one sentence describing the contract |
| `given.<ext>` | yes | the input document; the extension names its format |
| `expectation.<ext>` | `roundtrip/`, and a `transform/` that can succeed | the document the input must export to |

One rule follows from the third row: a **conversion without an expectation
asserts that it cannot be done**. That is how a format's *writer* obligation
is stated — a valid model that this format simply cannot carry — since such a
model cannot be written down in the format that refuses it.

## Running a case

- **`valid/`** — importing `given` must succeed.
- **`invalid/`** — importing `given` in the enclosing format must fail. The
  model does not prescribe a reason, so the broken rule is stated in
  `claim.txt` and never matched.
- **`roundtrip/`** — import `given` with the format's reader, export with
  the format's writer, import the result again, and require the resulting
  model to equal the model of `expectation.<ext>`. This is the
  losslessness the format promises: read your own bytes and get the same
  model back.
- **`transform/`** — import `given` with the reader for its format, export
  with the writer for the target its name names, import the result again,
  and require the resulting model to equal the model of
  `expectation.<ext>`. Without an expectation, the import must succeed and
  the export must fail: `json-to-jsonl-subtask-without-an-id/` is a valid
  JSON tree whose subtasks have no ids, and a JSONL can only carry a child
  as a reference.

Comparison is semantic. The standard fixes field names and mappings, not
key order, indentation, or trailing newline, so byte layout is not part of
the contract.

Implementations that do not claim a format skip its cases and must report
how many were skipped. Skipping is not passing.

## Where a case goes

The model is the standard and the formats map it, so a case's folder says
which one it tests:

- A **model** rule has a case in every format folder that can express it,
  so the formats stay at parity.
- A **format** rule has cases only in that format's folder — a CSV metadata
  column, Markdown frontmatter, or a constraint only that syntax imposes.
  A format that can carry every valid model therefore has none.
- A **writer** obligation is a conversion that cannot be done, and lives in
  `transform/` without an expectation: the model is valid, and only the
  format that refuses it is named.

Matters of **implementation policy** do not belong here at all: the size
before a document is refused, or which spec versions a tool implements. The
standard does not decide those, so they are tested by the implementation
that chose them.

## Adding a case

- One case per directory; the directory name is a short statement, and a
  conversion's names its two formats: `<from>-to-<to>`, then the statement.
- `valid/`, `roundtrip/`, and a `transform/` with an expectation hold
  documents that follow the rules; `invalid/` holds a document that breaks
  one model rule in its own format; a `transform/` without an expectation
  holds a valid model the target format cannot carry.
- Keep fixtures small; `claim.txt` states the contract.

## Why a portable corpus

The cases are files, not code, so any implementation in any language can
run them. The reference library generates one test per case and runs them
in place, which keeps the model, the formats, and the library from drifting
apart unnoticed.

Comparison is semantic on purpose: OPS defines the model and its mapping
onto each format, not one byte stream every tool must reproduce, so two
implementations may lay the same model out differently without failing
conformance. The `roundtrip/` cases are what pins a format down: within one
format, reading your own output has to give the same model back.

Layout is not free where it carries meaning — CSV and RFC 4180 quoting,
YAML block nesting, Markdown indentation — so a format fixes those, and a
writer must emit well-formed, idempotent output: read your own bytes and
write them again and nothing changes. The reference writer targets one
stable, minimal form, since it doubles as a formatter.
