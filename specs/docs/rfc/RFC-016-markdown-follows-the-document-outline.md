# RFC-016: Markdown follows the document outline

## Context

Markdown already has a tree: headings nest, and checkbox lists hang off
them. The first draft ignored that and required a document to be a single
`#` root with every task a checkbox beneath it — a data file written in
Markdown, not a document. It cannot read an ordinary document such as a
README, or this repository's own `TODOs.md`: every section heading becomes
a note and its tasks flatten onto the root.

The alternative was to leave heading text as prose and treat only
checkboxes as tasks. That is simpler and cannot mistake a section for a
task, but it throws away the structure a reader sees — the hierarchy the
document shows is not the hierarchy the model gets, and the sections that
give the tasks meaning disappear.

## Decision

ATX headings at column zero are tasks: heading level `L` is the task's
depth, so `#` is a root and `######` its six-level descendant. A heading
that skips a level attaches to the nearest shallower one. A document that
never reaches `#` is a forest whose top headings are roots.

A task's children may be deeper headings or checkbox list items, mixed —
they are two spellings of the same hierarchy, and which one a document
uses does not change the model. A heading is `open` unless its title
begins with `(Done) `, which makes it `done`; frontmatter may carry a
document `version` and a root `status` default that a local `(Done) `
overrides.

## Consequences

An ordinary Markdown document imports as the tree it shows, `TODOs.md`
included, and headings and checkboxes convert into one another freely.

The trade-offs: a section heading becomes a task, so a prose-only
document gains a task per heading, each carrying that section's text as
notes; a heading title that starts with `(Done) ` must be escaped as
`\(Done) `; and a heading level jump is normalized on write, since the
writer works from tree depth and chooses how many levels are headings via
`Format::Md { heading_levels }`. The same model may therefore be spelled
differently on the way out.
