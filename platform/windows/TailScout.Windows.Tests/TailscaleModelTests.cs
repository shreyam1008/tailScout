using Microsoft.VisualStudio.TestTools.UnitTesting;
using TailScout.Windows.Core.Models;

namespace TailScout.Windows.Tests;

[TestClass]
public sealed class TailscaleModelTests
{
    [TestMethod]
    public void ParsesSharedStatusAndPolicies()
    {
        var status = TailscaleStatus.Parse(Fixture("status.json"));

        Assert.AreEqual("1.98.4-example", status.Version);
        Assert.AreEqual("1.98.4-example", status.DisplayVersion);
        Assert.AreEqual("Connected", status.StatusLabel);
        Assert.IsTrue(status.Tun);
        Assert.AreEqual("Example Tailnet", status.CurrentTailnet?.Name);
        Assert.AreEqual("example-device", status.ThisNode?.DisplayName);
        Assert.AreEqual("100.64.0.10", status.ThisNode?.PrimaryIp);

        CollectionAssert.AreEqual(
            new[] { "guest-device", "example-phone", "example-desktop" },
            status.SortedPeers.Select(peer => peer.DisplayName).ToArray());
        var phone = status.Peers.Single(peer => peer.DisplayName == "example-phone");
        var guest = status.Peers.Single(peer => peer.DisplayName == "guest-device");
        Assert.AreEqual("Android", phone.OsLabel);
        Assert.AreEqual("peer-phone", phone.StableKey);
        Assert.IsTrue(status.CanSendTaildropTo(phone));
        Assert.IsTrue(guest.CanReceiveTaildrop);
        Assert.IsFalse(status.CanSendTaildropTo(guest));
        Assert.IsTrue(status.Peers.Single(peer => peer.DisplayName == "example-desktop").IsSubnetRouter);
        Assert.AreEqual("Example User", status.OwnerLabel(phone));
    }

    [TestMethod]
    public void HandlesSharedNullStatus()
    {
        var status = TailscaleStatus.Parse(Fixture("status-null.json"));

        Assert.AreEqual("Disconnected", status.StatusLabel);
        Assert.AreEqual("", status.Version);
        Assert.AreEqual(0, status.Health.Count);
        Assert.AreEqual(0, status.Peers.Count);
        Assert.AreEqual("unknown", status.ThisNode?.DisplayName);
    }

    [TestMethod]
    public void ParsesSharedProfiles()
    {
        var profiles = TailscaleProfile.ParseList(Fixture("profiles.json"));

        Assert.AreEqual(2, profiles.Count);
        Assert.AreEqual("Work", profiles[0].DisplayName);
        Assert.IsTrue(profiles[0].Selected);
        Assert.AreEqual("me@home.example", profiles[1].DisplayName);
        Assert.AreEqual("profile-b", profiles[1].SwitchKey);
    }

    private static string Fixture(string name) =>
        File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "Fixtures", name));
}
