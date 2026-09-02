using System.Text.Json;
using System.Text.Json.Serialization;

namespace TailScout.Windows.Core.Models;

internal static class JsonModel
{
    private static readonly JsonSerializerOptions Options = new()
    {
        PropertyNameCaseInsensitive = true,
        NumberHandling = JsonNumberHandling.AllowReadingFromString
    };

    public static T Parse<T>(string json) =>
        JsonSerializer.Deserialize<T>(json, Options)
        ?? throw new JsonException("Tailscale returned an empty JSON document.");
}
