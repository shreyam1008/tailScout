import Foundation
import TailScoutCore

extension AppViewModel {
    func refreshAll() async {
        await runBusy {
            let statusLoaded = await loadStatus()
            await loadProfiles()
            if statusLoaded { lastMessage = "Refreshed" }
        }
    }

    func connect() async {
        await runCommand(success: "Connected", operation: client.connect)
    }

    func disconnect() async {
        await runCommand(success: "Disconnected", operation: client.disconnect)
    }

    func login() async {
        await runBusy {
            do {
                let output = try await client.login()
                diagnosticResult = DiagnosticResult(
                    title: "Tailscale Log In",
                    output: output.isEmpty ? "Log in command completed." : output
                )
                await reloadAfterCommand()
            } catch {
                present(error, title: "Log In Failed")
            }
        }
    }

    func logout() async {
        await runCommand(success: "Logged out", operation: client.logout)
    }

    func switchSelectedProfile() async {
        guard !selectedProfileKey.isEmpty else {
            alert = AppAlert(title: "No Profile Selected", message: "Choose a saved account or tailnet first.")
            return
        }
        let key = selectedProfileKey
        await runCommand(success: "Switched account") { try await self.client.switchProfile(key) }
    }

    func setSelectedExitNode() async {
        guard let node = selectedExitNode else {
            alert = AppAlert(title: "No Exit Node Selected", message: "Choose an approved exit node first.")
            return
        }
        guard let target = node.cliTarget else {
            alert = AppAlert(title: "Exit Node Unavailable", message: "No usable Tailscale address was found for \(node.displayName).")
            return
        }
        await runCommand(success: "Exit node set to \(node.displayName)") {
            try await self.client.setExitNode(target)
        }
    }

    func clearExitNode() async {
        await runCommand(success: "Exit node cleared", operation: client.clearExitNode)
    }

    func advertiseExitNode(_ enabled: Bool) async {
        let message = enabled ? "This device is advertising as an exit node" : "Exit-node advertising stopped"
        await runCommand(success: message) { try await self.client.advertiseExitNode(enabled) }
    }

    func sendFile(_ fileURL: URL, to node: TailscaleNode) async {
        guard canSendTaildrop(to: node) else {
            let message = status?.hasSameOwner(as: node) == false
                ? "Taildrop only supports devices owned by the same Tailscale user."
                : "Tailscale did not report \(node.displayName) as an available Taildrop target."
            alert = AppAlert(title: "Taildrop Unavailable", message: message)
            return
        }
        guard let target = node.cliTarget else {
            alert = AppAlert(title: "Taildrop Unavailable", message: "No usable Tailscale address was found for \(node.displayName).")
            return
        }

        await withSecurityScope(fileURL) {
            await self.runCommand(success: "Sent \(fileURL.lastPathComponent) to \(node.displayName)") {
                try await self.client.sendFile(fileURL, to: target)
            }
        }
    }

    func receiveFiles(to folderURL: URL) async {
        await withSecurityScope(folderURL) {
            await self.runCommand(success: "Received Taildrop files into \(folderURL.path)") {
                try await self.client.receiveFiles(to: folderURL)
            }
        }
    }

    func runVersion() async {
        await runDiagnostic(title: "Tailscale Version", operation: client.version)
    }

    func runNetcheck() async {
        await runDiagnostic(title: "Network Check", operation: client.netcheck)
    }

    func runBugreport() async {
        await runDiagnostic(title: "Bug Report", operation: client.bugreport)
    }

    func presentImportError(_ error: Error) {
        present(error, title: "File Selection Failed")
    }

    private func withSecurityScope(_ url: URL, operation: () async -> Void) async {
        let accessed = url.startAccessingSecurityScopedResource()
        defer { if accessed { url.stopAccessingSecurityScopedResource() } }
        await operation()
    }
}
