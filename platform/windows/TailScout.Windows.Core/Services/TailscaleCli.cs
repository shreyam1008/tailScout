using System.ComponentModel;
using System.Diagnostics;
using TailScout.Windows.Core.Models;

namespace TailScout.Windows.Core.Services;

public sealed class TailscaleCommandException : Exception
{
    public TailscaleCommandException(string message, int? exitCode, string command, Exception? innerException = null)
        : base(message, innerException)
    {
        ExitCode = exitCode;
        Command = command;
    }

    public int? ExitCode { get; }

    public string Command { get; }
}

public sealed class TailscaleCli
{
    private const string BinaryOverrideEnv = "TAILSCOUT_TAILSCALE_BIN";
    private static readonly string[] CandidateExecutables = { "tailscale.exe", "tailscale" };

    public async Task<TailscaleStatus> GetStatusAsync(CancellationToken cancellationToken = default)
    {
        var json = await RunAsync(new[] { "status", "--json" }, cancellationToken);
        return TailscaleStatus.Parse(json);
    }

    public async Task<IReadOnlyList<TailscaleProfile>> GetProfilesAsync(CancellationToken cancellationToken = default)
    {
        var json = await RunAsync(new[] { "switch", "--list", "--json" }, cancellationToken);
        return TailscaleProfile.ParseList(json);
    }

    public Task ConnectAsync(CancellationToken cancellationToken = default) =>
        RunAsync(new[] { "up", "--timeout=30s" }, cancellationToken);

    public Task DisconnectAsync(CancellationToken cancellationToken = default) =>
        RunAsync(new[] { "down" }, cancellationToken);

    public Task<string> LoginAsync(CancellationToken cancellationToken = default) =>
        RunAsync(new[] { "login", "--timeout=30s" }, cancellationToken);

    public Task LogoutAsync(CancellationToken cancellationToken = default) =>
        RunAsync(new[] { "logout" }, cancellationToken);

    public Task SwitchProfileAsync(string idOrName, CancellationToken cancellationToken = default) =>
        RunAsync(new[] { "switch", idOrName }, cancellationToken);

    public Task SetExitNodeAsync(string target, CancellationToken cancellationToken = default) =>
        RunAsync(new[] { "set", $"--exit-node={target}" }, cancellationToken);

    public Task ClearExitNodeAsync(CancellationToken cancellationToken = default) =>
        RunAsync(new[] { "set", "--exit-node=" }, cancellationToken);

    public Task SetAdvertiseExitNodeAsync(bool enabled, CancellationToken cancellationToken = default) =>
        RunAsync(new[] { "set", $"--advertise-exit-node={enabled.ToString().ToLowerInvariant()}" }, cancellationToken);

    public Task<string> VersionAsync(CancellationToken cancellationToken = default) =>
        RunAsync(new[] { "version" }, cancellationToken);

    public Task<string> NetcheckAsync(CancellationToken cancellationToken = default) =>
        RunAsync(new[] { "netcheck" }, cancellationToken);

    public Task<string> BugreportAsync(CancellationToken cancellationToken = default) =>
        RunAsync(new[] { "bugreport" }, cancellationToken);

    public async Task SendFileAsync(string path, string target, CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(path))
        {
            throw new ArgumentException("Choose a file before sending.", nameof(path));
        }

        if (!File.Exists(path))
        {
            throw new FileNotFoundException("The selected file no longer exists.", path);
        }

        if (string.IsNullOrWhiteSpace(target))
        {
            throw new ArgumentException("Choose an online Taildrop target.", nameof(target));
        }

        var destination = $"{target.TrimEnd(':')}:";
        await RunAsync(new[] { "file", "cp", path, destination }, cancellationToken);
    }

    public async Task ReceiveFilesAsync(string directory, CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(directory))
        {
            throw new ArgumentException("Choose a folder before receiving files.", nameof(directory));
        }

        if (!Directory.Exists(directory))
        {
            throw new DirectoryNotFoundException($"The selected folder does not exist: {directory}");
        }

        await RunAsync(new[] { "file", "get", "--conflict=rename", directory }, cancellationToken);
    }

    public async Task<string> RunAsync(IReadOnlyList<string> arguments, CancellationToken cancellationToken = default)
    {
        var configuredExecutable = Environment.GetEnvironmentVariable(BinaryOverrideEnv);
        if (!string.IsNullOrWhiteSpace(configuredExecutable))
        {
            return await RunWithExecutableAsync(configuredExecutable, arguments, cancellationToken);
        }

        Exception? lastNotFound = null;

        foreach (var executable in CandidateExecutables)
        {
            try
            {
                return await RunWithExecutableAsync(executable, arguments, cancellationToken);
            }
            catch (Win32Exception exception) when (IsNotFound(exception))
            {
                lastNotFound = exception;
            }
        }

        throw new TailscaleCommandException(
            "Tailscale CLI was not found. Install Tailscale for Windows and make sure tailscale.exe is on PATH.",
            null,
            "tailscale",
            lastNotFound);
    }

    private static async Task<string> RunWithExecutableAsync(
        string executable,
        IReadOnlyList<string> arguments,
        CancellationToken cancellationToken)
    {
        var startInfo = new ProcessStartInfo
        {
            FileName = executable,
            RedirectStandardError = true,
            RedirectStandardOutput = true,
            UseShellExecute = false,
            CreateNoWindow = true
        };

        foreach (var argument in arguments)
        {
            startInfo.ArgumentList.Add(argument);
        }

        using var process = new Process { StartInfo = startInfo };
        var command = FormatCommand(executable, arguments);

        if (!process.Start())
        {
            throw new TailscaleCommandException("Could not start the Tailscale CLI.", null, command);
        }

        var stdoutTask = process.StandardOutput.ReadToEndAsync(cancellationToken);
        var stderrTask = process.StandardError.ReadToEndAsync(cancellationToken);

        try
        {
            await process.WaitForExitAsync(cancellationToken);
            var stdout = (await stdoutTask).Trim();
            var stderr = (await stderrTask).Trim();

            if (process.ExitCode == 0)
            {
                return stdout;
            }

            var message = !string.IsNullOrWhiteSpace(stderr)
                ? stderr
                : !string.IsNullOrWhiteSpace(stdout)
                    ? stdout
                    : $"Tailscale command failed with exit code {process.ExitCode}.";

            throw new TailscaleCommandException(message, process.ExitCode, command);
        }
        catch (OperationCanceledException)
        {
            TryKill(process);
            throw;
        }
    }

    private static bool IsNotFound(Win32Exception exception) =>
        exception.NativeErrorCode is 2 or 3;

    private static void TryKill(Process process)
    {
        try
        {
            if (!process.HasExited)
            {
                process.Kill(entireProcessTree: true);
            }
        }
        catch (InvalidOperationException)
        {
        }
        catch (Win32Exception)
        {
        }
    }

    private static string FormatCommand(string executable, IReadOnlyList<string> arguments) =>
        string.Join(" ", new[] { executable }.Concat(arguments.Select(Quote)));

    private static string Quote(string value) =>
        value.Contains(' ', StringComparison.Ordinal) ? $"\"{value.Replace("\"", "\\\"", StringComparison.Ordinal)}\"" : value;
}
