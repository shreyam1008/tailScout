namespace TailScout.Windows.Core.Models;

internal static class Text
{
    public static string Value(string? value) => value ?? "";

    public static string First(params string[] values) =>
        values.FirstOrDefault(value => !string.IsNullOrWhiteSpace(value)) ?? "";
}
