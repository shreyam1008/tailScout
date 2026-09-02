namespace TailScout.Windows.Core.Models;

public sealed record TailNode(
    string Id,
    string PublicKey,
    string HostName,
    string DnsName,
    string Os,
    IReadOnlyList<string> TailscaleIps,
    IReadOnlyList<string> AllowedIps,
    string CurAddr,
    string Relay,
    bool Online,
    bool ExitNode,
    bool ExitNodeOption,
    bool Active,
    long TaildropTarget,
    string NoFileSharingReason,
    ulong UserId,
    string KeyExpiry,
    string LastSeen,
    string LastHandshake,
    ulong RxBytes,
    ulong TxBytes)
{
    public string PrimaryIp =>
        TailscaleIps.FirstOrDefault(ip => ip.Contains('.')) ?? TailscaleIps.FirstOrDefault() ?? "";

    public string CleanDnsName => DnsName.TrimEnd('.');
    public string DisplayName => Text.First(HostName, CleanDnsName, PrimaryIp, "unknown");
    public string StableKey => Text.First(Id, PublicKey, PrimaryIp, DisplayName);
    public string StatusLabel => Online ? "Online" : "Offline";
    public string OsLabel => Os.ToLowerInvariant() switch
    {
        "" => "Unknown",
        "linux" => "Linux",
        "windows" => "Windows",
        "macos" or "darwin" => "macOS",
        "ios" => "iOS",
        "android" => "Android",
        "freebsd" => "FreeBSD",
        _ => char.ToUpperInvariant(Os[0]) + Os[1..]
    };
    public string CliTarget => Text.First(PrimaryIp, CleanDnsName);
    public bool CanReceiveTaildrop => Online && TaildropTarget > 0 && NoFileSharingReason.Length == 0;
    public bool IsSubnetRouter => AllowedIps.Any(ip => !ip.EndsWith("/32") && !ip.EndsWith("/128"));

    internal static TailNode From(RawNode node) => new(
        Text.Value(node.ID), Text.Value(node.PublicKey), Text.Value(node.HostName),
        Text.Value(node.DNSName), Text.Value(node.OS), node.TailscaleIPs ?? [],
        node.AllowedIPs ?? [], Text.Value(node.CurAddr), Text.Value(node.Relay),
        node.Online ?? false, node.ExitNode ?? false, node.ExitNodeOption ?? false,
        node.Active ?? false, node.TaildropTarget ?? 0, Text.Value(node.NoFileSharingReason),
        node.UserID ?? 0, Text.Value(node.KeyExpiry), Text.Value(node.LastSeen),
        Text.Value(node.LastHandshake), node.RxBytes ?? 0, node.TxBytes ?? 0);
}

internal sealed class RawNode
{
    public string? ID { get; init; }
    public string? PublicKey { get; init; }
    public string? HostName { get; init; }
    public string? DNSName { get; init; }
    public string? OS { get; init; }
    public string[]? TailscaleIPs { get; init; }
    public string[]? AllowedIPs { get; init; }
    public string? CurAddr { get; init; }
    public string? Relay { get; init; }
    public bool? Online { get; init; }
    public bool? ExitNode { get; init; }
    public bool? ExitNodeOption { get; init; }
    public bool? Active { get; init; }
    public long? TaildropTarget { get; init; }
    public string? NoFileSharingReason { get; init; }
    public ulong? UserID { get; init; }
    public string? KeyExpiry { get; init; }
    public string? LastSeen { get; init; }
    public string? LastHandshake { get; init; }
    public ulong? RxBytes { get; init; }
    public ulong? TxBytes { get; init; }
}
