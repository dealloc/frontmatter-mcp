namespace FrontmatterMCP.Core;

/// <summary>
/// The result of parsing a frontmatter block's raw YAML text into a dynamic key/value tree.
/// </summary>
internal sealed record YamlParseResult
{
    /// <summary>The parsed mapping, or <see langword="null"/> if parsing failed.</summary>
    public Dictionary<string, object?>? Value { get; init; }

    /// <summary>Set when the YAML is malformed or its root is not a mapping.</summary>
    public string? Error { get; init; }
}
