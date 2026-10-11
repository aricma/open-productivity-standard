# Changelog

All notable changes to the `ops-lib` crate are documented here, following
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versioning follows
[SemVer](https://semver.org/). Only changes to the public interface are listed.

> **Alpha.** Nothing has been released yet. The standard is still at
> version `0`; expect breaking changes.

## [0.1.0] - unreleased

### Added

- `v0::read(Format, &str) -> Result<Vec<Task>, Error>` and
  `v0::write(Format, &[Task]) -> Result<String, Error>` parse and serialize OPS
  version-0 documents.
- `Format` supports JSON, YAML, JSONL, and CSV.
- The version-0 model: `Task` and `Status`.
- `v0::detect_version(Format, &str)` probes a document's declared version.
- `Error::code()` and `ValidationError::code()` return stable, library defined
  rejection codes for invalid documents (`missing-title`, `invalid-status`,
  `nesting-too-deep`, …).
- `v0::limits` publishes the size guardrails (documents, tasks, depth, children,
  fields, metadata); over-limit input is rejected with
  `ValidationError::LimitExceeded`.
- `v0::limits::MAX_OUTPUT_BYTES`: a document a reader would refuse is not
  worth writing, so `write` applies the same bound to what it produces, and
  the flat writers stop as soon as they pass it.
- `Format::nests()` and `Format::display_name()`.
- `ValidationError::SubtaskWithoutId` (code `subtask-without-id`).

### Changed

- `Error::Syntax` carries a `Format` instead of a string, so a caller can
  match on it.
- `ValidationError::UnsupportedVersion` replaces `UnknownOPSVersion` (code
  `unsupported-version`). Which versions are accepted is this library's
  policy, not the standard's, and the message now says so.
- `MAX_DEPTH` is 48, down from 256. The tree parsers stop recursing at about
  64 levels of task nesting, and the exact edge moves with the document's
  shape and its format, so the bound sits inside that budget with room to
  spare — which is what keeps a written tree readable.
- A flat format references tasks by id, so `write` refuses a model whose
  subtask has none (`subtask-without-id`) instead of emitting a document
  where the child reads back as a separate root.

### Removed

- Markdown. The OPS specification defines the format, but this crate does
  not claim it: `Format::Md { heading_levels }` and `Format::md()` are gone,
  and the corpus cases for Markdown are ignored.

### Fixed

- A cycle that sits beside a root is rejected. Detection only looked at a
  document with no root at all, so the tasks in such a cycle were dropped
  silently and the rest of the document was imported as if it were whole.
- A CSV row wider than its header is rejected. The extra cells had no column
  to belong to and were read as nothing.
- `v0::detect_version` applies the input bound, like `read`. It parsed
  whatever it was handed, so the resource guardrails could be bypassed by
  calling it instead of `read`. It stays a probe: it bounds the input and
  does not validate the document.

### Security

- Safe Rust only: `unsafe` is denied crate-wide.
