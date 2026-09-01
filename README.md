# frontmatter-mcp

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL--3.0-blue.svg)](LICENSE.md)

An [MCP](https://modelcontextprotocol.io/) server that reads only the YAML
frontmatter block of markdown documents — skills, ADRs, PIRs, docs, anything
with a `---`-delimited header — without loading the rest of the document.

## Why this exists

When an agent needs to survey a folder of markdown for metadata, it's stuck
choosing between reading every file whole (spending its context window on
bodies it doesn't need) and hand-rolling shell pipelines that return
unstructured text and don't know frontmatter ends at the closing `---`, not
after some line count. frontmatter-mcp streams only the header block off
disk, stops at the closing `---`, and returns parsed key/value data.

## Status

This is a Rust rewrite of the original .NET implementation, in progress on
`feature/rust-rewrite`. Tools, install instructions, and release artifacts
will be documented here as each lands.

## License

[AGPL-3.0](LICENSE.md)
