using Microsoft.UI.Xaml.Controls;
using TailScout.Windows.Core.Models;

namespace TailScout.Windows;

public sealed partial class MainWindow
{
    private void DevicesList_SelectionChanged(object sender, SelectionChangedEventArgs e)
    {
        if (DevicesList.SelectedItem is not DeviceListItem item)
        {
            SelectedDeviceText.Text = "Choose a device";
            SelectedDeviceDetails.Text = "";
            return;
        }
        SelectedDeviceText.Text = item.DisplayName;
        SelectedDeviceDetails.Text = item.Details;
    }
}

public sealed record DeviceListItem(
    TailNode Node,
    string DetailLine,
    string Details)
{
    public string DisplayName => Node.DisplayName;
    public string StatusLabel => Node.StatusLabel;
    public override string ToString() => DisplayName;

    public static DeviceListItem Create(TailscaleStatus status, TailNode node)
    {
        var owner = status.OwnerLabel(node) ?? "Unknown";
        var summary = new List<string> { node.OsLabel, TextOr(node.PrimaryIp, "No Tailscale IP") };
        if (owner != "Unknown") summary.Add(owner);
        if (node.IsSubnetRouter) summary.Add("Subnet router");
        if (node.ExitNodeOption) summary.Add(node.ExitNode ? "Current exit node" : "Exit node");
        if (status.CanSendTaildropTo(node)) summary.Add("Taildrop");

        var taildrop = status.CanSendTaildropTo(node)
            ? "Available"
            : status.HasSameOwner(node)
                ? TextOr(node.NoFileSharingReason, "Unavailable")
                : "Unavailable: different Tailscale user";
        var state = node.Active ? $"{node.StatusLabel} · active" : node.StatusLabel;
        var details = string.Join(Environment.NewLine, new[]
        {
            $"Owner: {owner}",
            $"OS: {node.OsLabel}",
            $"Status: {state}",
            $"Tailscale IPs: {TextOr(string.Join(", ", node.TailscaleIps), "None")}",
            $"DNS: {TextOr(node.CleanDnsName, "None")}",
            $"Relay: {TextOr(node.Relay, "None")}",
            $"Endpoint: {TextOr(node.CurAddr, "None")}",
            $"Allowed IPs: {TextOr(string.Join(", ", node.AllowedIps), "None")}",
            $"Last Seen: {TextOr(node.LastSeen, "Unknown")}",
            $"Last Handshake: {TextOr(node.LastHandshake, "Unknown")}",
            $"Key Expiry: {TextOr(node.KeyExpiry, "Unknown")}",
            $"Exit Node: {(node.ExitNode ? "Currently selected" : node.ExitNodeOption ? "Available" : "No")}",
            $"Subnet Router: {(node.IsSubnetRouter ? "Yes" : "No")}",
            $"Taildrop: {taildrop}",
            $"Traffic: {FormatBytes(node.RxBytes)} received / {FormatBytes(node.TxBytes)} sent"
        });
        return new(node, string.Join(" · ", summary), details);
    }

    private static string TextOr(string value, string fallback) =>
        string.IsNullOrWhiteSpace(value) ? fallback : value;

    private static string FormatBytes(ulong value)
    {
        string[] units = ["B", "KB", "MB", "GB", "TB"];
        var amount = (double)value;
        var unit = 0;
        while (amount >= 1024 && unit < units.Length - 1)
        {
            amount /= 1024;
            unit++;
        }
        return unit == 0 ? $"{value} B" : $"{amount:0.0} {units[unit]}";
    }
}
