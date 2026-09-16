#!/bin/sh
# The Mac App Store screenshots, as recipes rather than as prose.
#
# `screenshot.sh` beside this is the driver and knows nothing about Tommy
# Flyleaf: it sizes the window, fronts it, drives the actions given, parks the
# pointer off the frame, captures, and refuses a result of the wrong size. This
# file is the other half, the part that is this application's - which document,
# which shot, and what has to happen in the window before the shutter.
#
#     ./packaging/macos/shots.sh --app 'dist-dev/Tommy Flyleaf.app'
#     ./packaging/macos/shots.sh --app 'dist-dev/Tommy Flyleaf.app' --lang de
#
# WHAT THE SET HAS TO SHOW, AND WHY
#
# The set this replaces was one window at rest, photographed twice in two
# appearances. Nothing was selected, no edit was under way, and Save, Undo and
# Redo were all greyed out. Apple rejected Segler's first set under guideline
# 2.3.3 - *the screenshots do not show the actual app in use* - for exactly
# that, and the same frames were sitting in this listing. A frame of an editor
# with nothing selected is a frame of a viewer, and this application stopped
# being a viewer at 0.2.0.
#
# So: an edit is under way in every shot below, and each shows something the
# others do not.
#
# THE ACTIONS STAY INSIDE THE DOCUMENT
#
# Coordinates are read off a frame at the size declared here, and the document
# area is the only part of the window that holds still between languages: the
# tree and its fields are laid out by the file, which is the same file. The
# toolbar is not. `Source` sits 240 pixels left of `Quelltext`, because every
# label before it is longer in German, so a recipe that clicked a toolbar
# button would photograph one thing in English and another in German.
#
# THE DOCUMENT IS STAGED
#
# The path is in the frame, in the toolbar, so whatever is photographed is what
# the listing advertises. Photographing the checkout put
# `/private/tmp/ship.DynEoj/repo/packaging/sample.toml` in the picture, and a
# runner would put its own workspace there. So the sample is copied somewhere
# that reads like a person's machine, under the name such a document would
# have, and that copy is what is opened.
#
# Author: David M. Anderson
# Built with AI assistance (Claude, Anthropic)

set -eu

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
root=$(CDPATH= cd -- "${here}/../.." && pwd)

# --- the configuration ------------------------------------------------------

# App Store Connect takes 1440x900 for macOS, and every coordinate below was
# read off a frame of that size. Change it and they all move.
WIDTH=1440
HEIGHT=900

OUTDIR="${root}/dist/screenshots"

# The two documents are the same document in two languages, and they are the
# same length: 55 lines each, with every comment occupying the same number of
# lines as its counterpart. That is a constraint rather than a nicety. The
# coordinates below are a row number in disguise, so a German header comment
# that wrapped onto a fifth line where the English takes four moved every row
# under it down by one, and shot 01 typed its new title into the comment.
#
# Which document, under which name, per language. The German set opens a German
# document: the interface is translated and the file's own words are not, which
# is this application's whole promise, so a German listing showing an English
# document would be mostly English pixels.
english_document="${root}/packaging/sample.toml"
english_name="job-ticket.toml"
german_document="${root}/packaging/sample.de.toml"
german_name="auftragszettel.toml"

# What gets typed into the title. Short enough to be read at a glance in a
# listing, and different from what is in the file so the edit is visible.
english_typed="Job ticket — rebind"
german_typed="Auftragszettel — Neubindung"

# What gets typed into the comment beside it. The `#` is the field's own and is
# not part of what is edited.
english_comment="printed at the head of the ticket"
german_comment="wird oben auf dem Zettel gedruckt"

# --- the shots --------------------------------------------------------------

# One shot to a line: name, then the actions that put the window into the state
# being photographed, in the order they are given.
#
#   --click X,Y    press a control
#   --double X,Y   press it twice inside the double-click interval
#   --type TEXT    type
#   --key NAME     one key, optionally with modifiers: cmd+a, return
shots() {
    # The title being rewritten: the field focused and holding new text, the
    # source pane beside it showing the same change, and Save and Undo come on.
    # This is the frame the old set most lacked - the application in use.
    shot 01-editing-a-value \
        --click 350,158 --key cmd+a --type "$typed"

    # A comment being edited in place, with the new words in the source pane
    # beside it. Keeping every comment is what this application promises, and a
    # window at rest cannot show it. Two attempts got this wrong. Clicking
    # alone put the caret in the comment and changed nothing, so Save stayed
    # grey and the frame read as the same window at rest. Typing was not enough
    # either: a value reaches the source pane as it is typed and a comment does
    # not, so the panes disagreed and the frame read as a defect. Return is
    # what commits it.
    shot 02-editing-a-comment \
        --click 620,158 --key cmd+a --type "$comment" --key return

    # The kind picker open. Every value is edited by its kind, with the four
    # datetime shapes told apart, and nothing in the listing showed it.
    shot 03-the-kind-picker \
        --click 280,398
}

# --- the driving ------------------------------------------------------------

app=""
lang=""
while [ $# -gt 0 ]; do
    case "$1" in
        --app) app="${2:?--app needs a bundle}"; shift 2 ;;
        --lang) lang="${2:?--lang needs a language tag}"; shift 2 ;;
        --outdir) OUTDIR="${2:?--outdir needs a directory}"; shift 2 ;;
        -h|--help) sed -n '2,45p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "shots.sh: unknown argument $1" >&2; exit 2 ;;
    esac
done

[ -n "$app" ] || {
    echo "shots.sh: --app needs the bundle to photograph" >&2
    echo "  ./packaging/macos/shots.sh --app 'dist-dev/Tommy Flyleaf.app'" >&2
    exit 2
}
[ -d "$app" ] || { echo "shots.sh: no bundle at $app" >&2; exit 1; }

case "$lang" in
    ""|en|en-US) document=$english_document; name=$english_name
                 typed=$english_typed; comment=$english_comment
                 OUTDIR="${OUTDIR}/en-US" ;;
    de|de-DE)    document=$german_document;  name=$german_name
                 typed=$german_typed;  comment=$german_comment
                 OUTDIR="${OUTDIR}/de-DE" ;;
    *) echo "shots.sh: no set is written for $lang" >&2; exit 2 ;;
esac
[ -f "$document" ] || { echo "shots.sh: no document at $document" >&2; exit 1; }

# Somewhere that reads like a person's machine rather than like a build.
staged="${HOME}/Documents"
mkdir -p "$staged"
cp "$document" "${staged}/${name}"
document="${staged}/${name}"

mkdir -p "$OUTDIR"

taken=0
shot() {
    shot_name=$1
    shift
    echo "shots.sh: ${shot_name}"
    "${here}/screenshot.sh" \
        --app "$app" \
        --document "$document" \
        --width "$WIDTH" --height "$HEIGHT" \
        ${lang:+--lang "$lang"} \
        --out "${OUTDIR}/${shot_name}.png" \
        "$@"
    taken=$((taken + 1))
}

shots

# A set that came back short is a set somebody submits without noticing, so the
# count is said rather than left to be counted.
echo "shots.sh: ${taken} shot(s) in ${OUTDIR}"
