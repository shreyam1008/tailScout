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

    let client: TailscaleCLI

    init(client: TailscaleCLI = TailscaleCLI()) {
        self.client = client
    }

    var peers: [TailscaleNode] { status?.sortedPeers ?? [] }

    var selectedNode: TailscaleNode? {
        selectedNodeKey.flatMap { key in peers.first { $0.stableKey == key } }
            ?? peers.first
    }

    var exitNodeOptions: [TailscaleNode] { status?.exitNodeOptions ?? [] }

    var selectedExitNode: TailscaleNode? {
        exitNodeOptions.first { $0.stableKey == selectedExitNodeKey }
    }

    var currentTailnetLabel: String {
        [status?.currentTailnet?.name, status?.magicDNSSuffix]
            .compactMap { $0 }
            .first(where: { !$0.isEmpty }) ?? "Unknown tailnet"
    }

    func canSendTaildrop(to node: TailscaleNode) -> Bool {
        status?.canSendTaildrop(to: node) == true
    }
}
