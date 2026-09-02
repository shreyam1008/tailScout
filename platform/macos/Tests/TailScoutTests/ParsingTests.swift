import Foundation
import XCTest
@testable import TailScoutCore

final class ParsingTests: XCTestCase {
    func testParsesSharedStatusAndPolicies() throws {
        let status = try TailscaleStatus.parse(fixture("status.json"))

        XCTAssertEqual(status.version, "1.98.4-t9e69045b2")
        XCTAssertEqual(status.displayVersion, "1.98.4-t9e69045b2")
        XCTAssertEqual(status.backendState, .running)
        XCTAssertTrue(status.tun)
        XCTAssertEqual(status.currentTailnet?.name, "jkp.org.in")
        XCTAssertEqual(status.thisNode?.displayName, "shre")
        XCTAssertEqual(status.thisNode?.primaryIP, "100.100.8.31")
        XCTAssertEqual(status.sortedPeers.map(\.displayName), ["guest-phone", "pixel", "dev-pc"])

        let phone = try XCTUnwrap(status.peers.first { $0.displayName == "pixel" })
        let guest = try XCTUnwrap(status.peers.first { $0.displayName == "guest-phone" })
        XCTAssertEqual(phone.osLabel, "Android")
        XCTAssertTrue(status.canSendTaildrop(to: phone))
        XCTAssertTrue(guest.canReceiveTaildrop)
        XCTAssertFalse(status.canSendTaildrop(to: guest))
        XCTAssertEqual(status.ownerLabel(for: phone), "Shreyam Adhikari")
        XCTAssertTrue(try XCTUnwrap(status.peers.first { $0.displayName == "dev-pc" }).isSubnetRouter)
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
