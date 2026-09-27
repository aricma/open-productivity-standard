# RFC-007: `notes` cannot contain an H1

## Context

Long-form text is usually written somewhere else — in a document, a ticket,
a thread — and Markdown is a common way to write it. Such a text tends to
open with a level-1 heading: the heading it had before it became a note.

In a Markdown document the H1 position belongs to the title. A column-zero
ATX heading is a task, and a level-1 heading is a root, so a `#` line
inside `notes` does not stay in the notes: it reads back as a new root,
splitting the task it belonged to and inventing a sibling nobody wrote.

## Decision

A task's `notes` must not contain a level-1 ATX heading. The H1 is the
title's.

## Consequences

This is a constraint on the value of `notes`, so it holds in every
serialization and not only in Markdown: a document whose notes carry an H1
is invalid, whichever format it arrived in.

It is not a general ban on headings in notes. A deeper heading does not
take the title's place, so this decision leaves it alone — the limitation
is only the one the title itself creates.

A tool that writes notes strips or demotes a leading H1 rather than emit a
document that cannot be read back as the thing it wrote.
