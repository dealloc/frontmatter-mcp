using System.Globalization;

using YamlDotNet.Core;
using YamlDotNet.Core.Events;

namespace FrontmatterMCP.Core;

/// <summary>
/// Parses frontmatter YAML into a dynamic Dictionary/List/scalar tree using YamlDotNet's low-level,
/// reflection-free event API (<see cref="Parser"/>), never YamlDotNet's Deserializer. Frontmatter has
/// no fixed schema to bind to, and the deserializer's reflection-based type resolution is not AOT-safe.
/// </summary>
internal static class FrontmatterYamlParser
{
    public static YamlParseResult Parse(string raw)
    {
        // An empty (or whitespace-only) frontmatter block produces no YAML document at all -
        // the parser goes straight from StreamStart to StreamEnd - so it must be special-cased.
        if (string.IsNullOrWhiteSpace(raw))
        {
            return new YamlParseResult { Value = [] };
        }

        try
        {
            IParser parser = new Parser(new StringReader(raw));
            parser.Consume<StreamStart>();
            parser.Consume<DocumentStart>();

            object? root = parser.Accept<DocumentEnd>(out _) ? null : ParseNode(parser);

            parser.Consume<DocumentEnd>();

            return root switch
            {
                null => new YamlParseResult { Value = [] },
                Dictionary<string, object?> map => new YamlParseResult { Value = map },
                _ => new YamlParseResult { Error = "frontmatter root must be a mapping" },
            };
        }
        catch (YamlException ex)
        {
            return new YamlParseResult { Error = ex.Message };
        }
    }

    private static object? ParseNode(IParser parser)
    {
        if (parser.TryConsume<MappingStart>(out _))
        {
            Dictionary<string, object?> map = [];
            while (!parser.TryConsume<MappingEnd>(out _))
            {
                Scalar key = parser.Consume<Scalar>();
                map[key.Value] = ParseNode(parser);
            }

            return map;
        }

        if (parser.TryConsume<SequenceStart>(out _))
        {
            List<object?> list = [];
            while (!parser.TryConsume<SequenceEnd>(out _))
            {
                list.Add(ParseNode(parser));
            }

            return list;
        }

        Scalar scalar = parser.Consume<Scalar>();
        return ResolveScalar(scalar);
    }

    /// <summary>
    /// Applies YAML 1.1 core-schema implicit typing to plain (unquoted) scalars. Quoted scalars are
    /// always returned as strings, since quoting is how YAML authors opt out of implicit typing.
    /// </summary>
    private static object? ResolveScalar(Scalar scalar)
    {
        if (scalar.Style != ScalarStyle.Plain)
        {
            return scalar.Value;
        }

        return scalar.Value switch
        {
            "" or "~" or "null" or "Null" or "NULL" => null,
            "true" or "True" or "TRUE" => true,
            "false" or "False" or "FALSE" => false,
            _ when long.TryParse(scalar.Value, NumberStyles.Integer, CultureInfo.InvariantCulture, out long integer) => integer,
            _ when double.TryParse(scalar.Value, NumberStyles.Float, CultureInfo.InvariantCulture, out double real) => real,
            _ => scalar.Value,
        };
    }
}
