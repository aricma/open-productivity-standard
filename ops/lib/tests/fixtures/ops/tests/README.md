# Tests

Conformance fixtures for the OPS Specifications version 0
([`specs/open_productivity_standard_v0.md`](../../../../../../specs/open_productivity_standard_v0.md)):
documents that a compliant tool must accept or reject. Write your parser
against these states, not against guesswork.

The model's rules are expressed in the canonical formats, one folder per
format:

- [`json/`](json/) — the canonical nested format: hierarchy via nested
  `subtasks` objects.
- [`jsonl/`](jsonl/) — the canonical flat format: hierarchy via child-id
  lists in `subtasks`.
- [`yaml/`](yaml/) — the JSON mapping in YAML, plus YAML's coercion traps
  (quote dates and other values YAML would retype).
- [`csv/`](csv/) — the CSV mapping: metadata columns, the `metadata_`
  prefix, type suffixes, and the optional `metadata` JSON column.
- [`markdown/`](markdown/) — the Markdown mapping: frontmatter root,
  checkbox tasks, bulleted metadata, and plain notes.

Each set has:

- `valid/` — documents that MUST parse and import cleanly.
- `invalid/` — documents that MUST be rejected (each breaks exactly one
  rule).

### Naming

Every case states the expected depth in its name — for valid and invalid
cases alike:

- `<count>-<roots>-with-<n>-levels-of-tasks` — depth is the task levels
  *below* the root; a root alone is `zero-levels`. For multi-root cases
  it is the maximum depth across roots.
- Valid: `<count>-<roots>-with-<n>-levels-of-tasks[-with-<feature>]`,
  with features like `id`, `ids`, `version`, `rich-metadata`,
  `mixed-statuses`.
- Invalid: same shape pattern, with the broken rule as the feature
  (`with-missing-title`, `with-shared-child`, `with-cyclic-reference`).
- Examples: `one-open-root-with-zero-levels-of-tasks`,
  `one-root-with-two-levels-of-tasks-with-rich-metadata`,
  `two-roots-with-one-level-of-tasks-with-shared-child`.

The pattern is open-ended — extend it the same way instead of inventing
new schemes.

### Adding fixtures

- One scenario per file; the name states what it demonstrates.
- Valid fixtures must not depend on lenient readers; invalid fixtures
  must not depend on strict ones beyond the rule they break.
- Keep each folder in its own format (JSON, JSONL, YAML, CSV,
  Markdown); other serializations are covered by the examples folder.

### Attachments

No test cases for attachments yet. Attachments are metadata-only (rule
7), and a portable fixture needs a concrete packaging decision (relative
paths inside a ZIP, how to point at them). We will add fixtures on
request when the first tool actually needs to exchange files.
