# frontmatter-mcp

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL--3.0-blue.svg)](LICENSE.md)

An [MCP](https://modelcontextprotocol.io/) server that reads only the YAML
frontmatter block of markdown documents — skills, ADRs, PIRs, docs, anything
with a `---`-delimited header — without loading the rest of the document.

## Why this exists

When an agent needs to survey a folder of markdown for metadata — which
skills are `status: active`, who owns each ADR, what `name` each file
declares — it's stuck choosing between reading every file whole (spending
its context window on bodies it doesn't need) and hand-rolling `grep`/`head`
pipelines that return unstructured text and don't know frontmatter ends at
the closing `---`, not after some line count.

frontmatter-mcp streams only the header block off disk, stops at the closing
`---`, and returns parsed key/value data — one call for one file, a whole
glob, or just the specific properties you want.

## Tools

### `read_frontmatter`

Reads the frontmatter of a single file.

```json
{ "path": "skills/my-skill/SKILL.md" }
```

```json
{
  "path": "skills/my-skill/SKILL.md",
  "hasFrontmatter": true,
  "raw": null,
  "parsed": { "name": "my-skill", "description": "..." },
  "parseError": null
}
```

Pass `"format": "Raw"` for the exact frontmatter text with no YAML parsing,
or `"format": "Both"` for both. If the YAML fails to parse, `raw` is
included automatically regardless of `format`, so you're never left with
nothing. Files with no `---`-delimited frontmatter report
`hasFrontmatter: false` rather than an error — that's a normal result.

### `read_frontmatter_batch`

Reads frontmatter from many files at once, given either explicit `paths` or
a `glob` pattern (`**` recurses) — provide exactly one:

```json
{ "glob": "docs/adr/*.md" }
```

Returns an array of the same shape as `read_frontmatter`, one per matched
file, read concurrently. `maxFiles` (default 500) caps how many files a
broad glob can return.

### `get_frontmatter_properties`

Extracts only specific named fields across many files:

```json
{ "glob": "skills/**/*.md", "properties": ["name", "status"] }
```

```json
[
  {
    "path": "skills/my-skill/SKILL.md",
    "values": { "name": "my-skill", "status": "active" },
    "missing": []
  }
]
```

`properties` supports dotted paths for nested keys (e.g. `"metadata.owner"`).
Requested properties that don't exist in a file are listed in `missing`
rather than silently omitted, so you can tell "field absent" apart from
"file unreadable".

## Install

### Claude Desktop

Download the `.mcpb` bundle for your platform from the
[latest release](https://github.com/dealloc/frontmatter-mcp/releases/latest)
(`frontmatter-mcp-<platform>.mcpb`) and double-click it, or drag it into the
Claude Desktop window.

### Any MCP client (binary on PATH)

Download the archive for your platform from the
[latest release](https://github.com/dealloc/frontmatter-mcp/releases/latest):

| Platform | Asset |
|---|---|
| Linux x86-64 | `frontmatter-mcp-linux-x64.tar.gz` |
| Linux ARM64 | `frontmatter-mcp-linux-arm64.tar.gz` |
| macOS Apple silicon | `frontmatter-mcp-osx-arm64.tar.gz` |
| macOS Intel | `frontmatter-mcp-osx-x64.tar.gz` |
| Windows x86-64 | `frontmatter-mcp-win-x64.zip` |

Every asset has a `.sha256` sibling and a
[build provenance attestation](https://github.com/dealloc/frontmatter-mcp/attestations)
(`gh attestation verify <asset> --repo dealloc/frontmatter-mcp`). The macOS
binaries are ad-hoc signed only — on first run you may need
`xattr -d com.apple.quarantine <binary>`.

Extract the binary somewhere on your `PATH`, then point your client at it:

**VS Code / GitHub Copilot** — `.vscode/mcp.json`:

```json
{ "servers": { "frontmatter": { "type": "stdio", "command": "frontmatter-mcp" } } }
```

**Cursor** — `~/.cursor/mcp.json`:

```json
{ "mcpServers": { "frontmatter": { "command": "frontmatter-mcp" } } }
```

**Claude Desktop (manual)** — `claude_desktop_config.json`: same as Cursor.

### From source

```sh
cargo build --release
# target/release/frontmatter-mcp
```

`frontmatter-mcp --help` for the (short) list of options; the server takes
no arguments and reads no configuration — run it with none to speak MCP over
stdio.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for the dev workflow (test-first,
`just precommit`) and [AI-POLICY.md](AI-POLICY.md) for how AI-assisted
contributions are handled.

## License

[AGPL-3.0](LICENSE.md)
