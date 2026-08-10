namespace FrontmatterMCP.Core;

/// <summary>
/// Extracts only the YAML frontmatter block of a markdown document by scanning line-by-line and
/// stopping as soon as the closing delimiter is found, without ever reading the document body.
/// </summary>
internal static class FrontmatterReader
{
    internal const string UnterminatedError = "unterminated frontmatter block";
    internal const string TooLargeError = "frontmatter block exceeds maximum size";

    private const string Delimiter = "---";

    // Frontmatter is metadata, not content; a well-formed block is at most a few dozen lines.
    // This bounds how much an unterminated block can buffer before giving up.
    private const int MaxFrontmatterLines = 1000;

    public static async Task<FrontmatterExtraction> ExtractAsync(string path, CancellationToken cancellationToken = default)
    {
        await using FileStream stream = new(
            path,
            FileMode.Open,
            FileAccess.Read,
            FileShare.Read,
            bufferSize: 4096,
            FileOptions.SequentialScan | FileOptions.Asynchronous);

        return await ExtractAsync(stream, cancellationToken).ConfigureAwait(false);
    }

    public static async Task<FrontmatterExtraction> ExtractAsync(Stream stream, CancellationToken cancellationToken = default)
    {
        using StreamReader reader = new(stream, detectEncodingFromByteOrderMarks: true, leaveOpen: true);

        string? firstLine = await reader.ReadLineAsync(cancellationToken).ConfigureAwait(false);
        if (firstLine != Delimiter)
        {
            return new FrontmatterExtraction { HasFrontmatter = false };
        }

        List<string> lines = [];
        while (true)
        {
            string? line = await reader.ReadLineAsync(cancellationToken).ConfigureAwait(false);

            if (line is null)
            {
                return new FrontmatterExtraction
                {
                    HasFrontmatter = true,
                    Raw = string.Join('\n', lines),
                    Error = UnterminatedError,
                };
            }

            if (line == Delimiter)
            {
                return new FrontmatterExtraction { HasFrontmatter = true, Raw = string.Join('\n', lines) };
            }

            if (lines.Count >= MaxFrontmatterLines)
            {
                return new FrontmatterExtraction
                {
                    HasFrontmatter = true,
                    Raw = string.Join('\n', lines),
                    Error = TooLargeError,
                };
            }

            lines.Add(line);
        }
    }
}