#!/bin/sh
# publish.sh DIR — copy the published sample set into DIR, which is the site
# repository's flyleaf/samples/.
#
# The set is `packaging/sample.toml`, which goes over as job-ticket.toml
# because that is what it is and "sample" says nothing beside four other
# samples, plus every .toml beside this script. One command rather than four
# `cp` lines, so that the copies cannot drift from the originals and a sample
# added here reaches the page by being added here.
#
# Unlike the web bundle, these files are committed on the site side: App
# Review downloads them, Apple asked for an address that stays put, and a
# gitignored directory is one deploy away from being empty.
#
# Author: David M. Anderson
# Built with AI assistance (Claude, Anthropic)
set -eu

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
dest="${1-}"

if [ -z "$dest" ]; then
    echo "usage: publish.sh DIR   (e.g. ~/clones/excelano.com/flyleaf/samples)" >&2
    exit 2
fi
if [ ! -d "$dest" ]; then
    echo "publish.sh: $dest is not a directory" >&2
    exit 1
fi

cp "${here}/../sample.toml" "${dest}/job-ticket.toml"
for sample in "${here}"/*.toml; do
    cp "$sample" "${dest}/"
done

# A sample taken out of the set here would otherwise stay on the page for
# ever, since nothing on the site side removes it. Said rather than deleted:
# this script is handed a directory, and a script that deletes files in a
# directory it was handed is one typo from taking something else with it.
for present in "${dest}"/*.toml; do
    name=$(basename "$present")
    if [ "$name" != "job-ticket.toml" ] && [ ! -f "${here}/${name}" ]; then
        echo "note: ${name} is on the page and no longer in the set; remove it by hand"
    fi
done

ls -l "$dest"
