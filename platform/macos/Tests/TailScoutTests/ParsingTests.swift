import Foundation
import XCTest
@testable import TailScoutCore

final class ParsingTests: XCTestCase {
    func testParsesSharedStatusAndPolicies() throws {
        let status = try TailscaleStatus.parse(fixture("status.json"))

        XCTAssertEqual(status.version, "1.98.4-example")
        XCTAssertEqual(status.displayVersion, "1.98.4-example")
        XCTAssertEqual(status.backendState, .running)
        XCTAssertTrue(status.tun)
        XCTAssertEqual(status.currentTailnet?.name, "Example Tailnet")
        XCTAssertEqual(status.thisNode?.displayName, "example-device")
        XCTAssertEqual(status.thisNode?.primaryIP, "100.64.0.10")
        XCTAssertEqual(status.sortedPeers.map(\.displayName), ["guest-device", "example-phone", "example-desktop"])

        let phone = try XCTUnwrap(status.peers.first { $0.displayName == "example-phone" })
        let guest = try XCTUnwrap(status.peers.first { $0.displayName == "guest-device" })
        XCTAssertEqual(phone.osLabel, "Android")
        XCTAssertTrue(status.canSendTaildrop(to: phone))
        XCTAssertTrue(guest.canReceiveTaildrop)
        XCTAssertFalse(status.canSendTaildrop(to: guest))
        XCTAssertEqual(status.ownerLabel(for: phone), "Example User")
        XCTAssertTrue(try XCTUnwrap(status.peers.first { $0.displayName == "example-desktop" }).isSubnetRouter)
    }

    func testHandlesSharedNullStatus() throws {
        let status = try TailscaleStatus.parse(fixture("status-null.json"))

        XCTAssertEqual(status.version, "")
        XCTAssertEqual(status.backendState, .stopped)
        XCTAssertTrue(status.health.isEmpty)
        XCTAssertTrue(status.peers.isEmpty)
        XCTAssertEqual(status.thisNode?.displayName, "unknown")
    }

    func testParsesSharedProfiles() throws {
        let profiles = try TailscaleProfile.parseList(
            String(decoding: fixture("profiles.json"), as: UTF8.self)
        )

        XCTAssertEqual(profiles.count, 2)
        XCTAssertEqual(profiles[0].displayName, "Work")
        XCTAssertTrue(profiles[0].selected)
        XCTAssertEqual(profiles[1].displayName, "me@home.example")
        XCTAssertEqual(profiles[1].switchKey, "profile-b")
    }

    func testRejectsInvalidJSON() {
        XCTAssertThrowsError(try TailscaleStatus.parse("not json"))
    }

    private func fixture(_ name: String) throws -> Data {
        let repository = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
        return try Data(contentsOf: repository
            .appendingPathComponent("shared/fixtures")
            .appendingPathComponent(name))
    }
}
