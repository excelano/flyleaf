# Tommy Flyleaf

A structure-aware TOML editor: a desktop application that shows a document as a tree, edits values by type, and keeps comments, key order and formatting when it saves. Rust, egui.

The editor widget is shared with [Slipcase Desktop](https://github.com/excelano/slipcase-desktop), which it was extracted from. `PROMPT.md` is the living plan.

    cargo run -- path/to/file.toml

opens a file and shows it as a tree. Edits stay in memory for now; saving arrives with open and save-as.
