using System.Text;

using FrontmatterMCP.Core;

namespace FrontmatterMCP.Tests;

public sealed class FrontmatterReaderTests
{
    private static string FixturePath(string name) => Path.Combine(AppContext.BaseDirectory, "Fixtures", name);

    private static MemoryStream Utf8Stream(string content) => new(Encoding.UTF8.GetBytes(content));

    [Test]
    public async Task ExtractsFlatFrontmatter()
    {
        FrontmatterExtraction result = await FrontmatterReader.ExtractAsync(FixturePath("valid-simple.md"));

        await Assert.That(result.HasFrontmatter).IsTrue();
        await Assert.That(result.Raw).IsEqualTo("title: Sample Document\nstatus: draft");
        await Assert.That(result.Error).IsNull();
    }

    [Test]
    public async Task ExtractsNestedFrontmatter()
    {
        FrontmatterExtraction result = await FrontmatterReader.ExtractAsync(FixturePath("valid-nested.md"));

        await Assert.That(result.HasFrontmatter).IsTrue();
        await Assert.That(result.Raw).IsEqualTo(
            "title: Sample Document\nmetadata:\n  owner: alice\n  tags:\n    - draft\n    - internal");
    }

    [Test]
    public async Task EmptyFrontmatterBlockIsValidWithEmptyRaw()
    {
        FrontmatterExtraction result = await FrontmatterReader.ExtractAsync(FixturePath("valid-empty-frontmatter.md"));

        await Assert.That(result.HasFrontmatter).IsTrue();
        await Assert.That(result.Raw).IsEqualTo(string.Empty);
        await Assert.That(result.Error).IsNull();
    }

    [Test]
    public async Task NoFrontmatterIsReportedWithoutReadingBody()
    {
        FrontmatterExtraction result = await FrontmatterReader.ExtractAsync(FixturePath("no-frontmatter.md"));

        await Assert.That(result.HasFrontmatter).IsFalse();
        await Assert.That(result.Raw).IsNull();
    }

    [Test]
    public async Task EmptyFileHasNoFrontmatter()
    {
        FrontmatterExtraction result = await FrontmatterReader.ExtractAsync(FixturePath("empty-file.md"));

        await Assert.That(result.HasFrontmatter).IsFalse();
        await Assert.That(result.Raw).IsNull();
    }

    [Test]
    public async Task UnterminatedFrontmatterReturnsCapturedRawWithError()
    {
        FrontmatterExtraction result = await FrontmatterReader.ExtractAsync(FixturePath("unterminated.md"));

        await Assert.That(result.HasFrontmatter).IsTrue();
        await Assert.That(result.Raw).IsEqualTo("title: Unterminated\nstatus: draft\n\nThis block never closes.");
        await Assert.That(result.Error).IsEqualTo(FrontmatterReader.UnterminatedError);
    }

    [Test]
    public async Task DashesInsideBodyAreNotTreatedAsDelimiters()
    {
        FrontmatterExtraction result = await FrontmatterReader.ExtractAsync(FixturePath("dashes-in-body.md"));

        await Assert.That(result.HasFrontmatter).IsTrue();
        await Assert.That(result.Raw).IsEqualTo("title: Has code fence in body");
    }

    [Test]
    public async Task NonYamlFenceIsReportedAsNoFrontmatter()
    {
        FrontmatterExtraction result = await FrontmatterReader.ExtractAsync(FixturePath("non-yaml-fence.md"));

        await Assert.That(result.HasFrontmatter).IsFalse();
        await Assert.That(result.Raw).IsNull();
    }

    [Test]
    public async Task CrlfLineEndingsAreNormalized()
    {
        FrontmatterExtraction result = await FrontmatterReader.ExtractAsync(FixturePath("crlf-line-endings.md"));

        await Assert.That(result.HasFrontmatter).IsTrue();
        await Assert.That(result.Raw).IsEqualTo("title: CRLF Document\nstatus: draft");
    }

    [Test]
    public async Task Utf8BomIsStripped()
    {
        FrontmatterExtraction result = await FrontmatterReader.ExtractAsync(FixturePath("bom-utf8.md"));

        await Assert.That(result.HasFrontmatter).IsTrue();
        await Assert.That(result.Raw).IsEqualTo("title: BOM Document\nstatus: draft");
    }

    [Test]
    public async Task LargeBodyIsNeverConsumedWhenReadingFrontmatter()
    {
        StringBuilder content = new();
        content.Append("---\ntitle: Huge Body Document\nstatus: draft\n---\n# Body\n\n");
        for (int i = 0; i < 200_000; i++)
        {
            content.Append("This line is repeated many times to make the document body large.\n");
        }

        using MemoryStream stream = Utf8Stream(content.ToString());

        FrontmatterExtraction result = await FrontmatterReader.ExtractAsync(stream);

        await Assert.That(result.HasFrontmatter).IsTrue();
        await Assert.That(result.Raw).IsEqualTo("title: Huge Body Document\nstatus: draft");
        // The body is over 13MB; the reader must stop well before consuming it, proving it never
        // buffers the document body, only the internal StreamReader read-ahead around the delimiter.
        await Assert.That(stream.Position).IsLessThan(65_536);
    }

    [Test]
    public async Task ExcessivelyLongFrontmatterIsCappedRatherThanBufferedIndefinitely()
    {
        StringBuilder content = new();
        content.Append("---\n");
        for (int i = 0; i < 50_000; i++)
        {
            content.Append("key").Append(i).Append(": value\n");
        }

        // Deliberately never closes the block.
        using MemoryStream stream = Utf8Stream(content.ToString());

        FrontmatterExtraction result = await FrontmatterReader.ExtractAsync(stream);

        await Assert.That(result.HasFrontmatter).IsTrue();
        await Assert.That(result.Error).IsEqualTo(FrontmatterReader.TooLargeError);
        await Assert.That(stream.Position).IsLessThan(content.Length);
    }
}