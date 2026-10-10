# RFC-020: The recommended priority scale is `A`, `B`, `C`, `D`

## Context

Priority is the key every tool has and every tool spells its own way:
`P0`–`P4`, `high`/`medium`/`low`, `1`–`5`, `critical`/`minor`. Numbers
make it worse, because the direction is a convention too — is `1` the most
urgent priority or the least? A reader comparing two exports guesses, and
a human reading one has to hold the tool's scale in mind.

The value is the tool's own and stays that way: a scale is not shared, so
nothing here can be a rule, and a reader keeps passing the value through.

What is missing is a scale for the tool that has none, and for the common
case of an export a human reads. The Eisenhower matrix gives one that is
already understood: two questions — is it important, is it urgent — make
four classes in an obvious order.

## Decision

A tool that needs to spell priority is recommended to use the four
classes:

| Value | Meaning |
|-------|---------|
| `A` | urgent and important |
| `B` | important, not urgent |
| `C` | urgent, not important |
| `D` | neither urgent nor important |

`A` first: the order is the order of attention.

This is a recommendation, not a rule. `priority` stays a convention — a
document with `priority: high` or `priority: 3` is valid, a tool that
already has a scale is recommended to keep it under a namespaced key
(RFC-019), and no reader compares priorities across tools or assumes they
mean these four classes.

## Consequences

A tool with no scale of its own has one to use, and two OPS exports read
side by side can be compared by eye. That is what a recommendation buys,
and all it buys: nothing is enforced, no reader may reject a document for
its priority values, and no tool has to adopt the scale.

Absence is not a value. A task with no priority has none; `D` says the
task was weighed and placed last, not that nobody set anything.
