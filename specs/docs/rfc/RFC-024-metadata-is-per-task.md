# RFC-024: `metadata` is per task

## Context

`version` is the only thing a document says about itself. A tool that
exports often knows more about the export than about any task in it —
which tool wrote it, when, from which project — and has nowhere to put it.

A document-level object would fit JSON and YAML, which wrap the tasks in a
document. It would not fit the flat serializations: JSONL is one task per
line and CSV one task per row, so neither has anywhere to hang an object
about the export — the version they carry lives on the rows. A property
only some serializations can express is not a property of the model.

## Decision

`metadata` belongs to a task; a document has none. A document is its tasks
and a version.

A tool that needs to record something about the export writes it onto the
tasks it applies to, or keeps it outside the document.

## Consequences

Every serialization can carry everything the model defines, and a reader
never looks for a second metadata block. The cost is that export-level
facts have no home in OPS, so a tool that needs them invents a per-task key
or keeps them beside the file — and a real need there is a proposal to
overturn this decision.
