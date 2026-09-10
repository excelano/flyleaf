# Samples

The TOML files published at <https://excelano.com/flyleaf/samples/>, kept here
for the reason `sample.toml`, `privacy-entry.html` and `store-listing.md` are:
each is a claim about what the application can open, and the application is
what moves. Edit here first, then `./publish.sh` into the site working copy.

**Why there is a page at all.** The Mac App Store submission of 0.2.1 came
back on 2026-09-10 under Guideline 2.1(a): the reviewer could not exercise an
editor without a file to edit, and asked for sample TOML "hosted at a location
that will remain available for future reviews". The page is that location. It
is written for anyone who wants something to open, not for App Review alone, a
reviewer being no better served by a page that reads as a formality.

**What each file is for.**

`job-ticket.toml` is `packaging/sample.toml` under the name the page gives it,
which is why it is not in this directory: it is the file the screenshots and
the hosted demo open with, and one copy of it is the point.

`every-kind.toml` is the specimen sheet. Every kind of value TOML has, the
four datetime shapes told apart, comments above, beside and below keys, and an
inline table written across lines. Someone verifying that the editor does what
its listing says should open this one: every renderer it claims is on the
screen at once.

What `every-kind.toml` does not carry is `nan`, or a float the editor draws as
a different number from the one in the file. Not because either is exotic:
because the editor gets both wrong today, and a specimen sheet that opens
already marked edited demonstrates the opposite of what it is for. `nan` becomes `inf` on open, and the two rounding
cases are beside it in `DESIGN.md` section 5, amended 2026-09-10 with what was
measured. They go back into the file when the float renderer is fixed.

`cargo-manifest.toml` and `pyproject.toml` are the two files a developer
actually opens, and they are here so that the set is not all specimen. They
describe projects that do not exist.

**Every one round-trips.** `flyleaf-core/tests/samples.rs` opens each file in
this set and asserts it comes back byte for byte, because a published sample
that the editor rewrites is a demonstration of the opposite of the product.
The test globs this directory, so a sample added here is covered by adding it.
