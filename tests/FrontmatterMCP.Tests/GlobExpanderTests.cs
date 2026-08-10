using FrontmatterMCP.Core;

namespace FrontmatterMCP.Tests;

public sealed class GlobExpanderTests
{
    private static string SampleDirectory => Path.Combine(AppContext.BaseDirectory, "Fixtures", "GlobSample");

    [Test]
    public async Task NonRecursivePatternMatchesOnlyTopLevelFiles()
    {
        IReadOnlyList<string> results = GlobExpander.Expand("*.md", SampleDirectory, maxFiles: 500);

        await Assert.That(results.Select(Path.GetFileName)).IsEquivalentTo(new List<string?> { "a.md", "b.md" });
    }

    [Test]
    public async Task RecursivePatternMatchesNestedFiles()
    {
        IReadOnlyList<string> results = GlobExpander.Expand("**/*.md", SampleDirectory, maxFiles: 500);

        await Assert.That(results.Select(Path.GetFileName)).IsEquivalentTo(new List<string?> { "a.md", "b.md", "c.md" });
    }

    [Test]
    public async Task NonMatchingExtensionsAreExcluded()
    {
        IReadOnlyList<string> results = GlobExpander.Expand("*.md", SampleDirectory, maxFiles: 500);

        await Assert.That(results.Any(path => path.EndsWith("notes.txt", StringComparison.Ordinal))).IsFalse();
    }

    [Test]
    public async Task MaxFilesCapsTheNumberOfResults()
    {
        IReadOnlyList<string> results = GlobExpander.Expand("**/*.md", SampleDirectory, maxFiles: 1);

        await Assert.That(results).Count().IsEqualTo(1);
    }

    [Test]
    public async Task PatternWithNoMatchesReturnsEmpty()
    {
        IReadOnlyList<string> results = GlobExpander.Expand("*.nonexistent", SampleDirectory, maxFiles: 500);

        await Assert.That(results).IsEmpty();
    }

    [Test]
    public async Task ResultsAreFullPaths()
    {
        IReadOnlyList<string> results = GlobExpander.Expand("a.md", SampleDirectory, maxFiles: 500);

        await Assert.That(results).Count().IsEqualTo(1);
        await Assert.That(Path.IsPathRooted(results[0])).IsTrue();
    }
}