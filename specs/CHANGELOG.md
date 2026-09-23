# Changelog

## v0 — draft, unreleased

OPS is a draft. No version is released, nothing is stable, and the
model may change before the first release.

### Added

- First draft of the OPS Specifications version 0
  (`open_productivity_standard_v0.md`): the task
  model (`title`/`status` required, everything else optional), the
  nine rules, common metadata keys, serializations, and conformance
  obligations for readers, writers, and transformers.
- Detailed CSV mapping in the Specifications: top-level fields as
  reserved columns, metadata spread over columns, the optional
  `metadata_` prefix for collisions, `_json`/`_string`/`_number`/`_bool`
  type suffixes, and the optional `metadata` JSON column merged with the
  rest.
- Markdown metadata as `- key: value` bullets instead of HTML comments,
  with the root's metadata in the body and only `version` and `status` in
  the frontmatter; notes stay plain lines, so a plain `key: value` is
  prose and only a bullet is metadata.
- First draft of the decision records (`docs/adr/`, index in
  `docs/adr/README.md`): one file per decision (ADR-001–011; ADR-009
  pending), with the CSV and Markdown metadata decisions included.
- Examples of every serialization, each named for what it showcases,
  now living as the reference library's fixtures under
  `ops/lib/tests/fixtures/ops/examples/`: the shared tree in nested
  JSON/YAML/Markdown and flat JSONL/CSV, a minimal export, a notes-only
  task, a minimal checkbox tree, the smallest CSV, a CSV focused on
  metadata prefixes and types, and a two-root forest.
- Conformance test fixtures, now at
  `ops/lib/tests/fixtures/ops/tests/`: valid and invalid documents for
  JSON, JSONL, YAML, CSV, and Markdown. Every invalid fixture breaks
  exactly one rule.
- A root `TODOs.md` for work beyond v0: release v1 and restore
  CSV/Markdown support in the library so those fixtures are tested too.
- Everything under the MIT license for now (see `LICENSE`).
