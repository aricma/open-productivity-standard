# RFC-013: `version` is one per document

## Context

A version names the spec a document is modeled after, and a reader has to
know it before it can parse: the syntax of one version's model is not the
syntax of the next one. Two ways to place it were considered.

Letting **each root declare its own version** keeps documents flexible,
but a single document can then mix versions — and the reader cannot pick
a module until it has parsed a document it can only parse with that
module. It also leaves the carriers inconsistent: Markdown has a
document-level frontmatter block, while JSON, YAML, JSONL, and CSV write
`version` on records.

Giving every format a **document envelope** — `{"version": …,
"tasks": […]}` for JSON and YAML, a header record for JSONL, a preamble
row for CSV — makes the version point uniform, but it is not actually
uniform (four different mechanisms), it breaks plain CSV consumers and
JSONL's one-record-per-line property, and it invalidates every existing
document.

## Decision

A document declares one version. It is optional; absent means the latest
released version applies. Every declaration in a document must be
identical, or the document is invalid, and a `version` never appears
below a root.

The carriers stay format-native, with no wrapper: Markdown uses
frontmatter, and JSON, YAML, JSONL, and CSV write `version` on root
records. Because the field name is fixed (rule 8), finding the declared
version is a version-agnostic syntax scan — the library exposes it as
`detect_version` — so the reader can choose a version module before
parsing the document.

## Consequences

A document maps to exactly one spec version and one module, and the
chicken-and-egg between "which version" and "how to parse" is gone. Mixed
versions are not valid; the library reports `mixed-versions`.

The trade-off is that a document cannot carry a different version per
root, and a rootless document can declare a version only where the format
has a document header — Markdown frontmatter. In the flat family the
version rides on root records, because CSV and JSONL have nowhere else to
put it. Detection is decoupled from the model: it reads a fixed key and
does not validate the value, which the chosen module does when it reads
the document.
