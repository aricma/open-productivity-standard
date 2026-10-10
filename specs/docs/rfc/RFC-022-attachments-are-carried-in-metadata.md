# RFC-022: Attachments are carried in `metadata`

## Context

Some platforms let users attach files to tasks. The attachment is useful
extra information about a task, but not the task itself: DR-001 defines
the task as its title and status, so attachments are metadata.

## Decision

Attachments are carried in `metadata`. The model has no attachment field,
and an attachment is a reference, never the file itself.

## Consequences

A task with no attachments writes none, and a reader that does not know
the key passes it through unchanged.
