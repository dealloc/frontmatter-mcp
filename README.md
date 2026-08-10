# frontmatter-mcp

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL--3.0-blue.svg)](LICENSE.md)

An [MCP](https://modelcontextprotocol.io/) server that reads only the YAML
frontmatter block of markdown documents — skills, ADRs, PIRs, docs, anything
with a `---`-delimited header — without loading the rest of the document.

## Why

AI assistants often need to survey many structured markdown documents just to
check their metadata (a `status` field, an `owner`, a `name`). Doing that with
a generic file-reading tool means reading the whole file — including a body
that might be many times larger than the header — for every single document.
frontmatter-mcp streams just the frontmatter block off disk (stopping at the
closing `---`, never touching the body) and returns structured data, so
surveying dozens of documents costs a fraction of the tokens and I/O.

## Tools

### `read_frontmatter`

Reads the frontmatter of a single file.

```json
{ "path": "skills/my-skill/SKILL.md" }
```

```json
{
  "Path": "skills/my-skill/SKILL.md",
  "HasFrontmatter": true,
  "Raw": null,
  "Parsed": { "name": "my-skill", "description": "..." },
  "ParseError": null
}
```

Pass `"format": "Raw"` for the exact frontmatter text with no YAML parsing,
or `"format": "Both"` for both. If the YAML fails to parse, `Raw` is included
automatically regardless of `format`, so you're never left with nothing.
Files with no `---`-delimited frontmatter report `HasFrontmatter: false`
rather than an error — that's a normal, expected result, not a failure.

### `read_frontmatter_batch`

Reads frontmatter from many files at once, given either explicit `paths` or
a `glob` pattern (`**` supported for recursion) — provide exactly one:

```json
{ "glob": "docs/adr/*.md" }
```

Returns an array of the same shape as `read_frontmatter`, one per matched
file, read concurrently. `maxFiles` (default 500) caps how many files an
overly broad glob can return.

### `get_frontmatter_properties`

Extracts only specific named fields across many files — useful when you want
one property (e.g. `status`) from a hundred documents without paying for the
rest of each one's frontmatter:

```json
{ "glob": "skills/**/*.md", "properties": ["name", "status"] }
```

```json
[
  {
    "Path": "skills/my-skill/SKILL.md",
    "Values": { "name": "my-skill", "status": "active" },
    "Missing": []
  }
]
```

`properties` supports dotted paths for nested keys (e.g. `"metadata.owner"`).
Requested properties that don't exist in a file are listed in `Missing`
rather than silently omitted, so you can tell "field absent" apart from
"file unreadable".

## Install

Self-contained, Native AOT binaries are attached to each
[GitHub release](https://github.com/dealloc/frontmatter-mcp/releases) for
`win-x64`, `osx-arm64`, `linux-x64`, and `linux-arm64` — no .NET runtime
required on the target machine. (`win-arm64` and `linux-musl-x64` are
supported build targets too, just not prebuilt by CI; run
`dotnet publish src/FrontmatterMCP -c Release -r <rid> --self-contained -p:PublishAot=true`
yourself for those.)

Point your MCP client at the downloaded binary, e.g. for a stdio-based client:

```json
{
  "servers": {
    "frontmatter": {
      "type": "stdio",
      "command": "/path/to/FrontmatterMCP"
    }
  }
}
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for the dev workflow and
[AI-POLICY.md](AI-POLICY.md) for how AI-assisted contributions are handled.

## License

[AGPL-3.0](LICENSE.md)
