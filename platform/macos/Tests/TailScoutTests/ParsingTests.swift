import XCTest
@testable import TailScoutCore

final class ParsingTests: XCTestCase {
    private let sampleStatus = """
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
        "OS": "macOS",
        "TailscaleIPs": ["100.100.8.31", "fd7a:115c:a1e0::1"],
        "Online": true,
        "UserID": 110841043178303
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
    """

    func testParsesTopLevelStatus() throws {
        let status = try TailscaleStatus.parse(sampleStatus)

        XCTAssertEqual(status.version, "1.98.4-t9e69045b2")
        XCTAssertEqual(status.clientVersion, "1.98.4")
        XCTAssertEqual(status.backendState, .running)
        XCTAssertTrue(status.backendState.isRunning)
        XCTAssertEqual(status.currentTailnet?.name, "jkp.org.in")
        XCTAssertEqual(status.health, ["relay warning"])
    }

    func testParsesNodesAndHelpers() throws {
        let status = try TailscaleStatus.parse(sampleStatus)
        let selfNode = try XCTUnwrap(status.thisNode)

        XCTAssertEqual(selfNode.displayName, "shre")
        XCTAssertEqual(selfNode.cleanDNSName, "shre.tail9e520a.ts.net")
        XCTAssertEqual(selfNode.primaryIP, "100.100.8.31")

        let sortedPeers = status.sortedPeers
        XCTAssertEqual(sortedPeers.first?.displayName, "pixel")
        XCTAssertTrue(try XCTUnwrap(sortedPeers.first).canReceiveTaildrop)
        XCTAssertTrue(try XCTUnwrap(sortedPeers.last).isSubnetRouter)
        XCTAssertEqual(status.ownerLabel(for: try XCTUnwrap(sortedPeers.first)), "Shreyam Adhikari")
    }

    func testHandlesNullAndMissingFields() throws {
        let status = try TailscaleStatus.parse(
            """
            {
              "Version": null,
              "BackendState": "Stopped",
              "Health": null,
              "Peer": null,
              "User": null,
              "Self": {
                "HostName": null,
                "TailscaleIPs": null,
                "AllowedIPs": null,
                "Online": null,
                "TaildropTarget": null
              }
            }
            """
        )

        XCTAssertEqual(status.version, "")
        XCTAssertEqual(status.backendState, .stopped)
        XCTAssertTrue(status.health.isEmpty)
        XCTAssertTrue(status.peers.isEmpty)
        XCTAssertEqual(status.thisNode?.displayName, "unknown")
    }

    func testParsesSwitchProfiles() throws {
        let profiles = try TailscaleProfile.parseList(
            """
            [
              {
                "id": "profile-a",
                "nickname": "Work",
                "tailnet": "example.com",
                "account": "me@example.com",
                "selected": true
              },
              {
                "id": "profile-b",
                "nickname": null,
                "tailnet": "personal.ts.net",
                "account": "me@home.example",
                "selected": false
              }
            ]
            """
        )

        XCTAssertEqual(profiles.count, 2)
        XCTAssertEqual(profiles[0].displayName, "Work")
        XCTAssertEqual(profiles[0].switchKey, "profile-a")
        XCTAssertTrue(profiles[0].selected)
        XCTAssertEqual(profiles[1].displayName, "me@home.example")
    }

    func testRejectsInvalidJson() {
        XCTAssertThrowsError(try TailscaleStatus.parse("not json"))
    }
}
