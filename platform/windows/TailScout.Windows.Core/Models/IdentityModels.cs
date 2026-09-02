namespace TailScout.Windows.Core.Models;

public sealed record TailnetInfo(string Name, string MagicDnsSuffix, bool MagicDnsEnabled);

public sealed record UserProfile(ulong Id, string LoginName, string DisplayName)
{
    public string DisplayLabel => Text.First(
        DisplayName,
        LoginName,
        Id == 0 ? "" : Id.ToString(),
        "Unknown user");
}
