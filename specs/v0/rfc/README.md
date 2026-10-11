# Requests for Comments (RFC)

What is still open. Each RFC names the part of v0 it proposes: the model,
or one format.

## v0 model

| RFC | Proposal |
|-----|----------|
| [RFC-001](RFC-001-id-optional-where-referenced.md) | `id` is optional — required only where a format references by id |
| [RFC-002](RFC-002-binary-status.md) | The top-level status is binary |
| [RFC-003](RFC-003-status-is-a-string.md) | `status` is a string, not a boolean |
| [RFC-004](RFC-004-status-values.md) | The status values are `open` and `done` |
| [RFC-005](RFC-005-optional-long-form-text.md) | Long-form text is an optional string |
| [RFC-006](RFC-006-the-name-is-notes.md) | The field is called `notes` |
| [RFC-008](RFC-008-the-model-needs-an-escape-hatch.md) | The model needs an escape hatch |
| [RFC-009](RFC-009-the-open-field-is-called-metadata.md) | The open field is called `metadata` |
| [RFC-010](RFC-010-metadata-key-charset.md) | metadata keys must match a charset |
| [RFC-011](RFC-011-metadata-values-are-json-values.md) | Metadata values are JSON values |
| [RFC-012](RFC-012-done-task-has-no-open-subtask.md) | A `done` task has no open subtask |
| [RFC-013](RFC-013-version-is-one-per-document.md) | `version` is one per document |
| [RFC-017](RFC-017-the-core-data-model.md) | The core data model |
| [RFC-018](RFC-018-three-levels-of-ops-support.md) | Three levels of OPS support |
| [RFC-019](RFC-019-namespaced-tool-keys.md) | Namespaced keys carry tool-specific data |
| [RFC-020](RFC-020-the-recommended-priority-scale.md) | The recommended priority scale is `A`, `B`, `C`, `D` |
| [RFC-021](RFC-021-common-metadata-keys.md) | The common metadata keys are recommendations |
| [RFC-022](RFC-022-attachments-are-carried-in-metadata.md) | Attachments are carried in `metadata` |
| [RFC-023](RFC-023-a-metadata-key-is-unique.md) | A metadata key is unique |
| [RFC-024](RFC-024-metadata-is-per-task.md) | `metadata` is per task |
| [RFC-026](RFC-026-the-attachment-keys.md) | The recommended keys are `attachment` and `attachments` |
| [RFC-027](RFC-027-empty-metadata-fields-are-not-written.md) | An empty metadata field is not written |
| [RFC-028](RFC-028-attachment-values-are-urls-or-relative-paths.md) | An attachment value is a URL or a relative path |
| [RFC-029](RFC-029-attachments-are-not-validated-or-resolved.md) | Attachments are not validated or resolved |
| [RFC-030](RFC-030-a-title-is-one-non-empty-line.md) | A title is one non-empty line |
| [RFC-031](RFC-031-the-standard-fixes-the-model-not-the-bytes.md) | The standard fixes the model, not the bytes |

## v0 formats

| RFC | Proposal |
|-----|----------|
| [RFC-014](RFC-014-csv-metadata-columns.md) | CSV spreads metadata over columns |
| [RFC-007](RFC-007-notes-cannot-contain-an-h1.md) | `notes` cannot contain an H1 |
| [RFC-015](RFC-015-markdown-metadata-bullets.md) | Markdown metadata is a bulleted `key: value` |
| [RFC-016](RFC-016-markdown-follows-the-document-outline.md) | Markdown follows the document outline |

## Superseded

Folded into other decisions; kept as the record of the proposal.

| RFC | Proposal |
|-----|----------|
| [RFC-025](RFC-025-a-metadata-value-may-contain-line-breaks.md) | A metadata value may contain line breaks |
