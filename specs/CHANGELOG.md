# Changelog

All notable changes to the OPS Specifications are documented here, following
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Each version of the
specification is a directory — [`v0/`](v0/README.md) — and a later version lands
beside it. Only changes a document author or an implementation can observe are
listed.

> **Draft.** No version of OPS is released yet, so version 0 may change in any
> way.

## [Unreleased]

### Added

- The model: one task type, nested arbitrarily deep, in a forest of roots;
  `title` and `status` required and the rest optional; and the rules that make a
  document valid.
- `metadata` as the escape hatch, with the key charset, values of six shapes,
  one value per key, the recommended common keys, the priority scale, and
  attachments as references that are never resolved.
- Conformance: the core model, three levels of support, and what a reader, a
  writer, and a transformer each owe a document.
- The formats, one document each: JSON, YAML, JSONL, CSV, and Markdown. A format
  is normative for a tool that claims it, and no format is required.
- The CSV mapping: reserved columns for the fields, metadata over columns, the
  optional `metadata_` prefix for collisions, the `_json`, `_string`, `_number`,
  and `_bool` type suffixes, and an optional `metadata` JSON column the reader
  merges with the columns.
- The Markdown mapping, marked draft: headings as tasks, checkboxes as subtasks,
  `- key: value` bullets as metadata, a note that may not contain a level-1
  heading, and the collisions in notes it does not settle yet.
- The portable corpus: one file-based case per rule, grouped by format, for any
  implementation to run — semantic comparison, and a model rule carrying a case
  in every format that can express it.
- The decision records and the open proposals, indexed per version, with an RFC
  proposing that the standard fixes the model and its mapping rather than the
  bytes.
- The specifications under the MIT license for now.
