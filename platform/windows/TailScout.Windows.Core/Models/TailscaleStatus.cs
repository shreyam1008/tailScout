namespace TailScout.Windows.Core.Models;

public sealed record TailscaleStatus(
    string Version,
    string ClientVersion,
    string BackendState,
    bool Tun,
    string MagicDnsSuffix,
    TailnetInfo? CurrentTailnet,
    IReadOnlyList<string> Health,
    TailNode? ThisNode,
    IReadOnlyList<TailNode> Peers,
    IReadOnlyDictionary<ulong, UserProfile> Users)
{
    public string StatusLabel => BackendState switch
    {
        "NeedsLogin" => "Logged out",
        "Running" => "Connected",
        "Stopped" => "Disconnected",
        "Starting" => "Starting…",
        "" => "Unknown",
        _ => BackendState
    };

    public string DisplayVersion => Text.First(Version, ClientVersion);
    public IReadOnlyList<TailNode> SortedPeers => Peers
        .OrderByDescending(peer => peer.Online)
        .ThenBy(peer => peer.DisplayName, StringComparer.OrdinalIgnoreCase)
        .ToArray();
    public string? OwnerLabel(TailNode node) =>
        Users.TryGetValue(node.UserId, out var user) ? user.DisplayLabel : null;

    public bool HasSameOwner(TailNode node) =>
        ThisNode is null || ThisNode.UserId == 0 || node.UserId == 0 || ThisNode.UserId == node.UserId;

    public bool CanSendTaildropTo(TailNode node) => node.CanReceiveTaildrop && HasSameOwner(node);

    public static TailscaleStatus Parse(string json)
    {
        var raw = JsonModel.Parse<RawStatus>(json);
        var users = (raw.User ?? []).Select(entry =>
        {
            var id = ulong.TryParse(entry.Key, out var keyId) ? keyId : entry.Value.ID ?? 0;
            var value = entry.Value;
            return new UserProfile(id, Text.Value(value.LoginName), Text.Value(value.DisplayName));
        }).Where(user => user.Id > 0).ToDictionary(user => user.Id);

        return new(
            Text.Value(raw.Version), Text.Value(raw.ClientVersion), Text.Value(raw.BackendState),
            raw.TUN ?? false, Text.Value(raw.MagicDNSSuffix), raw.CurrentTailnet is null ? null : new(
                Text.Value(raw.CurrentTailnet.Name), Text.Value(raw.CurrentTailnet.MagicDNSSuffix),
                raw.CurrentTailnet.MagicDNSEnabled ?? false), raw.Health ?? [],
            raw.Self is null ? null : TailNode.From(raw.Self),
            (raw.Peer ?? []).Values.Select(TailNode.From).ToArray(), users);
    }
}

internal sealed class RawStatus
{
    public string? Version { get; init; }
    public string? ClientVersion { get; init; }
    public string? BackendState { get; init; }
    public bool? TUN { get; init; }
    public string? MagicDNSSuffix { get; init; }
    public RawTailnet? CurrentTailnet { get; init; }
    public string[]? Health { get; init; }
    public RawNode? Self { get; init; }
    public Dictionary<string, RawNode>? Peer { get; init; }
    public Dictionary<string, RawUser>? User { get; init; }
}

internal sealed class RawTailnet
{
    public string? Name { get; init; }
    public string? MagicDNSSuffix { get; init; }
    public bool? MagicDNSEnabled { get; init; }
}

internal sealed class RawUser
{
    public ulong? ID { get; init; }
    public string? LoginName { get; init; }
    public string? DisplayName { get; init; }
}
