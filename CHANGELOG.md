# Changelog

The record for people who take these crates from crates.io, where the git log
does not travel. Each entry says what changed for a consumer of `flyleaf-core`
or the `flyleaf` widget; the reasoning is in the commits.

## 0.2.1 — 2026-09-07

`flyleaf`, the application:

- An About box, from a button at the bar's right end: the icon, the version,
  what the application is, where it comes from, and the name's story. Escape,
  the backdrop or its button closes it.
- The icon: a bound book with its cover folded back to the flyleaf, in the
  oxblood, brass and cream of excelano.com/flyleaf, replacing the placeholder.
  The application paints it from the same geometry, so no raster ships.
- The web build takes a size profile and the glow renderer, 4.8 MB from
  7.5 MB, and opens on a sample file. Its page carries the icon.

## 0.2.0 — 2026-09-07

The general editor. `flyleaf-core`:

- `Document`: a file's tree with the three things `toml_edit` drops put back on
  render (byte order mark, CRLF, a missing final newline), an edited baseline,
  undo and redo with edits to one row coalesced, `lines_of` for the position of
  a path in the rendered text, and `from_path` and an atomic `save_to` behind
  the default `fs` feature.
- `Kind` replaces `NewKey` and names all eleven kinds, the four datetime shapes
  told apart. `convert` and `convert_key` change a value's kind where it reads
  as the target, and a table becomes an inline table and back in place.
- Array operations: `push_element`, `remove_element`, `set_element`,
  `convert_element`.
- Comments: `comments_before`, `comment_beside`, `trailing_comments` and their
  setters, keeping the whitespace that shares the decor.
- Every valid TOML 1.1.0 case of toml-test round-trips byte for byte through
  `Document` except seven that `toml_edit` itself changes, named in the test;
  see `UPSTREAM.md`.

`flyleaf`:

- `render` returns the path of the row with keyboard focus; `open_all` opens or
  closes every section; `forget_typing` drops in-progress field text, which a
  caller must do before undoing. Sections start closed above 200 entries.
- Arrays are editable sections. Every row and section has a kind menu offering
  the conversions the value allows and a comment to add. Comments are fields:
  edited on blur, removed when emptied.
- `source` draws the rendered document with a range of lines highlighted.
- A comment above a key is a line of its own rather than part of a label after
  the value.
- The application binary, behind the default `app` feature: open, save, save
  as, undo, redo, the source pane, and a prompt before unsaved changes are lost.

## 0.1.0 — 2026-09-07

The editor as extracted from slipcase-desktop: the tree with one renderer per
TOML type, scalar editing, add, rename and remove for tables and inline tables,
and a `Policy` for what an application protects.
