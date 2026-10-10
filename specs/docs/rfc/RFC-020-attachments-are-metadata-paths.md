# RFC-020: Attachments are metadata paths

## Context

A task often needs files beside it — a screenshot, a spec, a log. The model
stores none of them: a task list is not storage, and the tool that holds
the file (a tracker, a repository, a drive) holds it its own way. What the
task needs is the reference, so a reader can find the file.

Attachments belong in the metadata, and the common keys name them. Two
things are still open: what a path may be, and whether a single attachment
is also a list.

## Decision

A task's attachments are the `attachments` key, whose value is a list of
strings. Each string is one path: a URL, or a file path relative to the
export (absolute is allowed when the writer has no better anchor).

A writer always writes a list, even for a single attachment. A reader
accepts one path as a one-element list, so a hand-written document does
not have to bracket it.

A format that carries no lists carries the JSON text of this one, as it
does for every other non-string value.

## Consequences

The files stay where the tool keeps them and the document says where they
are, so the same export works for a tracker that links and a repository
that stores alongside.

A path promises a location, not reachability: a reader that cannot fetch
one keeps it. Order in the list is the writer's, and a reader must not read
meaning into it.
