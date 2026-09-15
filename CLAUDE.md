# CLAUDE.md

Tommy Flyleaf is a structure-aware TOML editor. `flyleaf-core` is the document model over
`toml_edit`, the edit operations and what a save writes, with no user interface; `flyleaf`
is the editor as an egui widget, which slipcase-desktop draws inside its own window, plus
the application binary behind the default `app` feature. `DESIGN.md` is the authority.

## Commands

    cargo build --all-targets
    cargo test
    cargo clippy --all-targets -- -D warnings             # must be silent
    cargo fmt --check
    cargo +1.95 build --all-targets                       # the floor, read from Cargo.toml
    cargo check -p flyleaf --no-default-features          # the widget alone
    cargo run -- path/to/file.toml
    ./flyleaf/po/update-po.sh          # after changing any sentence a person reads
    ./flyleaf/po/pseudo.sh             # then a debug build with POTEXT_LANG=en-x-pseudo
    cargo check -p flyleaf --target wasm32-unknown-unknown
    trunk build --release                                 # the web page, into dist/
    ./packaging/debian/build-deb.sh                       # after a release build
    ./packaging/linux/check-libraries.sh   # needs a display; run it after touching a dependency
    cargo clippy -p flyleaf --all-targets --target x86_64-pc-windows-msvc -- -D warnings
    cargo clippy --workspace --all-targets --target aarch64-apple-darwin -- -D warnings

Windows and macOS each have their lane in their `packaging/` README, and are changed only
on their own machine. Releases: run `ship flyleaf`. There is no release document.

## Rules

`flyleaf-core` and the widget are `forbid(unsafe_code)`; the binary is `deny`, with one
`allow` on `src/opened_document.rs`, where receiving a macOS Apple Event means defining an
Objective-C class. A second module wanting it is David's decision. Nothing in the tree
compiles C: `ci.yml` refuses any `.o` or `.a` under the build directory and holds `ldd` to
libc, libgcc_s and libm (`~/notes/pure_rust_preference.md`). `flyleaf-core`, `slpc` and
slipcase-desktop resolve to one `toml_edit`, or `DocumentMut` becomes two types; the
manifests say so beside the number and move together. The widget never mutates the
document: every change goes through core, so undo has one chokepoint, and an operation
core lacks goes into core. Round-trip fidelity is the product — comments, key order,
whitespace, quoting and table layout survive an edit elsewhere in the document, and the
goldens make losing any of it a decision. Never run `cargo fmt` in slipcase-desktop; it is
not fmt-clean and is edited by hand. Every test's doc comment says what defect it would
catch, and a new test is broken deliberately once to watch it fail. The commit trailer is
one line, a `Co-Authored-By` naming the model: this repository is public, so no session URL.
