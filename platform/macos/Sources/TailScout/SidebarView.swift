import SwiftUI
import TailScoutCore

struct SidebarView: View {
    @EnvironmentObject private var model: AppViewModel

    var body: some View {
        List(selection: $model.selectedNodeKey) {
            Section("Devices") {
                if model.peers.isEmpty {
                    Text("No peers found").foregroundStyle(.secondary)
                } else {
                    ForEach(model.peers, id: \.stableKey) { node in
                        DeviceListRow(
                            node: node,
                            owner: model.status?.ownerLabel(for: node),
                            canSendTaildrop: model.canSendTaildrop(to: node)
                        )
                            .tag(Optional(node.stableKey))
                    }
                }
            }
        }
        .navigationTitle("TailScout")
    }
}

private struct DeviceListRow: View {
    let node: TailscaleNode
    let owner: String?
    let canSendTaildrop: Bool

    var body: some View {
        HStack(spacing: 10) {
            Image(systemName: node.online ? "circle.fill" : "circle")
                .foregroundStyle(node.online ? .green : .secondary)
                .font(.caption)
            VStack(alignment: .leading, spacing: 2) {
                Text(node.displayName).lineLimit(1)
                Text(subtitle).foregroundStyle(.secondary).font(.caption).lineLimit(1)
            }
            Spacer()
            Text(node.online ? "Online" : "Offline").foregroundStyle(.secondary)
        }
    }

    private var subtitle: String {
        var parts = [node.osLabel, node.primaryIP ?? "No Tailscale IP"]
        if let owner { parts.append(owner) }
        if node.isSubnetRouter { parts.append("Subnet router") }
        if node.exitNodeOption { parts.append(node.exitNode ? "Current exit node" : "Exit node") }
        if canSendTaildrop { parts.append("Taildrop") }
        return parts.joined(separator: " · ")
    }
}
