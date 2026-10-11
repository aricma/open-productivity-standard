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
| `expectation.<ext>` | `roundtrip/`, `transform/` | the document the input must export to |

## Running a case

- **`valid/`** — importing `given` must succeed.
- **`invalid/`** — the enclosing format must reject `given`, one way or the
  other. When `given` is in that same format, its **reader** must fail.
  When `given` is in another format it is a valid model that can only be
  rejected on the way out: the enclosing format's **writer** must fail.
  Markdown cannot carry a level-1 heading inside `notes`, so
  `markdown/invalid/notes-with-an-h1/` reads JSON and fails on write. The
  model does not prescribe a reason, so the broken rule is stated in
  `claim.txt` and never matched.
- **`roundtrip/`** — import `given` with the format's reader, export with
  the format's writer, import the result again, and require the resulting
  model to equal the model of `expectation.<ext>`. This is the
  losslessness the format promises: read your own bytes and get the same
  model back.
- **`transform/`** — import `given` with the reader for its format, export
  with the writer for the expectation's format, import the result again,
  and require the resulting model to equal the model of
  `expectation.<ext>`.

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

## Adding a case

- One case per directory; the directory name is a short statement.
- `valid/`, `roundtrip/`, and `transform/` hold documents that follow the
  rules; `invalid/` holds either a document that breaks one model rule or a
  valid model the enclosing format cannot write.
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
