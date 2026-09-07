#!/bin/sh
# Install the desktop integration: the desktop entry, the application icon,
# and optionally the binary alongside them, so that a file manager offers
# Tommy Flyleaf for a .toml file and can open one with it.
#
# The media type is not this repository's. `application/toml` has been in
# shared-mime-info since 2.1, in 2021, so every current desktop already knows
# what a .toml file is and this only has to say what opens one. The check at
# the end says so where a machine's database is older than that.
#
# For a person installing by hand and for testing the association without
# building a package. The Debian package, when there is one, ships the same
# files, and the two must agree about where things go.
#
# Author: David M. Anderson
# Built with AI assistance (Claude, Anthropic)
set -eu

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
prefix="${HOME}/.local"
binary=""
found_binary=""
make_default=""

usage() {
    cat <<'USAGE'
usage: install.sh [--prefix DIR] [--binary PATH] [--no-binary] [--default]

  --prefix DIR   where to install (default: ~/.local; use /usr/local for all users)
  --binary PATH  the executable to install into PREFIX/bin
  --no-binary    install the desktop integration only
  --default      make Tommy Flyleaf the default application for .toml files

With neither --binary nor --no-binary, a built executable is looked for under
the cargo target directory, release before debug, and installed if found.
USAGE
}

while [ $# -gt 0 ]; do
    case "$1" in
        --prefix) prefix="${2:?--prefix needs a directory}"; shift 2 ;;
        --binary) binary="${2:?--binary needs a path}"; shift 2 ;;
        --no-binary) binary="none"; shift ;;
        --default) make_default=yes; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "install.sh: unknown argument $1" >&2; usage >&2; exit 2 ;;
    esac
done

# The executable, where one was not named and one was not refused.
#
# Cargo is asked where its target directory is rather than guessed at, because
# `[build] target-dir` in a Cargo configuration file moves it and no environment
# variable then says so; on the machine this was written on it is under
# ~/.cache, and guessing found nothing.
if [ -z "$binary" ]; then
    target_dir=""
    if command -v cargo >/dev/null 2>&1; then
        target_dir=$(cd "${here}/../.." && cargo metadata --format-version 1 --no-deps 2>/dev/null |
            sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
    fi
    [ -n "$target_dir" ] || target_dir="${here}/../../target"

    for candidate in "${target_dir}/release/flyleaf" "${target_dir}/debug/flyleaf"
    do
        if [ -x "$candidate" ]; then found_binary="$candidate"; break; fi
    done
elif [ "$binary" != "none" ]; then
    [ -x "$binary" ] || { echo "install.sh: $binary is not an executable" >&2; exit 1; }
    found_binary="$binary"
fi

mkdir -p \
    "${prefix}/share/applications" \
    "${prefix}/share/icons/hicolor/scalable/apps"

install -m 0644 "${here}/flyleaf.desktop" \
    "${prefix}/share/applications/flyleaf.desktop"
install -m 0644 "${here}/icons/flyleaf.svg" \
    "${prefix}/share/icons/hicolor/scalable/apps/flyleaf.svg"

if [ -n "$found_binary" ]; then
    mkdir -p "${prefix}/bin"
    install -m 0755 "$found_binary" "${prefix}/bin/flyleaf"
    echo "installed ${prefix}/bin/flyleaf from ${found_binary}"
else
    echo "no executable installed; flyleaf must be on PATH for the entry to work"
fi

# Each is absent on a minimal system and each failure is survivable: the files
# are in place either way and the next login or the next package installation
# rebuilds these caches.
[ -x "$(command -v update-desktop-database || true)" ] &&
    update-desktop-database "${prefix}/share/applications" || true
[ -x "$(command -v gtk-update-icon-cache || true)" ] &&
    gtk-update-icon-cache -q -t -f "${prefix}/share/icons/hicolor" || true

echo "installed the Tommy Flyleaf desktop entry and application icon under ${prefix}"

# Asked for, never assumed. A .toml file may already open in somebody's text
# editor by their choice, and an install script that took that over would be
# the kind of thing a person uninstalls over.
if [ -n "$make_default" ]; then
    if [ -x "$(command -v xdg-mime || true)" ]; then
        xdg-mime default flyleaf.desktop application/toml
        echo "made Tommy Flyleaf the default for application/toml"
    else
        echo "xdg-mime is not installed, so the default could not be set" >&2
    fi
fi

# Said rather than assumed, the other way round from a product with a type of
# its own: here the type is the desktop's, and a database old enough to lack
# it makes the entry above one no file manager will offer for a .toml.
if ! grep -qsx 'application/toml' \
        "${prefix}/share/mime/types" \
        /usr/local/share/mime/types \
        /usr/share/mime/types
then
    echo
    echo "This machine's shared-mime-info does not declare application/toml,"
    echo "which arrived in shared-mime-info 2.1 in 2021. Until it is updated,"
    echo "nothing will associate a .toml with this application."
fi

echo
echo "check it with:"
echo "  xdg-mime query filetype SOME.toml       # application/toml"
echo "  xdg-mime query default application/toml # flyleaf.desktop, once --default or chosen in a file manager"
