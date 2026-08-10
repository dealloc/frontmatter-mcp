using FrontmatterMCP.Core;
using FrontmatterMCP.Tools;

namespace FrontmatterMCP.Tests;

public sealed class ReadFrontmatterBatchTests
{
    private static string FixturePath(string name) => Path.Combine(AppContext.BaseDirectory, "Fixtures", name);

    private static string GlobSampleDirectory => Path.Combine(AppContext.BaseDirectory, "Fixtures", "GlobSample");

    [Test]
    public async Task ExplicitPathsAreReadInOrder()
    {
        string[] paths = [FixturePath("valid-simple.md"), FixturePath("no-frontmatter.md")];

        IReadOnlyList<FrontmatterResult> results = await FrontmatterTools.ReadFrontmatterBatch(paths, glob: null);

        await Assert.That(results).Count().IsEqualTo(2);
        await Assert.That(results[0].Path).IsEqualTo(paths[0]);
        await Assert.That(results[0].HasFrontmatter).IsTrue();
        await Assert.That(results[1].Path).IsEqualTo(paths[1]);
        await Assert.That(results[1].HasFrontmatter).IsFalse();
    }

    [Test]
    public async Task GlobPatternResolvesMatchingFilesRelativeToGivenBaseDirectory()
    {
        // ResolvePaths takes an explicit base directory precisely so glob resolution can be tested
        // without mutating the process-wide current directory (which the real tool call always uses).
        IReadOnlyList<string> resolvedPaths = FrontmatterTools.ResolvePaths(
            paths: null, glob: "*.md", maxFiles: 500, globBaseDirectory: GlobSampleDirectory);

        IReadOnlyList<FrontmatterResult> results = await FrontmatterTools.ReadFrontmatterBatch(resolvedPaths.ToArray(), glob: null);

        await Assert.That(results).Count().IsEqualTo(2);
        await Assert.That(results.All(r => r.HasFrontmatter)).IsTrue();
    }

    [Test]
    public async Task ProvidingBothPathsAndGlobThrows()
    {
        await Assert.That(async () => await FrontmatterTools.ReadFrontmatterBatch(["a.md"], "*.md"))
            .Throws<ArgumentException>();
    }

    [Test]
    public async Task ProvidingNeitherPathsNorGlobThrows()
    {
        await Assert.That(async () => await FrontmatterTools.ReadFrontmatterBatch(paths: null, glob: null))
            .Throws<ArgumentException>();
    }

    [Test]
    public async Task MaxFilesCapsExplicitPathsToo()
    {
        string[] paths = [FixturePath("valid-simple.md"), FixturePath("valid-nested.md"), FixturePath("no-frontmatter.md")];

        IReadOnlyList<FrontmatterResult> results = await FrontmatterTools.ReadFrontmatterBatch(paths, glob: null, maxFiles: 2);

        await Assert.That(results).Count().IsEqualTo(2);
    }
}