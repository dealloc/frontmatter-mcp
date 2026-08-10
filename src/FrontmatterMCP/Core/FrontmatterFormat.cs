using System.Text.Json.Serialization;

namespace FrontmatterMCP.Core;

/// <summary>How a frontmatter tool should shape its response.</summary>
// JsonStringEnumConverter<T> (the generic, AOT-safe form) is used explicitly here because under Native
// AOT the SDK's reflection-based fallback that would otherwise add a string enum converter is unavailable,
// so without this the tool schema would expose the format as an opaque integer instead of named values.
[JsonConverter(typeof(JsonStringEnumConverter<FrontmatterFormat>))]
internal enum FrontmatterFormat
{
    /// <summary>Return the parsed key/value data only (default). Falls back to raw text if parsing fails.</summary>
    Parsed,

    /// <summary>Return the exact frontmatter text only, with no YAML parsing performed.</summary>
    Raw,

    /// <summary>Return both the parsed data and the exact frontmatter text.</summary>
    Both,
}