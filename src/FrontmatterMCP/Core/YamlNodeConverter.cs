using System.Text.Json.Nodes;

namespace FrontmatterMCP.Core;

/// <summary>
/// Converts the dynamic Dictionary/List/scalar tree produced by <see cref="FrontmatterYamlParser"/> into
/// <see cref="JsonNode"/>s for MCP tool responses. JsonNode's serialization walks the node tree directly,
/// unlike serializing a boxed <see cref="object"/> graph, which needs per-runtime-type metadata that isn't
/// available under Native AOT once reflection-based System.Text.Json is unavailable.
/// </summary>
internal static class YamlNodeConverter
{
    public static JsonObject ToJsonObject(Dictionary<string, object?> map)
    {
        JsonObject result = [];
        foreach (KeyValuePair<string, object?> pair in map)
        {
            result[pair.Key] = ToJsonNode(pair.Value);
        }

        return result;
    }

    private static JsonNode? ToJsonNode(object? value) => value switch
    {
        null => null,
        Dictionary<string, object?> map => ToJsonObject(map),
        List<object?> list => ToJsonArray(list),
        string s => JsonValue.Create(s),
        bool b => JsonValue.Create(b),
        long l => JsonValue.Create(l),
        double d => JsonValue.Create(d),
        _ => throw new NotSupportedException($"Unexpected parsed YAML value type: {value.GetType()}"),
    };

    private static JsonArray ToJsonArray(List<object?> list)
    {
        JsonArray result = [];
        foreach (object? item in list)
        {
            result.Add(ToJsonNode(item));
        }

        return result;
    }
}