#!/usr/bin/env bash
# Renders packaging/manifest.json.tmpl to stdout. Used by both the CI
# manifest-validation job and the release workflow, so the two can't drift.
#
# Usage: render-manifest.sh <version> <entry-point-basename> <platform> <tools-json-file>
#   e.g. render-manifest.sh 0.1.0 frontmatter-mcp linux tools.json
#
# <tools-json-file> holds a JSON array of {name, description} (see
# tool-list.sh); it is spliced into the manifest's "tools" field.
set -euo pipefail

if [ "$#" -ne 4 ]; then
  echo "usage: $0 <version> <entry> <platform> <tools-json-file>" >&2
  exit 2
fi

version=$1
entry=$2
platform=$3
tools_file=$4
here=$(dirname "$0")

sed \
  -e "s/__VERSION__/${version}/g" \
  -e "s/__ENTRY__/${entry}/g" \
  -e "s/__PLATFORM__/${platform}/g" \
  "${here}/manifest.json.tmpl" \
  | jq --slurpfile tools "$tools_file" '.tools = $tools[0]'
