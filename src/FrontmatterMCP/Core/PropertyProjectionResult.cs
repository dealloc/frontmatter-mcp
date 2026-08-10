using System.Text.Json.Nodes;

namespace FrontmatterMCP.Core;

/// <summary>The response shape for <c>get_frontmatter_properties</c>, one per requested file.</summary>
internal sealed record PropertyProjectionResult
{
    public required string Path { get; init; }

    /// <summary>The requested properties that were found, keyed by the property name/dotted path requested.</summary>
    public required JsonObject Values { get; init; }

    /// <summary>Requested properties that were absent, so callers can tell "absent" from "file unreadable".</summary>
    public required IReadOnlyList<string> Missing { get; init; }
}