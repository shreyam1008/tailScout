using TailScout.Windows.Core.Models;

namespace TailScout.Windows.Core.Services;

public sealed class TailscaleCli(ICommandRunner? runner = null)
{
    private readonly ICommandRunner runner = runner ?? new TailscaleProcessRunner();

    public async Task<TailscaleStatus> GetStatusAsync(CancellationToken cancellationToken = default) =>
        TailscaleStatus.Parse(await RunAsync(["status", "--json"], cancellationToken));

    public async Task<IReadOnlyList<TailscaleProfile>> GetProfilesAsync(
        CancellationToken cancellationToken = default) =>
        TailscaleProfile.ParseList(await RunAsync(["switch", "--list", "--json"], cancellationToken));

    public Task ConnectAsync(CancellationToken cancellationToken = default) =>
        RunAsync(["up", "--timeout=30s"], cancellationToken);

    public Task DisconnectAsync(CancellationToken cancellationToken = default) =>
        RunAsync(["down"], cancellationToken);

    public Task<string> LoginAsync(CancellationToken cancellationToken = default) =>
        RunAsync(["login", "--timeout=30s"], cancellationToken);

    public Task LogoutAsync(CancellationToken cancellationToken = default) =>
        RunAsync(["logout"], cancellationToken);

    public Task SwitchProfileAsync(string idOrName, CancellationToken cancellationToken = default) =>
        RunAsync(["switch", idOrName], cancellationToken);

    public Task SetExitNodeAsync(string target, CancellationToken cancellationToken = default) =>
        RunAsync(["set", $"--exit-node={target}"], cancellationToken);

    public Task ClearExitNodeAsync(CancellationToken cancellationToken = default) =>
        RunAsync(["set", "--exit-node="], cancellationToken);

    public Task SetAdvertiseExitNodeAsync(bool enabled, CancellationToken cancellationToken = default) =>
        RunAsync(["set", $"--advertise-exit-node={enabled.ToString().ToLowerInvariant()}"], cancellationToken);

    public Task<string> VersionAsync(CancellationToken cancellationToken = default) =>
        RunAsync(["version"], cancellationToken);

    public Task<string> NetcheckAsync(CancellationToken cancellationToken = default) =>
        RunAsync(["netcheck"], cancellationToken);

    public Task<string> BugreportAsync(CancellationToken cancellationToken = default) =>
        RunAsync(["bugreport"], cancellationToken);

    public Task SendFileAsync(string path, string target, CancellationToken cancellationToken = default)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(path);
        ArgumentException.ThrowIfNullOrWhiteSpace(target);
        if (!File.Exists(path))
        {
            throw new FileNotFoundException("The selected file no longer exists.", path);
        }
        return RunAsync(["file", "cp", path, $"{target.TrimEnd(':')}:"], cancellationToken);
    }

    public Task ReceiveFilesAsync(string directory, CancellationToken cancellationToken = default)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(directory);
        if (!Directory.Exists(directory))
        {
            throw new DirectoryNotFoundException($"The selected folder does not exist: {directory}");
        }
        return RunAsync(["file", "get", "--conflict=rename", directory], cancellationToken);
    }

    public Task<string> RunAsync(
        IReadOnlyList<string> arguments,
        CancellationToken cancellationToken = default) => runner.RunAsync(arguments, cancellationToken);
}
