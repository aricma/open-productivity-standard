# RFC-029: Attachments are not validated or resolved

*Scope: v0 model.*

## Context

An attachment is a reference, and a reference may point at a file that no
longer exists, a URL that has expired, or a path only the exporter's
machine understands. The OPS reader and writer describe tasks; finding
files is the surrounding tool's business, not theirs.

## Decision

A reader does not validate an attachment and a writer accepts it as given:
the OPS reader and writer carry the reference through verbatim, without
checking that it points anywhere or fetching it.

The surrounding code — the application that imports or exports — resolves
the references, when it needs the files, with the access it has.

## Consequences

The reader and writer stay simple and stay about tasks: a document with a
dead link is still valid, and a round trip keeps every reference. A tool
that resolves them reports the failures — OPS does not.
