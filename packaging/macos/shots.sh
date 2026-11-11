#!/bin/sh
# The Mac App Store screenshots, as recipes rather than as prose.
#
# `screenshot.sh` beside this is the driver and knows nothing about Tommy
# Flyleaf: it sizes the window, fronts it, drives the actions given, parks the
# pointer off the frame, captures, and refuses a result of the wrong size. This
# file is the other half, the part that is this application's - which document,
# which shot, and what has to happen in the window before the shutter.
#
#     ./packaging/macos/shots.sh --app "dist/Tommy Flyleaf.app"
#     ./packaging/macos/shots.sh --app 'dist-dev/Tommy Flyleaf.app' --lang de
#
# WHAT THE SET HAS TO SHOW
#
# An edit is under way in every shot below, and each shows something the
# others do not. A frame of an editor with nothing selected is a frame of a
# viewer, and Save, Undo and Redo greyed out say the window is at rest.
#
# LIGHT AND DARK
#
# Every frame is taken in both appearances, light first: the light frames are
# numbered before the dark repeats, so the file names alone keep light first
# without depending on where `shots()` calls `appearance`.
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
# the listing advertises, and a checkout's path would put a build directory in
# the picture. So the sample is copied somewhere that reads like a person's
# machine, under the name such a document would have, and that copy is what is
# opened.
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
# lines as its counterpart. The coordinates below are a row number in
# disguise, so a comment that wraps onto one more line in one language moves
# every row under it, and a recipe types into the wrong field.
#
# Which document, under which name, per language. The German set opens a German
# document: the interface is translated and the file's own words are not, so a
# German listing showing an English
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
    appearance light

    # The title being rewritten: the field focused and holding new text, the
    # source pane beside it showing the same change, and Save and Undo come on.
    shot 01-editing-a-value \
        --click 350,158 --key cmd+a --type "$typed"

    # A comment being edited in place, with the new words in the source pane
    # beside it. Keeping every comment is what this application promises, and a
    # window at rest cannot show it. A value reaches the source pane as it is
    # typed and a comment does not, so Return commits it and the two panes
    # agree.
    shot 02-editing-a-comment \
        --click 620,158 --key cmd+a --type "$comment" --key return

    # The kind picker open. Every value is edited by its kind, with the four
    # datetime shapes told apart.
    shot 03-the-kind-picker \
        --click 280,398

    appearance dark

    shot 04-editing-a-value-dark \
        --click 350,158 --key cmd+a --type "$typed"

    shot 05-editing-a-comment-dark \
        --click 620,158 --key cmd+a --type "$comment" --key return

    shot 06-the-kind-picker-dark \
        --click 280,398
}

# --- the driving ------------------------------------------------------------

# Which document this language opens, under what name, and what its recipes
# type. `take-shots.sh` beside this does the rest.
for_language() {
    case "$1" in
        en|en-US|en-us)
            document=$english_document; staged_name=$english_name
            typed=$english_typed; comment=$english_comment ;;
        de|de-DE|de-de)
            document=$german_document; staged_name=$german_name
            typed=$german_typed; comment=$german_comment ;;
        *) echo "shots.sh: no set is written for $1" >&2; exit 2 ;;
    esac
}

. "$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)/take-shots.sh"
take_shots "$@"
