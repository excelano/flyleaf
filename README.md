# Tommy Flyleaf

A structure-aware TOML editor: a desktop application that shows a document as a tree, edits values by type, and keeps comments, key order and formatting when it saves. Rust, egui.

The editor widget is shared with [Slipcase Desktop](https://github.com/excelano/slipcase-desktop), which it was extracted from. `PROMPT.md` is the living plan.

    cargo run -- path/to/file.toml

opens a file and shows it as a tree beside what a save would write. Ctrl+O, Ctrl+S and Ctrl+Shift+S open, save and save as; Ctrl+Z and Ctrl+Shift+Z undo and redo. A save keeps every comment, the key order, the whitespace and the quoting of everything that was not edited.

On Linux, `packaging/linux/install.sh` installs the desktop entry and icon under `~/.local` so a file manager offers Tommy Flyleaf for `.toml` files; `--default` makes it the default.
