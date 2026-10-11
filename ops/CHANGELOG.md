# Changelog

All notable changes to `ops-lib` are documented here, following
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versioning follows
[SemVer](https://semver.org/). Only changes to the public interface are listed.

> **Alpha.** Nothing has been released yet. Version 0.x may change in any
> release.

## [Unreleased]

### Added

- `v0::read(Format, &str)` and `v0::write(Format, &[Task])` read and write OPS
  version-0 documents, in a `v0` module so a later specification version lands
  beside it.
- The version-0 model: `Task`, `Status`, `Metadata`, `MetadataValue`, and
  `Number`.
- `Format` claims JSON, YAML, JSONL, and CSV, with `Format::ALL`, `name()`,
  `display_name()`, `from_name()`, and `nests()` for the tree and flat families.
- One cargo feature per format, all on by default, so a build can claim a
  subset: a format whose feature is off has no `Format` variant, its code is not
  compiled in, and its dependency is not pulled in — `yaml` is the only one with
  a dependency of its own. At least one format is required.
- `v0::detect_version(Format, &str)` reads the version a document declares,
  matching it against nothing, under the same input bound as `read`.
- `Error` and `ValidationError` carry stable, library-defined codes
  (`missing-title`, `invalid-status`, `nesting-too-deep`, …), so a diagnostic
  does not depend on the message text.
- `v0::limits` publishes the resource guardrails — document size, task count,
  nesting depth, children, field and metadata sizes — and `write` holds its
  output to `MAX_OUTPUT_BYTES`.
- Both sides validate: a parse never returns an invalid forest, and a model is
  checked before serializing — duplicate ids, the metadata key charset, version
  placement and supported versions, a `done` task with an `open` subtask, a
  shared or cyclic reference. A value that carries nothing, such as an empty
  string, list, map, or `null`, is read as absent and never written.
- A flat format carries the hierarchy as child id references, so `write` refuses
  a model whose subtask has no id (`subtask-without-id`) rather than emitting a
  document where the child reads back as a separate root.
- Reading a writer's own output and writing it again produces the same bytes, so
  reformatting a file never churns it.
- Safe Rust only: `unsafe` is denied in the crate and in its tools.
