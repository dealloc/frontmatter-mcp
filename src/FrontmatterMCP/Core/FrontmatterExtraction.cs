namespace FrontmatterMCP.Core;

/// <summary>
/// The result of streaming just the frontmatter block out of a markdown document, before any YAML parsing.
/// </summary>
internal sealed record FrontmatterExtraction
{
    public required bool HasFrontmatter { get; init; }

    /// <summary>The exact text between the opening and closing delimiters, or <see langword="null"/> if there is no frontmatter block.</summary>
    public string? Raw { get; init; }

    /// <summary>Set when a frontmatter block was opened but could not be closed cleanly (unterminated or too large).</summary>
    public string? Error { get; init; }
}
