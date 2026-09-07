#!/bin/sh
# Remove what install.sh put in place, and rebuild the caches that indexed it.
#
# The media type is the desktop's own and is left alone. A default set with
# --default is undone, since a default pointing at an entry that is gone is
# what a file manager reports as a broken association.
#
# Author: David M. Anderson
# Built with AI assistance (Claude, Anthropic)
set -eu

prefix="${HOME}/.local"
keep_binary=""

while [ $# -gt 0 ]; do
    case "$1" in
        --prefix) prefix="${2:?--prefix needs a directory}"; shift 2 ;;
        --keep-binary) keep_binary=yes; shift ;;
        -h|--help)
            echo "usage: uninstall.sh [--prefix DIR] [--keep-binary]"; exit 0 ;;
        *) echo "uninstall.sh: unknown argument $1" >&2; exit 2 ;;
    esac
done

rm -f \
    "${prefix}/share/applications/flyleaf.desktop" \
    "${prefix}/share/icons/hicolor/scalable/apps/flyleaf.svg"

[ -n "$keep_binary" ] || rm -f "${prefix}/bin/flyleaf"

# The user's own list of defaults, edited only where it names this entry.
mimeapps="${XDG_CONFIG_HOME:-${HOME}/.config}/mimeapps.list"
if [ -f "$mimeapps" ] && grep -qs 'flyleaf.desktop' "$mimeapps"; then
    sed -i '/=flyleaf\.desktop;\{0,1\}$/d' "$mimeapps"
fi

[ -x "$(command -v update-desktop-database || true)" ] &&
    update-desktop-database "${prefix}/share/applications" || true
[ -x "$(command -v gtk-update-icon-cache || true)" ] &&
    gtk-update-icon-cache -q -t -f "${prefix}/share/icons/hicolor" || true

echo "removed the Tommy Flyleaf desktop integration from ${prefix}"
