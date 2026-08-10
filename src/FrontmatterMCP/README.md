# FrontmatterMCP

An [MCP](https://modelcontextprotocol.io/) server that reads only the YAML
frontmatter block of markdown documents — skills, ADRs, PIRs, docs, anything
with a `---`-delimited header — without loading the rest of the document.

See the [project README](https://github.com/dealloc/frontmatter-mcp) for the
full tool reference (`read_frontmatter`, `read_frontmatter_batch`,
`get_frontmatter_properties`) with example requests and responses.

## Developing locally

To run this server from source without a built package, point your MCP
client at `dotnet run`:

```json
{
  "servers": {
    "FrontmatterMCP": {
      "type": "stdio",
      "command": "dotnet",
      "args": ["run", "--project", "<PATH TO PROJECT DIRECTORY>"]
    }
  }
}
```

## Using a published binary

Self-contained, Native AOT binaries are attached to each
[GitHub release](https://github.com/dealloc/frontmatter-mcp/releases). Point
your MCP client's `command` directly at the downloaded executable — no
runtime install, no `dnx`/`dotnet` needed on the target machine.

## More information

FrontmatterMCP uses the [ModelContextProtocol](https://www.nuget.org/packages/ModelContextProtocol)
C# SDK. For more on MCP itself:

- [Official Documentation](https://modelcontextprotocol.io/)
- [Protocol Specification](https://spec.modelcontextprotocol.io/)
- [MCP C# SDK](https://modelcontextprotocol.github.io/csharp-sdk)
