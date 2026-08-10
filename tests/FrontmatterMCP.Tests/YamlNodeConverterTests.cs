using System.Text.Json;
using System.Text.Json.Nodes;

using FrontmatterMCP.Core;

namespace FrontmatterMCP.Tests;

public sealed class YamlNodeConverterTests
{
    [Test]
    public async Task ConvertsScalarTypesToTheirJsonKinds()
    {
        Dictionary<string, object?> map = new()
        {
            ["text"] = "hello",
            ["integer"] = 42L,
            ["real"] = 3.14,
            ["flag"] = true,
            ["nothing"] = null,
        };

        JsonObject result = YamlNodeConverter.ToJsonObject(map);

        await Assert.That(result["text"]!.GetValue<string>()).IsEqualTo("hello");
        await Assert.That(result["integer"]!.GetValue<long>()).IsEqualTo(42L);
        await Assert.That(result["real"]!.GetValue<double>()).IsEqualTo(3.14);
        await Assert.That(result["flag"]!.GetValue<bool>()).IsTrue();
        await Assert.That(result["nothing"]).IsNull();
    }

    [Test]
    public async Task ConvertsNestedMappingsAndLists()
    {
        Dictionary<string, object?> map = new()
        {
            ["metadata"] = new Dictionary<string, object?>
            {
                ["owner"] = "alice",
                ["tags"] = new List<object?> { "draft", "internal" },
            },
        };

        JsonObject result = YamlNodeConverter.ToJsonObject(map);

        JsonObject metadata = result["metadata"]!.AsObject();
        await Assert.That(metadata["owner"]!.GetValue<string>()).IsEqualTo("alice");

        JsonArray tags = metadata["tags"]!.AsArray();
        await Assert.That(tags[0]!.GetValue<string>()).IsEqualTo("draft");
        await Assert.That(tags[1]!.GetValue<string>()).IsEqualTo("internal");
    }

    [Test]
    public async Task RoundTripsThroughJsonSerialization()
    {
        Dictionary<string, object?> map = new() { ["count"] = 7L, ["ratio"] = 1.5, ["ok"] = false };

        JsonObject result = YamlNodeConverter.ToJsonObject(map);
        string json = result.ToJsonString();
        JsonElement parsed = JsonDocument.Parse(json).RootElement;

        await Assert.That(parsed.GetProperty("count").GetInt64()).IsEqualTo(7L);
        await Assert.That(parsed.GetProperty("ratio").GetDouble()).IsEqualTo(1.5);
        await Assert.That(parsed.GetProperty("ok").GetBoolean()).IsFalse();
    }
}