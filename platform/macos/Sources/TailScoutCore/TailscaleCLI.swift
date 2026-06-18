import Foundation

public enum TailscaleCLIError: LocalizedError, Sendable {
    case launchFailed(String)
    case commandFailed(arguments: [String], code: Int32, message: String)
    case invalidFile(URL)
    case foldersCannotBeSent(URL)
    case invalidDirectory(URL)
    case missingTarget(String)

    public var errorDescription: String? {
        switch self {
        case .launchFailed(let message):
            "Could not start tailscale: \(message)"
        case .commandFailed(let arguments, let code, let message):
            let command = (["tailscale"] + arguments).joined(separator: " ")
            if message.isEmpty {
                return "\(command) failed with exit code \(code)."
            }
            return "\(command) failed with exit code \(code):\n\(message)"
        case .invalidFile(let url):
            "File does not exist: \(url.path)"
        case .foldersCannotBeSent(let url):
            "Taildrop send expects a file, not a folder: \(url.path)"
        case .invalidDirectory(let url):
            "Folder does not exist: \(url.path)"
        case .missingTarget(let device):
            "No usable Tailscale address was found for \(device)."
        }
    }
}

public struct TailscaleCLI: Sendable {
    public static let defaultBinaryName = "tailscale"

    private let executablePath: String?

    public init(executablePath: String? = nil) {
        self.executablePath = executablePath
    }

    public func status() async throws -> TailscaleStatus {
        let output = try await run(["status", "--json"])
        return try TailscaleStatus.parse(output)
    }

    public func profiles() async throws -> [TailscaleProfile] {
        let output = try await run(["switch", "--list", "--json"])
        return try TailscaleProfile.parseList(output)
    }

    public func connect() async throws {
        _ = try await run(["up", "--timeout=30s"])
    }

    public func disconnect() async throws {
        _ = try await run(["down"])
    }

    public func login() async throws -> String {
        try await run(["login", "--timeout=30s"])
    }

    public func logout() async throws {
        _ = try await run(["logout"])
    }

    public func switchProfile(_ idOrName: String) async throws {
        _ = try await run(["switch", idOrName])
    }

    public func setExitNode(_ target: String) async throws {
        _ = try await run(["set", "--exit-node=\(target)"])
    }

    public func clearExitNode() async throws {
        _ = try await run(["set", "--exit-node="])
    }

    public func advertiseExitNode(_ enabled: Bool) async throws {
        _ = try await run(["set", "--advertise-exit-node=\(enabled ? "true" : "false")"])
    }

    public func sendFile(_ fileURL: URL, to target: String) async throws {
        var isDirectory = ObjCBool(false)
        guard FileManager.default.fileExists(atPath: fileURL.path, isDirectory: &isDirectory) else {
            throw TailscaleCLIError.invalidFile(fileURL)
        }
        guard !isDirectory.boolValue else {
            throw TailscaleCLIError.foldersCannotBeSent(fileURL)
        }

        let destination = target.hasSuffix(":") ? target : "\(target):"
        _ = try await run(["file", "cp", fileURL.path, destination])
    }

    public func receiveFiles(to folderURL: URL) async throws {
        var isDirectory = ObjCBool(false)
        guard FileManager.default.fileExists(atPath: folderURL.path, isDirectory: &isDirectory),
              isDirectory.boolValue else {
            throw TailscaleCLIError.invalidDirectory(folderURL)
        }

        _ = try await run(["file", "get", "--conflict=rename", folderURL.path])
    }

    public func version() async throws -> String {
        try await run(["version"])
    }

    public func netcheck() async throws -> String {
        try await run(["netcheck"])
    }

    public func bugreport() async throws -> String {
        try await run(["bugreport"])
    }

    public func run(_ arguments: [String]) async throws -> String {
        let executablePath = self.executablePath
        return try await Task.detached(priority: .userInitiated) {
            try runSynchronous(executablePath: executablePath, arguments: arguments)
        }.value
    }
}

private func runSynchronous(executablePath: String?, arguments: [String]) throws -> String {
    let process = Process()
    if let executablePath {
        process.executableURL = URL(fileURLWithPath: executablePath)
        process.arguments = arguments
    } else {
        process.executableURL = URL(fileURLWithPath: "/usr/bin/env")
        process.arguments = [TailscaleCLI.defaultBinaryName] + arguments
    }

    let stdout = Pipe()
    let stderr = Pipe()
    process.standardOutput = stdout
    process.standardError = stderr

    do {
        try process.run()
    } catch {
        throw TailscaleCLIError.launchFailed(error.localizedDescription)
    }

    process.waitUntilExit()

    let outputData = stdout.fileHandleForReading.readDataToEndOfFile()
    let errorData = stderr.fileHandleForReading.readDataToEndOfFile()
    let output = String(data: outputData, encoding: .utf8)?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
    let errorOutput = String(data: errorData, encoding: .utf8)?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""

    guard process.terminationStatus == 0 else {
        throw TailscaleCLIError.commandFailed(
            arguments: arguments,
            code: process.terminationStatus,
            message: errorOutput.isEmpty ? output : errorOutput
        )
    }

    return output
}
