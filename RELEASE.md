# Releasing Flyleaf

The release loop lives in `~/notes/releasing.md` — the ordered steps, the apt
step, crates.io, and the spent-tag rule. Failure recipes are in
`~/notes/build_release_gotchas.md`. The loop is slipcase-desktop's, which is
the hand-cut one: no workflow creates the release, so publishing it is a
command you type, and that is what starts the packaging.

| | |
|---|---|
| Loop | hand-cut |
| Version lives in | `version` in the workspace `Cargo.toml` |
| Read by | `packaging/version.sh`, and nothing else |
| `apt-ship` argument | `flyleaf` |
| Packages per release | two, amd64 and arm64, built by `linux.yml` |
| crates | `flyleaf-core`, then `flyleaf` |
| Store lanes | Microsoft Store (MSIX), App Store |

`packaging/windows/SUBMITTING.local.md` and `packaging/macos/SUBMITTING.local.md`
are named here and not committed, because each carries an account's own
identifiers. `packaging/windows/identity.psd1` is the same and has an
`identity.psd1.example` beside it.

## The order

Debian first: the tag, the crates, the GitHub release, apt. Then each store
when its lane is ready, in whichever order the lanes are available, each behind
its own readiness review reading the listing against that platform's built
artefact. apt may be ahead of a store for a while, and that is a stated fact
rather than an exception. Amended fleet-wide on 2026-09-08; the reasoning is in
`~/notes/releasing.md` and in slipcase-desktop's `RELEASE.md`.

## One number, four spellings

The workspace `Cargo.toml` holds the version and nothing else should.
`packaging/version.sh` is the only thing that reads it.

| Where | Shape | Rule |
| --- | --- | --- |
| `Cargo.toml` | `X.Y.Z` | The source. |
| `AppxManifest.xml` | `X.Y.Z.0` | Four parts, and the Store requires the fourth to be `0`. |
| `Info.plist` `CFBundleShortVersionString` | `X.Y.Z` | What a person sees. |
| `Info.plist` `CFBundleVersion` | the first-parent commit count | Must increase on every upload, including a rejected one resubmitted unchanged. |

**Bump only for a number that has been tagged.** A code change costs a
certification re-run either way; only a published number costs a version as
well.

## What is different about this application

**Two crates ship, and one of them is somebody's dependency.** `flyleaf-core`
and `flyleaf` publish together from one `cargo publish --workspace`, core
first, and the widget is the editor Slipcase Desktop draws — from crates.io at
`flyleaf = "0.2"`, never a path dependency. So this repository runs a step the
applications do not: **the consumer has to be released to carry the fix.** The
caret requirement means no manifest edit for a patch, which is what makes this
easy to miss — `cargo update -p flyleaf` there picks the new widget up, and
until Slipcase Desktop cuts its own release nobody is running it. Nothing fails
meanwhile, and no report can see it; the same blind spot `xaddr/RELEASING.md`
describes for a library.

**The tag publishes the crates before the release page exists.**
`publish-crate.yml` fires on the `v*` tag, and the release is created by hand
afterwards. So crates.io is the first channel to go live here, and a tag pushed
without following through leaves two published crates pointing at a version
with no release, no package, and nothing in apt. `ship flyleaf` is what says so.

**The Linux packages come from CI, not from this machine.** Publishing the
release fires `linux.yml`, which builds amd64 and arm64 on runners of their
own architecture, puts both through the checks a push gets, installs each one
and asks whether the loader will take the executable, and attaches them.
`apt-ship` reads what the release carries, so it waits for that run.

Until 2026-09-09 the package was built here by hand and there was only ever
one of it, which is why the arm64 half of the apt repository has nothing in it
for Flyleaf and will stay that way until the next release. `build-deb.sh`
still works and is what the workflow calls; running it locally is for looking
at a package, not for shipping one.

**`windows.yml` and `apple-silicon.yml` test, they do not package.** They run
the suite where nothing else can and check what only that platform can answer.
The MSIX and the `.app` come from `packaging/windows/` and `packaging/macos/`,
run on those machines by hand. `linux.yml` is the exception and the only
workflow that produces something a person installs.

## The steps

    ./packaging/version.sh                      # confirm the number
    cargo test --workspace
    ./packaging/linux/check-libraries.sh        # needs a display; run it

    git tag v0.2.2 && git push origin main --tags
    gh release create v0.2.2 --title 'Flyleaf 0.2.2' --notes-file <file>
    # linux.yml attaches both .deb packages; wait for it
    gh run watch "$(gh run list --workflow=linux.yml --limit 1 --json databaseId --jq '.[0].databaseId')"
    apt-ship flyleaf v0.2.2

Create the release yourself rather than from a workflow: a release created with
the default `GITHUB_TOKEN` fires no event, and on a hand-cut repo the event is
what the loop is built on. Attach the `.deb` to the release before `apt-ship`,
which downloads what the release carries.

Author: David M. Anderson. Built with AI assistance (Claude, Anthropic).
