# Architectural Decision Records (ADR)

This folder records the decisions that shape the OPS model and the
reasoning behind them. A proposal to change a decision must overturn this
reasoning, not just edit the specification.

Each decision lives in its own file, `ADR-NNN-short-title.md`.

| ADR | Decision | Status |
|-----|----------|--------|
| [ADR-001](ADR-001-id-optional-where-referenced.md) | `id` is optional — required only where a format references by id | Accepted |
| [ADR-002](ADR-002-title-and-status-required.md) | Only `title` and `status` are required | Accepted |
| [ADR-003](ADR-003-binary-status.md) | Binary status (`open` / `done`) | Accepted |
| [ADR-004](ADR-004-version-optional-on-root.md) | `version` is optional and lives on the root | Accepted |
| [ADR-005](ADR-005-notes-is-a-field.md) | `notes` is a model field, not metadata | Accepted |
| [ADR-006](ADR-006-subtasks-field.md) | `subtasks` instead of `children` | Accepted |
| [ADR-007](ADR-007-metadata-escape-hatch.md) | `metadata` is the escape hatch | Accepted |
| [ADR-008](ADR-008-metadata-key-charset.md) | Metadata keys must match a charset | Accepted |
| [ADR-009](ADR-009-status-string-enum.md) | `status` is a string enum, not a boolean | Pending |
| [ADR-010](ADR-010-csv-metadata-columns.md) | CSV spreads metadata over columns | Accepted |
| [ADR-011](ADR-011-markdown-metadata-bullets.md) | Markdown metadata is a bulleted `key: value` | Accepted |
