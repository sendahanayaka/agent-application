using System.Text.Json.Serialization;

namespace DotnetMcpServer.Models;

public class ToolResponse
{
    [JsonPropertyName("success")]
    public bool Success { get; set; }

    [JsonPropertyName("message")]
    public string Message { get; set; } = string.Empty;

    [JsonPropertyName("count")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public int? Count { get; set; }

    [JsonPropertyName("data")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public object? Data { get; set; }

    [JsonPropertyName("error")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public string? Error { get; set; }

    public static ToolResponse Ok(string message, object? data = null, int? count = null)
    {
        return new ToolResponse
        {
            Success = true,
            Message = message,
            Data = data,
            Count = count
        };
    }

    public static ToolResponse Fail(string errorMessage)
    {
        return new ToolResponse
        {
            Success = false,
            Message = errorMessage,
            Error = errorMessage
        };
    }
}
