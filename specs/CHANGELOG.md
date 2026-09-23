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
- First draft of the decision records (`adr/`, index in
  `adr/README.md`): one file per decision (ADR-001–011; ADR-009
  pending), with the CSV and Markdown metadata decisions included.
- Everything under the MIT license for now (see `LICENSE`).
