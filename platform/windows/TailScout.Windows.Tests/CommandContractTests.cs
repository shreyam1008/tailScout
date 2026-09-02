using Microsoft.VisualStudio.TestTools.UnitTesting;
using TailScout.Windows.Core.Services;

namespace TailScout.Windows.Tests;

[TestClass]
public sealed class CommandContractTests
{
    [TestMethod]
    public async Task UsesSharedCliContract()
    {
        var runner = new RecordingRunner();
        var client = new TailscaleCli(runner);

        await client.ConnectAsync();
        await client.DisconnectAsync();
        await client.LoginAsync();
        await client.LogoutAsync();
        await client.SwitchProfileAsync("work");
        await client.SetExitNodeAsync("100.64.0.1");
        await client.ClearExitNodeAsync();
        await client.SetAdvertiseExitNodeAsync(true);
        await client.VersionAsync();
        await client.NetcheckAsync();
        await client.BugreportAsync();

        CollectionAssert.AreEqual(
            new[]
            {
                "up --timeout=30s",
                "down",
                "login --timeout=30s",
                "logout",
                "switch work",
                "set --exit-node=100.64.0.1",
                "set --exit-node=",
                "set --advertise-exit-node=true",
                "version",
                "netcheck",
                "bugreport"
            },
            runner.Calls.ToArray());
    }

    private sealed class RecordingRunner : ICommandRunner
    {
        public List<string> Calls { get; } = [];

        public Task<string> RunAsync(
            IReadOnlyList<string> arguments,
            CancellationToken cancellationToken = default)
        {
            Calls.Add(string.Join(' ', arguments));
            return Task.FromResult("");
        }
    }
}
