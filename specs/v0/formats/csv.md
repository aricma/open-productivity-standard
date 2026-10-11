# CSV

CSV carries the flat family: one task per row. The first row is the
header and names the columns; it is required. Column order is free, and
this is the recommended order:

`id`, `title`, `status`, `version`, `notes`, `subtasks`, `metadata`,
then one column per metadata key.

Columns come in two kinds:

- **Reserved columns** are the top-level field names (`version`, `id`,
  `title`, `status`, `notes`, `subtasks`, `metadata`) and map to the
  model fields unchanged. The `metadata` column holds a JSON object,
  exactly as in the model.
- **Metadata columns** are every other column. A column names a metadata
  key, optionally prefixed with `metadata_` and optionally suffixed with
  a type:
  1. Strip one leading `metadata_` prefix, if present.
  2. If the prefix was present and the remainder ends in `_json`,
     `_string`, `_number`, or `_bool`, that suffix names the value type
     and is stripped.
  3. The remainder is the metadata key. It must match the metadata key
     charset — a document whose keys break it is invalid.

The prefix is optional and only needed to avoid a collision — with a
reserved column, or because the key itself starts with `metadata_`
(such a key is written prefixed, and stripping recovers it). A column
without the prefix is read as the literal key; a type suffix is
recognized only on a prefixed column. One key must not be supplied
twice. (RFC-014)

Cell values:

- A prefixed column with a type suffix is read as that type. If the cell
  cannot be read as it, it is kept as a string — reading never fails.
- Otherwise, a cell that parses as JSON is read as that JSON value, and
  any other cell is kept as a string. So `3` is a number, `true` is a
  boolean, `["a","b"]` is a list, and `2026-07-01` is a string.
- An empty cell means the key is absent, not an empty value.

The `metadata` column and the metadata columns are merged; a metadata
column that repeats a key already in the `metadata` object is invalid, a
duplicate. Any non-reserved column is metadata, so an unexpected column
adds data instead of failing the read — but a row is **no wider than the
header**: extra cells have no column to belong to, and would be read as
nothing, so a wider row is invalid. A row without `title` or `status`
is invalid.
`subtasks` is a JSON array of child ids, as in JSONL, and roots carry no
id — so **every subtask needs one**: the column has nothing else to carry the
hierarchy with, and a writer that cannot name a subtask must refuse the model
rather than let the child out as a root. (RFC-001) `version` is a document property: every root row that declares it must
declare the same value, and absent leaves the choice to the reader. Values
containing commas, quotes, or newlines are quoted per RFC 4180.
