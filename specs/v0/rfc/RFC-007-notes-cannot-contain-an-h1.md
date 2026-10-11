# RFC-007: `notes` cannot contain an H1

*Scope: v0 markdown format.*

## Context

Long-form text is usually written somewhere else — in a document, a ticket,
a thread — and Markdown is a common way to write it. Such a text tends to
open with a level-1 heading: the heading it had before it became a note.

In a Markdown document the H1 position belongs to the title. A column-zero
ATX heading is a task, and a level-1 heading is a root, so a `#` line
inside `notes` does not stay in the notes: it reads back as a new root,
splitting the task it belonged to and inventing a sibling nobody wrote.
That is a property of the Markdown format, not of the model.

## Decision

A Markdown note must not contain a level-1 ATX heading. The H1 is the
title's, and a writer must reject a note that carries one.

## Consequences

The constraint belongs to the Markdown format, not to the model: a note may
carry an H1 in the model, and every other format carries it unchanged. Only
a Markdown writer refuses it, because only Markdown would read it back as
something else.

It is not a general ban on headings in notes, and it is not a claim that
notes and structure are otherwise unambiguous: a deeper column-zero heading
inside a note still reads back as a child task, and a line shaped like a
checkbox or a metadata bullet still reads back as a subtask or as metadata.
This RFC settles the level-1 case; the rest is left to the format document.

A tool that is handed notes containing an H1 may strip or demote it before
writing. What a writer must not do is emit the document anyway and let the
note read back as a task.
