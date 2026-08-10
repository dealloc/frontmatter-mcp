using FrontmatterMCP.Core;

namespace FrontmatterMCP.Tests;

public sealed class FrontmatterYamlParserTests
{
    [Test]
    public async Task ParsesFlatMapping()
    {
        YamlParseResult result = FrontmatterYamlParser.Parse("title: Sample Document\nstatus: draft");

        await Assert.That(result.Error).IsNull();
        await Assert.That(result.Value).IsNotNull();
        await Assert.That(result.Value!["title"]).IsEqualTo("Sample Document");
        await Assert.That(result.Value!["status"]).IsEqualTo("draft");
    }

    [Test]
    public async Task ParsesNestedMappingsAndBlockLists()
    {
        const string raw = "title: Sample Document\nmetadata:\n  owner: alice\n  tags:\n    - draft\n    - internal";

        YamlParseResult result = FrontmatterYamlParser.Parse(raw);

        await Assert.That(result.Error).IsNull();
        Dictionary<string, object?> metadata = (Dictionary<string, object?>)result.Value!["metadata"]!;
        await Assert.That(metadata["owner"]).IsEqualTo("alice");
        List<object?> tags = (List<object?>)metadata["tags"]!;
        await Assert.That(tags).IsEquivalentTo(new List<object?> { "draft", "internal" });
    }

    [Test]
    public async Task ParsesInlineLists()
    {
        YamlParseResult result = FrontmatterYamlParser.Parse("tags: [draft, internal]");

        List<object?> tags = (List<object?>)result.Value!["tags"]!;
        await Assert.That(tags).IsEquivalentTo(new List<object?> { "draft", "internal" });
    }

    [Test]
    public async Task ResolvesImplicitScalarTypes()
    {
        const string raw = "count: 42\nratio: 3.14\nenabled: true\ndisabled: false\nnothing: null\nquoted_number: \"42\"";

        YamlParseResult result = FrontmatterYamlParser.Parse(raw);

        await Assert.That(result.Error).IsNull();
        await Assert.That(result.Value!["count"]).IsEqualTo(42L);
        await Assert.That(result.Value!["ratio"]).IsEqualTo(3.14);
        await Assert.That((bool)result.Value!["enabled"]!).IsTrue();
        await Assert.That((bool)result.Value!["disabled"]!).IsFalse();
        await Assert.That(result.Value!["nothing"]).IsNull();
        await Assert.That(result.Value!["quoted_number"]).IsEqualTo("42");
    }

    [Test]
    public async Task EmptyFrontmatterParsesToEmptyMapping()
    {
        YamlParseResult result = FrontmatterYamlParser.Parse(string.Empty);

        await Assert.That(result.Error).IsNull();
        await Assert.That(result.Value).IsNotNull();
        await Assert.That(result.Value!).IsEmpty();
    }

    [Test]
    public async Task MalformedYamlReturnsErrorWithoutThrowing()
    {
        const string raw = "title: \"unterminated string\nstatus droid twelve: : :";

        YamlParseResult result = FrontmatterYamlParser.Parse(raw);

        await Assert.That(result.Value).IsNull();
        await Assert.That(result.Error).IsNotNull();
    }

    [Test]
    public async Task NonMappingRootReturnsError()
    {
        YamlParseResult result = FrontmatterYamlParser.Parse("- just\n- a\n- list");

        await Assert.That(result.Value).IsNull();
        await Assert.That(result.Error).IsNotNull();
    }
}
