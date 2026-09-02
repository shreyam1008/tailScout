using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using TailScout.Windows.Core.Services;

namespace TailScout.Windows;

public sealed partial class MainWindow
{
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
            try
            {
                await LoadStateAsync();
                ShowInfo("Command finished.", InfoBarSeverity.Success);
            }
            catch (Exception exception) when (IsExpected(exception))
            {
                ShowInfo(FriendlyMessage(exception), InfoBarSeverity.Warning);
            }
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

    private async Task RunDiagnosticAsync(string busyText, Func<CancellationToken, Task<string>> action)
    {
        if (busy)
        {
            return;
        }
        SetBusy(true, busyText);
        try
        {
            DiagnosticsOutput.Text = TextOr(await action(CancellationToken.None), "(no output)");
            ShowInfo("Diagnostic command finished.", InfoBarSeverity.Success);
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

    private void SetBusy(bool isBusy, string text = "")
    {
        busy = isBusy;
        RootCommandBar.IsEnabled = !isBusy;
        ActionsPanel.IsHitTestVisible = !isBusy;
        ActionsPanel.Opacity = isBusy ? 0.6 : 1;
        BusyRing.IsActive = isBusy;
        BusyRing.Visibility = isBusy ? Visibility.Visible : Visibility.Collapsed;
        BusyText.Text = isBusy ? text : "";
    }

    private void ShowInfo(string message, InfoBarSeverity severity)
    {
        StatusInfo.Message = message;
        StatusInfo.Severity = severity;
        StatusInfo.IsOpen = true;
    }

    private async Task ShowMessageAsync(string title, string message)
    {
        if (Content is not FrameworkElement { XamlRoot: { } root })
        {
            return;
        }
        await new ContentDialog
        {
            Title = title,
            CloseButtonText = "Close",
            XamlRoot = root,
            Content = new ScrollViewer
            {
                MaxHeight = 420,
                Content = new TextBlock { Text = message, TextWrapping = TextWrapping.Wrap }
            }
        }.ShowAsync();
    }

    private static bool IsExpected(Exception exception) => exception is
        TailscaleCommandException or System.Text.Json.JsonException or IOException or
        UnauthorizedAccessException or ArgumentException;

    private static string FriendlyMessage(Exception exception) => exception switch
    {
        TailscaleCommandException { ExitCode: { } code } command =>
            $"{command.Message} ({command.Command}, exit {code})",
        TailscaleCommandException command => $"{command.Message} ({command.Command})",
        System.Text.Json.JsonException => "Tailscale returned JSON in an unexpected shape.",
        UnauthorizedAccessException => "TailScout could not access the selected file.",
        _ => exception.Message
    };
}
