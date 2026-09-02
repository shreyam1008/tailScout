import Foundation

func runTailscaleProcess(executablePath: String?, arguments: [String]) async throws -> String {
    try await Task.detached(priority: .userInitiated) {
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

        // Drain both pipes while the process runs; waiting first can deadlock when
        // a diagnostic command fills an OS pipe buffer.
        let stdoutReader = Task.detached { stdout.fileHandleForReading.readDataToEndOfFile() }
        let stderrReader = Task.detached { stderr.fileHandleForReading.readDataToEndOfFile() }
        process.waitUntilExit()
        let output = String(data: await stdoutReader.value, encoding: .utf8)?
            .trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        let errorOutput = String(data: await stderrReader.value, encoding: .utf8)?
            .trimmingCharacters(in: .whitespacesAndNewlines) ?? ""

        guard process.terminationStatus == 0 else {
            throw TailscaleCLIError.commandFailed(
                arguments: arguments,
                code: process.terminationStatus,
                message: errorOutput.isEmpty ? output : errorOutput
            )
        }
        return output
    }.value
}
