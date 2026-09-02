import Foundation
import SwiftUI
import TailScoutCore

struct DetailView: View {
    @EnvironmentObject private var model: AppViewModel
    let receiveFiles: () -> Void
    let sendFile: (TailscaleNode) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HeaderView()
            Divider()
            Form {
                connection
                accounts
                selectedDevice
                taildrop
                exitNodes
                diagnostics
            }
            .formStyle(.grouped)
            .scrollContentBackground(.hidden)
        }
    }

    private var connection: some View {
        Section("Connection") {
            LabeledContent("State", value: model.status?.backendState.label ?? "Unknown")
            LabeledContent("Tailnet", value: model.currentTailnetLabel)
            LabeledContent("Tailscale Version", value: model.status?.displayVersion ?? "Unknown")
            HStack {
                action("Connect", "power", model.connect)
                    .disabled(model.isWorking || model.status?.backendState.isRunning == true)
                action("Disconnect", "poweroff", model.disconnect)
                    .disabled(model.isWorking || model.status?.backendState.isRunning != true)
                action("Log In", "person.crop.circle.badge.plus", model.login).disabled(model.isWorking)
                Button(role: .destructive) {
                    Task { await model.logout() }
                } label: {
                    Label("Log Out", systemImage: "person.crop.circle.badge.minus")
                }
                .disabled(model.isWorking)
            }
            if let message = model.lastMessage { Text(message).foregroundStyle(.secondary) }
            if let health = model.status?.health, !health.isEmpty {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Health").font(.headline)
                    ForEach(health, id: \.self) { Text($0).foregroundStyle(.secondary) }
                }
            }
        }
    }

    private var accounts: some View {
        Section("Accounts and Tailnets") {
            if model.profiles.isEmpty {
                Text("No saved accounts reported by tailscale switch --list --json.")
                    .foregroundStyle(.secondary)
            } else {
                Picker("Saved account/tailnet", selection: $model.selectedProfileKey) {
                    ForEach(model.profiles, id: \.switchKey) {
                        Text($0.displayName).tag($0.switchKey)
                    }
                }
                action("Switch", "person.2", model.switchSelectedProfile)
                    .disabled(model.isWorking || model.selectedProfileKey.isEmpty)
            }
        }
    }

    private var selectedDevice: some View {
        Section("Selected Device") {
            if let node = model.selectedNode {
                LabeledContent("Name", value: node.displayName)
                LabeledContent("Owner", value: model.status?.ownerLabel(for: node) ?? "Unknown")
                LabeledContent("OS", value: node.osLabel)
                LabeledContent("Tailscale IPs", value: value(node.tailscaleIPs.joined(separator: ", ")))
                LabeledContent("DNS", value: value(node.cleanDNSName))
                LabeledContent(
                    "Status",
                    value: "\(node.online ? "Online" : "Offline")\(node.active ? " · active" : "")"
                )
                LabeledContent("Relay", value: value(node.relay))
                LabeledContent("Endpoint", value: value(node.curAddr))
                LabeledContent("Allowed IPs", value: value(node.allowedIPs.joined(separator: ", ")))
                LabeledContent("Last Seen", value: value(node.lastSeen, fallback: "Unknown"))
                LabeledContent("Last Handshake", value: value(node.lastHandshake, fallback: "Unknown"))
                LabeledContent("Key Expiry", value: value(node.keyExpiry, fallback: "Unknown"))
                LabeledContent(
                    "Exit Node",
                    value: node.exitNode ? "Currently selected" : node.exitNodeOption ? "Available" : "No"
                )
                LabeledContent("Subnet Router", value: node.isSubnetRouter ? "Yes" : "No")
                LabeledContent("Taildrop", value: taildropLabel(for: node))
                LabeledContent(
                    "Traffic",
                    value: "\(formatBytes(node.rxBytes)) received / \(formatBytes(node.txBytes)) sent"
                )
            } else {
                Text("Refresh to load this tailnet.").foregroundStyle(.secondary)
            }
        }
    }

    private var taildrop: some View {
        Section("Taildrop") {
            HStack {
                Button(action: receiveFiles) {
                    Label("Receive Files", systemImage: "tray.and.arrow.down")
                }
                .disabled(model.isWorking)

                if let node = model.selectedNode {
                    Button { sendFile(node) } label: {
                        Label("Send File", systemImage: "paperplane")
                    }
                    .disabled(model.isWorking || !model.canSendTaildrop(to: node))
                    .help(taildropHelp(for: node))
                }
            }
            Text("Receive uses conflict renaming. Send availability follows Tailscale Taildrop policy.")
                .foregroundStyle(.secondary)
        }
    }

    private var exitNodes: some View {
        Section("Exit Node") {
            if model.exitNodeOptions.isEmpty {
                Text("No approved exit nodes are currently reported.").foregroundStyle(.secondary)
            } else {
                Picker("Exit node", selection: $model.selectedExitNodeKey) {
                    ForEach(model.exitNodeOptions, id: \.stableKey) {
                        Text($0.displayName).tag($0.stableKey)
                    }
                }
                HStack {
                    action(
                        "Use Exit Node",
                        "point.topleft.down.curvedto.point.bottomright.up",
                        model.setSelectedExitNode
                    )
                    .disabled(model.isWorking || model.selectedExitNodeKey.isEmpty)
                    action("Clear Exit Node", "xmark.circle", model.clearExitNode)
                        .disabled(model.isWorking)
                }
            }
            HStack {
                action("Advertise This Device", "antenna.radiowaves.left.and.right") {
                    await model.advertiseExitNode(true)
                }
                action("Stop Advertising", "antenna.radiowaves.left.and.right.slash") {
                    await model.advertiseExitNode(false)
                }
            }
            .disabled(model.isWorking)
        }
    }

    private var diagnostics: some View {
        Section("Diagnostics") {
            HStack {
                action("Tailscale Version", "number", model.runVersion)
                action("Network Check", "network", model.runNetcheck)
                action("Bug Report", "ladybug", model.runBugreport)
            }
            .disabled(model.isWorking)
        }
    }

    private func action(
        _ title: String,
        _ icon: String,
        _ operation: @escaping () async -> Void
    ) -> some View {
        Button { Task { await operation() } } label: { Label(title, systemImage: icon) }
    }

    private func taildropHelp(for node: TailscaleNode) -> String {
        if model.status?.hasSameOwner(as: node) == false {
            return "Taildrop only supports devices owned by the same Tailscale user."
        }
        if model.canSendTaildrop(to: node) { return "Send one file to \(node.displayName)" }
        if !node.noFileSharingReason.isEmpty { return node.noFileSharingReason }
        return node.online
            ? "Tailscale did not report this device as a Taildrop target."
            : "The device is offline."
    }

    private func taildropLabel(for node: TailscaleNode) -> String {
        if model.canSendTaildrop(to: node) { return "Available" }
        if model.status?.hasSameOwner(as: node) == false {
            return "Unavailable: different Tailscale user"
        }
        return value(node.noFileSharingReason, fallback: "Unavailable")
    }

    private func value(_ text: String, fallback: String = "None") -> String {
        text.isEmpty ? fallback : text
    }

    private func formatBytes(_ value: UInt64) -> String {
        let units = ["B", "KB", "MB", "GB", "TB"]
        var amount = Double(value)
        var unit = 0
        while amount >= 1024, unit < units.count - 1 {
            amount /= 1024
            unit += 1
        }
        return unit == 0 ? "\(value) B" : String(format: "%.1f %@", amount, units[unit])
    }
}

private struct HeaderView: View {
    @EnvironmentObject private var model: AppViewModel

    var body: some View {
        HStack {
            VStack(alignment: .leading, spacing: 4) {
                Text("TailScout").font(.largeTitle).bold()
                Text("\(model.status?.backendState.label ?? "Unknown") · \(model.currentTailnetLabel)")
                    .foregroundStyle(.secondary)
            }
            Spacer()
            if let node = model.status?.thisNode {
                VStack(alignment: .trailing, spacing: 4) {
                    Text(node.displayName).font(.headline)
                    Text(
                        "\(node.osLabel) · \(node.primaryIP ?? "No Tailscale IP")" +
                        (model.status?.ownerLabel(for: node).map { " · \($0)" } ?? "")
                    )
                    .foregroundStyle(.secondary)
                }
            }
        }
        .padding()
    }
}
