# RFC-028: An attachment value is a URL or a relative path

*Scope: v0 model.*

## Context

Where a file is depends on how the document travels: a tracker keeps it
behind a URL, a directory or zip export keeps it beside the document.

## Decision

We recommend that an attachment value be a URL — `https://` (presigned
URLs included) or `file://` — or a path relative to the document. An
absolute location is a `file://` URL, not a path.

## Consequences

One export serves both worlds: a tracker keeps its URLs and a snapshot
that ships its files keeps its paths. A presigned URL expires, so a
document holding one goes stale.
