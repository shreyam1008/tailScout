using System.Collections.ObjectModel;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using TailScout.Windows.Core.Models;
using TailScout.Windows.Core.Services;
using Windows.Storage.Pickers;
using WinRT.Interop;

namespace TailScout.Windows;

public sealed partial class MainWindow : Window
{
    private readonly TailscaleCli tailscale = new();
    private readonly ObservableCollection<TailNode> devices = new();
    private readonly ObservableCollection<TailscaleProfile> profiles = new();
    private readonly ObservableCollection<TailNode> exitNodes = new();
    private bool busy;
    private bool updatingAdvertiseToggle;

    public MainWindow()
    {
        InitializeComponent();

        Title = "TailScout";
        DevicesList.ItemsSource = devices;
        ProfilesCombo.ItemsSource = profiles;
        ExitNodeCombo.ItemsSource = exitNodes;

        _ = RefreshAsync();
    }

    private async void Refresh_Click(object sender, RoutedEventArgs e) => await RefreshAsync();

    private async void Connect_Click(object sender, RoutedEventArgs e) =>
        await RunActionAsync("Connecting", ct => tailscale.ConnectAsync(ct));

    private async void Disconnect_Click(object sender, RoutedEventArgs e) =>
        await RunActionAsync("Disconnecting", ct => tailscale.DisconnectAsync(ct));

    private async void Login_Click(object sender, RoutedEventArgs e)
    {
        await RunActionAsync(
            "Starting login",
            async ct =>
            {
                var output = await tailscale.LoginAsync(ct);
                if (!string.IsNullOrWhiteSpace(output))
                {
                    await ShowMessageAsync("Tailscale login", output);
                }
            });
    }

    private async void Logout_Click(object sender, RoutedEventArgs e) =>
        await RunActionAsync("Logging out", ct => tailscale.LogoutAsync(ct));

    private async void SwitchProfile_Click(object sender, RoutedEventArgs e)
    {
        if (ProfilesCombo.SelectedItem is not TailscaleProfile profile || string.IsNullOrWhiteSpace(profile.SwitchKey))
        {
            ShowInfo("Choose a saved account before switching.", InfoBarSeverity.Warning);
            return;
        }

        await RunActionAsync("Switching account", ct => tailscale.SwitchProfileAsync(profile.SwitchKey, ct));
    }

    private async void UseExitNode_Click(object sender, RoutedEventArgs e)
    {
        if (ExitNodeCombo.SelectedItem is not TailNode node || string.IsNullOrWhiteSpace(node.TaildropDestination))
        {
            ShowInfo("Choose an available exit node.", InfoBarSeverity.Warning);
            return;
        }

        await RunActionAsync("Setting exit node", ct => tailscale.SetExitNodeAsync(node.TaildropDestination, ct));
    }

    private async void ClearExitNode_Click(object sender, RoutedEventArgs e) =>
        await RunActionAsync("Clearing exit node", ct => tailscale.ClearExitNodeAsync(ct));

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

    private async void SendFile_Click(object sender, RoutedEventArgs e)
    {
        if (DevicesList.SelectedItem is not TailNode node)
        {
            ShowInfo("Choose a device before sending a file.", InfoBarSeverity.Warning);
            return;
        }

        if (!node.CanReceiveTaildrop)
        {
            ShowInfo("The selected device is not available for Taildrop.", InfoBarSeverity.Warning);
            return;
        }

        var picker = new FileOpenPicker
        {
            SuggestedStartLocation = PickerLocationId.DocumentsLibrary
        };
        picker.FileTypeFilter.Add("*");

        var windowHandle = WindowNative.GetWindowHandle(this);
        InitializeWithWindow.Initialize(picker, windowHandle);

        var file = await picker.PickSingleFileAsync();
        if (file is null)
        {
            return;
        }

        await RunActionAsync(
            "Sending file",
            ct => tailscale.SendFileAsync(file.Path, node.TaildropDestination, ct));
    }

    private async void ReceiveFiles_Click(object sender, RoutedEventArgs e)
    {
        var picker = new FolderPicker
        {
            SuggestedStartLocation = PickerLocationId.Downloads
        };
        picker.FileTypeFilter.Add("*");

        var windowHandle = WindowNative.GetWindowHandle(this);
        InitializeWithWindow.Initialize(picker, windowHandle);

        var folder = await picker.PickSingleFolderAsync();
        if (folder is null)
        {
            return;
        }

        await RunActionAsync(
            "Receiving files",
            ct => tailscale.ReceiveFilesAsync(folder.Path, ct));
    }

    private async void Version_Click(object sender, RoutedEventArgs e) =>
        await RunDiagnosticAsync("Running version", ct => tailscale.VersionAsync(ct));

    private async void Netcheck_Click(object sender, RoutedEventArgs e) =>
        await RunDiagnosticAsync("Running netcheck", ct => tailscale.NetcheckAsync(ct));

    private async void Bugreport_Click(object sender, RoutedEventArgs e) =>
        await RunDiagnosticAsync("Creating bugreport", ct => tailscale.BugreportAsync(ct));

    private void DevicesList_SelectionChanged(object sender, SelectionChangedEventArgs e)
    {
        UpdateSelectedDeviceText();
    }

    private void UpdateSelectedDeviceText()
    {
        SelectedDeviceText.Text = DevicesList.SelectedItem is TailNode node
            ? $"{node.DisplayName} - {node.DetailLine}"
            : "Choose a device";
    }

    private async Task RefreshAsync()
    {
        if (busy)
        {
            return;
        }

        SetBusy(true, "Refreshing");

        try
        {
            var status = await tailscale.GetStatusAsync();
            IReadOnlyList<TailscaleProfile> savedProfiles;

            try
            {
                savedProfiles = await tailscale.GetProfilesAsync();
            }
            catch (Exception exception) when (exception is TailscaleCommandException or System.Text.Json.JsonException)
            {
                savedProfiles = Array.Empty<TailscaleProfile>();
                DiagnosticsOutput.Text = FriendlyMessage(exception);
            }

            ApplyStatus(status, savedProfiles);
            ShowInfo("Status refreshed.", InfoBarSeverity.Success);
        }
        catch (Exception exception) when (exception is TailscaleCommandException or System.Text.Json.JsonException)
        {
            ShowInfo(FriendlyMessage(exception), InfoBarSeverity.Error);
        }
        finally
        {
            SetBusy(false);
        }
    }

    private async Task RunActionAsync(string busyText, Func<CancellationToken, Task> action)
    {
        if (busy)
        {
            return;
        }

        SetBusy(true, busyText);

        try
        {
            await action(CancellationToken.None);
            await RefreshAsyncAfterAction();
        }
        catch (Exception exception) when (exception is TailscaleCommandException or IOException or UnauthorizedAccessException or ArgumentException)
        {
            ShowInfo(FriendlyMessage(exception), InfoBarSeverity.Error);
        }
        finally
        {
            SetBusy(false);
        }
    }

    private async Task RunDiagnosticAsync(string busyText, Func<CancellationToken, Task<string>> action)
    {
        if (busy)
        {
            return;
        }

        SetBusy(true, busyText);

        try
        {
            var output = await action(CancellationToken.None);
            DiagnosticsOutput.Text = string.IsNullOrWhiteSpace(output) ? "(no output)" : output;
            ShowInfo("Diagnostic command finished.", InfoBarSeverity.Success);
        }
        catch (Exception exception) when (exception is TailscaleCommandException or System.Text.Json.JsonException)
        {
            ShowInfo(FriendlyMessage(exception), InfoBarSeverity.Error);
        }
        finally
        {
            SetBusy(false);
        }
    }

    private async Task RefreshAsyncAfterAction()
    {
        try
        {
            var status = await tailscale.GetStatusAsync();
            IReadOnlyList<TailscaleProfile> savedProfiles;

            try
            {
                savedProfiles = await tailscale.GetProfilesAsync();
            }
            catch (Exception exception) when (exception is TailscaleCommandException or System.Text.Json.JsonException)
            {
                savedProfiles = profiles.ToArray();
                DiagnosticsOutput.Text = FriendlyMessage(exception);
            }

            ApplyStatus(status, savedProfiles);
            ShowInfo("Command finished.", InfoBarSeverity.Success);
        }
        catch (Exception exception) when (exception is TailscaleCommandException or System.Text.Json.JsonException)
        {
            ShowInfo(FriendlyMessage(exception), InfoBarSeverity.Warning);
        }
    }

    private void ApplyStatus(TailscaleStatus status, IReadOnlyList<TailscaleProfile> savedProfiles)
    {
        StateText.Text = status.StatusLabel;
        TailnetText.Text = TailnetLabel(status);
        SelfText.Text = status.ThisNode is null
            ? "This device is not reported by tailscaled."
            : $"This device: {status.ThisNode.DisplayName} {status.ThisNode.PrimaryIp}";
        VersionText.Text = VersionLabel(status);
        HealthText.Text = status.Health.Count == 0 ? string.Empty : string.Join(Environment.NewLine, status.Health);

        devices.Clear();
        foreach (var peer in status.SortedPeers)
        {
            devices.Add(peer);
        }

        exitNodes.Clear();
        foreach (var node in status.ExitNodeOptions)
        {
            exitNodes.Add(node);
        }

        profiles.Clear();
        foreach (var profile in savedProfiles)
        {
            profiles.Add(profile);
        }

        ProfilesCombo.SelectedItem = profiles.FirstOrDefault(profile => profile.Selected) ?? profiles.FirstOrDefault();
        ExitNodeCombo.SelectedItem = exitNodes.FirstOrDefault(node => node.ExitNode) ?? exitNodes.FirstOrDefault();

        updatingAdvertiseToggle = true;
        AdvertiseExitNodeToggle.IsOn = status.ThisNode?.ExitNodeOption == true;
        updatingAdvertiseToggle = false;

        DevicesList.SelectedItem = devices.FirstOrDefault();
        UpdateSelectedDeviceText();
    }

    private void SetBusy(bool isBusy, string text = "")
    {
        busy = isBusy;
        RootCommandBar.IsEnabled = !isBusy;
        ActionsPanel.IsHitTestVisible = !isBusy;
        ActionsPanel.Opacity = isBusy ? 0.6 : 1.0;
        BusyRing.IsActive = isBusy;
        BusyRing.Visibility = isBusy ? Visibility.Visible : Visibility.Collapsed;
        BusyText.Text = isBusy ? text : string.Empty;
    }

    private void ShowInfo(string message, InfoBarSeverity severity)
    {
        StatusInfo.Message = message;
        StatusInfo.Severity = severity;
        StatusInfo.IsOpen = true;
    }

    private async Task ShowMessageAsync(string title, string message)
    {
        if (Content is not FrameworkElement root || root.XamlRoot is null)
        {
            return;
        }

        var dialog = new ContentDialog
        {
            Title = title,
            CloseButtonText = "Close",
            XamlRoot = root.XamlRoot,
            Content = new ScrollViewer
            {
                MaxHeight = 420,
                Content = new TextBlock
                {
                    Text = message,
                    TextWrapping = TextWrapping.Wrap
                }
            }
        };

        await dialog.ShowAsync();
    }

    private static string TailnetLabel(TailscaleStatus status)
    {
        if (status.CurrentTailnet is { } tailnet && !string.IsNullOrWhiteSpace(tailnet.Name))
        {
            return $"Tailnet: {tailnet.Name}";
        }

        return !string.IsNullOrWhiteSpace(status.MagicDnsSuffix)
            ? $"Tailnet: {status.MagicDnsSuffix}"
            : "Tailnet: unknown";
    }

    private static string VersionLabel(TailscaleStatus status)
    {
        var version = !string.IsNullOrWhiteSpace(status.ClientVersion)
            ? status.ClientVersion
            : status.Version;

        return string.IsNullOrWhiteSpace(version) ? "Tailscale version: unknown" : $"Tailscale version: {version}";
    }

    private static string FriendlyMessage(Exception exception) =>
        exception switch
        {
            TailscaleCommandException commandException when commandException.ExitCode is { } code =>
                $"{commandException.Message} ({commandException.Command}, exit {code})",
            TailscaleCommandException commandException =>
                $"{commandException.Message} ({commandException.Command})",
            System.Text.Json.JsonException =>
                "Tailscale returned JSON in an unexpected shape.",
            FileNotFoundException fileNotFound =>
                fileNotFound.Message,
            UnauthorizedAccessException =>
                "TailScout could not access the selected file.",
            DirectoryNotFoundException directoryNotFound =>
                directoryNotFound.Message,
            ArgumentException argumentException =>
                argumentException.Message,
            _ => exception.Message
        };
}
