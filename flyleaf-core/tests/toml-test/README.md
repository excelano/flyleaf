# toml-test, vendored

The TOML 1.1.0 cases of [toml-lang/toml-test](https://github.com/toml-lang/toml-test),
copied from commit `bc8f2c2cca601ea91d482046ea9fef3bf7a26c28` of 2026-08-25,
under that project's MIT license (`LICENSE` beside this file). Only the
`.toml` files named in `files-toml-1.1.0` are here; the `.json` expectations
are not, because `tests/roundtrip.rs` asks a different question of them than
toml-test does. toml-test asks whether a parser reads each valid case as the
right value and refuses each invalid one. This asks whether a valid case comes
back byte for byte after a parse and a render, which is what an editor that
promises round-trip fidelity has to answer, and it still asks that every
invalid case is refused.

Refresh by copying from a newer commit and updating this file; the test names
every case it expects to differ and fails when one stops differing, so a
refresh that fixes something upstream is a deliberate edit here.
