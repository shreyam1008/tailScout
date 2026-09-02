import Foundation
import TailScoutCore

extension AppViewModel {
    func runBusy(_ operation: () async -> Void) async {
        guard !isWorking else { return }
        isWorking = true
        defer { isWorking = false }
        await operation()
    }

    func runCommand(success: String, operation: () async throws -> Void) async {
        await runBusy {
            do {
                try await operation()
                lastMessage = success
                await reloadAfterCommand()
            } catch {
                present(error, title: "Tailscale Command Failed")
            }
        }
    }

    func runDiagnostic(title: String, operation: () async throws -> String) async {
        await runBusy {
            do {
                let output = try await operation()
                diagnosticResult = DiagnosticResult(
                    title: title,
                    output: output.isEmpty ? "Command completed with no output." : output
                )
            } catch {
                present(error, title: "\(title) Failed")
            }
        }
    }

    func reloadAfterCommand() async {
        await loadStatus()
        await loadProfiles()
    }

    @discardableResult
    func loadStatus() async -> Bool {
        do {
            status = try await client.status()
            syncSelections()
            return true
        } catch {
            present(error, title: "Could Not Refresh Status")
            return false
        }
    }

    func loadProfiles() async {
        do {
            profiles = try await client.profiles()
            if !profiles.contains(where: { $0.switchKey == selectedProfileKey }) {
                selectedProfileKey = profiles.first(where: \.selected)?.switchKey
                    ?? profiles.first?.switchKey
                    ?? ""
            }
        } catch {
            profiles = []
            selectedProfileKey = ""
        }
    }

    func syncSelections() {
        let availablePeers = peers
        let nodeKeys = Set(availablePeers.map(\.stableKey))
        if selectedNodeKey.map({ nodeKeys.contains($0) }) != true {
            selectedNodeKey = availablePeers.first?.stableKey
        }
        let availableExitNodes = availablePeers.filter(\.exitNodeOption)
        if !availableExitNodes.contains(where: { $0.stableKey == selectedExitNodeKey }) {
            selectedExitNodeKey = availableExitNodes.first(where: { $0.exitNode || $0.active })?.stableKey
                ?? availableExitNodes.first?.stableKey
                ?? ""
        }
    }

    func present(_ error: Error, title: String) {
        alert = AppAlert(
            title: title,
            message: (error as? LocalizedError)?.errorDescription ?? error.localizedDescription
        )
    }
}
