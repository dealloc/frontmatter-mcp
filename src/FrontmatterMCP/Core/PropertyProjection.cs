namespace FrontmatterMCP.Core;

/// <summary>Extracts a subset of named (optionally dotted-path) properties from a parsed frontmatter map.</summary>
internal static class PropertyProjection
{
    public static (Dictionary<string, object?> Values, List<string> Missing) Project(
        Dictionary<string, object?> map, IReadOnlyList<string> properties)
    {
        Dictionary<string, object?> values = [];
        List<string> missing = [];

        foreach (string property in properties)
        {
            if (TryGetDottedValue(map, property, out object? value))
            {
                values[property] = value;
            }
            else
            {
                missing.Add(property);
            }
        }

        return (values, missing);
    }

    private static bool TryGetDottedValue(Dictionary<string, object?> root, string dottedPath, out object? value)
    {
        object? current = root;

        foreach (string segment in dottedPath.Split('.'))
        {
            if (current is not Dictionary<string, object?> map || !map.TryGetValue(segment, out current))
            {
                value = null;
                return false;
            }
        }

        value = current;
        return true;
    }
}