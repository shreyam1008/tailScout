import Foundation
import XCTest
@testable import TailScoutCore

final class CommandContractTests: XCTestCase {
    func testUsesSharedCLIContract() async throws {
        let recorder = CommandRecorder()
        let client = TailscaleCLI { arguments in await recorder.run(arguments) }
        let folder = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString, isDirectory: true)
        let file = folder.appendingPathComponent("sample.txt")
        try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
        try Data("sample".utf8).write(to: file)
        defer { try? FileManager.default.removeItem(at: folder) }

        try await client.connect()
        try await client.disconnect()
        _ = try await client.login()
        try await client.logout()
        try await client.switchProfile("work")
        try await client.setExitNode("100.64.0.1")
        try await client.clearExitNode()
        try await client.advertiseExitNode(true)
        try await client.sendFile(file, to: "100.64.0.2")
        try await client.receiveFiles(to: folder)
        _ = try await client.version()
        _ = try await client.netcheck()
        _ = try await client.bugreport()

        let calls = await recorder.calls
        XCTAssertEqual(calls, [
            ["up", "--timeout=30s"],
            ["down"],
            ["login", "--timeout=30s"],
            ["logout"],
            ["switch", "work"],
            ["set", "--exit-node=100.64.0.1"],
            ["set", "--exit-node="],
            ["set", "--advertise-exit-node=true"],
            ["file", "cp", file.path, "100.64.0.2:"],
            ["file", "get", "--conflict=rename", folder.path],
            ["version"],
            ["netcheck"],
            ["bugreport"]
        ])
    }
}

private actor CommandRecorder {
    private(set) var calls: [[String]] = []

    func run(_ arguments: [String]) -> String {
        calls.append(arguments)
        return ""
    }
}
