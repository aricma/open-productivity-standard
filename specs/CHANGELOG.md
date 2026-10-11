# Changelog

## v0 — draft, unreleased

OPS is a draft. No version is released, nothing is stable, and the
model may change before the first release.

### Changed

- The specification is split into the model and its formats. `v0/model.md`
  is now the whole standard — the fields, the rules, and conformance — and
  each format has its own document under `v0/formats/`. A format is
  normative for a tool that claims it; no format is required.
- The "no level-1 heading in `notes`" rule left the model and became a
  Markdown format rule, where its reason lives. It no longer constrains
  documents in other formats.
- Version 0's decisions, proposals, and test corpus moved under `v0/`,
  alongside the model they belong to.
- A document's `version` is now what it claims to follow, not a promise
  about a release list: the standard asks that a declaration be findable
  and that a document declare at most one, and leaves which versions are
  accepted to each implementation. (RFC-013)
- A CSV row may be no wider than its header. Extra cells had no column to
  belong to and were read as nothing, so the row is invalid instead.
- `formats/markdown.md` is marked draft: the profile is not settled and its
  notes still collide with its structure.

### Added

- First draft of the OPS Specifications version 0: the task
  model (`title`/`status` required, everything else optional), the
  rules, common metadata keys, formats, and conformance
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
- First draft of the decision records: one file per decision, numbered
  once and living in `rfc/` while open or in `dr/` once locked (indexes
  in each folder's `README.md`), with the CSV and Markdown metadata
  decisions included.
- Everything under the MIT license for now (see `LICENSE`).
