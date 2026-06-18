import Foundation
import SwiftUI
import TailScoutCore

struct AppAlert: Identifiable {
    let id = UUID()
    let title: String
    let message: String
}

struct DiagnosticResult: Identifiable {
    let id = UUID()
    let title: String
    let output: String
}

@MainActor
final class AppViewModel: ObservableObject {
    @Published var status: TailscaleStatus?
    @Published var profiles: [TailscaleProfile] = []
    @Published var selectedNodeKey: String?
    @Published var selectedProfileKey = ""
    @Published var selectedExitNodeKey = ""
    @Published var isWorking = false
    @Published var lastMessage: String?
    @Published var alert: AppAlert?
    @Published var diagnosticResult: DiagnosticResult?

    private let client: TailscaleCLI

    init(client: TailscaleCLI = TailscaleCLI()) {
        self.client = client
    }

    var allNodes: [TailscaleNode] {
        var nodes: [TailscaleNode] = []
        if let thisNode = status?.thisNode {
            nodes.append(thisNode)
        }
        nodes.append(contentsOf: peers)
        return nodes
    }

    var peers: [TailscaleNode] {
        status?.sortedPeers ?? []
    }

    var selectedNode: TailscaleNode? {
        if let selectedNodeKey,
           let node = allNodes.first(where: { $0.stableKey == selectedNodeKey }) {
            return node
        }
        return status?.thisNode ?? peers.first
    }

    var exitNodeOptions: [TailscaleNode] {
        status?.exitNodeOptions ?? []
    }

    var selectedExitNode: TailscaleNode? {
        exitNodeOptions.first { $0.stableKey == selectedExitNodeKey }
    }

    var currentTailnetLabel: String {
        if let name = status?.currentTailnet?.name, !name.isEmpty {
            return name
        }
        if let suffix = status?.magicDNSSuffix, !suffix.isEmpty {
            return suffix
        }
        return "Unknown tailnet"
    }

    func refreshAll() async {
        await runBusy {
            let statusLoaded = await loadStatus()
            await loadProfiles(showErrors: false)
            if statusLoaded {
                lastMessage = "Refreshed"
            }
        }
    }

    func connect() async {
        await runCommand(success: "Connected") {
            try await client.connect()
        }
    }

    func disconnect() async {
        await runCommand(success: "Disconnected") {
            try await client.disconnect()
        }
    }

    func login() async {
        await runBusy {
            do {
                let output = try await client.login()
                diagnosticResult = DiagnosticResult(
                    title: "Tailscale Login",
                    output: output.isEmpty ? "Login command completed." : output
                )
                await reloadAfterCommand()
            } catch {
                present(error, title: "Login Failed")
            }
        }
    }

    func logout() async {
        await runCommand(success: "Logged out") {
            try await client.logout()
        }
    }

    func switchSelectedProfile() async {
        let key = selectedProfileKey
        guard !key.isEmpty else {
            alert = AppAlert(title: "No Profile Selected", message: "Choose a saved account or tailnet first.")
            return
        }

        await runCommand(success: "Switched account") {
            try await client.switchProfile(key)
        }
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
            try await client.setExitNode(target)
        }
    }

    func clearExitNode() async {
        await runCommand(success: "Exit node cleared") {
            try await client.clearExitNode()
        }
    }

    func advertiseExitNode(_ enabled: Bool) async {
        await runCommand(success: enabled ? "This Mac is advertising as an exit node" : "Exit-node advertising stopped") {
            try await client.advertiseExitNode(enabled)
        }
    }

    func sendFile(_ fileURL: URL, to node: TailscaleNode) async {
        guard let target = node.cliTarget else {
            alert = AppAlert(title: "Taildrop Unavailable", message: "No usable Tailscale address was found for \(node.displayName).")
            return
        }

        let didAccess = fileURL.startAccessingSecurityScopedResource()
        defer {
            if didAccess {
                fileURL.stopAccessingSecurityScopedResource()
            }
        }

        await runCommand(success: "Sent \(fileURL.lastPathComponent) to \(node.displayName)") {
            try await client.sendFile(fileURL, to: target)
        }
    }

    func receiveFiles(to folderURL: URL) async {
        let didAccess = folderURL.startAccessingSecurityScopedResource()
        defer {
            if didAccess {
                folderURL.stopAccessingSecurityScopedResource()
            }
        }

        await runCommand(success: "Received Taildrop files into \(folderURL.path)") {
            try await client.receiveFiles(to: folderURL)
        }
    }

    func runVersion() async {
        await runDiagnostic(title: "Tailscale Version") {
            try await client.version()
        }
    }

    func runNetcheck() async {
        await runDiagnostic(title: "Tailscale Netcheck") {
            try await client.netcheck()
        }
    }

    func runBugreport() async {
        await runDiagnostic(title: "Tailscale Bug Report") {
            try await client.bugreport()
        }
    }

    func presentImportError(_ error: Error) {
        present(error, title: "File Selection Failed")
    }

    private func runBusy(_ operation: () async -> Void) async {
        if isWorking {
            return
        }
        isWorking = true
        defer {
            isWorking = false
        }
        await operation()
    }

    private func runCommand(success: String, operation: () async throws -> Void) async {
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

    private func runDiagnostic(title: String, operation: () async throws -> String) async {
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

    private func reloadAfterCommand() async {
        await loadStatus()
        await loadProfiles(showErrors: false)
    }

    @discardableResult
    private func loadStatus() async -> Bool {
        do {
            status = try await client.status()
            syncNodeSelection()
            syncExitNodeSelection()
            return true
        } catch {
            present(error, title: "Could Not Refresh Status")
            return false
        }
    }

    private func loadProfiles(showErrors: Bool) async {
        do {
            profiles = try await client.profiles()
            syncProfileSelection()
        } catch {
            profiles = []
            selectedProfileKey = ""
            if showErrors {
                present(error, title: "Could Not Load Saved Accounts")
            }
        }
    }

    private func syncNodeSelection() {
        let keys = Set(allNodes.map(\.stableKey))
        if let selectedNodeKey, keys.contains(selectedNodeKey) {
            return
        }
        selectedNodeKey = status?.thisNode?.stableKey ?? peers.first?.stableKey
    }

    private func syncProfileSelection() {
        if profiles.contains(where: { $0.switchKey == selectedProfileKey }) {
            return
        }
        selectedProfileKey = profiles.first(where: \.selected)?.switchKey ?? profiles.first?.switchKey ?? ""
    }

    private func syncExitNodeSelection() {
        if exitNodeOptions.contains(where: { $0.stableKey == selectedExitNodeKey }) {
            return
        }
        selectedExitNodeKey = exitNodeOptions.first(where: { $0.exitNode || $0.active })?.stableKey ?? exitNodeOptions.first?.stableKey ?? ""
    }

    private func present(_ error: Error, title: String) {
        let message = (error as? LocalizedError)?.errorDescription ?? error.localizedDescription
        alert = AppAlert(title: title, message: message)
    }
}
