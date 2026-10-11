# Markdown

Markdown is tree-preserving and follows the document's outline. ATX
headings at column zero are tasks: heading level `L` is the task's depth,
so `#` is a root and `######` its six-level descendant. A heading's title
is the text after the `#`s. A heading that skips a level attaches to the
nearest shallower one. (RFC-016)

A heading is `open` unless its title begins with `(Done) `, which makes it
`done`; `\(Done) ` at the start is a literal `(Done) ` and stays `open`.

A task's children may be deeper headings or checkbox list items, mixed:

- `- [ ]` is `open`, `- [x]` is `done`; the rest of the line is the title.
- A checkbox at column zero belongs to the nearest heading. A checkbox
  belongs to the nearest task whose content indent it matches: a heading's
  content indent is column zero, a checkbox's is its own indent plus two.
- A checkbox with no preceding task is a root.

Optional YAML frontmatter carries the document `version` and an optional
root `status` default: a root's `(Done) ` overrides it, and non-root
headings are never affected. Any other frontmatter key is invalid, and a
leading `---` block that is not a mapping is ordinary content.

Any other line in a task's scope is a note on that task; `- key: value`
with a valid metadata key is metadata — the value is the text after the
colon, so a value read from Markdown is a string. (RFC-015) A value with a
line break is written on one line, double-quoted with the break escaped
(`\n`, JSON-style), and a reader decodes the escapes back to the real
characters. Content before the first task is not part of the document.

## Notes carry no level-1 heading

A task's `notes` must not contain a level-1 ATX heading. The H1 belongs to
the title: a column-zero `#` line reads back as a new root, splitting the
task it belonged to and inventing a sibling nobody wrote. A writer must
reject such a note rather than emit a document that reads back as
something else. (RFC-007)

It is not a general ban on headings in notes, and it does not make notes
and structure unambiguous overall: a deeper column-zero heading inside a
note reads back as a child task, and a line shaped like a checkbox or a
metadata bullet reads back as a subtask or as metadata. Those collisions
are a known gap, to be settled when this format is implemented.

```markdown
---
version: "0"
status: open
---

# Acme product backlog

The product backlog for the Acme platform.

- [ ] Fix memory leak in auth service
  - priority: high
  - [ ] Reproduce the leak

## (Done) Shipped in 2026.3

- [x] Retire the legacy importer
```
