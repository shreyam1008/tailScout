import Foundation

public struct TailscaleStatus: Decodable, Equatable, Sendable {
    public let version: String
    public let clientVersion: String
    public let backendState: BackendState
    public let tun: Bool
    public let magicDNSSuffix: String
    public let currentTailnet: Tailnet?
    public let health: [String]
    public let thisNode: TailscaleNode?
    public let peers: [TailscaleNode]
    public let users: [UInt64: UserProfile]

    enum CodingKeys: String, CodingKey {
        case version = "Version"
        case clientVersion = "ClientVersion"
        case backendState = "BackendState"
        case tun = "TUN"
        case magicDNSSuffix = "MagicDNSSuffix"
        case currentTailnet = "CurrentTailnet"
        case health = "Health"
        case thisNode = "Self"
        case peer = "Peer"
        case user = "User"
    }

    public init(from decoder: Decoder) throws {
        let values = try decoder.container(keyedBy: CodingKeys.self)
        version = try values.decodeString(forKey: .version)
        clientVersion = try values.decodeString(forKey: .clientVersion)
        backendState = BackendState(rawValue: try values.decodeString(forKey: .backendState))
        tun = try values.decodeBool(forKey: .tun)
        magicDNSSuffix = try values.decodeString(forKey: .magicDNSSuffix)
        currentTailnet = try values.decodeIfPresent(Tailnet.self, forKey: .currentTailnet)
        health = try values.decodeStringArray(forKey: .health)
        thisNode = try values.decodeIfPresent(TailscaleNode.self, forKey: .thisNode)
        let peerMap = try values.decodeIfPresent([String: TailscaleNode].self, forKey: .peer) ?? [:]
        peers = Array(peerMap.values)

        let rawUsers = try values.decodeIfPresent([String: UserProfile].self, forKey: .user) ?? [:]
        users = rawUsers.reduce(into: [:]) { result, entry in
            guard let id = UInt64(entry.key) else { return }
            var profile = entry.value
            profile.id = id
            result[id] = profile
        }
    }

    public static func parse(_ input: String) throws -> Self {
        try parse(Data(input.utf8))
    }

    public static func parse(_ data: Data) throws -> Self {
        try JSONDecoder().decode(Self.self, from: data)
    }

    public var displayVersion: String {
        version.isEmpty ? clientVersion : version
    }

    public var sortedPeers: [TailscaleNode] {
        peers.sorted {
            $0.online == $1.online
                ? $0.displayName.localizedCaseInsensitiveCompare($1.displayName) == .orderedAscending
                : $0.online
        }
    }

    public var exitNodeOptions: [TailscaleNode] {
        sortedPeers.filter(\.exitNodeOption)
    }

    public func ownerLabel(for node: TailscaleNode) -> String? {
        users[node.userID]?.displayLabel
    }

    public func hasSameOwner(as node: TailscaleNode) -> Bool {
        guard let thisNode else { return true }
        return thisNode.userID == 0 || node.userID == 0 || thisNode.userID == node.userID
    }

    public func canSendTaildrop(to node: TailscaleNode) -> Bool {
        node.canReceiveTaildrop && hasSameOwner(as: node)
    }
}
