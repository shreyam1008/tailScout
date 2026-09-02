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

        Assert.AreEqual("1.98.4-t9e69045b2", status.Version);
        Assert.AreEqual("1.98.4-t9e69045b2", status.DisplayVersion);
        Assert.AreEqual("Connected", status.StatusLabel);
        Assert.IsTrue(status.Tun);
        Assert.AreEqual("jkp.org.in", status.CurrentTailnet?.Name);
        Assert.AreEqual("shre", status.ThisNode?.DisplayName);
        Assert.AreEqual("100.100.8.31", status.ThisNode?.PrimaryIp);

        CollectionAssert.AreEqual(
            new[] { "guest-phone", "pixel", "dev-pc" },
            status.SortedPeers.Select(peer => peer.DisplayName).ToArray());
        var phone = status.Peers.Single(peer => peer.DisplayName == "pixel");
        var guest = status.Peers.Single(peer => peer.DisplayName == "guest-phone");
        Assert.AreEqual("Android", phone.OsLabel);
        Assert.AreEqual("peer-phone", phone.StableKey);
        Assert.IsTrue(status.CanSendTaildropTo(phone));
        Assert.IsTrue(guest.CanReceiveTaildrop);
        Assert.IsFalse(status.CanSendTaildropTo(guest));
        Assert.IsTrue(status.Peers.Single(peer => peer.DisplayName == "dev-pc").IsSubnetRouter);
        Assert.AreEqual("Shreyam Adhikari", status.OwnerLabel(phone));
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
