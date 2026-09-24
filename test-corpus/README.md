# OPS Conformance Corpus

Language-agnostic, file-based conformance cases for the
[Open Productivity Standard](../specs/open_productivity_standard_v0.md).
Any implementation can run them; nothing here is specific to one
language.

## A case is a directory

```text
test-corpus/
├── valid/<case>/{claim.txt, given.<ext>}
├── invalid/<case>/{claim.txt, given.<ext>}
├── convert/<case>/{claim.txt, given.<ext>, expectation.<ext>}
└── roundtrips/<case>/{claim.txt, given.<ext>, expectation.<ext>}
```

`<ext>` is the serialization: `json`, `yaml`, `jsonl`, `csv`, or `md`.
Every case directory contains:

| File | Required | Meaning |
|---|---|---|
| `claim.txt` | yes | one sentence describing the contract |
| `given.<ext>` | yes | the input document; the extension names its format |
| `expectation.<ext>` | for `convert/`, `roundtrips/` | the document the input must export to |

## Running a case

- **`valid/`** — importing `given` must succeed.
- **`invalid/`** — importing `given` must fail. The spec requires readers
  to reject invalid documents; it does not prescribe a reason, so the
  broken rule is stated in `claim.txt` and never matched.
- **`convert/`** — import `given` with the reader for its format, export
  with the writer for the expectation's format, import the result again,
  and require the resulting model to equal the model of
  `expectation.<ext>`. The two extensions are the transformation: the same
  format is a round-trip, different formats a conversion.
- **`roundtrips/`** — same as `convert/`, but `given` and `expectation`
  share one format: read, write, read back, and require the same model.

Comparison is semantic. The standard fixes field names and mappings, not
key order, indentation, or trailing newline, so byte layout is not part
of the contract.

Implementations that do not support a format skip its cases and must
report how many were skipped. Skipping is not passing.

## Adding a case

- One case per directory; the directory name is a short statement.
- `valid/`, `convert/`, and `roundtrips/` hold documents that follow the
  rules; `invalid/` holds documents that break exactly one rule.
- Keep fixtures small; `claim.txt` states the contract.

## Why a portable corpus

The corpus is independent of the specification and of every
implementation: the reference library runs it, anything else may too, and
the spec neither owns nor references it. Sharing one set of cases keeps
them honest — a case an implementation cannot run still documents the
contract.

Comparison is semantic on purpose: OPS defines the model and its
mapping onto each format, not one byte stream every tool must
reproduce, so two implementations may lay out the same model
differently without failing conformance.

Layout is not free where it carries meaning — Markdown's two-space
indentation, CSV and RFC 4180 quoting, YAML block nesting — so the spec
fixes those, and a writer must emit well-formed, idempotent output:
read your own bytes and write them again and nothing changes. The
reference writer targets one stable, minimal form, since it doubles as
a formatter.
