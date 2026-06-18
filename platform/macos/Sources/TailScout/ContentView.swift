import SwiftUI
import TailScoutCore
import UniformTypeIdentifiers

struct ContentView: View {
    @EnvironmentObject private var model: AppViewModel
    @State private var showingSendImporter = false
    @State private var showingReceiveImporter = false
    @State private var sendTarget: TailscaleNode?

    var body: some View {
        NavigationSplitView {
            sidebar
        } detail: {
            detail
        }
        .frame(minWidth: 980, minHeight: 640)
        .toolbar {
            ToolbarItemGroup {
                if model.isWorking {
                    ProgressView()
                        .controlSize(.small)
                }
                Button {
                    Task { await model.refreshAll() }
                } label: {
                    Label("Refresh", systemImage: "arrow.clockwise")
                }
                .help("Refresh status and devices")
            }
        }
        .alert(item: $model.alert) { alert in
            Alert(
                title: Text(alert.title),
                message: Text(alert.message),
                dismissButton: .default(Text("OK"))
            )
        }
        .sheet(item: $model.diagnosticResult) { result in
            DiagnosticView(result: result)
        }
        .fileImporter(
            isPresented: $showingSendImporter,
            allowedContentTypes: [.item],
            allowsMultipleSelection: false
        ) { selection in
            handleSendSelection(selection)
        }
        .fileImporter(
            isPresented: $showingReceiveImporter,
            allowedContentTypes: [.folder],
            allowsMultipleSelection: false
        ) { selection in
            handleReceiveSelection(selection)
        }
    }

    private var sidebar: some View {
        List(selection: $model.selectedNodeKey) {
            if let thisNode = model.status?.thisNode {
                Section("This Mac") {
                    DeviceListRow(
                        node: thisNode,
                        owner: model.status?.ownerLabel(for: thisNode),
                        isSelf: true
                    )
                    .tag(Optional(thisNode.stableKey))
                }
            }

            Section("Devices") {
                if model.peers.isEmpty {
                    Text("No peers found")
                        .foregroundStyle(.secondary)
                } else {
                    ForEach(model.peers, id: \.stableKey) { node in
                        DeviceListRow(
                            node: node,
                            owner: model.status?.ownerLabel(for: node),
                            isSelf: false
                        )
                        .tag(Optional(node.stableKey))
                    }
                }
            }
        }
        .navigationTitle("TailScout")
    }

    private var detail: some View {
        VStack(alignment: .leading, spacing: 0) {
            HeaderView()
                .environmentObject(model)

            Divider()

            Form {
                connectionSection
                accountSection
                selectedDeviceSection
                taildropSection
                exitNodeSection
                diagnosticsSection
            }
            .formStyle(.grouped)
            .scrollContentBackground(.hidden)
        }
    }

    private var connectionSection: some View {
        Section("Connection") {
            LabeledContent("State", value: model.status?.backendState.label ?? "Unknown")
            LabeledContent("Tailnet", value: model.currentTailnetLabel)
            LabeledContent("Tailscale Version", value: model.status?.displayVersion ?? "Unknown")

            HStack {
                Button {
                    Task { await model.connect() }
                } label: {
                    Label("Connect", systemImage: "power")
                }
                .disabled(model.isWorking || model.status?.backendState.isRunning == true)

                Button {
                    Task { await model.disconnect() }
                } label: {
                    Label("Disconnect", systemImage: "poweroff")
                }
                .disabled(model.isWorking || model.status?.backendState.isRunning != true)

                Button {
                    Task { await model.login() }
                } label: {
                    Label("Login", systemImage: "person.crop.circle.badge.plus")
                }
                .disabled(model.isWorking)

                Button(role: .destructive) {
                    Task { await model.logout() }
                } label: {
                    Label("Logout", systemImage: "person.crop.circle.badge.minus")
                }
                .disabled(model.isWorking)
            }

            if let message = model.lastMessage {
                Text(message)
                    .foregroundStyle(.secondary)
            }

            if let health = model.status?.health, !health.isEmpty {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Health")
                        .font(.headline)
                    ForEach(health, id: \.self) { item in
                        Text(item)
                            .foregroundStyle(.secondary)
                    }
                }
            }
        }
    }

    private var accountSection: some View {
        Section("Saved Accounts and Tailnets") {
            if model.profiles.isEmpty {
                Text("No saved accounts reported by tailscale switch --list --json.")
                    .foregroundStyle(.secondary)
            } else {
                Picker("Saved account/tailnet", selection: $model.selectedProfileKey) {
                    ForEach(model.profiles, id: \.switchKey) { profile in
                        Text(profilePickerLabel(profile))
                            .tag(profile.switchKey)
                    }
                }

                Button {
                    Task { await model.switchSelectedProfile() }
                } label: {
                    Label("Switch", systemImage: "person.2")
                }
                .disabled(model.isWorking || model.selectedProfileKey.isEmpty)
            }
        }
    }

    private var selectedDeviceSection: some View {
        Section("Selected Device") {
            if let node = model.selectedNode {
                LabeledContent("Name", value: node.displayName)
                LabeledContent("Owner", value: model.status?.ownerLabel(for: node) ?? "Unknown")
                LabeledContent("OS", value: node.os.isEmpty ? "Unknown" : node.os)
                LabeledContent("Tailscale IPs", value: node.tailscaleIPs.joined(separator: ", "))
                LabeledContent("DNS", value: node.cleanDNSName.isEmpty ? "None" : node.cleanDNSName)
                LabeledContent("Online", value: node.online ? "Yes" : "No")
                LabeledContent("Relay", value: node.relay.isEmpty ? "None" : node.relay)
                LabeledContent("Endpoint", value: node.curAddr.isEmpty ? "None" : node.curAddr)
                LabeledContent("Allowed IPs", value: node.allowedIPs.joined(separator: ", "))
                LabeledContent("Last Seen", value: node.lastSeen.isEmpty ? "Unknown" : node.lastSeen)
                LabeledContent("Traffic", value: "\(formatBytes(node.rxBytes)) received / \(formatBytes(node.txBytes)) sent")
            } else {
                Text("Refresh to load this tailnet.")
                    .foregroundStyle(.secondary)
            }
        }
    }

    private var taildropSection: some View {
        Section("Taildrop") {
            HStack {
                Button {
                    showingReceiveImporter = true
                } label: {
                    Label("Receive Files...", systemImage: "tray.and.arrow.down")
                }
                .disabled(model.isWorking)

                if let node = model.selectedNode,
                   model.status?.thisNode?.stableKey != node.stableKey {
                    Button {
                        sendTarget = node
                        showingSendImporter = true
                    } label: {
                        Label("Send File to \(node.displayName)...", systemImage: "paperplane")
                    }
                    .disabled(model.isWorking || !node.canReceiveTaildrop)
                    .help(taildropHelp(for: node))
                }
            }

            Text("Receive uses tailscale file get with conflict renaming. Send availability follows Tailscale Taildrop policy for the selected device.")
                .foregroundStyle(.secondary)
        }
    }

    private var exitNodeSection: some View {
        Section("Exit Nodes") {
            if model.exitNodeOptions.isEmpty {
                Text("No approved exit nodes are currently reported.")
                    .foregroundStyle(.secondary)
            } else {
                Picker("Exit node", selection: $model.selectedExitNodeKey) {
                    ForEach(model.exitNodeOptions, id: \.stableKey) { node in
                        Text(node.displayName)
                            .tag(node.stableKey)
                    }
                }

                HStack {
                    Button {
                        Task { await model.setSelectedExitNode() }
                    } label: {
                        Label("Use Exit Node", systemImage: "point.topleft.down.curvedto.point.bottomright.up")
                    }
                    .disabled(model.isWorking || model.selectedExitNodeKey.isEmpty)

                    Button {
                        Task { await model.clearExitNode() }
                    } label: {
                        Label("Clear Exit Node", systemImage: "xmark.circle")
                    }
                    .disabled(model.isWorking)
                }
            }

            HStack {
                Button {
                    Task { await model.advertiseExitNode(true) }
                } label: {
                    Label("Advertise This Mac", systemImage: "antenna.radiowaves.left.and.right")
                }
                .disabled(model.isWorking)

                Button {
                    Task { await model.advertiseExitNode(false) }
                } label: {
                    Label("Stop Advertising", systemImage: "antenna.radiowaves.left.and.right.slash")
                }
                .disabled(model.isWorking)
            }
        }
    }

    private var diagnosticsSection: some View {
        Section("Diagnostics") {
            HStack {
                Button {
                    Task { await model.runVersion() }
                } label: {
                    Label("Version", systemImage: "number")
                }
                .disabled(model.isWorking)

                Button {
                    Task { await model.runNetcheck() }
                } label: {
                    Label("Netcheck", systemImage: "network")
                }
                .disabled(model.isWorking)

                Button {
                    Task { await model.runBugreport() }
                } label: {
                    Label("Bug Report", systemImage: "ladybug")
                }
                .disabled(model.isWorking)
            }
        }
    }

    private func profilePickerLabel(_ profile: TailscaleProfile) -> String {
        if profile.detail.isEmpty {
            return profile.selected ? "\(profile.displayName) (current)" : profile.displayName
        }
        let label = "\(profile.displayName) - \(profile.detail)"
        return profile.selected ? "\(label) (current)" : label
    }

    private func taildropHelp(for node: TailscaleNode) -> String {
        if node.canReceiveTaildrop {
            return "Send one file to \(node.displayName)"
        }
        if !node.noFileSharingReason.isEmpty {
            return node.noFileSharingReason
        }
        if !node.online {
            return "The device is offline."
        }
        return "Tailscale did not report this device as a Taildrop target."
    }

    private func handleSendSelection(_ selection: Result<[URL], Error>) {
        switch selection {
        case .success(let urls):
            guard let url = urls.first, let sendTarget else {
                return
            }
            Task { await model.sendFile(url, to: sendTarget) }
        case .failure(let error):
            model.presentImportError(error)
        }
    }

    private func handleReceiveSelection(_ selection: Result<[URL], Error>) {
        switch selection {
        case .success(let urls):
            guard let url = urls.first else {
                return
            }
            Task { await model.receiveFiles(to: url) }
        case .failure(let error):
            model.presentImportError(error)
        }
    }

    private func formatBytes(_ value: UInt64) -> String {
        ByteCountFormatter.string(fromByteCount: Int64(value), countStyle: .file)
    }
}

private struct HeaderView: View {
    @EnvironmentObject private var model: AppViewModel

    var body: some View {
        HStack(alignment: .center) {
            VStack(alignment: .leading, spacing: 4) {
                Text("TailScout")
                    .font(.largeTitle)
                    .bold()
                Text("\(model.status?.backendState.label ?? "Unknown") - \(model.currentTailnetLabel)")
                    .foregroundStyle(.secondary)
            }

            Spacer()

            if let selfNode = model.status?.thisNode {
                VStack(alignment: .trailing, spacing: 4) {
                    Text(selfNode.displayName)
                        .font(.headline)
                    Text(selfNode.primaryIP ?? "No Tailscale IP")
                        .foregroundStyle(.secondary)
                }
            }
        }
        .padding()
    }
}

private struct DeviceListRow: View {
    let node: TailscaleNode
    let owner: String?
    let isSelf: Bool

    var body: some View {
        HStack(spacing: 10) {
            Image(systemName: node.online ? "circle.fill" : "circle")
                .foregroundStyle(node.online ? .green : .secondary)
                .font(.caption)

            VStack(alignment: .leading, spacing: 2) {
                Text(isSelf ? "\(node.displayName) (This Mac)" : node.displayName)
                    .lineLimit(1)
                Text(subtitle)
                    .foregroundStyle(.secondary)
                    .font(.caption)
                    .lineLimit(1)
            }

            Spacer()

            if node.exitNodeOption {
                Image(systemName: "point.topleft.down.curvedto.point.bottomright.up")
                    .foregroundStyle(.secondary)
                    .help("Approved exit node")
            }
            if node.canReceiveTaildrop {
                Image(systemName: "paperplane")
                    .foregroundStyle(.secondary)
                    .help("Taildrop target")
            }
        }
    }

    private var subtitle: String {
        let ip = node.primaryIP ?? "no IP"
        let os = node.os.isEmpty ? "unknown OS" : node.os
        let ownerText = owner.map { " - \($0)" } ?? ""
        return "\(os) - \(ip)\(ownerText)"
    }
}

private struct DiagnosticView: View {
    let result: DiagnosticResult
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text(result.title)
                    .font(.title2)
                    .bold()
                Spacer()
                Button("Done") {
                    dismiss()
                }
                .keyboardShortcut(.defaultAction)
            }

            ScrollView {
                Text(result.output)
                    .font(.system(.body, design: .monospaced))
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .textSelection(.enabled)
            }
            .frame(minWidth: 640, minHeight: 360)
        }
        .padding()
    }
}
