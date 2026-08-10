using System.Text.Json.Serialization;

using FrontmatterMCP.Core;

namespace FrontmatterMCP;

/// <summary>
/// Source-generated JSON metadata for the custom types crossing the MCP tool boundary (parameters and
/// return values). Required for Native AOT: without it, System.Text.Json has no reflection fallback to
/// resolve these types, and tool registration fails at startup.
/// </summary>
[JsonSerializable(typeof(FrontmatterResult))]
[JsonSerializable(typeof(FrontmatterFormat))]
[JsonSerializable(typeof(IReadOnlyList<FrontmatterResult>))]
[JsonSerializable(typeof(PropertyProjectionResult))]
[JsonSerializable(typeof(IReadOnlyList<PropertyProjectionResult>))]
[JsonSerializable(typeof(string[]))]
internal sealed partial class FrontmatterJsonContext : JsonSerializerContext;