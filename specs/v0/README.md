# OPS version 0

Version 0 of the [OPS Specifications](../README.md). Draft: not released,
so the model may still change.

**The model is the standard.** Everything else here either records a
decision about the model, or maps the model onto a concrete syntax.

## Read order

1. [`model.md`](model.md) — the model: the fields, the rules that make a
   collection of tasks valid, and conformance. The normative document.
2. [`formats/`](formats/README.md) — the mappings onto concrete syntaxes,
   one document per format. Normative for a tool that claims the format;
   nothing requires a tool to claim one.
3. [`tests/`](tests/README.md) — the corpus: portable, language-agnostic
   cases, grouped by format.
4. [`dr/`](dr/README.md) — decision records: what is locked.
5. [`rfc/`](rfc/README.md) — requests for comments: what is still open.
