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
        CancellationToken cancellationToken = default) =>
        await ReadOne(path, format, cancellationToken).ConfigureAwait(false);

    [McpServerTool]
    [Description(
        "Reads only the YAML frontmatter block from multiple markdown files, given either explicit paths " +
        "or a glob pattern (e.g. 'docs/adr/*.md' or 'skills/**/*.md' for recursive matching), without " +
        "loading document bodies. Provide exactly one of 'paths' or 'glob'.")]
    public static async Task<IReadOnlyList<FrontmatterResult>> ReadFrontmatterBatch(
        [Description("Explicit file paths to read. Mutually exclusive with 'glob'.")]
        string[]? paths = null,
        [Description("Glob pattern (supports ** for recursion), resolved relative to the server's working directory. Mutually exclusive with 'paths'.")]
        string? glob = null,
        [Description("How to shape each result: Parsed (default), Raw, or Both.")]
        FrontmatterFormat format = FrontmatterFormat.Parsed,
        [Description("Maximum number of files to process, guarding against an overly broad glob. Default 500.")]
        int maxFiles = 500,
        CancellationToken cancellationToken = default)
    {
        IReadOnlyList<string> resolvedPaths = ResolvePaths(paths, glob, maxFiles, Directory.GetCurrentDirectory());

        Task<FrontmatterResult>[] tasks = new Task<FrontmatterResult>[resolvedPaths.Count];
        for (int i = 0; i < resolvedPaths.Count; i++)
        {
            tasks[i] = ReadOne(resolvedPaths[i], format, cancellationToken);
        }

        return await Task.WhenAll(tasks).ConfigureAwait(false);
    }

    /// <summary>
    /// <paramref name="globBaseDirectory"/> is a test seam for resolving relative glob patterns without
    /// mutating the process-wide current directory; the real tool always passes the actual CWD.
    /// </summary>
    internal static IReadOnlyList<string> ResolvePaths(string[]? paths, string? glob, int maxFiles, string globBaseDirectory)
    {
        bool hasPaths = paths is { Length: > 0 };
        bool hasGlob = !string.IsNullOrWhiteSpace(glob);

        if (hasPaths == hasGlob)
        {
            throw new ArgumentException("Provide exactly one of 'paths' or 'glob'.");
        }

        return hasPaths
            ? paths!.Take(maxFiles).ToList()
            : GlobExpander.Expand(glob!, globBaseDirectory, maxFiles);
    }

    private static async Task<FrontmatterResult> ReadOne(string path, FrontmatterFormat format, CancellationToken cancellationToken)
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
