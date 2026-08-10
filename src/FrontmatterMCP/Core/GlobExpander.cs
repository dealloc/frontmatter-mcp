using Microsoft.Extensions.FileSystemGlobbing;

namespace FrontmatterMCP.Core;

/// <summary>Expands a glob pattern into matching file paths, relative to a base directory.</summary>
internal static class GlobExpander
{
    public static IReadOnlyList<string> Expand(string pattern, string baseDirectory, int maxFiles)
    {
        Matcher matcher = new();
        matcher.AddInclude(pattern);

        return matcher.GetResultsInFullPath(baseDirectory)
            .Order(StringComparer.Ordinal)
            .Take(maxFiles)
            .ToList();
    }
}
