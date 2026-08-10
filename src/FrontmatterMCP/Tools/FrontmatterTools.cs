using System.ComponentModel;

using FrontmatterMCP.Core;

using ModelContextProtocol.Server;

namespace FrontmatterMCP.Tools;

[McpServerToolType]
internal sealed class FrontmatterTools
{
    [McpServerTool]
    [Description(
        "Reads only the YAML frontmatter block of a single markdown file, without loading the document " +
        "body. Files with no --- delimited frontmatter report HasFrontmatter = false rather than an error.")]
    public static async Task<FrontmatterResult> ReadFrontmatter(
        [Description("Absolute or relative path to the markdown file.")]
        string path,
        [Description("How to shape the response: Parsed (default), Raw, or Both.")]
        FrontmatterFormat format = FrontmatterFormat.Parsed,
        CancellationToken cancellationToken = default)
    {
        FrontmatterExtraction extraction = await FrontmatterReader.ExtractAsync(path, cancellationToken).ConfigureAwait(false);
        return BuildResult(path, extraction, format);
    }

    internal static FrontmatterResult BuildResult(string path, FrontmatterExtraction extraction, FrontmatterFormat format)
    {
        if (!extraction.HasFrontmatter)
        {
            return new FrontmatterResult { Path = path, HasFrontmatter = false };
        }

        bool wantsParsed = format is FrontmatterFormat.Parsed or FrontmatterFormat.Both;
        bool wantsRaw = format is FrontmatterFormat.Raw or FrontmatterFormat.Both;

        // A reader-level error (unterminated/too-large block) means there is nothing to parse. Parsing
        // is also skipped outright when the caller only asked for Raw, to avoid the wasted work.
        YamlParseResult? parseResult = extraction.Error is null && wantsParsed
            ? FrontmatterYamlParser.Parse(extraction.Raw!)
            : null;

        // Fall back to raw text whenever parsing was requested but didn't produce data, so the caller
        // isn't left with nothing.
        bool parseFailed = wantsParsed && parseResult?.Value is null;
        bool includeRaw = wantsRaw || parseFailed;

        return new FrontmatterResult
        {
            Path = path,
            HasFrontmatter = true,
            Raw = includeRaw ? extraction.Raw : null,
            Parsed = parseResult?.Value is { } map ? YamlNodeConverter.ToJsonObject(map) : null,
            ParseError = extraction.Error ?? parseResult?.Error,
        };
    }
}
