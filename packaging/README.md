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

**One sample and one statement.** `sample.toml` is what the screenshots, the
marketing page and the hosted demo open with, and `privacy-entry.html` is the
privacy statement copied verbatim into excelano.com/legal.
