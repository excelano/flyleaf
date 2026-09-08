# Packaging

One directory per platform, plus `debian` for the way Linux is distributed.
Each platform's README says what was decided there; the measurements behind
most of them were made in slipcase-desktop, whose `packaging/` these were
adapted from, and are recorded there rather than restated.

**One drawing.** `linux/icons/flyleaf.svg` is the source of every platform's
icon: Linux ships it under hicolor, `windows/make-ico` rasterizes it into
`windows/flyleaf.ico` and the Store assets, which are committed because
nothing on Windows rasterizes an SVG, and `macos/build-app.sh` renders it into
the `.icns` at build time. The application also paints it from the same
geometry in its About box. Any change to the drawing is checked at 16, 24,
32, 48 and 128 pixels on a light ground and a dark one, and `make-ico` is run
again so `windows.yml` finds the committed rasters current.

**One version.** `version.sh` is the only thing that reads it out of
`Cargo.toml`, in whichever spelling a platform wants.

**One sample, one statement and one listing.** `sample.toml` is what the
screenshots, the marketing page and the hosted demo open with;
`privacy-entry.html` is the privacy statement copied verbatim into
excelano.com/legal; and `store-listing.md` is the copy a store listing is
filled in from, including the notes a reviewer reads. All three are here rather
than in the place they are pasted into, because each is a set of claims about
the code and the code is what moves. Edit here first. The identity a store
assigns is the one thing that stays out: `windows/identity.psd1` holds it and
is not committed.
