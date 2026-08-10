using System.Text.Json;

using FrontmatterMCP;
using FrontmatterMCP.Tools;

using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;

using ModelContextProtocol;

var builder = Host.CreateApplicationBuilder(args);

// Configure all logs to go to stderr (stdout is used for the MCP protocol messages).
builder.Logging.AddConsole(o => o.LogToStandardErrorThreshold = LogLevel.Trace);

// Under Native AOT there is no reflection fallback, so every custom type used as a tool parameter or
// return value needs source-generated JSON metadata, chained after the SDK's own default resolver.
JsonSerializerOptions toolSerializerOptions = new()
{
    TypeInfoResolverChain =
    {
        McpJsonUtilities.DefaultOptions.TypeInfoResolver!,
        FrontmatterJsonContext.Default,
    },
};

// Add the MCP services: the transport to use (stdio) and the tools to register.
builder.Services
    .AddMcpServer()
    .WithStdioServerTransport()
    .WithTools<FrontmatterTools>(toolSerializerOptions);

await builder.Build().RunAsync();