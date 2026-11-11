# Samples

The TOML files published at <https://excelano.com/flyleaf/samples/>, kept here
for the reason `sample.toml`, `privacy-entry.html` and `store-listing.toml` are:
each is a claim about what the application can open, and the application is
what moves. Edit here first, then `./publish.sh` into the site working copy.

The page gives anyone who wants to try an editor a file to edit, a store
reviewer included; both stores' review notes point here.

**What each file is for.**

`job-ticket.toml` is `packaging/sample.toml` under the name the page gives it,
which is why it is not in this directory: it is the file the screenshots and
the hosted demo open with, and one copy of it is the point.

`every-kind.toml` is the specimen sheet. Every kind of value TOML has, the
four datetime shapes told apart, comments above, beside and below keys, and an
inline table written across lines. Someone verifying that the editor does what
its listing says should open this one: every renderer it claims is on the
screen at once.

`every-kind.toml` carries no `nan`, and no float the editor draws as a
different number from the one in the file, because the float field changes
both on open and a specimen sheet that opens already marked edited
demonstrates the opposite of what it is for. Issue #10 tracks the float field;
they go back into the file when it is fixed.

`cargo-manifest.toml` and `pyproject.toml` are the two files a developer
actually opens, and they are here so that the set is not all specimen. They
describe projects that do not exist.

**Every one round-trips.** `flyleaf-core/tests/samples.rs` opens each file in
this set and asserts it comes back byte for byte, because a published sample
that the editor rewrites is a demonstration of the opposite of the product.
The test globs this directory, so a sample added here is covered by adding it.
