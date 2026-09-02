using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using TailScout.Windows.Core.Models;
using Windows.Storage.Pickers;
using WinRT.Interop;

namespace TailScout.Windows;

public sealed partial class MainWindow
{
    private async void SendFile_Click(object sender, RoutedEventArgs e)
    {
        if (DevicesList.SelectedItem is not DeviceListItem { Node: var node })
        {
            ShowInfo("Choose a device before sending a file.", InfoBarSeverity.Warning);
            return;
        }
        if (currentStatus?.CanSendTaildropTo(node) != true)
        {
            ShowInfo(
                currentStatus?.HasSameOwner(node) == false
                    ? "Taildrop only supports devices owned by the same Tailscale user."
                    : "The selected device is not available for Taildrop.",
                InfoBarSeverity.Warning);
            return;
        }

        var picker = new FileOpenPicker { SuggestedStartLocation = PickerLocationId.DocumentsLibrary };
        picker.FileTypeFilter.Add("*");
        InitializeWithWindow.Initialize(picker, WindowNative.GetWindowHandle(this));
        if (await picker.PickSingleFileAsync() is { } file)
        {
            await RunActionAsync("Sending file", ct => tailscale.SendFileAsync(file.Path, node.CliTarget, ct));
        }
    }

    private async void ReceiveFiles_Click(object sender, RoutedEventArgs e)
    {
        var picker = new FolderPicker { SuggestedStartLocation = PickerLocationId.Downloads };
        picker.FileTypeFilter.Add("*");
        InitializeWithWindow.Initialize(picker, WindowNative.GetWindowHandle(this));
        if (await picker.PickSingleFolderAsync() is { } folder)
        {
            await RunActionAsync("Receiving files", ct => tailscale.ReceiveFilesAsync(folder.Path, ct));
        }
    }
}
