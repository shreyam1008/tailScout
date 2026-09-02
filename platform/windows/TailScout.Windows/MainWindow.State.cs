using Microsoft.UI.Xaml.Controls;
using TailScout.Windows.Core.Models;
using TailScout.Windows.Core.Services;

namespace TailScout.Windows;

public sealed partial class MainWindow
{
    private async Task RefreshAsync()
    {
        if (busy)
        {
            return;
        }
        SetBusy(true, "Refreshing");
        try
        {
            await LoadStateAsync();
            ShowInfo("Status refreshed.", InfoBarSeverity.Success);
        }
        catch (Exception exception) when (IsExpected(exception))
        {
            ShowInfo(FriendlyMessage(exception), InfoBarSeverity.Error);
        }
        finally
        {
            SetBusy(false);
        }
    }

    private async Task LoadStateAsync()
    {
        var status = await tailscale.GetStatusAsync();
        var savedProfiles = profiles.ToArray();
        try
        {
            savedProfiles = (await tailscale.GetProfilesAsync()).ToArray();
        }
        catch (Exception exception) when (IsExpected(exception))
        {
            DiagnosticsOutput.Text = FriendlyMessage(exception);
        }
        ApplyStatus(status, savedProfiles);
    }

    private void ApplyStatus(TailscaleStatus status, IReadOnlyList<TailscaleProfile> savedProfiles)
    {
        var selectedDeviceKey = (DevicesList.SelectedItem as DeviceListItem)?.Node.StableKey;
        currentStatus = status;
        StateText.Text = status.StatusLabel;
        TailnetText.Text = status.CurrentTailnet is { Name.Length: > 0 } tailnet
            ? $"Tailnet: {tailnet.Name}"
            : $"Tailnet: {TextOr(status.MagicDnsSuffix, "unknown")}";
        var self = status.ThisNode;
        SelfText.Text = self is null
            ? "This device is not reported by tailscaled."
            : $"This device: {self.DisplayName} · {self.OsLabel} · " +
              $"{TextOr(self.PrimaryIp, "No Tailscale IP")}" +
              (status.OwnerLabel(self) is { Length: > 0 } owner ? $" · {owner}" : "");
        VersionText.Text = $"Tailscale Version: {TextOr(status.DisplayVersion, "unknown")}";
        HealthText.Text = string.Join(Environment.NewLine, status.Health);

        var peers = status.SortedPeers;
        Replace(devices, peers.Select(node => DeviceListItem.Create(status, node)));
        Replace(exitNodes, peers.Where(node => node.ExitNodeOption));
        Replace(profiles, savedProfiles);
        ProfilesCombo.SelectedItem = profiles.FirstOrDefault(profile => profile.Selected) ?? profiles.FirstOrDefault();
        ExitNodeCombo.SelectedItem = exitNodes.FirstOrDefault(node => node.ExitNode) ?? exitNodes.FirstOrDefault();

        updatingAdvertiseToggle = true;
        AdvertiseExitNodeToggle.IsOn = status.ThisNode?.ExitNodeOption == true;
        updatingAdvertiseToggle = false;
        DevicesList.SelectedItem = devices.FirstOrDefault(item => item.Node.StableKey == selectedDeviceKey)
            ?? devices.FirstOrDefault();
    }

    private static void Replace<T>(ICollection<T> target, IEnumerable<T> values)
    {
        target.Clear();
        foreach (var value in values)
        {
            target.Add(value);
        }
    }

    private static string TextOr(string value, string fallback) =>
        string.IsNullOrWhiteSpace(value) ? fallback : value;
}
