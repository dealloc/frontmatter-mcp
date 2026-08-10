using FrontmatterMCP.Core;

namespace FrontmatterMCP.Tests;

public sealed class PropertyProjectionTests
{
    private static readonly Dictionary<string, object?> Map = new()
    {
        ["title"] = "Sample Document",
        ["nothing"] = null,
        ["metadata"] = new Dictionary<string, object?>
        {
            ["owner"] = "alice",
            ["tags"] = new List<object?> { "draft", "internal" },
        },
    };

    [Test]
    public async Task FindsTopLevelProperty()
    {
        (Dictionary<string, object?> values, List<string> missing) = PropertyProjection.Project(Map, ["title"]);

        await Assert.That(values["title"]).IsEqualTo("Sample Document");
        await Assert.That(missing).IsEmpty();
    }

    [Test]
    public async Task FindsNestedDottedPathProperty()
    {
        (Dictionary<string, object?> values, List<string> missing) = PropertyProjection.Project(Map, ["metadata.owner"]);

        await Assert.That(values["metadata.owner"]).IsEqualTo("alice");
        await Assert.That(missing).IsEmpty();
    }

    [Test]
    public async Task ReportsAbsentPropertyAsMissingNotAsNull()
    {
        (Dictionary<string, object?> values, List<string> missing) = PropertyProjection.Project(Map, ["does_not_exist"]);

        await Assert.That(values.ContainsKey("does_not_exist")).IsFalse();
        await Assert.That(missing).IsEquivalentTo(new List<string> { "does_not_exist" });
    }

    [Test]
    public async Task PresentButNullPropertyIsNotReportedAsMissing()
    {
        (Dictionary<string, object?> values, List<string> missing) = PropertyProjection.Project(Map, ["nothing"]);

        await Assert.That(values.ContainsKey("nothing")).IsTrue();
        await Assert.That(values["nothing"]).IsNull();
        await Assert.That(missing).IsEmpty();
    }

    [Test]
    public async Task NavigatingThroughANonMappingSegmentIsMissing()
    {
        (Dictionary<string, object?> values, List<string> missing) = PropertyProjection.Project(Map, ["metadata.tags.owner"]);

        await Assert.That(values.ContainsKey("metadata.tags.owner")).IsFalse();
        await Assert.That(missing).IsEquivalentTo(new List<string> { "metadata.tags.owner" });
    }

    [Test]
    public async Task ProjectsMultiplePropertiesInOneCall()
    {
        (Dictionary<string, object?> values, List<string> missing) = PropertyProjection.Project(
            Map, ["title", "metadata.owner", "does_not_exist"]);

        await Assert.That(values).Count().IsEqualTo(2);
        await Assert.That(missing).IsEquivalentTo(new List<string> { "does_not_exist" });
    }
}