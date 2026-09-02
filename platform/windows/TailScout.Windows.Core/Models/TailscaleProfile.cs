namespace TailScout.Windows.Core.Models;

public sealed record TailscaleProfile(
    string Id,
    string Nickname,
    string Tailnet,
    string Account,
    bool Selected)
{
    public string DisplayName => Text.First(Nickname, Account, Tailnet, Id);
    public string SwitchKey => Text.First(Id, Nickname, Account, Tailnet);

    public static IReadOnlyList<TailscaleProfile> ParseList(string json) =>
        JsonModel.Parse<RawProfile[]>(json)
            .Select(profile => new TailscaleProfile(
                Text.Value(profile.Id), Text.Value(profile.Nickname), Text.Value(profile.Tailnet),
                Text.Value(profile.Account), profile.Selected ?? false))
            .Where(profile => profile.DisplayName.Length > 0)
            .ToArray();
}

internal sealed class RawProfile
{
    public string? Id { get; init; }
    public string? Nickname { get; init; }
    public string? Tailnet { get; init; }
    public string? Account { get; init; }
    public bool? Selected { get; init; }
}
