# RFC-006: The field is called `notes`

## Context

The optional multi-line string needs a name. The obvious candidate,
`description`, is narrower than the field it would name. A description
suggests a statement of what the task is, which presumes the task has a
shape worth describing — and plenty do not. Most to-dos need no
description at all.

What a to-do actually needs is context: a hint, a reminder of why it
exists, what it is for, or information that helps do it. That is broader
than a description, and it is the shape we already know — a plain notepad
with a few to-dos on it, and some notes beside them giving the context,
the hints, and the extra information the one-line entry could not hold.

## Decision

The field is called `notes`.

## Consequences

Broader meaning, same mapping. A tool whose own model calls the field a
description maps it onto `notes`; a tool that calls it a body, a comment,
or a memo does too.

A narrower name would have implied a narrower field, and the pressure to
add a second field for "extra information" would split one body of text
across two places — a tool would then have to guess which one to read, and
a round trip would have to guess which one to write. One name, one field,
one body of text.
