using System.Text.Json.Nodes;

namespace FrontmatterMCP.Core;

/// <summary>The response shape shared by all frontmatter MCP tools.</summary>
internal sealed record FrontmatterResult
{
    public required string Path { get; init; }

    public required bool HasFrontmatter { get; init; }

    /// <summary>The exact frontmatter text. Present when explicitly requested, or when parsing failed.</summary>
    public string? Raw { get; init; }

    /// <summary>The parsed frontmatter data. Absent when parsing failed or was not requested.</summary>
    public JsonObject? Parsed { get; init; }

    /// <summary>Set when the frontmatter block could not be read cleanly or its YAML could not be parsed.</summary>
    public string? ParseError { get; init; }
}