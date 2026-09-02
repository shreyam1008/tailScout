using System.ComponentModel;
using System.Diagnostics;
using TailScout.Windows.Core.Models;

namespace TailScout.Windows.Core.Services;

public sealed class TailscaleCommandException(
    string message,
    int? exitCode,
    string command,
    Exception? innerException = null) : Exception(message, innerException)
{
    public int? ExitCode { get; } = exitCode;
    public string Command { get; } = command;
}

internal sealed class TailscaleProcessRunner : ICommandRunner
{
    private const string BinaryOverrideEnv = "TAILSCOUT_TAILSCALE_BIN";
    private static readonly string[] CandidateExecutables = ["tailscale.exe", "tailscale"];

    public async Task<string> RunAsync(
        IReadOnlyList<string> arguments,
        CancellationToken cancellationToken = default)
    {
        var configured = Environment.GetEnvironmentVariable(BinaryOverrideEnv);
        if (!string.IsNullOrWhiteSpace(configured))
        {
            return await RunWithExecutableAsync(configured, arguments, cancellationToken);
        }

        Exception? lastNotFound = null;
        foreach (var executable in CandidateExecutables)
        {
            try
            {
                return await RunWithExecutableAsync(executable, arguments, cancellationToken);
            }
            catch (Win32Exception exception) when (exception.NativeErrorCode is 2 or 3)
            {
                lastNotFound = exception;
            }
        }

        throw new TailscaleCommandException(
            "Tailscale CLI was not found. Install Tailscale and make sure tailscale.exe is on PATH.",
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
            throw new TailscaleCommandException(
                Text.First(stderr, stdout, $"Tailscale command failed with exit code {process.ExitCode}."),
                process.ExitCode,
                command);
        }
        catch (OperationCanceledException)
        {
            TryKill(process);
            throw;
        }
    }

    private static void TryKill(Process process)
    {
        try
        {
            if (!process.HasExited)
            {
                process.Kill(entireProcessTree: true);
            }
        }
        catch (Exception exception) when (exception is InvalidOperationException or Win32Exception)
        {
        }
    }

    private static string FormatCommand(string executable, IReadOnlyList<string> arguments) =>
        string.Join(' ', new[] { executable }.Concat(arguments.Select(Quote)));

    private static string Quote(string value) => value.Contains(' ')
        ? $"\"{value.Replace("\"", "\\\"")}\""
        : value;
}
