using System.Text.Json;

namespace TailScout.Windows.Core.Models;

public sealed record TailnetInfo(string Name, string MagicDnsSuffix, bool MagicDnsEnabled);

public sealed record UserProfile(ulong Id, string LoginName, string DisplayName, string ProfilePicUrl)
{
    public string DisplayLabel =>
        !string.IsNullOrWhiteSpace(DisplayName) ? DisplayName :
        !string.IsNullOrWhiteSpace(LoginName) ? LoginName :
        Id.ToString();
}

public sealed record TailNode(
    string Id,
    string PublicKey,
    string HostName,
    string DnsName,
    string Os,
    IReadOnlyList<string> TailscaleIps,
    IReadOnlyList<string> AllowedIps,
    IReadOnlyList<string> Addrs,
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
        TailscaleIps.FirstOrDefault(ip => ip.Contains('.')) ??
        TailscaleIps.FirstOrDefault() ??
        string.Empty;

    public string CleanDnsName => DnsName.TrimEnd('.');

    public string DisplayName =>
        !string.IsNullOrWhiteSpace(HostName) ? HostName :
        !string.IsNullOrWhiteSpace(CleanDnsName) ? CleanDnsName :
        !string.IsNullOrWhiteSpace(PrimaryIp) ? PrimaryIp :
        "unknown";

    public string StatusLabel => Online ? "Online" : "Offline";

    public string TaildropDestination =>
        !string.IsNullOrWhiteSpace(PrimaryIp) ? PrimaryIp : CleanDnsName;

    public bool CanReceiveTaildrop =>
        Online && TaildropTarget > 0 && string.IsNullOrWhiteSpace(NoFileSharingReason);

    public bool IsSubnetRouter =>
        AllowedIps.Any(ip => !ip.EndsWith("/32", StringComparison.Ordinal) &&
                             !ip.EndsWith("/128", StringComparison.Ordinal));

    public string DetailLine
    {
        get
        {
            var parts = new List<string>();
            if (!string.IsNullOrWhiteSpace(PrimaryIp))
            {
                parts.Add(PrimaryIp);
            }

            if (!string.IsNullOrWhiteSpace(Os))
            {
                parts.Add(Os);
            }

            if (ExitNodeOption)
            {
                parts.Add(ExitNode ? "Current exit node" : "Exit node");
            }

            if (CanReceiveTaildrop)
            {
                parts.Add("Taildrop");
            }

            return parts.Count == 0 ? "No device details" : string.Join(" - ", parts);
        }
    }
}

public sealed record TailscaleProfile(
    string Id,
    string Nickname,
    string Tailnet,
    string Account,
    bool Selected)
{
    public string DisplayName =>
        !string.IsNullOrWhiteSpace(Nickname) ? Nickname :
        !string.IsNullOrWhiteSpace(Account) ? Account :
        !string.IsNullOrWhiteSpace(Tailnet) ? Tailnet :
        Id;

    public string SwitchKey =>
        !string.IsNullOrWhiteSpace(Id) ? Id :
        !string.IsNullOrWhiteSpace(Account) ? Account :
        !string.IsNullOrWhiteSpace(Tailnet) ? Tailnet :
        Nickname;

    public static IReadOnlyList<TailscaleProfile> ParseList(string json)
    {
        using var document = JsonDocument.Parse(json);
        var root = document.RootElement;
        var source = JsonHelpers.GetArraySource(root, "Profiles", "profiles");

        return source
            .Select(Parse)
            .Where(profile => !string.IsNullOrWhiteSpace(profile.DisplayName))
            .ToArray();
    }

    private static TailscaleProfile Parse(JsonElement element) =>
        new(
            JsonHelpers.GetString(element, "ID", "Id", "id"),
            JsonHelpers.GetString(element, "Nickname", "nickname"),
            JsonHelpers.GetString(element, "Tailnet", "tailnet"),
            JsonHelpers.GetString(element, "Account", "account"),
            JsonHelpers.GetBool(element, "Selected", "selected"));
}

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
        "Starting" => "Starting",
        "" => "Unknown",
        _ => BackendState
    };

    public IReadOnlyList<TailNode> SortedPeers =>
        Peers
            .OrderByDescending(peer => peer.Online)
            .ThenBy(peer => peer.DisplayName, StringComparer.OrdinalIgnoreCase)
            .ToArray();

    public IReadOnlyList<TailNode> ExitNodeOptions =>
        Peers
            .Where(peer => peer.ExitNodeOption)
            .OrderBy(peer => peer.DisplayName, StringComparer.OrdinalIgnoreCase)
            .ToArray();

    public static TailscaleStatus Parse(string json)
    {
        using var document = JsonDocument.Parse(json);
        var root = document.RootElement;

        var users = ParseUsers(JsonHelpers.GetObject(root, "User", "user"));
        var peers = ParsePeers(JsonHelpers.GetObject(root, "Peer", "peer"));

        return new TailscaleStatus(
            JsonHelpers.GetString(root, "Version", "version"),
            JsonHelpers.GetString(root, "ClientVersion", "clientVersion"),
            JsonHelpers.GetString(root, "BackendState", "backendState"),
            JsonHelpers.GetBool(root, "TUN", "tun"),
            JsonHelpers.GetString(root, "MagicDNSSuffix", "magicDNSSuffix"),
            ParseTailnet(JsonHelpers.GetObject(root, "CurrentTailnet", "currentTailnet")),
            JsonHelpers.GetStringArray(root, "Health", "health"),
            JsonHelpers.GetObject(root, "Self", "self") is { } self ? ParseNode(self) : null,
            peers,
            users);
    }

    public string? OwnerLabel(TailNode node) =>
        Users.TryGetValue(node.UserId, out var user) ? user.DisplayLabel : null;

    private static IReadOnlyDictionary<ulong, UserProfile> ParseUsers(JsonElement? element)
    {
        if (element is not { ValueKind: JsonValueKind.Object } usersElement)
        {
            return new Dictionary<ulong, UserProfile>();
        }

        var users = new Dictionary<ulong, UserProfile>();
        foreach (var property in usersElement.EnumerateObject())
        {
            var id = ulong.TryParse(property.Name, out var parsedId)
                ? parsedId
                : JsonHelpers.GetUlong(property.Value, "ID", "id");

            if (id == 0)
            {
                continue;
            }

            users[id] = new UserProfile(
                id,
                JsonHelpers.GetString(property.Value, "LoginName", "loginName"),
                JsonHelpers.GetString(property.Value, "DisplayName", "displayName"),
                JsonHelpers.GetString(property.Value, "ProfilePicURL", "profilePicURL"));
        }

        return users;
    }

    private static IReadOnlyList<TailNode> ParsePeers(JsonElement? element)
    {
        if (element is not { ValueKind: JsonValueKind.Object } peersElement)
        {
            return Array.Empty<TailNode>();
        }

        return peersElement.EnumerateObject()
            .Select(peer => ParseNode(peer.Value))
            .ToArray();
    }

    private static TailnetInfo? ParseTailnet(JsonElement? element)
    {
        if (element is not { ValueKind: JsonValueKind.Object } tailnetElement)
        {
            return null;
        }

        return new TailnetInfo(
            JsonHelpers.GetString(tailnetElement, "Name", "name"),
            JsonHelpers.GetString(tailnetElement, "MagicDNSSuffix", "magicDNSSuffix"),
            JsonHelpers.GetBool(tailnetElement, "MagicDNSEnabled", "magicDNSEnabled"));
    }

    private static TailNode ParseNode(JsonElement element) =>
        new(
            JsonHelpers.GetString(element, "ID", "id"),
            JsonHelpers.GetString(element, "PublicKey", "publicKey"),
            JsonHelpers.GetString(element, "HostName", "hostName"),
            JsonHelpers.GetString(element, "DNSName", "dnsName"),
            JsonHelpers.GetString(element, "OS", "os"),
            JsonHelpers.GetStringArray(element, "TailscaleIPs", "tailscaleIPs"),
            JsonHelpers.GetStringArray(element, "AllowedIPs", "allowedIPs"),
            JsonHelpers.GetStringArray(element, "Addrs", "addrs"),
            JsonHelpers.GetString(element, "CurAddr", "curAddr"),
            JsonHelpers.GetString(element, "Relay", "relay"),
            JsonHelpers.GetBool(element, "Online", "online"),
            JsonHelpers.GetBool(element, "ExitNode", "exitNode"),
            JsonHelpers.GetBool(element, "ExitNodeOption", "exitNodeOption"),
            JsonHelpers.GetBool(element, "Active", "active"),
            JsonHelpers.GetLong(element, "TaildropTarget", "taildropTarget"),
            JsonHelpers.GetString(element, "NoFileSharingReason", "noFileSharingReason"),
            JsonHelpers.GetUlong(element, "UserID", "userID"),
            JsonHelpers.GetString(element, "KeyExpiry", "keyExpiry"),
            JsonHelpers.GetString(element, "LastSeen", "lastSeen"),
            JsonHelpers.GetString(element, "LastHandshake", "lastHandshake"),
            JsonHelpers.GetUlong(element, "RxBytes", "rxBytes"),
            JsonHelpers.GetUlong(element, "TxBytes", "txBytes"));
}

internal static class JsonHelpers
{
    public static JsonElement? GetObject(JsonElement element, params string[] names)
    {
        var property = GetProperty(element, names);
        return property is { ValueKind: JsonValueKind.Object } ? property : null;
    }

    public static IEnumerable<JsonElement> GetArraySource(JsonElement root, params string[] names)
    {
        if (root.ValueKind == JsonValueKind.Array)
        {
            return root.EnumerateArray().ToArray();
        }

        var property = GetProperty(root, names);
        return property is { ValueKind: JsonValueKind.Array }
            ? property.Value.EnumerateArray().ToArray()
            : Array.Empty<JsonElement>();
    }

    public static string GetString(JsonElement element, params string[] names)
    {
        var property = GetProperty(element, names);
        if (property is null || property.Value.ValueKind == JsonValueKind.Null)
        {
            return string.Empty;
        }

        return property.Value.ValueKind switch
        {
            JsonValueKind.String => property.Value.GetString() ?? string.Empty,
            JsonValueKind.Number => property.Value.GetRawText(),
            JsonValueKind.True => "true",
            JsonValueKind.False => "false",
            _ => string.Empty
        };
    }

    public static IReadOnlyList<string> GetStringArray(JsonElement element, params string[] names)
    {
        var property = GetProperty(element, names);
        if (property is null || property.Value.ValueKind == JsonValueKind.Null)
        {
            return Array.Empty<string>();
        }

        if (property.Value.ValueKind == JsonValueKind.String)
        {
            var single = property.Value.GetString();
            return string.IsNullOrWhiteSpace(single) ? Array.Empty<string>() : new[] { single };
        }

        if (property.Value.ValueKind != JsonValueKind.Array)
        {
            return Array.Empty<string>();
        }

        return property.Value.EnumerateArray()
            .Select(item => item.ValueKind == JsonValueKind.String ? item.GetString() ?? string.Empty : item.GetRawText())
            .Where(value => !string.IsNullOrWhiteSpace(value))
            .ToArray();
    }

    public static bool GetBool(JsonElement element, params string[] names)
    {
        var property = GetProperty(element, names);
        if (property is null || property.Value.ValueKind == JsonValueKind.Null)
        {
            return false;
        }

        return property.Value.ValueKind switch
        {
            JsonValueKind.True => true,
            JsonValueKind.False => false,
            JsonValueKind.Number => property.Value.TryGetInt64(out var number) && number != 0,
            JsonValueKind.String => bool.TryParse(property.Value.GetString(), out var parsed) && parsed,
            _ => false
        };
    }

    public static long GetLong(JsonElement element, params string[] names)
    {
        var property = GetProperty(element, names);
        if (property is null || property.Value.ValueKind == JsonValueKind.Null)
        {
            return 0;
        }

        return property.Value.ValueKind switch
        {
            JsonValueKind.Number => property.Value.TryGetInt64(out var number) ? number : 0,
            JsonValueKind.String => long.TryParse(property.Value.GetString(), out var parsed) ? parsed : 0,
            _ => 0
        };
    }

    public static ulong GetUlong(JsonElement element, params string[] names)
    {
        var property = GetProperty(element, names);
        if (property is null || property.Value.ValueKind == JsonValueKind.Null)
        {
            return 0;
        }

        return property.Value.ValueKind switch
        {
            JsonValueKind.Number => property.Value.TryGetUInt64(out var number) ? number : 0,
            JsonValueKind.String => ulong.TryParse(property.Value.GetString(), out var parsed) ? parsed : 0,
            _ => 0
        };
    }

    private static JsonElement? GetProperty(JsonElement element, params string[] names)
    {
        if (element.ValueKind != JsonValueKind.Object)
        {
            return null;
        }

        foreach (var property in element.EnumerateObject())
        {
            if (names.Any(name => string.Equals(property.Name, name, StringComparison.OrdinalIgnoreCase)))
            {
                return property.Value;
            }
        }

        return null;
    }
}
