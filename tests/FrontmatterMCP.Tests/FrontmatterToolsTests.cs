using FrontmatterMCP.Core;
using FrontmatterMCP.Tools;

namespace FrontmatterMCP.Tests;

public sealed class FrontmatterToolsTests
{
    private static string FixturePath(string name) => Path.Combine(AppContext.BaseDirectory, "Fixtures", name);

    [Test]
    public async Task ReadFrontmatterReturnsParsedDataByDefault()
    {
        FrontmatterResult result = await FrontmatterTools.ReadFrontmatter(FixturePath("valid-simple.md"));

        await Assert.That(result.HasFrontmatter).IsTrue();
        await Assert.That(result.Parsed!["title"]!.GetValue<string>()).IsEqualTo("Sample Document");
        await Assert.That(result.Raw).IsNull();
        await Assert.That(result.ParseError).IsNull();
    }

    [Test]
    public async Task ReadFrontmatterWithRawFormatOmitsParsedData()
    {
        FrontmatterResult result = await FrontmatterTools.ReadFrontmatter(FixturePath("valid-simple.md"), FrontmatterFormat.Raw);

        await Assert.That(result.Raw).IsEqualTo("title: Sample Document\nstatus: draft");
        await Assert.That(result.Parsed).IsNull();
    }

    [Test]
    public async Task ReadFrontmatterWithBothFormatIncludesRawAndParsed()
    {
        FrontmatterResult result = await FrontmatterTools.ReadFrontmatter(FixturePath("valid-simple.md"), FrontmatterFormat.Both);

        await Assert.That(result.Raw).IsNotNull();
        await Assert.That(result.Parsed).IsNotNull();
    }

    [Test]
    public async Task ReadFrontmatterOnMissingFrontmatterReportsFalseWithoutError()
    {
        FrontmatterResult result = await FrontmatterTools.ReadFrontmatter(FixturePath("no-frontmatter.md"));

        await Assert.That(result.HasFrontmatter).IsFalse();
        await Assert.That(result.Raw).IsNull();
        await Assert.That(result.Parsed).IsNull();
        await Assert.That(result.ParseError).IsNull();
    }

    [Test]
    public async Task ReadFrontmatterOnMalformedYamlFallsBackToRawWithError()
    {
        FrontmatterResult result = await FrontmatterTools.ReadFrontmatter(FixturePath("malformed-yaml.md"));

        await Assert.That(result.HasFrontmatter).IsTrue();
        await Assert.That(result.Parsed).IsNull();
        await Assert.That(result.Raw).IsNotNull();
        await Assert.That(result.ParseError).IsNotNull();
    }

    [Test]
    public async Task ReadFrontmatterOnUnterminatedBlockFallsBackToRawWithError()
    {
        FrontmatterResult result = await FrontmatterTools.ReadFrontmatter(FixturePath("unterminated.md"));

        await Assert.That(result.HasFrontmatter).IsTrue();
        await Assert.That(result.Parsed).IsNull();
        await Assert.That(result.Raw).IsNotNull();
        await Assert.That(result.ParseError).IsEqualTo(FrontmatterReader.UnterminatedError);
    }

    [Test]
    public async Task ReadFrontmatterOnNestedDocumentParsesNestedStructure()
    {
        FrontmatterResult result = await FrontmatterTools.ReadFrontmatter(FixturePath("valid-nested.md"));

        await Assert.That(result.Parsed!["metadata"]!["owner"]!.GetValue<string>()).IsEqualTo("alice");
    }
}
