using System;
using System.Text.Json;
using System.Threading.Tasks;
using DotnetMcpServer.Data;
using DotnetMcpServer.Tools;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;
using ModelContextProtocol.Protocol;
using ModelContextProtocol.Server;

namespace DotnetMcpServer;

public class Program
{
    public static async Task Main(string[] args)
    {
        // Direct all framework logging strictly to STDERR so STDOUT remains 100% clean for MCP JSON-RPC
        var builder = Host.CreateApplicationBuilder(args);

        builder.Logging.ClearProviders();
        builder.Logging.AddConsole(options =>
        {
            options.LogToStandardErrorThreshold = LogLevel.Trace;
        });

        // Register Repository & Tools
        builder.Services.AddSingleton<EmployeeRepository>();
        builder.Services.AddSingleton<EmployeeTools>();

        // Register Official MCP Server with STDIO Transport
        builder.Services.AddMcpServer(options =>
        {
            options.ServerInfo = new Implementation
            {
                Name = "dotnet-employee-mcp-server",
                Version = "1.0.0"
            };
        })
        .WithTools<EmployeeTools>()
        .WithStdioServerTransport();

        Console.Error.WriteLine("[DotnetMcpServer] Initializing .NET 8 MCP Server over STDIO...");

        var app = builder.Build();
        await app.RunAsync();
    }
}
