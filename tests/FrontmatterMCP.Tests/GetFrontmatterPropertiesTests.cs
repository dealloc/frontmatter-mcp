using FrontmatterMCP.Core;
using FrontmatterMCP.Tools;

namespace FrontmatterMCP.Tests;

public sealed class GetFrontmatterPropertiesTests
{
    private static string FixturePath(string name) => Path.Combine(AppContext.BaseDirectory, "Fixtures", name);

    [Test]
    public async Task ProjectsRequestedPropertiesAcrossFiles()
    {
        string[] paths = [FixturePath("valid-simple.md"), FixturePath("valid-nested.md")];

        IReadOnlyList<PropertyProjectionResult> results = await FrontmatterTools.GetFrontmatterProperties(
            properties: ["title"], paths: paths);

        await Assert.That(results).Count().IsEqualTo(2);
        await Assert.That(results[0].Values["title"]!.GetValue<string>()).IsEqualTo("Sample Document");
        await Assert.That(results[1].Values["title"]!.GetValue<string>()).IsEqualTo("Sample Document");
        await Assert.That(results[0].Missing).IsEmpty();
    }

    [Test]
    public async Task ReportsMissingPropertiesPerFile()
    {
        string[] paths = [FixturePath("valid-simple.md")];

        IReadOnlyList<PropertyProjectionResult> results = await FrontmatterTools.GetFrontmatterProperties(
            properties: ["title", "does_not_exist"], paths: paths);

        await Assert.That(results[0].Missing).IsEquivalentTo(new List<string> { "does_not_exist" });
    }

    [Test]
    public async Task FileWithoutFrontmatterReportsAllPropertiesMissing()
    {
        string[] paths = [FixturePath("no-frontmatter.md")];

        IReadOnlyList<PropertyProjectionResult> results = await FrontmatterTools.GetFrontmatterProperties(
            properties: ["title", "status"], paths: paths);

        await Assert.That(results[0].Missing).IsEquivalentTo(new List<string> { "title", "status" });
        await Assert.That(results[0].Values).IsEmpty();
    }

    [Test]
    public async Task NestedDottedPathIsProjected()
    {
        string[] paths = [FixturePath("valid-nested.md")];

        IReadOnlyList<PropertyProjectionResult> results = await FrontmatterTools.GetFrontmatterProperties(
            properties: ["metadata.owner"], paths: paths);

        await Assert.That(results[0].Values["metadata.owner"]!.GetValue<string>()).IsEqualTo("alice");
    }
}