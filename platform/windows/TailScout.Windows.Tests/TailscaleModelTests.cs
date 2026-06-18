using Microsoft.VisualStudio.TestTools.UnitTesting;
using TailScout.Windows.Core.Models;

namespace TailScout.Windows.Tests;

[TestClass]
public sealed class TailscaleModelTests
{
    private const string SampleStatus = """
        {
          "Version": "1.98.4-t9e69045b2",
          "ClientVersion": "1.98.4",
          "TUN": true,
          "BackendState": "Running",
          "MagicDNSSuffix": "tail9e520a.ts.net",
          "CurrentTailnet": {
            "Name": "jkp.org.in",
            "MagicDNSSuffix": "tail9e520a.ts.net",
            "MagicDNSEnabled": true
          },
          "Health": ["relay warning"],
          "User": {
            "110841043178303": {
              "ID": 110841043178303,
              "LoginName": "shreyama@jkp.org.in",
              "DisplayName": "Shreyam Adhikari",
              "ProfilePicURL": "https://example.invalid/photo.png"
            }
          },
          "Self": {
            "ID": "self-1",
            "HostName": "shre",
            "DNSName": "shre.tail9e520a.ts.net.",
            "OS": "windows",
            "TailscaleIPs": ["100.100.8.31", "fd7a:115c:a1e0::1"],
            "Online": true,
            "UserID": 110841043178303,
            "ExitNodeOption": true
          },
          "Peer": {
            "key1": {
              "ID": "peer-win",
              "HostName": "dev-pc",
              "DNSName": "dev-pc.tail9e520a.ts.net.",
              "OS": "windows",
              "TailscaleIPs": ["100.100.8.30"],
              "AllowedIPs": ["100.100.8.30/32", "10.10.0.0/16"],
              "Online": false,
              "RxBytes": 2048,
              "TxBytes": 1024
            },
            "key2": {
              "ID": "peer-phone",
              "HostName": "pixel",
              "DNSName": "pixel.tail9e520a.ts.net.",
              "OS": "android",
              "TailscaleIPs": ["100.100.8.32"],
              "Online": true,
              "ExitNodeOption": true,
              "TaildropTarget": 3,
              "UserID": 110841043178303
            }
          }
        }
        """;

    [TestMethod]
    public void ParsesStatusAndPeers()
    {
        var status = TailscaleStatus.Parse(SampleStatus);

        Assert.AreEqual("1.98.4-t9e69045b2", status.Version);
        Assert.AreEqual("Connected", status.StatusLabel);
        Assert.IsTrue(status.Tun);
        Assert.AreEqual("jkp.org.in", status.CurrentTailnet?.Name);
        Assert.AreEqual("shre", status.ThisNode?.DisplayName);
        Assert.AreEqual("100.100.8.31", status.ThisNode?.PrimaryIp);

        var sorted = status.SortedPeers;
        Assert.AreEqual(2, sorted.Count);
        Assert.AreEqual("pixel", sorted[0].DisplayName);
        Assert.IsTrue(sorted[0].CanReceiveTaildrop);
        Assert.IsTrue(sorted[1].IsSubnetRouter);
        Assert.AreEqual("Shreyam Adhikari", status.OwnerLabel(sorted[0]));
    }

    [TestMethod]
    public void HandlesMissingAndNullFields()
    {
        var status = TailscaleStatus.Parse("""
            {
              "BackendState": "NeedsLogin",
              "Version": null,
              "Health": null,
              "Peer": null,
              "Self": {
                "HostName": null,
                "TailscaleIPs": null,
                "Online": null
              }
            }
            """);

        Assert.AreEqual("Logged out", status.StatusLabel);
        Assert.AreEqual(string.Empty, status.Version);
        Assert.AreEqual(0, status.Health.Count);
        Assert.AreEqual(0, status.Peers.Count);
        Assert.AreEqual("unknown", status.ThisNode?.DisplayName);
    }

    [TestMethod]
    public void ParsesSwitchProfiles()
    {
        var profiles = TailscaleProfile.ParseList("""
            [
              {"id": "alice@example.com", "account": "alice@example.com", "tailnet": "alice.ts.net", "selected": true},
              {"id": "work", "nickname": "Work", "account": "alice@work.example", "tailnet": "corp.ts.net"}
            ]
            """);

        Assert.AreEqual(2, profiles.Count);
        Assert.AreEqual("alice@example.com", profiles[0].DisplayName);
        Assert.IsTrue(profiles[0].Selected);
        Assert.AreEqual("Work", profiles[1].DisplayName);
        Assert.AreEqual("work", profiles[1].SwitchKey);
    }
}
