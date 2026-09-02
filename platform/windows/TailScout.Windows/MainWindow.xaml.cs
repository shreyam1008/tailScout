using System.Collections.ObjectModel;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using TailScout.Windows.Core.Models;
using TailScout.Windows.Core.Services;

namespace TailScout.Windows;

public sealed partial class MainWindow : Window
{
    private readonly TailscaleCli tailscale = new();
    private readonly ObservableCollection<DeviceListItem> devices = [];
    private readonly ObservableCollection<TailscaleProfile> profiles = [];
    private readonly ObservableCollection<TailNode> exitNodes = [];
    private TailscaleStatus? currentStatus;
    private bool busy;
    private bool updatingAdvertiseToggle;

    public MainWindow()
    {
        InitializeComponent();
        Title = "TailScout";
        DevicesList.ItemsSource = devices;
        ProfilesCombo.ItemsSource = profiles;
        ExitNodeCombo.ItemsSource = exitNodes;

        if (IsStartupRefreshDisabled())
        {
            ShowInfo("Startup refresh skipped.", InfoBarSeverity.Informational);
        }
        else
        {
            _ = RefreshAsync();
        }
    }

    private async void Refresh_Click(object sender, RoutedEventArgs e) => await RefreshAsync();
    private async void Connect_Click(object sender, RoutedEventArgs e) =>
        await RunActionAsync("Connecting", tailscale.ConnectAsync);
    private async void Disconnect_Click(object sender, RoutedEventArgs e) =>
        await RunActionAsync("Disconnecting", tailscale.DisconnectAsync);
    private async void Logout_Click(object sender, RoutedEventArgs e) =>
        await RunActionAsync("Logging out", tailscale.LogoutAsync);

    private async void Login_Click(object sender, RoutedEventArgs e) =>
        await RunActionAsync("Starting login", async cancellationToken =>
        {
            var output = await tailscale.LoginAsync(cancellationToken);
            if (output.Length > 0)
            {
                await ShowMessageAsync("Tailscale Log In", output);
            }
        });

    private async void SwitchProfile_Click(object sender, RoutedEventArgs e)
    {
        if (ProfilesCombo.SelectedItem is not TailscaleProfile { SwitchKey.Length: > 0 } profile)
        {
            ShowInfo("Choose a saved account before switching.", InfoBarSeverity.Warning);
            return;
        }
        await RunActionAsync("Switching account", ct => tailscale.SwitchProfileAsync(profile.SwitchKey, ct));
    }

    private async void UseExitNode_Click(object sender, RoutedEventArgs e)
    {
        if (ExitNodeCombo.SelectedItem is not TailNode { CliTarget.Length: > 0 } node)
        {
            ShowInfo("Choose an available exit node.", InfoBarSeverity.Warning);
            return;
        }
        await RunActionAsync("Setting exit node", ct => tailscale.SetExitNodeAsync(node.CliTarget, ct));
    }

    private async void ClearExitNode_Click(object sender, RoutedEventArgs e) =>
        await RunActionAsync("Clearing exit node", tailscale.ClearExitNodeAsync);

    private async void AdvertiseExitNode_Toggled(object sender, RoutedEventArgs e)
    {
        if (updatingAdvertiseToggle)
        {
            return;
        }
        var enabled = AdvertiseExitNodeToggle.IsOn;
        await RunActionAsync(
            enabled ? "Advertising exit node" : "Stopping exit node advertisement",
            ct => tailscale.SetAdvertiseExitNodeAsync(enabled, ct));
    }

    private async void Version_Click(object sender, RoutedEventArgs e) =>
        await RunDiagnosticAsync("Running version", tailscale.VersionAsync);
    private async void Netcheck_Click(object sender, RoutedEventArgs e) =>
        await RunDiagnosticAsync("Running network check", tailscale.NetcheckAsync);
    private async void Bugreport_Click(object sender, RoutedEventArgs e) =>
        await RunDiagnosticAsync("Creating bug report", tailscale.BugreportAsync);

    private static bool IsStartupRefreshDisabled()
    {
        var value = Environment.GetEnvironmentVariable("TAILSCOUT_SKIP_STARTUP_REFRESH");
        return value?.ToLowerInvariant() is "1" or "true" or "yes";
    }
}
