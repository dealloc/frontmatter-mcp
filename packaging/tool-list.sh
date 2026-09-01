#!/usr/bin/env bash
# Asks a built frontmatter-mcp binary for its tool list over stdio and
# emits a compact JSON array of {name, description} - the shape the .mcpb
# manifest's optional "tools" field wants. Generating it this way means the
# manifest never drifts from what the server actually exposes.
#
# Usage: tool-list.sh <path-to-binary>
set -euo pipefail

bin=$1

printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"manifest-gen","version":"0"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
  | "$bin" 2>/dev/null \
  | jq -c 'select(.id == 2) | [.result.tools[] | {name, description}]'
